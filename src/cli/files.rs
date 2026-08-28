use std::ffi::OsString;
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use anyhow::{Context, Result, bail, ensure};
use ignore::types::{Types, TypesBuilder};
use ignore::{WalkBuilder, WalkState};

static TEMP_SEQUENCE: AtomicU64 = AtomicU64::new(0);

pub fn discover(inputs: &[PathBuf], threads: usize) -> Result<Vec<PathBuf>> {
    ensure!(threads > 0, "discovery thread count must be greater than zero");
    let types = ruby_types()?;
    let mut files = Vec::new();
    for input in inputs {
        let metadata =
            fs::symlink_metadata(input).with_context(|| format!("cannot inspect input {}", input.display()))?;
        ensure!(
            !metadata.file_type().is_symlink(),
            "input is a symbolic link: {}",
            input.display()
        );
        if metadata.is_file() {
            files.push(input.clone());
        } else if metadata.is_dir() {
            discover_directory(input, types.clone(), threads, &mut files)?;
        } else {
            bail!("input is not a regular file or directory: {}", input.display());
        }
    }
    files.sort_unstable();
    files.dedup();
    Ok(files)
}

fn ruby_types() -> Result<Types> {
    let mut builder = TypesBuilder::new();
    builder.add("ruby", "*.rb")?;
    builder.add("ruby", "*.rbi")?;
    builder.select("ruby");
    builder.build().context("build Ruby file matcher")
}

fn discover_directory(root: &Path, types: Types, threads: usize, files: &mut Vec<PathBuf>) -> Result<()> {
    let mut builder = WalkBuilder::new(root);
    builder
        .types(types)
        .hidden(false)
        .follow_links(false)
        .standard_filters(true);
    if threads > 1 {
        builder.threads(threads);
        let (sender, receiver) = std::sync::mpsc::channel::<Result<PathBuf>>();
        builder.build_parallel().run(|| {
            let sender = sender.clone();
            Box::new(move |entry| {
                let entry = match entry.with_context(|| format!("walk input directory {}", root.display())) {
                    Ok(entry) => entry,
                    Err(error) => {
                        sender
                            .send(Err(error))
                            .expect("discovery receiver lives through the walk");
                        return WalkState::Quit;
                    }
                };
                if entry.file_type().is_some_and(|file_type| file_type.is_file()) {
                    sender
                        .send(Ok(entry.into_path()))
                        .expect("discovery receiver lives through the walk");
                }
                WalkState::Continue
            })
        });
        drop(sender);
        for path in receiver {
            files.push(path?);
        }
        return Ok(());
    }
    for entry in builder.build() {
        let entry = entry.with_context(|| format!("walk input directory {}", root.display()))?;
        if entry.file_type().is_some_and(|file_type| file_type.is_file()) {
            files.push(entry.into_path());
        }
    }
    Ok(())
}

pub fn replace(path: &Path, before: &[u8], after: &[u8]) -> Result<()> {
    let metadata = fs::symlink_metadata(path).with_context(|| format!("inspect {} before writing", path.display()))?;
    ensure!(
        !metadata.file_type().is_symlink(),
        "refusing to replace symbolic link {}",
        path.display()
    );
    ensure!(metadata.is_file(), "not a regular file: {}", path.display());

    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let (temporary_path, mut temporary) = create_temporary(parent, path.file_name())?;
    let write_result = (|| -> Result<()> {
        temporary
            .set_permissions(metadata.permissions())
            .with_context(|| format!("copy permissions to {}", temporary_path.display()))?;
        temporary
            .write_all(after)
            .with_context(|| format!("write temporary file {}", temporary_path.display()))?;
        temporary
            .flush()
            .with_context(|| format!("flush temporary file {}", temporary_path.display()))?;
        drop(temporary);

        let current = fs::read(path).with_context(|| format!("re-read {} before replacing it", path.display()))?;
        ensure!(
            current == before,
            "{} changed while it was being formatted",
            path.display()
        );
        fs::rename(&temporary_path, path)
            .with_context(|| format!("replace {} with {}", path.display(), temporary_path.display()))?;
        Ok(())
    })();

    if let Err(error) = write_result {
        return match fs::remove_file(&temporary_path) {
            Ok(()) => Err(error),
            Err(cleanup) if cleanup.kind() == std::io::ErrorKind::NotFound => Err(error),
            Err(cleanup) => Err(error.context(format!(
                "also failed to remove temporary file {}: {cleanup}",
                temporary_path.display()
            ))),
        };
    }
    Ok(())
}

fn create_temporary(parent: &Path, file_name: Option<&std::ffi::OsStr>) -> Result<(PathBuf, File)> {
    let file_name = file_name.unwrap_or_default();
    for _ in 0..100 {
        let sequence = TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let mut temporary_name = OsString::from(".");
        temporary_name.push(file_name);
        temporary_name.push(format!(".alofmt.{}.{sequence}.tmp", std::process::id()));
        let path = parent.join(temporary_name);
        match OpenOptions::new().write(true).create_new(true).open(&path) {
            Ok(file) => return Ok((path, file)),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(error) => {
                return Err(error).with_context(|| format!("create temporary file next to {}", parent.display()));
            }
        }
    }
    bail!("could not create a unique temporary file next to {}", parent.display())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temporary_directory(test: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock after epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("alofmt-{test}-{}-{nonce}", std::process::id()));
        fs::create_dir(&path).expect("create test directory");
        path
    }

    #[test]
    fn discovers_ruby_files_in_deterministic_order() {
        let root = temporary_directory("discover");
        fs::create_dir(root.join("nested")).expect("create nested directory");
        fs::write(root.join("z.rb"), b"").expect("write Ruby file");
        fs::write(root.join("nested/a.rbi"), b"").expect("write RBI file");
        fs::write(root.join("ignored.txt"), b"").expect("write other file");

        let files = discover(std::slice::from_ref(&root), 4).expect("discover files");
        assert_eq!(files, vec![root.join("nested/a.rbi"), root.join("z.rb")]);

        fs::remove_dir_all(root).expect("remove test directory");
    }

    #[test]
    fn replaces_a_file_without_leaving_a_temporary_file() {
        let root = temporary_directory("replace");
        let path = root.join("example.rb");
        fs::write(&path, b"before").expect("write source");

        replace(&path, b"before", b"after").expect("replace source");

        assert_eq!(fs::read(&path).expect("read replacement"), b"after");
        assert_eq!(fs::read_dir(&root).expect("read directory").count(), 1);
        fs::remove_dir_all(root).expect("remove test directory");
    }

    #[test]
    fn refuses_to_overwrite_a_concurrent_change() {
        let root = temporary_directory("concurrent");
        let path = root.join("example.rb");
        fs::write(&path, b"changed").expect("write source");

        let error = replace(&path, b"before", b"after").expect_err("concurrent change should fail");

        assert!(error.to_string().contains("changed while it was being formatted"));
        assert_eq!(fs::read(&path).expect("read source"), b"changed");
        assert_eq!(fs::read_dir(&root).expect("read directory").count(), 1);
        fs::remove_dir_all(root).expect("remove test directory");
    }

    #[cfg(unix)]
    #[test]
    fn refuses_symbolic_link_inputs() {
        use std::os::unix::fs::symlink;

        let root = temporary_directory("symlink");
        let target = root.join("target.rb");
        let link = root.join("link.rb");
        fs::write(&target, b"x = 1\n").expect("write target");
        symlink(&target, &link).expect("create symbolic link");

        let error = discover(std::slice::from_ref(&link), 1).expect_err("symbolic link should fail");

        assert!(error.to_string().contains("input is a symbolic link"));
        fs::remove_dir_all(root).expect("remove test directory");
    }
}
