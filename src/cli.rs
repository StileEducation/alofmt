mod config;
mod files;

use std::ffi::OsStr;
use std::io::{Read, Write};
use std::num::NonZeroUsize;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use anyhow::{Context, Result, bail};
use clap::{Args, Parser, ValueEnum};
use rayon::prelude::*;

const MAX_DISCOVERY_THREADS: usize = 4;

#[derive(Parser, Debug)]
#[command(name = "alofmt", about = "Format Ruby source code", version)]
struct Options {
    /// Ruby files or directories to format. `-` reads standard input.
    #[arg(required = true)]
    paths: Vec<PathBuf>,

    /// Rewrite files in place.
    #[arg(short, long, conflicts_with_all = ["check", "diff"])]
    write: bool,

    /// Exit with status 1 if any input would change.
    #[arg(short, long)]
    check: bool,

    /// Print a unified diff for each input that would change.
    #[arg(long, requires = "check")]
    diff: bool,

    /// Suppress changed-file status and the summary. Errors and requested
    /// diffs are still printed.
    #[arg(short, long)]
    quiet: bool,

    /// Number of worker threads. Directory discovery uses at most four.
    #[arg(long)]
    threads: Option<NonZeroUsize>,

    /// Read formatting policy from this TOML file instead of discovering
    /// `.alofmt.toml` from the current directory.
    #[arg(long, value_name = "PATH", conflicts_with = "no_config")]
    config: Option<PathBuf>,

    /// Do not discover or load a formatting configuration.
    #[arg(long, conflicts_with = "config")]
    no_config: bool,

    #[command(flatten)]
    style: StyleOptions,
}

#[derive(Args, Clone, Debug, Default)]
struct StyleOptions {
    /// Maximum line width.
    #[arg(long, value_name = "COLUMNS")]
    line_width: Option<usize>,

    /// Spaces emitted for one indentation level.
    #[arg(long, value_name = "SPACES")]
    indent_width: Option<usize>,

    /// Width charged to one indentation level while deciding line breaks.
    #[arg(long, value_name = "COLUMNS")]
    fit_indent_width: Option<usize>,

    /// Preferred quote style for plain strings and symbols.
    #[arg(long)]
    quote_style: Option<QuoteStyle>,

    /// Add trailing commas to broken collections and argument lists.
    #[arg(long, value_name = "BOOL", num_args = 0..=1, require_equals = true, default_missing_value = "true")]
    trailing_commas: Option<bool>,

    /// Whether an array of single-word strings or symbols collapses to `%w` or `%i`.
    #[arg(long, value_enum)]
    percent_arrays: Option<PercentArrays>,

    /// Add thousands separators to eligible decimal integers.
    #[arg(long, value_name = "BOOL", num_args = 0..=1, require_equals = true, default_missing_value = "true")]
    normalize_number_separators: Option<bool>,

    /// Spell an omitted rescue class as `StandardError`.
    #[arg(long, value_name = "BOOL", num_args = 0..=1, require_equals = true, default_missing_value = "true")]
    explicit_standard_error: Option<bool>,

    /// Replace the default ignore directives with this repeatable value.
    #[arg(long = "ignore-directive", value_name = "COMMENT")]
    ignore_directives: Vec<String>,

    /// Disable all ignore directives.
    #[arg(long, conflicts_with = "ignore_directives")]
    no_ignore_directives: bool,

    /// Number of chained calls that makes a chain break.
    #[arg(long)]
    chain_break_threshold: Option<usize>,

    /// Chain threshold inside configured compact-chain blocks.
    #[arg(long)]
    compact_chain_break_threshold: Option<usize>,

    /// Replace the default compact-chain block names with this repeatable
    /// value.
    #[arg(long = "compact-chain-block", value_name = "METHOD")]
    compact_chain_blocks: Vec<String>,

    /// Disable compact-chain block special cases.
    #[arg(long, conflicts_with = "compact_chain_blocks")]
    no_compact_chain_blocks: bool,

    /// Replace the default unaligned command names with this repeatable value.
    #[arg(long = "unaligned-command-call", value_name = "METHOD")]
    unaligned_command_calls: Vec<String>,

    /// Disable unaligned command special cases.
    #[arg(long, conflicts_with = "unaligned_command_calls")]
    no_unaligned_command_calls: bool,

    /// Maximum command prefix width eligible for continuation alignment.
    #[arg(long)]
    max_command_alignment: Option<usize>,

    /// Where a command call's sole bracketed argument sits when it breaks.
    #[arg(long, value_enum)]
    delimited_argument_alignment: Option<DelimitedArgumentAlignment>,

    /// Where the value of an assignment that cannot fit on one line goes.
    #[arg(long, value_enum)]
    multiline_assignment_layout: Option<MultilineAssignmentLayout>,

    /// Which delimiters a block prints with, where either would parse the same.
    #[arg(long, value_enum)]
    block_delimiters: Option<BlockDelimiters>,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum BlockDelimiters {
    LineCountBased,
    AlwaysBraces,
    AlwaysDoEnd,
    Preserve,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum DelimitedArgumentAlignment {
    Aligned,
    Consistent,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum MultilineAssignmentLayout {
    NewLine,
    SameLine,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum PercentArrays {
    Prefer,
    Preserve,
    Avoid,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum QuoteStyle {
    Single,
    Double,
    Preserve,
}

impl StyleOptions {
    fn apply(self, options: &mut alofmt::FormatOptions) -> Result<()> {
        set_if_some(&mut options.line_width, self.line_width);
        set_if_some(&mut options.indent_width, self.indent_width);
        set_if_some(&mut options.fit_indent_width, self.fit_indent_width);
        set_if_some(&mut options.trailing_commas, self.trailing_commas);
        set_if_some(
            &mut options.normalize_number_separators,
            self.normalize_number_separators,
        );
        set_if_some(&mut options.explicit_standard_error, self.explicit_standard_error);
        set_if_some(&mut options.chain_break_threshold, self.chain_break_threshold);
        set_if_some(
            &mut options.compact_chain_break_threshold,
            self.compact_chain_break_threshold,
        );
        set_if_some(&mut options.max_command_alignment, self.max_command_alignment);
        if let Some(alignment) = self.delimited_argument_alignment {
            options.delimited_argument_alignment = match alignment {
                DelimitedArgumentAlignment::Aligned => alofmt::DelimitedArgumentAlignment::Aligned,
                DelimitedArgumentAlignment::Consistent => alofmt::DelimitedArgumentAlignment::Consistent,
            };
        }
        if let Some(layout) = self.multiline_assignment_layout {
            options.multiline_assignment_layout = match layout {
                MultilineAssignmentLayout::NewLine => alofmt::MultilineAssignmentLayout::NewLine,
                MultilineAssignmentLayout::SameLine => alofmt::MultilineAssignmentLayout::SameLine,
            };
        }
        if let Some(style) = self.quote_style {
            options.quote_style = match style {
                QuoteStyle::Single => alofmt::QuoteStyle::Single,
                QuoteStyle::Double => alofmt::QuoteStyle::Double,
                QuoteStyle::Preserve => alofmt::QuoteStyle::Preserve,
            };
        }
        if let Some(mode) = self.percent_arrays {
            options.percent_arrays = match mode {
                PercentArrays::Prefer => alofmt::PercentArrays::Prefer,
                PercentArrays::Preserve => alofmt::PercentArrays::Preserve,
                PercentArrays::Avoid => alofmt::PercentArrays::Avoid,
            };
        }
        if let Some(delimiters) = self.block_delimiters {
            options.block_delimiters = match delimiters {
                BlockDelimiters::LineCountBased => alofmt::BlockDelimiters::LineCountBased,
                BlockDelimiters::AlwaysBraces => alofmt::BlockDelimiters::AlwaysBraces,
                BlockDelimiters::AlwaysDoEnd => alofmt::BlockDelimiters::AlwaysDoEnd,
                BlockDelimiters::Preserve => alofmt::BlockDelimiters::Preserve,
            };
        }
        replace_list(
            &mut options.ignore_directives,
            self.ignore_directives,
            self.no_ignore_directives,
        );
        replace_list(
            &mut options.compact_chain_blocks,
            self.compact_chain_blocks,
            self.no_compact_chain_blocks,
        );
        replace_list(
            &mut options.unaligned_command_calls,
            self.unaligned_command_calls,
            self.no_unaligned_command_calls,
        );
        options.validate()?;
        Ok(())
    }
}

fn set_if_some<T>(target: &mut T, value: Option<T>) {
    if let Some(value) = value {
        *target = value;
    }
}

fn replace_list(target: &mut Vec<String>, values: Vec<String>, clear: bool) {
    if clear {
        target.clear();
    } else if !values.is_empty() {
        *target = values;
    }
}

#[derive(Clone, Copy)]
enum Mode {
    Check { diff: bool },
    Write,
}

enum Outcome {
    Unchanged,
    Changed { diff: Option<String> },
    Failed(String),
}

pub fn run() -> Result<ExitCode> {
    run_with(Options::parse())
}

fn run_with(options: Options) -> Result<ExitCode> {
    if options.quiet && !options.check && !options.write {
        bail!("--quiet requires --check or --write");
    }
    let mut format_options = config::load(options.config.as_deref(), options.no_config)?;
    options.style.clone().apply(&mut format_options)?;
    let stdin = options
        .paths
        .iter()
        .filter(|path| path.as_os_str() == OsStr::new("-"))
        .count();
    if stdin > 0 {
        if options.paths.len() != 1 {
            bail!("standard input (`-`) cannot be combined with file paths");
        }
        if options.write {
            bail!("--write cannot be used with standard input");
        }
        return run_stdin(&options, &format_options);
    }

    let threads = match options.threads {
        Some(threads) => threads.get(),
        None => std::thread::available_parallelism()
            .context("determine available formatter parallelism")?
            .get(),
    };
    // Directory walking is syscall-bound; more walkers add contention while
    // file formatting continues to benefit from the full worker count.
    let paths = files::discover(&options.paths, threads.min(MAX_DISCOVERY_THREADS))?;
    if !options.check && !options.write {
        return print_one(&paths, &format_options);
    }
    let mode = if options.write {
        Mode::Write
    } else {
        Mode::Check { diff: options.diff }
    };
    let outcomes: Vec<_> = if paths.len() <= 1 || threads == 1 {
        paths
            .iter()
            .map(|path| process_file(path, mode, &format_options))
            .collect()
    } else {
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(threads)
            .build()
            .context("create formatter thread pool")?;
        pool.install(|| {
            paths
                .par_iter()
                .map(|path| process_file(path, mode, &format_options))
                .collect()
        })
    };

    let mut stdout = std::io::BufWriter::new(std::io::stdout().lock());
    let mut stderr = std::io::BufWriter::new(std::io::stderr().lock());
    let (mut unchanged, mut changed, mut failed) = (0usize, 0usize, 0usize);
    for (path, outcome) in paths.iter().zip(outcomes) {
        match outcome {
            Outcome::Unchanged => unchanged += 1,
            Outcome::Changed { diff } => {
                changed += 1;
                if options.check && !options.quiet {
                    writeln!(stdout, "changed: {}", path.display())?;
                }
                if let Some(diff) = diff {
                    stdout.write_all(diff.as_bytes())?;
                }
            }
            Outcome::Failed(error) => {
                failed += 1;
                writeln!(stderr, "failed: {}: {error}", path.display())?;
            }
        }
    }
    if !options.quiet {
        writeln!(
            stderr,
            "alofmt: {unchanged} unchanged, {changed} changed, {failed} failed ({} total)",
            paths.len()
        )?;
    }
    stdout.flush()?;
    stderr.flush()?;
    Ok(exit_status(options.check, changed, failed))
}

fn run_stdin(options: &Options, format_options: &alofmt::FormatOptions) -> Result<ExitCode> {
    let mut source = Vec::new();
    std::io::stdin()
        .read_to_end(&mut source)
        .context("read standard input")?;
    let formatted = alofmt::format_with_options(&source, format_options)?;
    let changed = formatted.as_bytes() != source;
    let mut stdout = std::io::BufWriter::new(std::io::stdout().lock());
    let mut stderr = std::io::BufWriter::new(std::io::stderr().lock());
    if options.check {
        if changed && !options.quiet {
            writeln!(stdout, "changed: stdin")?;
        }
        if changed && options.diff {
            stdout.write_all(unified_diff(&source, &formatted, Path::new("stdin")).as_bytes())?;
        }
        if !options.quiet {
            let unchanged = usize::from(!changed);
            writeln!(
                stderr,
                "alofmt: {unchanged} unchanged, {} changed, 0 failed (1 total)",
                usize::from(changed)
            )?;
        }
    } else {
        stdout.write_all(formatted.as_bytes())?;
    }
    stdout.flush()?;
    stderr.flush()?;
    Ok(if options.check && changed {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    })
}

fn print_one(paths: &[PathBuf], options: &alofmt::FormatOptions) -> Result<ExitCode> {
    let [path] = paths else {
        bail!(
            "printing requires exactly one Ruby file; use --write or --check for {} files",
            paths.len()
        );
    };
    let source = std::fs::read(path).with_context(|| format!("read {}", path.display()))?;
    let formatted =
        alofmt::format_with_options(&source, options).with_context(|| format!("format {}", path.display()))?;
    let mut stdout = std::io::BufWriter::new(std::io::stdout().lock());
    stdout.write_all(formatted.as_bytes())?;
    stdout.flush()?;
    Ok(ExitCode::SUCCESS)
}

fn process_file(path: &Path, mode: Mode, options: &alofmt::FormatOptions) -> Outcome {
    let result = (|| -> Result<Outcome> {
        let source = std::fs::read(path).with_context(|| format!("read: {}", path.display()))?;
        if matches!(mode, Mode::Check { diff: false }) {
            return Ok(if alofmt::is_formatted_with_options(&source, options)? {
                Outcome::Unchanged
            } else {
                Outcome::Changed { diff: None }
            });
        }
        let formatted = alofmt::format_with_options(&source, options)?;
        if formatted.as_bytes() == source {
            return Ok(Outcome::Unchanged);
        }
        match mode {
            Mode::Check { diff } => Ok(Outcome::Changed {
                diff: diff.then(|| unified_diff(&source, &formatted, path)),
            }),
            Mode::Write => {
                files::replace(path, &source, formatted.as_bytes())?;
                Ok(Outcome::Changed { diff: None })
            }
        }
    })();
    result.unwrap_or_else(|error| Outcome::Failed(format!("{error:#}")))
}

fn unified_diff(before: &[u8], after: &str, path: &Path) -> String {
    let before = std::str::from_utf8(before).expect("formatting validates UTF-8");
    similar::TextDiff::from_lines(before, after)
        .unified_diff()
        .context_radius(2)
        .header(&format!("a/{}", path.display()), &format!("b/{}", path.display()))
        .to_string()
}

fn exit_status(check: bool, changed: usize, failed: usize) -> ExitCode {
    if failed > 0 {
        ExitCode::from(2)
    } else if check && changed > 0 {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn style_flags_override_defaults() {
        let style = StyleOptions {
            line_width: Some(100),
            quote_style: Some(QuoteStyle::Preserve),
            trailing_commas: Some(false),
            no_ignore_directives: true,
            ..StyleOptions::default()
        };

        let mut options = alofmt::FormatOptions::default();
        style.apply(&mut options).expect("valid options");
        assert_eq!(options.line_width, 100);
        assert_eq!(options.quote_style, alofmt::QuoteStyle::Preserve);
        assert!(!options.trailing_commas);
        assert!(options.ignore_directives.is_empty());
    }

    #[test]
    fn exit_status_prioritises_failures() {
        assert_eq!(exit_status(true, 1, 1), ExitCode::from(2));
        assert_eq!(exit_status(true, 1, 0), ExitCode::from(1));
        assert_eq!(exit_status(true, 0, 0), ExitCode::SUCCESS);
        assert_eq!(exit_status(false, 1, 0), ExitCode::SUCCESS);
    }
}
