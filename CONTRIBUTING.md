# Contributing

Focused bug reports and formatting fixtures are welcome.

alofmt requires Rust 1.89 or newer. Run these checks from the repository root:

```sh
cargo fmt --check
cargo build --locked --release
cargo test --locked
```

The build and test commands are separate so a release build never runs the
test suite as a side effect.

## Formatting changes

Add one focused fixture for each layout rule that changes. Place the fixture
under the matching language family in `tests/fixtures`.

The fixture suite treats every file as canonical output. Each fixture must
format to itself byte for byte.

The process-level tests in `tests/cli.rs` cover CLI behavior such as config
discovery, standard input, checks, traversal, and safe writes.

## Source layout

`src/options.rs` defines public policy. `src/comments.rs` attaches comments.
`src/doc.rs` contains the document model and renderer. `src/format.rs` owns
per-file formatter state, while `src/format/` groups layouts by Ruby language
family.

The CLI lives in `src/cli.rs` and `src/cli/`. The project uses Rust 2024
sibling module files and does not use `mod.rs`.

The optional scripts under `oracle/` compare output with Syntax Tree 6.2.0 and
exercise formatting round trips.
