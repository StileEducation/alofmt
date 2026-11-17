//! Formatting throughput over a corpus of Ruby files, to compare two builds of
//! alofmt on one machine:
//!
//! ```sh
//! cargo run --locked --release --example bench_format                    # the test fixtures
//! cargo run --locked --release --example bench_format -- ~/src/backend   # any directory
//! ```
//!
//! It times `alofmt::format` alone, with each file already in memory, so the
//! number moves with parsing, comment attachment and layout and not with the
//! filesystem. `docs/behavior.md` has the end-to-end figures.

const ROUNDS: usize = 15;

fn ruby_files(root: &std::path::Path, found: &mut Vec<std::path::PathBuf>) -> anyhow::Result<()> {
    for entry in std::fs::read_dir(root)? {
        let path = entry?.path();
        if path.is_dir() {
            ruby_files(&path, found)?;
        } else if path.extension().is_some_and(|extension| extension == "rb") {
            found.push(path);
        }
    }
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let root = std::env::args().nth(1).map_or_else(
        || std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests"),
        std::path::PathBuf::from,
    );
    let mut paths = Vec::new();
    ruby_files(&root, &mut paths)?;
    paths.sort();
    // A file alofmt can't format (a fixture of invalid Ruby) fails the same way in every build.
    let sources = paths
        .iter()
        .map(std::fs::read)
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .filter(|source| alofmt::format(source).is_ok())
        .collect::<Vec<_>>();
    let bytes: usize = sources.iter().map(Vec::len).sum();
    anyhow::ensure!(bytes > 0, "no Ruby that alofmt can format under {}", root.display());

    let mut rounds = (0..ROUNDS)
        .map(|_| {
            let start = std::time::Instant::now();
            for source in &sources {
                std::hint::black_box(alofmt::format(std::hint::black_box(source))?);
            }
            Ok(start.elapsed())
        })
        .collect::<anyhow::Result<Vec<_>>>()?;
    rounds.sort();
    let median = rounds[ROUNDS / 2];
    println!(
        "{} files, {:.1} KiB: median {:.2} ms a pass over {ROUNDS} passes, fastest {:.2} ms, {:.1} MiB/s",
        sources.len(),
        bytes as f64 / 1024.0,
        median.as_secs_f64() * 1000.0,
        rounds[0].as_secs_f64() * 1000.0,
        bytes as f64 / (1024.0 * 1024.0) / median.as_secs_f64(),
    );
    Ok(())
}
