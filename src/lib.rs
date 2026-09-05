//! A fast, deterministic Ruby formatter built on Prism.

mod comments;
mod doc;
mod format;
mod options;

pub use format::{format, format_with_options, is_formatted, is_formatted_with_options};
pub use options::{DelimitedArgumentAlignment, FormatOptions, MultilineAssignmentLayout, PercentArrays, QuoteStyle};
