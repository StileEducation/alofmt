use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

fn alofmt(arguments: &[&str], stdin: &[u8]) -> Output {
    run_alofmt(arguments, stdin, None, true)
}

fn run_alofmt(arguments: &[&str], stdin: &[u8], directory: Option<&std::path::Path>, no_config: bool) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_alofmt"));
    if no_config {
        command.arg("--no-config");
    }
    command
        .args(arguments)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    if let Some(directory) = directory {
        command.current_dir(directory);
    }
    let mut child = command.spawn().expect("start alofmt");
    child
        .stdin
        .take()
        .expect("stdin pipe")
        .write_all(stdin)
        .expect("write stdin");
    child.wait_with_output().expect("wait for alofmt")
}

#[test]
fn discovers_configuration_and_applies_cli_overrides_last() {
    let root = temporary_directory("config");
    fs::write(root.join(".alofmt.toml"), "quote_style = \"double\"\n").expect("write config");

    let configured = run_alofmt(&["-"], b"x='hello'\n", Some(&root), false);
    let overridden = run_alofmt(&["--quote-style", "single", "-"], b"x=\"hello\"\n", Some(&root), false);

    assert!(
        configured.status.success(),
        "{}",
        String::from_utf8_lossy(&configured.stderr)
    );
    assert_eq!(configured.stdout, b"x = \"hello\"\n");
    assert!(
        overridden.status.success(),
        "{}",
        String::from_utf8_lossy(&overridden.stderr)
    );
    assert_eq!(overridden.stdout, b"x = 'hello'\n");
    fs::remove_dir_all(root).expect("remove test directory");
}

fn temporary_directory(test: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock after epoch")
        .as_nanos();
    let path = std::env::temp_dir().join(format!("alofmt-cli-{test}-{}-{nonce}", std::process::id()));
    fs::create_dir(&path).expect("create test directory");
    path
}

#[test]
fn formats_standard_input_and_honours_style_flags() {
    let output = alofmt(&["--quote-style", "preserve", "-"], b"x=\"hello\"\n");

    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    assert_eq!(output.stdout, b"x = \"hello\"\n");
    assert!(output.stderr.is_empty());
}

#[test]
fn check_on_standard_input_uses_documented_exit_statuses() {
    let changed = alofmt(&["--check", "--quiet", "-"], b"x=1\n");
    let clean = alofmt(&["--check", "--quiet", "-"], b"x = 1\n");

    assert_eq!(changed.status.code(), Some(1));
    assert!(changed.stdout.is_empty());
    assert!(changed.stderr.is_empty());
    assert_eq!(clean.status.code(), Some(0));
}

#[test]
fn write_walks_directories_and_respects_gitignore() {
    let root = temporary_directory("walk");
    fs::create_dir(root.join(".git")).expect("create repository marker");
    fs::write(root.join(".gitignore"), b"ignored.rb\n").expect("write ignore file");
    fs::write(root.join("keep.rb"), b"x=1\n").expect("write included Ruby file");
    fs::write(root.join("ignored.rb"), b"y=2\n").expect("write ignored Ruby file");

    let output = alofmt(&["--write", "--quiet", root.to_str().expect("UTF-8 path")], b"");

    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    assert_eq!(fs::read(root.join("keep.rb")).expect("read included file"), b"x = 1\n");
    assert_eq!(fs::read(root.join("ignored.rb")).expect("read ignored file"), b"y=2\n");
    fs::remove_dir_all(root).expect("remove test directory");
}

#[test]
fn refuses_ambiguous_multi_file_stdout() {
    let root = temporary_directory("multiple");
    let first = root.join("a.rb");
    let second = root.join("b.rb");
    fs::write(&first, b"a=1\n").expect("write first file");
    fs::write(&second, b"b=2\n").expect("write second file");

    let output = alofmt(
        &[
            first.to_str().expect("UTF-8 path"),
            second.to_str().expect("UTF-8 path"),
        ],
        b"",
    );

    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("printing requires exactly one Ruby file"));
    fs::remove_dir_all(root).expect("remove test directory");
}

#[test]
fn parse_failures_are_reported_without_rewriting_the_file() {
    let root = temporary_directory("parse-error");
    let path = root.join("invalid.rb");
    let source = b"def broken(\n";
    fs::write(&path, source).expect("write invalid file");

    let output = alofmt(&["--write", "--quiet", path.to_str().expect("UTF-8 path")], b"");

    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("parse error"));
    assert_eq!(fs::read(&path).expect("read invalid file"), source);
    fs::remove_dir_all(root).expect("remove test directory");
}
