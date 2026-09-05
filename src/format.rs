//! Turns a Prism tree into a document. This module owns the formatter state,
//! statement lists, and comment placement; each family module owns the
//! layouts for its node kinds.

mod assign;
mod calls;
mod control;
mod defs;
mod dispatch;
mod literals;
mod patterns;
mod strings;

use anyhow::{Context, bail};
use ruby_prism::{Location, Node, ProgramNode, StatementsNode};

use crate::FormatOptions;
use crate::comments::{Comment, CommentMap, Key};
use crate::doc::{self, Builder, COMMENT_PRIORITY, HARD, PrintOptions, RETURN};

const BOM: &[u8] = b"\xEF\xBB\xBF";

pub fn format(source: &[u8]) -> anyhow::Result<String> {
    format_with_options(source, &FormatOptions::default())
}

pub fn format_with_options(source: &[u8], options: &FormatOptions) -> anyhow::Result<String> {
    let document = document(source, options)?;
    let output = doc::print(
        &document.document,
        document.source,
        print_options(options, document.source.len()),
    );
    if document.has_bom {
        let mut with_bom = String::with_capacity(BOM.len().saturating_add(output.len()));
        with_bom.push('\u{feff}');
        with_bom.push_str(&output);
        Ok(with_bom)
    } else {
        Ok(output)
    }
}

/// Returns whether `source` is already formatted using the default policy,
/// without allocating the rendered output.
pub fn is_formatted(source: &[u8]) -> anyhow::Result<bool> {
    is_formatted_with_options(source, &FormatOptions::default())
}

/// Returns whether `source` is already formatted using `options`, without
/// allocating the rendered output.
pub fn is_formatted_with_options(source: &[u8], options: &FormatOptions) -> anyhow::Result<bool> {
    let document = document(source, options)?;
    Ok(doc::matches_source(
        &document.document,
        document.source,
        print_options(options, 0),
    ))
}

struct Document<'a> {
    source: &'a str,
    document: doc::Document,
    has_bom: bool,
}

fn document<'a>(source: &'a [u8], options: &FormatOptions) -> anyhow::Result<Document<'a>> {
    options.validate()?;

    // An empty source has no node capable of owning the final newline.
    if source.is_empty() {
        return Ok(Document {
            source: "",
            document: Builder::new().finish(),
            has_bom: false,
        });
    }
    let (has_bom, source) = match source.strip_prefix(BOM) {
        Some(source) => (true, source),
        None => (false, source),
    };
    if u32::try_from(source.len()).is_err() {
        bail!("source exceeds the 4 GiB document-offset limit");
    }
    let source_text = std::str::from_utf8(source).context("source is not valid UTF-8")?;
    let result = ruby_prism::parse(source);
    if let Some(error) = result.errors().next() {
        let line = line_of(source, error.location().start_offset());
        bail!("parse error at line {line}: {}", error.message());
    }
    let document = {
        let root = result.node();
        let comments = crate::comments::attach(source, &result);
        let line_starts: Vec<usize> = std::iter::once(0)
            .chain(memchr::memchr_iter(b'\n', source).map(|offset| offset + 1))
            .collect();
        let control = control::State::analyze(&root);
        // Layout nodes scale with both syntax nodes and line-level separators.
        // Reserving once avoids moving large arenas while they grow.
        let document_capacity = control.node_count().saturating_add(line_starts.len()).saturating_mul(3);
        let mut formatter = Formatter {
            source,
            options,
            line_starts,
            b: Builder::with_capacity(document_capacity),
            comments,
            unsupported: Vec::new(),
            last_end: None,
            header_break: false,
            calls: calls::State::default(),
            control,
        };
        formatter.node(&root);
        match result.data_loc() {
            Some(data) => formatter.data(&data),
            None => formatter.b.line(HARD),
        }
        if !formatter.unsupported.is_empty() {
            formatter.unsupported.sort();
            formatter.unsupported.dedup();
            bail!("unsupported nodes: {}", formatter.unsupported.join(", "));
        }
        formatter.b.finish()
    };
    drop(result);
    Ok(Document {
        source: source_text,
        document,
        has_bom,
    })
}

fn print_options(options: &FormatOptions, output_capacity: usize) -> PrintOptions {
    PrintOptions {
        line_width: options.line_width,
        indent_width: options.indent_width,
        fit_indent_width: options.fit_indent_width,
        output_capacity: output_capacity.saturating_add(1),
    }
}

pub struct Formatter<'a> {
    pub source: &'a [u8],
    pub options: &'a FormatOptions,
    line_starts: Vec<usize>,
    pub b: Builder,
    comments: CommentMap,
    unsupported: Vec<String>,
    /// End of the last top-level entry, for the blank line before `__END__`.
    last_end: Option<usize>,
    /// Set while a node with header comments has not opened a group yet: the
    /// first group it opens breaks, so its children leave the header line.
    header_break: bool,
    /// Layout context threaded through nested calls.
    pub calls: calls::State,
    /// Analysis and layout context used by control-flow nodes.
    pub control: control::State,
}

impl<'a> Formatter<'a> {
    /// Formats a node together with its attached comments. A configured
    /// ignore directive on the final leading comment copies the node verbatim.
    pub fn node(&mut self, node: &Node<'_>) {
        self.with_comments(node, |f| f.dispatch(node));
    }

    /// Prints `node`'s comments around `layout`, for families that lay a
    /// child out by hand instead of calling [`Formatter::node`]. The layout
    /// is skipped when a configured ignore directive copies the node verbatim.
    pub fn with_comments(&mut self, node: &Node<'_>, layout: impl FnOnce(&mut Self)) {
        // A child printed before its parent opens a group takes the header
        // break with it; the parent then keeps the comment on its line only.
        self.header_break = false;
        let key = Key::of(node);
        let Some((leading_len, header_len, trailing_len)) = self.comments.get_key(&key).map(|attached| {
            (
                attached.leading().len(),
                attached.header().len(),
                attached.trailing().len(),
            )
        }) else {
            layout(self);
            self.header_break = false;
            return;
        };
        let mut last_leading = None;
        for index in 0..leading_len {
            let comment = self.attached_comment_key(&key, AttachedSlot::Leading, index);
            self.comment(&comment);
            self.b.line(HARD);
            last_leading = Some(comment);
        }
        if header_len > 0 {
            self.trailing_attached_count(&key, AttachedSlot::Header, header_len);
            self.header_break = true;
        }
        if last_leading.is_some_and(|comment| self.is_ignore(&comment)) {
            self.verbatim(&node.location());
        } else {
            layout(self);
        }
        self.header_break = false;
        self.trailing_attached_count(&key, AttachedSlot::Trailing, trailing_len);
    }

    pub(super) fn attached_len(&self, node: &Node<'_>, slot: AttachedSlot) -> usize {
        self.comments.get(node).map_or(0, |attached| match slot {
            AttachedSlot::Leading => attached.leading().len(),
            AttachedSlot::Header => attached.header().len(),
            AttachedSlot::Trailing => attached.trailing().len(),
        })
    }

    pub(super) fn attached_comment(&self, node: &Node<'_>, slot: AttachedSlot, index: usize) -> Comment {
        self.attached_comment_key(&Key::of(node), slot, index)
    }

    fn attached_comment_key(&self, key: &Key, slot: AttachedSlot, index: usize) -> Comment {
        let attached = self.comments.get_key(key).expect("attached comment exists");
        match slot {
            AttachedSlot::Leading => attached.leading()[index],
            AttachedSlot::Header => attached.header()[index],
            AttachedSlot::Trailing => attached.trailing()[index],
        }
    }

    pub(super) fn trailing_attached(&mut self, node: &Node<'_>, slot: AttachedSlot) {
        let key = Key::of(node);
        let len = self.comments.get_key(&key).map_or(0, |attached| match slot {
            AttachedSlot::Leading => attached.leading().len(),
            AttachedSlot::Header => attached.header().len(),
            AttachedSlot::Trailing => attached.trailing().len(),
        });
        self.trailing_attached_count(&key, slot, len);
    }

    fn trailing_attached_count(&mut self, key: &Key, slot: AttachedSlot, len: usize) {
        for index in 0..len {
            let comment = self.attached_comment_key(key, slot, index);
            self.trailing_comments(std::slice::from_ref(&comment));
        }
    }

    /// Queues comments for the end of the current line; an own-line comment
    /// goes on a line of its own after it, at the current indentation.
    fn trailing_comments(&mut self, comments: &[Comment]) {
        for comment in comments {
            self.line_suffix(COMMENT_PRIORITY, |f| {
                if comment.own_line {
                    f.b.line(HARD);
                } else {
                    f.b.text(" ");
                }
                f.comment(comment);
                f.b.break_parent();
            });
        }
    }

    /// Prints a comment at the current position. Embdocs start at column
    /// zero, whatever the indentation.
    pub fn comment(&mut self, comment: &Comment) {
        if comment.embdoc {
            self.b.trim();
            self.source_lines(comment.start, comment.end, |f| f.b.line(RETURN));
        } else {
            self.b.source(comment.start, comment.end);
        }
    }

    /// A source range spanning several lines, printed as-is. Continuation
    /// lines keep their source indentation instead of the printer's.
    pub fn source_lines(&mut self, start: usize, end: usize, mut newline: impl FnMut(&mut Self)) {
        let mut cursor = start;
        while let Some(relative) = memchr::memchr(b'\n', &self.source[cursor..end]) {
            let newline_offset = cursor + relative;
            self.b.source(cursor, newline_offset);
            newline(self);
            cursor = newline_offset + 1;
        }
        self.b.source(cursor, end);
    }

    pub fn is_ignore(&self, comment: &Comment) -> bool {
        if comment.embdoc {
            return false;
        }
        let body = self.source[comment.start + 1..comment.end].trim_ascii();
        self.options
            .ignore_directives
            .iter()
            .any(|directive| body == directive.as_bytes())
    }

    /// Copies a node's source text unchanged.
    pub fn verbatim(&mut self, loc: &Location<'_>) {
        self.source_lines(loc.start_offset(), loc.end_offset(), |f| f.b.line(RETURN));
    }

    pub fn unsupported(&mut self, kind: &str, loc: &Location<'_>) {
        self.unsupported.push(kind.to_string());
        self.b.source(loc.start_offset(), loc.end_offset());
    }

    /// Source text under a location. The entire input is validated before
    /// formatting, so every Prism location is valid UTF-8.
    pub fn slice(&self, loc: &Location<'_>) -> &'a str {
        std::str::from_utf8(&self.source[loc.start_offset()..loc.end_offset()])
            .expect("the source was validated as UTF-8")
    }

    pub fn text_of(&mut self, loc: &Location<'_>) {
        self.b.source(loc.start_offset(), loc.end_offset());
    }

    /// One-based line number of a byte offset.
    pub fn line_of(&self, offset: usize) -> usize {
        self.line_starts.partition_point(|&start| start <= offset)
    }

    pub fn contains_heredoc(&self, node: &Node<'_>) -> bool {
        let loc = node.location();
        let openings = self.comments.heredoc_openings();
        let first = openings.partition_point(|&start| start < loc.start_offset());
        openings.get(first).is_some_and(|&start| start < loc.end_offset())
    }

    /// Whether `node` has comments on its header line (`x = # c`): a layout
    /// that would otherwise keep the first child on that line must move it
    /// down, since the comment ends the line.
    pub fn has_header_comments(&self, node: &Node<'_>) -> bool {
        self.comments
            .get(node)
            .is_some_and(|attached| !attached.header().is_empty())
    }

    /// Whether `node` has trailing comments, which end the line it closes on.
    pub fn has_trailing_comments(&self, node: &Node<'_>) -> bool {
        self.comments
            .get(node)
            .is_some_and(|attached| !attached.trailing().is_empty())
    }

    pub fn comma_separated<'pr>(&mut self, nodes: impl Iterator<Item = Node<'pr>>) {
        for (index, node) in nodes.enumerate() {
            if index > 0 {
                self.b.text(",");
                self.b.line(crate::doc::SPACE);
            }
            self.node(&node);
        }
    }

    /// The top-level statements, without the newline that ends the file.
    fn program(&mut self, node: &ProgramNode<'_>) {
        let statements = node.statements();
        self.last_end = self.statements(&statements);
    }

    /// `__END__` and everything after it, byte for byte; the source's own
    /// final newline (or lack of one) is kept.
    fn data(&mut self, loc: &Location<'_>) {
        if let Some(end) = self.last_end {
            self.b.line(HARD);
            if self.line_of(loc.start_offset()) - self.line_of(end) > 1 {
                self.b.line(HARD);
            }
        }
        let start = loc.start_offset();
        let end = loc.end_offset();
        const DATA_MARKER: &[u8] = b"__END__";
        assert!(
            self.source[start..end].starts_with(DATA_MARKER),
            "data section starts with __END__"
        );
        self.b.source(start, start + DATA_MARKER.len());
        if start + DATA_MARKER.len() == end {
            self.b.line(HARD);
        } else {
            self.source_lines(start + DATA_MARKER.len(), end, |f| f.b.line(RETURN));
        }
    }

    /// Statements separated by newlines, keeping at most one blank line
    /// between neighbours where the source had any. Own-line comments are
    /// interleaved as entries of their own, except configured ignore
    /// directives, which join the entry after them and copy it from source.
    /// Returns where the last entry ends.
    pub fn statements(&mut self, node: &StatementsNode<'_>) -> Option<usize> {
        let nodes: Vec<_> = node.body().iter().collect();
        let entries = self.prepare_entries(&nodes, self.comments.body(&node.as_node()));
        self.print_entries(nodes, entries)
    }

    /// Indented statements on their own lines, or nothing for an empty body.
    pub fn body(&mut self, statements: Option<StatementsNode<'_>>) {
        if let Some(statements) = statements {
            self.indent(|f| {
                f.b.line(HARD);
                f.statements(&statements);
            });
        }
    }

    /// Like [`Formatter::body`], but an empty body still prints the own-line
    /// comments dangling on `parent` (those before `end`, `else`, `rescue`,
    /// ...): `def foo\n  # todo\nend`.
    pub fn body_of(&mut self, statements: Option<StatementsNode<'_>>, parent: &Node<'_>) {
        match statements {
            Some(statements) => self.body(Some(statements)),
            None => {
                if self.has_dangling(parent) {
                    let nodes = Vec::new();
                    let entries = self.prepare_entries(&nodes, self.comments.dangling(parent));
                    self.indent(|f| {
                        f.b.line(HARD);
                        f.print_entries(nodes, entries);
                    });
                }
            }
        }
    }

    /// Own-line comments dangling on a node that no child claims. Families
    /// with several bodies (`if`/`else`, `case`/`when`) print each clause's
    /// comments from the clause node; this is for the parent's own span.
    /// Own-line comments interleaved in a statement list.
    pub fn dangling_body(&self, statements: &StatementsNode<'_>) -> Vec<Comment> {
        self.comments.body(&statements.as_node()).to_vec()
    }

    pub fn dangling(&self, node: &Node<'_>) -> Vec<Comment> {
        self.comments.dangling(node).to_vec()
    }

    pub(super) fn dangling_len(&self, node: &Node<'_>) -> usize {
        self.comments.dangling(node).len()
    }

    pub(super) fn dangling_comment(&self, node: &Node<'_>, index: usize) -> Comment {
        self.comments.dangling(node)[index]
    }

    pub fn has_dangling_body(&self, statements: &StatementsNode<'_>) -> bool {
        !self.comments.body(&statements.as_node()).is_empty()
    }

    pub fn has_dangling(&self, node: &Node<'_>) -> bool {
        !self.comments.dangling(node).is_empty()
    }

    fn prepare_entries(&self, nodes: &[Node<'_>], comments: &[Comment]) -> Vec<Entry> {
        let mut entries: Vec<Entry> = (0..nodes.len())
            .map(|index| Entry::Node {
                index,
                ignores: Vec::new(),
            })
            .collect();
        entries.extend(comments.iter().map(|c| Entry::Comment(*c)));
        entries.sort_by_key(|e| e.start(nodes));

        let mut folded: Vec<Entry> = Vec::with_capacity(entries.len());
        let mut pending: Vec<Comment> = Vec::new();
        for entry in entries {
            match entry {
                Entry::Comment(c) if self.is_ignore(&c) => pending.push(c),
                Entry::Comment(_) => {
                    pending.clear();
                    folded.push(entry);
                }
                Entry::Node { index, .. } => folded.push(Entry::Node {
                    index,
                    ignores: std::mem::take(&mut pending),
                }),
            }
        }
        folded.extend(pending.into_iter().map(Entry::Comment));
        folded
    }

    /// Prints nodes and own-line comments in source order.
    fn print_entries(&mut self, nodes: Vec<Node<'_>>, entries: Vec<Entry>) -> Option<usize> {
        let mut previous_end: Option<usize> = None;
        for entry in &entries {
            if let Some(end) = previous_end {
                self.b.line(HARD);
                if self.line_of(entry.start(&nodes)) - self.line_of(end) > 1 {
                    self.b.line(HARD);
                }
            }
            match entry {
                Entry::Node { index, ignores } => {
                    let node = &nodes[*index];
                    if ignores.is_empty() {
                        self.node(node);
                    } else {
                        for comment in ignores {
                            self.comment(comment);
                            self.b.line(HARD);
                        }
                        self.verbatim(&node.location());
                        self.trailing_attached(node, AttachedSlot::Trailing);
                    }
                }
                Entry::Comment(comment) => self.comment(comment),
            }
            let mut end = entry.end(&nodes);
            if let Entry::Node { index, .. } = &entry {
                // A heredoc's node ends at its opener; the gap to the next
                // statement starts after its body.
                let node = &nodes[*index];
                if self.contains_heredoc(node)
                    && let Some(body_end) = strings::heredoc_end_offset(self.source, node)
                {
                    end = end.max(body_end);
                }
            }
            previous_end = Some(end);
        }
        previous_end
    }

    pub fn group(&mut self, f: impl FnOnce(&mut Self)) {
        self.b.open_group();
        if std::mem::take(&mut self.header_break) {
            self.b.break_parent();
        }
        f(self);
        self.b.close_group();
    }

    pub fn indent(&mut self, f: impl FnOnce(&mut Self)) {
        self.b.push_target();
        f(self);
        self.b.close_indent();
    }

    pub fn align(&mut self, units: usize, f: impl FnOnce(&mut Self)) {
        self.b.push_target();
        f(self);
        self.b.close_align(units);
    }

    pub fn if_break(&mut self, broken: impl FnOnce(&mut Self), flat: impl FnOnce(&mut Self)) {
        self.b.push_target();
        broken(self);
        let broken = self.b.pop_target();
        let flat = if self.b.current_group_broken() {
            let flags = self.b.broken_flags();
            self.b.push_target();
            flat(self);
            self.b.pop_target();
            self.b.restore_broken_flags(flags);
            doc::Fragment::EMPTY
        } else {
            self.b.push_target();
            flat(self);
            self.b.pop_target()
        };
        self.b.close_if_break(broken, flat);
    }

    pub fn line_suffix(&mut self, priority: u8, f: impl FnOnce(&mut Self)) {
        self.b.push_target();
        f(self);
        self.b.close_line_suffix(priority);
    }
}

enum Entry {
    Node { index: usize, ignores: Vec<Comment> },
    Comment(Comment),
}

#[derive(Clone, Copy)]
pub(super) enum AttachedSlot {
    Leading,
    Header,
    Trailing,
}

impl Entry {
    fn start(&self, nodes: &[Node<'_>]) -> usize {
        match self {
            Entry::Node { index, .. } => nodes[*index].location().start_offset(),
            Entry::Comment(c) => c.start,
        }
    }

    fn end(&self, nodes: &[Node<'_>]) -> usize {
        match self {
            Entry::Node { index, .. } => nodes[*index].location().end_offset(),
            Entry::Comment(c) => c.end,
        }
    }
}

pub fn line_of(source: &[u8], offset: usize) -> usize {
    source[..offset.min(source.len())]
        .iter()
        .filter(|&&b| b == b'\n')
        .count()
        + 1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_non_utf8_input() {
        let error = format(b"x = \xff\n").expect_err("invalid UTF-8 should fail");

        assert_eq!(error.to_string(), "source is not valid UTF-8");
    }

    #[test]
    fn preserves_a_byte_order_mark() {
        assert_eq!(format(b"\xef\xbb\xbfx=1\n").expect("valid Ruby"), "\u{feff}x = 1\n");
        assert_eq!(format(BOM).expect("bare BOM"), "\u{feff}\n");
    }

    #[test]
    fn checks_formatting_without_rendering_an_output_string() {
        assert!(is_formatted(b"x = 1\n").expect("valid Ruby"));
        assert!(is_formatted(b"\xef\xbb\xbfx = 1\n").expect("valid Ruby with BOM"));
        assert!(!is_formatted(b"x=1\n").expect("valid Ruby"));
        assert!(!is_formatted(BOM).expect("bare BOM"));
    }

    #[test]
    fn supports_custom_width_and_indentation() {
        let options = FormatOptions {
            line_width: 8,
            indent_width: 2,
            fit_indent_width: 2,
            ..FormatOptions::default()
        };

        assert_eq!(
            format_with_options(b"x = [1, 2]\n", &options).expect("valid Ruby"),
            "x = [\n  1,\n  2\n]\n"
        );
    }

    #[test]
    fn consistent_delimited_arguments_break_at_the_line_indent() {
        let options = FormatOptions {
            line_width: 30,
            indent_width: 4,
            fit_indent_width: 4,
            delimited_argument_alignment: crate::DelimitedArgumentAlignment::Consistent,
            ..FormatOptions::default()
        };

        assert_eq!(
            format_with_options(b"run App.new(alpha: service.alpha, beta: service.beta)\n", &options)
                .expect("valid Ruby"),
            "run App.new(\n    alpha: service.alpha,\n    beta: service.beta\n)\n"
        );
        // Bare argument lists keep their aligned continuations.
        assert_eq!(
            format_with_options(b"raise ArgumentError, 'a message that will not fit'\n", &options).expect("valid Ruby"),
            "raise ArgumentError,\n      'a message that will not fit'\n"
        );
    }

    #[test]
    fn same_line_assignments_keep_a_breakable_value_on_the_assignment_line() {
        let options = FormatOptions {
            line_width: 30,
            indent_width: 4,
            fit_indent_width: 4,
            multiline_assignment_layout: crate::MultilineAssignmentLayout::SameLine,
            ..FormatOptions::default()
        };

        assert_eq!(
            format_with_options(b"result = Open3.capture2('git', 'diff', '--name-only')\n", &options)
                .expect("valid Ruby"),
            "result = Open3.capture2(\n    'git',\n    'diff',\n    '--name-only'\n)\n"
        );
        assert_eq!(
            format_with_options(
                b"out, status = Open3.capture2('git', 'diff', '--name-only')\n",
                &options
            )
            .expect("valid Ruby"),
            "out, status = Open3.capture2(\n    'git',\n    'diff',\n    '--name-only'\n)\n"
        );
        // A value with no internal break still moves to its own line.
        assert_eq!(
            format_with_options(b"a_variable = another_quite_long_variable\n", &options).expect("valid Ruby"),
            "a_variable =\n    another_quite_long_variable\n"
        );
    }

    #[test]
    fn supports_custom_ignore_directives() {
        let options = FormatOptions {
            ignore_directives: vec!["keep".to_owned()],
            ..FormatOptions::default()
        };

        assert_eq!(
            format_with_options(b"# keep\nx=  1\n", &options).expect("valid Ruby"),
            "# keep\nx=  1\n"
        );
    }

    #[test]
    fn supports_quote_policies() {
        let preserve = FormatOptions {
            quote_style: crate::QuoteStyle::Preserve,
            ..FormatOptions::default()
        };
        let double = FormatOptions {
            quote_style: crate::QuoteStyle::Double,
            ..FormatOptions::default()
        };

        assert_eq!(
            format_with_options(b"x=\"hello\"\n", &preserve).expect("valid Ruby"),
            "x = \"hello\"\n"
        );
        assert_eq!(
            format_with_options(b"x='hello'\n", &double).expect("valid Ruby"),
            "x = \"hello\"\n"
        );
        assert_eq!(
            format_with_options(b"x='#@name'\n", &double).expect("valid Ruby"),
            "x = '#@name'\n"
        );
    }

    #[test]
    fn comments_between_a_pairs_key_and_value_move_the_value_down() {
        let source = b"foo(\n  bounds: # why\n  # more\n  Bar.new(a: 1),\n  other: # only here\n  { a: 1 },\n  plain: Baz.new,\n)\n";
        assert_eq!(
            format(source).expect("valid Ruby"),
            "foo(\n  bounds: # why\n    # more\n    Bar.new(a: 1),\n  other: # only here\n    { a: 1 },\n  plain: Baz.new\n)\n"
        );
    }

    #[test]
    fn avoiding_percent_arrays_spells_them_out_where_the_words_allow() {
        let avoid = FormatOptions {
            percent_arrays: crate::options::PercentArrays::Avoid,
            ..FormatOptions::default()
        };
        let source = b"w = %w[c d]\ns = %i[a b?]\nq = %i[foo-bar]\na = %w[it's]\ne = %w[]\n";
        assert_eq!(
            format_with_options(source, &avoid).expect("valid Ruby"),
            "w = ['c', 'd']\ns = [:a, :b?]\nq = [:'foo-bar']\na = [\"it's\"]\ne = []\n"
        );
        // Interpolation and escapes keep the literal as written.
        let kept = b"i = %W[#{x} y]\nb = %w[a\\ b]\n";
        assert_eq!(
            format_with_options(kept, &avoid).expect("valid Ruby"),
            "i = %W[#{x} y]\nb = %w[a\\ b]\n"
        );
        let double = FormatOptions {
            quote_style: crate::options::QuoteStyle::Double,
            ..avoid
        };
        assert_eq!(
            format_with_options(b"w = %w[c d]\n", &double).expect("valid Ruby"),
            "w = [\"c\", \"d\"]\n"
        );
    }

    #[test]
    fn percent_arrays_collapse_only_when_preferred_and_never_expand() {
        let source = b"w = ['a', 'b']\ns = [:a, :b]\nk = %w[c d]\n";
        let preserve = FormatOptions {
            percent_arrays: crate::options::PercentArrays::Preserve,
            ..FormatOptions::default()
        };
        assert_eq!(
            format_with_options(source, &preserve).expect("valid Ruby"),
            "w = ['a', 'b']\ns = [:a, :b]\nk = %w[c d]\n"
        );
        let prefer = FormatOptions {
            percent_arrays: crate::options::PercentArrays::Prefer,
            ..FormatOptions::default()
        };
        assert_eq!(
            format_with_options(source, &prefer).expect("valid Ruby"),
            "w = %w[a b]\ns = %i[a b]\nk = %w[c d]\n"
        );
    }

    #[test]
    fn supports_collection_and_literal_policies() {
        let options = FormatOptions {
            line_width: 8,
            trailing_commas: false,
            percent_arrays: crate::options::PercentArrays::Preserve,
            normalize_number_separators: false,
            ..FormatOptions::default()
        };

        assert_eq!(
            format_with_options(b"x=['a','b']\nn=10000\n", &options).expect("valid Ruby"),
            "x = [\n  'a',\n  'b'\n]\nn =\n  10000\n"
        );
    }

    #[test]
    fn supports_implicit_rescue_policy() {
        let options = FormatOptions {
            explicit_standard_error: false,
            ..FormatOptions::default()
        };

        assert_eq!(
            format_with_options(b"begin\nfoo\nrescue\nbar\nend\n", &options).expect("valid Ruby"),
            "begin\n  foo\nrescue\n  bar\nend\n"
        );
    }
}
