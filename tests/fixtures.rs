//! Every file under `tests/fixtures` is canonical output under the explicit
//! compatibility profile, so formatting it must change nothing.

use std::path::Path;

fn walk(dir: &Path, out: &mut Vec<std::path::PathBuf>) {
    for entry in std::fs::read_dir(dir).expect("fixture dir") {
        let path = entry.expect("dir entry").path();
        if path.is_dir() {
            walk(&path, out);
        } else if path.extension().is_some_and(|e| e == "rb") {
            out.push(path);
        }
    }
}

#[test]
fn fixtures_are_fixpoints() {
    let options = alofmt::FormatOptions::from_toml(include_str!("compatibility.toml"))
        .expect("valid fixture compatibility configuration");
    let mut paths = Vec::new();
    walk(
        Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures")),
        &mut paths,
    );
    paths.sort();
    assert!(!paths.is_empty(), "no fixtures found");
    let mut failures = Vec::new();
    for path in &paths {
        let source = std::fs::read(path).expect("read fixture");
        match alofmt::is_formatted_with_options(&source, &options) {
            Ok(true) => {}
            Ok(false) => failures.push(format!(
                "{}: allocation-free check reported a canonical fixture as changed",
                path.display()
            )),
            Err(e) => failures.push(format!("{}: allocation-free check failed: {e:#}", path.display())),
        }
        match alofmt::format_with_options(&source, &options) {
            Ok(out) if out.as_bytes() == source.as_slice() => {}
            Ok(out) => {
                let diff = similar::TextDiff::from_lines(std::str::from_utf8(&source).expect("utf8"), &out)
                    .unified_diff()
                    .header("expected", "alofmt")
                    .to_string();
                failures.push(format!("{}:\n{diff}", path.display()));
            }
            Err(e) => failures.push(format!("{}: {e:#}", path.display())),
        }
    }
    assert!(
        failures.is_empty(),
        "{} of {} fixtures differ:\n\n{}",
        failures.len(),
        paths.len(),
        failures.join("\n")
    );
}
