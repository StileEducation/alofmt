# alofmt

[![Crates.io](https://img.shields.io/crates/v/alofmt.svg)](https://crates.io/crates/alofmt)
[![Documentation](https://docs.rs/alofmt/badge.svg)](https://docs.rs/alofmt)
[![MIT licence](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

A fast, deterministic Ruby formatter powered by
[Prism](https://github.com/ruby/prism) and written in Rust. Pronounced like "elephant", but with more Aluminium Oxide.

alofmt turns this:

```ruby
published=Catalog.fetch(ids).filter_map{|record|
# Keep drafts out of the index.
next unless record.published?
{id:record.id,title:record.title,topics:record.topics,metadata:record.metadata}
}.group_by{|record|record[:topics].first}.transform_values(&:count)
```

into this:

```ruby
published =
  Catalog
    .fetch(ids)
    .filter_map do |record|
      # Keep drafts out of the index.
      next unless record.published?
      {
        id: record.id,
        title: record.title,
        topics: record.topics,
        metadata: record.metadata
      }
    end
    .group_by { |record| record[:topics].first }
    .transform_values(&:count)
```

## Quick start

Install alofmt from source with Cargo:

```sh
cargo install --locked alofmt
```

Then format or check a project:

```sh
alofmt --write .         # Format every Ruby file in place.
alofmt --check .         # Check formatting without changing files.
alofmt --check --diff .  # Show what would change.
```

alofmt finds `.rb` and `.rbi` files, respects `.gitignore`, and formats files
in parallel. Pass one file without `--write` or `--check` to print the result,
or use `-` to read standard input.

## Highlights

- **Fast:** directory discovery and formatting both run in parallel.
- **Deterministic:** parallel work still produces stable output and reporting.
- **Configurable:** project policy lives in `.alofmt.toml`, not formatter
  special cases.
- **Strict:** invalid input, parse errors, unsupported syntax, and misspelled
  configuration keys fail visibly.
- **Embeddable:** the Rust library exposes the same formatter and
  allocation-free checks as the CLI.

> [!NOTE]
> alofmt is pre-1.0. It supports a broad and growing portion of Ruby syntax.
> Unsupported Prism nodes return an error instead of partial or lossy output.

## Configuration

The defaults are pretty standard; two-space indentation, preserved quotes,
and no method-name or optional syntax special cases.

Add `.alofmt.toml` at the root of a project to change that:

```toml
line_width = 100
indent_width = 4
quote_style = "single"
trailing_commas = true
```

alofmt searches the current directory and its parents for the nearest config
file. CLI options override the file for one run.

See the [configuration reference](docs/configuration.md) for every option and
its default value.

## Library

Use alofmt as a Rust library when formatting needs to live inside another
tool:

```rust
use alofmt::{FormatOptions, QuoteStyle, format_with_options};

let options = FormatOptions {
    quote_style: QuoteStyle::Single,
    ..FormatOptions::default()
};

let output = format_with_options(b"message = \"hello\"\n", &options)?;
assert_eq!(output, "message = 'hello'\n");
# Ok::<(), anyhow::Error>(())
```

Read the [API documentation](https://docs.rs/alofmt) for the complete library
interface.

## More

- [Behavior, safety, and performance](docs/behavior.md)
- [Configuration reference](docs/configuration.md)
- [Contributing](CONTRIBUTING.md)

## Licence

alofmt is available under the [MIT licence](LICENSE).
