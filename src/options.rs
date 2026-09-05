use anyhow::{Context, Result, ensure};
use serde::Deserialize;

/// Formatting policy used by [`crate::format_with_options`].
///
/// Defaults are project-neutral and avoid method-name special cases or
/// syntax rewrites that are better selected by repository configuration.
#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
pub struct FormatOptions {
    /// Maximum width used when deciding whether a document group fits.
    pub line_width: usize,
    /// Spaces emitted for one indentation level.
    pub indent_width: usize,
    /// Width charged to one indentation level while measuring a line.
    pub fit_indent_width: usize,
    /// Preferred delimiter for plain strings and quoted symbols.
    pub quote_style: QuoteStyle,
    /// Add a trailing comma when a collection or argument list breaks.
    pub trailing_commas: bool,
    /// Whether an array of single-word strings or symbols collapses to a
    /// `%w` or `%i` literal.
    pub percent_arrays: PercentArrays,
    /// Add thousands separators to eligible decimal integer literals.
    pub normalize_number_separators: bool,
    /// Spell an omitted rescue class as `StandardError`.
    pub explicit_standard_error: bool,
    /// Comment bodies that make the following node print verbatim.
    pub ignore_directives: Vec<String>,
    /// Number of chained calls that makes a chain break by default.
    pub chain_break_threshold: usize,
    /// Chain threshold inside blocks named by [`Self::compact_chain_blocks`].
    pub compact_chain_break_threshold: usize,
    /// Block-call names that use [`Self::compact_chain_break_threshold`].
    pub compact_chain_blocks: Vec<String>,
    /// Command-call names whose continuations do not align under the first
    /// argument.
    pub unaligned_command_calls: Vec<String>,
    /// Do not align a command continuation when its prefix exceeds this many
    /// columns. Zero disables prefix alignment.
    pub max_command_alignment: usize,
    /// Where the contents of a command call's sole bracketed argument sit
    /// when the call has to break.
    pub delimited_argument_alignment: DelimitedArgumentAlignment,
    /// Where the value of an assignment that cannot fit on one line goes.
    pub multiline_assignment_layout: MultilineAssignmentLayout,
    /// Which delimiters a block prints with, where either would parse the same.
    pub block_delimiters: BlockDelimiters,
}

impl Default for FormatOptions {
    fn default() -> Self {
        Self {
            line_width: 80,
            indent_width: 2,
            fit_indent_width: 2,
            quote_style: QuoteStyle::Preserve,
            trailing_commas: false,
            percent_arrays: PercentArrays::Preserve,
            normalize_number_separators: false,
            explicit_standard_error: false,
            ignore_directives: vec!["alofmt-ignore".to_owned()],
            chain_break_threshold: 3,
            compact_chain_break_threshold: 3,
            compact_chain_blocks: Vec::new(),
            unaligned_command_calls: Vec::new(),
            max_command_alignment: 40,
            delimited_argument_alignment: DelimitedArgumentAlignment::Aligned,
            multiline_assignment_layout: MultilineAssignmentLayout::NewLine,
            block_delimiters: BlockDelimiters::LineCountBased,
        }
    }
}

/// Which delimiters a block prints with. A block whose delimiters change what
/// it binds to, inside a paren-less command call or a conditional's predicate,
/// keeps the delimiters that preserve its meaning whatever the setting.
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BlockDelimiters {
    /// `{ }` when the block fits on one line, `do`/`end` when it breaks.
    #[default]
    LineCountBased,
    /// `{ }` whether the block fits or breaks.
    AlwaysBraces,
    /// `do`/`end` on its own lines, even for a block that would fit.
    AlwaysDoEnd,
    /// The source's delimiters, flat or broken as the block fits.
    Preserve,
}

impl FormatOptions {
    /// Parse a strict TOML configuration, filling omitted fields from the
    /// project-neutral defaults.
    pub fn from_toml(source: &str) -> Result<Self> {
        let options: Self = toml::from_str(source).context("parse formatter configuration")?;
        options.validate()?;
        Ok(options)
    }

    /// Validate options assembled programmatically.
    pub fn validate(&self) -> Result<()> {
        ensure!(self.line_width > 0, "line width must be greater than zero");
        ensure!(self.indent_width > 0, "indent width must be greater than zero");
        ensure!(self.fit_indent_width > 0, "fit indent width must be greater than zero");
        ensure!(
            self.chain_break_threshold > 0,
            "chain break threshold must be greater than zero"
        );
        ensure!(
            self.compact_chain_break_threshold > 0,
            "compact chain break threshold must be greater than zero"
        );
        for directive in &self.ignore_directives {
            ensure!(!directive.is_empty(), "ignore directives cannot be empty");
            ensure!(
                !directive.bytes().any(|byte| byte == b'\n' || byte == b'\r'),
                "ignore directives cannot contain line breaks"
            );
        }
        Ok(())
    }
}

/// Where the contents of a command call's sole bracketed argument sit when
/// the call has to break.
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DelimitedArgumentAlignment {
    /// Under the argument's own column, closing bracket alongside.
    #[default]
    Aligned,
    /// One level in from the start of the line, closing bracket back at the
    /// line's indent. RuboCop calls this first-argument style `consistent`;
    /// rustfmt calls the same layout `overflow_delimited_expr`.
    Consistent,
}

/// Where the value of an assignment that cannot fit on one line goes. The
/// names are RuboCop's `Layout/MultilineAssignmentLayout` styles.
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MultilineAssignmentLayout {
    /// The whole value moves to an indented line, unless it opens a bracket.
    #[default]
    NewLine,
    /// A value that can break — a call with parentheses, a block, or a dot
    /// chain — keeps its head on the assignment line and breaks below it. A
    /// value with nothing to break still moves down whole.
    SameLine,
}

/// How an array of single-word strings or symbols is spelled.
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum PercentArrays {
    /// Collapse an eligible bracketed array to its `%w` or `%i` literal.
    Prefer,
    /// Keep every array as written.
    #[default]
    Preserve,
    /// Rewrite a `%w` or `%i` literal as a bracketed array. An interpolating
    /// `%W` or `%I`, and a literal with a word neither string delimiter can
    /// hold verbatim, stay as written.
    Avoid,
}

/// Preferred delimiter for plain strings and quoted symbols.
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum QuoteStyle {
    /// Prefer single quotes where changing delimiters is semantics-preserving.
    Single,
    /// Prefer double quotes where changing delimiters is semantics-preserving.
    Double,
    /// Keep the source delimiter.
    #[default]
    Preserve,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_do_not_encode_project_method_names_or_syntax_rewrites() {
        let options = FormatOptions::default();

        assert_eq!(options.indent_width, options.fit_indent_width);
        assert_eq!(options.quote_style, QuoteStyle::Preserve);
        assert!(!options.trailing_commas);
        assert_eq!(options.percent_arrays, PercentArrays::Preserve);
        assert!(!options.normalize_number_separators);
        assert!(!options.explicit_standard_error);
        assert_eq!(options.ignore_directives, ["alofmt-ignore"]);
        assert!(options.compact_chain_blocks.is_empty());
        assert!(options.unaligned_command_calls.is_empty());
        assert_eq!(
            options.delimited_argument_alignment,
            DelimitedArgumentAlignment::Aligned
        );
        assert_eq!(options.multiline_assignment_layout, MultilineAssignmentLayout::NewLine);
        assert_eq!(options.block_delimiters, BlockDelimiters::LineCountBased);
    }

    #[test]
    fn rejects_zero_widths() {
        let options = FormatOptions {
            line_width: 0,
            ..FormatOptions::default()
        };

        assert_eq!(
            options.validate().expect_err("zero width should fail").to_string(),
            "line width must be greater than zero"
        );
    }

    #[test]
    fn rejects_multiline_ignore_directives() {
        let options = FormatOptions {
            ignore_directives: vec!["alofmt-ignore\nnext".to_owned()],
            ..FormatOptions::default()
        };

        assert_eq!(
            options
                .validate()
                .expect_err("multiline directive should fail")
                .to_string(),
            "ignore directives cannot contain line breaks"
        );
    }

    #[test]
    fn parses_partial_configuration_over_defaults() {
        let options = FormatOptions::from_toml(
            r#"
                line_width = 100
                quote_style = "single"
                percent_arrays = "prefer"
                block_delimiters = "always_braces"
                compact_chain_blocks = ["typed"]
            "#,
        )
        .expect("valid configuration");

        assert_eq!(options.line_width, 100);
        assert_eq!(options.indent_width, 2);
        assert_eq!(options.quote_style, QuoteStyle::Single);
        assert_eq!(options.percent_arrays, PercentArrays::Prefer);
        assert_eq!(options.block_delimiters, BlockDelimiters::AlwaysBraces);
        assert_eq!(options.compact_chain_blocks, ["typed"]);
    }

    #[test]
    fn rejects_unknown_configuration_fields() {
        let error = FormatOptions::from_toml("line_wdith = 100").expect_err("misspelled field should fail");

        assert!(error.to_string().contains("parse formatter configuration"));
        assert!(format!("{error:#}").contains("unknown field `line_wdith`"));
    }
}
