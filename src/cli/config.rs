use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, ensure};

const FILE_NAME: &str = ".alofmt.toml";

pub fn load(explicit: Option<&Path>, disabled: bool) -> Result<alofmt::FormatOptions> {
    ensure!(
        explicit.is_none() || !disabled,
        "--config cannot be combined with --no-config"
    );
    if disabled {
        return Ok(alofmt::FormatOptions::default());
    }

    let path = match explicit {
        Some(path) => Some(path.to_owned()),
        None => {
            let current = std::env::current_dir().context("determine current directory")?;
            discover_from(&current)?
        }
    };
    let Some(path) = path else {
        return Ok(alofmt::FormatOptions::default());
    };

    let metadata =
        fs::metadata(&path).with_context(|| format!("inspect formatter configuration {}", path.display()))?;
    ensure!(
        metadata.is_file(),
        "formatter configuration is not a regular file: {}",
        path.display()
    );
    let source =
        fs::read_to_string(&path).with_context(|| format!("read formatter configuration {}", path.display()))?;
    alofmt::FormatOptions::from_toml(&source)
        .with_context(|| format!("load formatter configuration {}", path.display()))
}

fn discover_from(start: &Path) -> Result<Option<PathBuf>> {
    for directory in start.ancestors() {
        let candidate = directory.join(FILE_NAME);
        match fs::metadata(&candidate) {
            Ok(metadata) if metadata.is_file() => return Ok(Some(candidate)),
            Ok(_) => {
                anyhow::bail!("formatter configuration is not a regular file: {}", candidate.display());
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => {
                return Err(error).with_context(|| format!("inspect formatter configuration {}", candidate.display()));
            }
        }
    }
    Ok(None)
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
        let path = std::env::temp_dir().join(format!("alofmt-config-{test}-{}-{nonce}", std::process::id()));
        fs::create_dir(&path).expect("create test directory");
        path
    }

    #[test]
    fn discovers_the_nearest_configuration() {
        let root = temporary_directory("discovery");
        let nested = root.join("one/two");
        fs::create_dir_all(&nested).expect("create nested directory");
        fs::write(root.join(FILE_NAME), "line_width = 100\n").expect("write root config");
        fs::write(root.join("one").join(FILE_NAME), "line_width = 90\n").expect("write nearest config");

        assert_eq!(
            discover_from(&nested).expect("discover config"),
            Some(root.join("one").join(FILE_NAME))
        );

        fs::remove_dir_all(root).expect("remove test directory");
    }
}
