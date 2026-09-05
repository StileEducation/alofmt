//! Strings, symbols, regular expressions, command strings, heredocs, and the
//! `#{}` interpolation they share.
//!
//! Delimiter selection obeys [`QuoteStyle`]:
//!
//! - Plain strings and quoted symbols use the requested quote only when the
//!   conversion preserves escapes and interpolation semantics. `Preserve`
//!   retains source delimiters.
//! - Interpolated strings keep their delimiters; `#@x` prints as `#{@x}`.
//! - Regexes prefer `/…/`; `%r{…}` is used when the content contains `/`,
//!   except that a `{` or `}` in the content keeps the original delimiters.
//! - Command strings always print with backticks.
//! - Heredoc bodies are verbatim and deferred to the end of the line.

use ruby_prism::{
    EmbeddedStatementsNode, EmbeddedVariableNode, InterpolatedMatchLastLineNode, InterpolatedRegularExpressionNode,
    InterpolatedStringNode, InterpolatedSymbolNode, InterpolatedXStringNode, Location, MatchLastLineNode, Node,
    NodeList, RegularExpressionNode, StringNode, SymbolNode, Visit, XStringNode,
};

use super::Formatter;
use crate::QuoteStyle;
use crate::comments::Comment;
use crate::doc::{self, HARD, HEREDOC_PRIORITY, RETURN};

pub fn string_node(f: &mut Formatter<'_>, node: &StringNode<'_>) {
    let Some(opening) = node.opening_loc() else {
        // A part of an interpolated string: content verbatim.
        let content = f.slice(&node.content_loc());
        literal_text(f, content);
        return;
    };
    if is_heredoc(f, &opening) {
        let closing = node.closing_loc().expect("heredoc has a closing");
        let body = node.content_loc();
        heredoc(f, &opening, &closing, |f| {
            f.source_lines(body.start_offset(), body.end_offset(), return_line)
        });
        return;
    }
    let content = f.slice(&node.content_loc());
    if f.slice(&opening) == "?" {
        match f.options.quote_style {
            QuoteStyle::Single => character_literal(f, content),
            QuoteStyle::Double if can_use_quote(content, '"') => double_quoted(f, content),
            QuoteStyle::Double | QuoteStyle::Preserve => original_string(f, node, &opening, content),
        }
        return;
    }
    match f.options.quote_style {
        QuoteStyle::Single if can_use_quote(content, '\'') => single_quoted(f, content),
        QuoteStyle::Double if can_use_quote(content, '"') => double_quoted(f, content),
        QuoteStyle::Single | QuoteStyle::Double | QuoteStyle::Preserve => original_string(f, node, &opening, content),
    }
}

fn original_string(f: &mut Formatter<'_>, node: &StringNode<'_>, opening: &Location<'_>, content: &str) {
    f.text_of(opening);
    literal_text(f, content);
    f.text_of(&node.closing_loc().expect("quoted string has a closing"));
}

fn can_use_quote(content: &str, quote: char) -> bool {
    !content.contains(['\\', quote])
        && (quote != '"' || (!content.contains("#{") && !content.contains("#@") && !content.contains("#$")))
}

/// String and symbol content. A literal newline restarts the column at
/// zero and, unlike one in a regex or command string, breaks every
/// enclosing group.
fn literal_text(f: &mut Formatter<'_>, text: &str) {
    for (i, line) in text.split('\n').enumerate() {
        if i > 0 {
            f.b.line(RETURN);
        }
        f.b.text_ref(line);
    }
}

fn character_literal(f: &mut Formatter<'_>, content: &str) {
    match content {
        "'" => f.b.text("'''"),
        "\"" => f.b.text("'\\\"'"),
        _ if content.starts_with('\\') => {
            f.b.text("?");
            f.b.text_ref(content);
        }
        _ => single_quoted(f, content),
    }
}

/// The delimiter a bare word from a `%w` literal takes as a plain string
/// under the quote style, or `None` when neither delimiter holds it verbatim.
pub fn word_quote(f: &Formatter<'_>, content: &str) -> Option<char> {
    let (first, second) = match f.options.quote_style {
        QuoteStyle::Double => ('"', '\''),
        QuoteStyle::Single | QuoteStyle::Preserve => ('\'', '"'),
    };
    [first, second].into_iter().find(|quote| can_use_quote(content, *quote))
}

pub fn quoted_word(f: &mut Formatter<'_>, quote: char, content: &str) {
    match quote {
        '\'' => single_quoted(f, content),
        _ => double_quoted(f, content),
    }
}

/// Whether a bare word from a `%i` literal prints as `:word`.
pub fn is_bare_symbol(value: &str) -> bool {
    is_label_identifier(value)
}

fn single_quoted(f: &mut Formatter<'_>, content: &str) {
    f.b.text("'");
    literal_text(f, content);
    f.b.text("'");
}

fn double_quoted(f: &mut Formatter<'_>, content: &str) {
    f.b.text("\"");
    literal_text(f, content);
    f.b.text("\"");
}

pub fn interpolated_string_node(f: &mut Formatter<'_>, node: &InterpolatedStringNode<'_>) {
    let parts = node.parts();
    let Some(opening) = node.opening_loc() else {
        concatenation(f, &parts);
        return;
    };
    if is_heredoc(f, &opening) {
        let closing = node.closing_loc().expect("heredoc has a closing");
        heredoc(f, &opening, &closing, |f| heredoc_parts(f, &parts, &closing));
        return;
    }
    f.text_of(&opening);
    for part in parts.iter() {
        f.node(&part);
    }
    f.text_of(&node.closing_loc().expect("quoted string has a closing"));
}

/// Adjacent literals (`'a' 'b'`) always print one per line, joined by `\`.
fn concatenation(f: &mut Formatter<'_>, parts: &NodeList<'_>) {
    let mut parts = parts.iter();
    let first = parts.next().expect("concatenation has parts");
    f.node(&first);
    f.indent(|f| {
        for part in parts {
            f.b.text(" \\");
            f.b.line(HARD);
            f.node(&part);
        }
    });
}

/// The body of `#{...}` prints flat whatever its width. Only an
/// interpolation whose source spans lines (a heredoc body counts) is a real
/// group that breaks into the `#{`/indent/`}` form when too wide. Statements
/// on one source line stay `; `-joined either way; statements on separate
/// lines, or body comments, always take the broken form, one source line
/// of statements per line.
pub fn embedded_statements_node(f: &mut Formatter<'_>, node: &EmbeddedStatementsNode<'_>) {
    f.b.text("#{");
    if let Some(statements) = node.statements() {
        let body = statements.body();
        let separate_lines = body
            .iter()
            .zip(body.iter().skip(1))
            .any(|(a, b)| f.line_of(a.location().end_offset()) != f.line_of(b.location().start_offset()));
        let comments = f.dangling_body(&statements);
        if separate_lines || !comments.is_empty() {
            f.indent(|f| {
                f.b.line(HARD);
                commented_body(f, &body, &comments);
            });
            f.b.line(HARD);
        } else {
            let closing_end = node.closing_loc().end_offset();
            let end = body
                .iter()
                .filter_map(|statement| heredoc_end_offset(f.source, &statement))
                .max()
                .map_or(closing_end, |heredoc_end| heredoc_end.max(closing_end));
            let spans_lines = f.line_of(node.opening_loc().start_offset()) != f.line_of(end);
            let flat = if spans_lines { None } else { flat_body(f, &body) };
            match flat {
                Some(docs) => f.b.append(docs),
                None => f.group(|f| {
                    f.indent(|f| {
                        f.b.soft();
                        joined_body(f, &body);
                    });
                    f.b.soft();
                }),
            }
        }
    }
    f.b.text("}");
}

/// Statements and own-line comments in source order, one line each except
/// that statements sharing a source line stay `; `-joined; a blank line in
/// the source before an entry survives. An ignore directive always gets a
/// blank line before it and copies the next statement from the source.
fn commented_body(f: &mut Formatter<'_>, body: &NodeList<'_>, comments: &[Comment]) {
    let mut comments = comments.iter().peekable();
    let mut previous_end: Option<usize> = None;
    let separate = |f: &mut Formatter<'_>, start: usize, previous_end: Option<usize>, blank: bool| {
        if let Some(end) = previous_end {
            f.b.line(HARD);
            if blank || f.line_of(start) - f.line_of(end) > 1 {
                f.b.line(HARD);
            }
        }
    };
    let mut ignored = false;
    for statement in body.iter() {
        let start = statement.location().start_offset();
        while let Some(comment) = comments.next_if(|c| c.start < start) {
            ignored = f.is_ignore(comment);
            separate(f, comment.start, previous_end, ignored);
            f.comment(comment);
            previous_end = Some(comment.end);
        }
        match previous_end {
            Some(end) if f.line_of(end) == f.line_of(start) => f.b.text("; "),
            _ => separate(f, start, previous_end, false),
        }
        if std::mem::take(&mut ignored) {
            f.verbatim(&statement.location());
        } else {
            f.node(&statement);
        }
        previous_end = Some(statement.location().end_offset());
    }
    for comment in comments {
        separate(f, comment.start, previous_end, false);
        f.comment(comment);
        previous_end = Some(comment.end);
    }
}

fn joined_body(f: &mut Formatter<'_>, body: &NodeList<'_>) {
    for (i, statement) in body.iter().enumerate() {
        if i > 0 {
            f.b.text("; ");
        }
        f.node(&statement);
    }
}

/// Builds the joined body into a discarded probe and returns its flat
/// rendering, or `None` when a forced break (a literal newline) makes a
/// single line impossible.
fn flat_body(f: &mut Formatter<'_>, body: &NodeList<'_>) -> Option<doc::Fragment> {
    let flags = f.b.broken_flags();
    f.b.push_target();
    f.b.open_group();
    joined_body(f, body);
    let broken = f.b.current_group_broken();
    f.b.close_group();
    let docs = f.b.pop_target();
    f.b.restore_broken_flags(flags);
    (!broken).then(|| f.b.flatten(docs))
}

pub fn embedded_variable_node(f: &mut Formatter<'_>, node: &EmbeddedVariableNode<'_>) {
    f.b.text("#{");
    f.node(&node.variable());
    f.b.text("}");
}

pub fn symbol_node(f: &mut Formatter<'_>, node: &SymbolNode<'_>) {
    let Some(opening) = quote_opening(f, node) else {
        f.text_of(&node.location());
        return;
    };
    let closing = node.closing_loc().expect("quoted symbol has a closing");
    let label = f.slice(&closing).ends_with(':');
    let closing = f.slice(&closing).to_owned();
    let value = symbol_value(f, node);
    quoted_symbol(f, &opening, &value, &closing, label);
}

/// The delimiter of a quoted symbol (`:"`, `'`, `%s(`, ...); `None` for a
/// bare `:sym` or `sym:`.
fn quote_opening(f: &Formatter<'_>, node: &SymbolNode<'_>) -> Option<String> {
    let opening = f.slice(&node.opening_loc()?).to_owned();
    (opening != ":").then_some(opening)
}

fn symbol_value(f: &Formatter<'_>, node: &SymbolNode<'_>) -> String {
    node.value_loc().map(|loc| f.slice(&loc).to_owned()).unwrap_or_default()
}

/// Whether the calls family may print this `=>` key as a label (`a:`).
/// Bare symbols qualify only when their name is a plain identifier; every
/// quoted symbol qualifies.
pub fn symbol_can_be_label(f: &Formatter<'_>, node: &SymbolNode<'_>) -> bool {
    quote_opening(f, node).is_some() || is_label_identifier(&symbol_value(f, node))
}

/// Prints a `:sym` key in label position: `:a` as `a:`, `:"a b"` as `'a b':`.
pub fn symbol_as_label(f: &mut Formatter<'_>, node: &SymbolNode<'_>) {
    let value = symbol_value(f, node);
    match quote_opening(f, node) {
        None => {
            f.b.text(value);
            f.b.text(":");
        }
        Some(opening) => {
            let opening = opening.trim_start_matches(':').to_string();
            // A source-side label (`"a'b":`) already carries the colon.
            let closing = node
                .closing_loc()
                .map(|loc| f.slice(&loc).trim_end_matches(':').to_string())
                .unwrap_or_default();
            quoted_symbol(f, &opening, &value, &format!("{closing}:"), true);
        }
    }
}

/// Prints a symbol key in rocket position (`=>`), whatever its source
/// form: `e:` and `"e":` as `:e`, `"a b":` as `:'a b'`, `"a'b":` as
/// `:"a'b"`.
pub fn symbol_as_rocket_key(f: &mut Formatter<'_>, node: &SymbolNode<'_>) {
    let value = symbol_value(f, node);
    match quote_opening(f, node) {
        None => {
            f.b.text(":");
            f.b.text(value);
        }
        Some(opening) => {
            let opening = if opening.starts_with(':') {
                opening
            } else {
                format!(":{opening}")
            };
            let closing = node
                .closing_loc()
                .map(|loc| f.slice(&loc).trim_end_matches(':').to_string())
                .unwrap_or_default();
            quoted_symbol(f, &opening, &value, &closing, false);
        }
    }
}

/// One `%W[]`/`%I[]` word: its literal parts verbatim, its interpolations
/// laid out like any other (`#{f("x")}` becomes `#{f('x')}`).
pub fn interpolated_word(f: &mut Formatter<'_>, parts: &NodeList<'_>) {
    for part in parts.iter() {
        match part.as_string_node() {
            Some(s) => f.text_of(&s.content_loc()),
            None => f.node(&part),
        }
    }
}

/// Prints a `:"#{x}"` key in label position: `"#{x}":`.
pub fn interpolated_symbol_as_label(f: &mut Formatter<'_>, node: &InterpolatedSymbolNode<'_>) {
    let opening = node.opening_loc().expect("interpolated symbol has an opening");
    let quote = f.slice(&opening).trim_start_matches(':').to_string();
    f.b.text(quote.clone());
    for part in node.parts().iter() {
        f.node(&part);
    }
    f.b.text(quote);
    f.b.text(":");
}

/// `opening` and `closing` are the source delimiters (`:"`, `%s(`, `'`,
/// `":`, ...); `label` selects the `'x':` form over `:'x'`.
fn quoted_symbol(f: &mut Formatter<'_>, opening: &str, value: &str, closing: &str, label: bool) {
    if label && is_label_identifier(value) {
        f.b.text_ref(value);
        f.b.text(":");
        return;
    }
    let quote = match f.options.quote_style {
        QuoteStyle::Single if can_use_quote(value, '\'') => Some('\''),
        QuoteStyle::Double if can_use_quote(value, '"') => Some('"'),
        QuoteStyle::Single | QuoteStyle::Double | QuoteStyle::Preserve => None,
    };
    let Some(quote) = quote else {
        f.b.text_ref(opening);
        literal_text(f, value);
        f.b.text_ref(closing);
        return;
    };
    if !label {
        f.b.text(":");
    }
    match quote {
        '\'' => single_quoted(f, value),
        '"' => double_quoted(f, value),
        _ => unreachable!("quote style only supports Ruby string delimiters"),
    }
    if label {
        f.b.text(":");
    }
}

fn is_label_identifier(value: &str) -> bool {
    let body = value.strip_suffix(['?', '!']).unwrap_or(value);
    let mut chars = body.chars();
    chars.next().is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

pub fn interpolated_symbol_node(f: &mut Formatter<'_>, node: &InterpolatedSymbolNode<'_>) {
    f.text_of(&node.opening_loc().expect("interpolated symbol has an opening"));
    for part in node.parts().iter() {
        f.node(&part);
    }
    f.text_of(&node.closing_loc().expect("interpolated symbol has a closing"));
}

pub fn regular_expression_node(f: &mut Formatter<'_>, node: &RegularExpressionNode<'_>) {
    let content = f.slice(&node.content_loc());
    let content_loc = node.content_loc();
    regex(
        f,
        &node.opening_loc(),
        &node.closing_loc(),
        content,
        |f, unescape_slash| {
            if unescape_slash {
                f.b.text(content.replace("\\/", "/"));
            } else {
                f.text_of(&content_loc);
            }
        },
    );
}

pub fn interpolated_regular_expression_node(f: &mut Formatter<'_>, node: &InterpolatedRegularExpressionNode<'_>) {
    let parts = node.parts();
    let content = parts_text(f, &parts);
    regex(
        f,
        &node.opening_loc(),
        &node.closing_loc(),
        &content,
        |f, unescape_slash| regex_parts(f, &parts, unescape_slash),
    );
}

pub fn match_last_line_node(f: &mut Formatter<'_>, node: &MatchLastLineNode<'_>) {
    let content = f.slice(&node.content_loc());
    let content_loc = node.content_loc();
    regex(
        f,
        &node.opening_loc(),
        &node.closing_loc(),
        content,
        |f, unescape_slash| {
            if unescape_slash {
                f.b.text(content.replace("\\/", "/"));
            } else {
                f.text_of(&content_loc);
            }
        },
    );
}

pub fn interpolated_match_last_line_node(f: &mut Formatter<'_>, node: &InterpolatedMatchLastLineNode<'_>) {
    let parts = node.parts();
    let content = parts_text(f, &parts);
    regex(
        f,
        &node.opening_loc(),
        &node.closing_loc(),
        &content,
        |f, unescape_slash| regex_parts(f, &parts, unescape_slash),
    );
}

/// Concatenated source of the literal parts, used only to choose delimiters.
fn parts_text(f: &Formatter<'_>, parts: &NodeList<'_>) -> String {
    parts
        .iter()
        .filter_map(|part| part.as_string_node().map(|string| f.slice(&string.content_loc())))
        .collect()
}

fn regex_parts(f: &mut Formatter<'_>, parts: &NodeList<'_>, unescape_slash: bool) {
    for part in parts.iter() {
        match part.as_string_node() {
            Some(s) => {
                if unescape_slash {
                    f.b.text(f.slice(&s.content_loc()).replace("\\/", "/"));
                } else {
                    f.text_of(&s.content_loc());
                }
            }
            None => f.node(&part),
        }
    }
}

/// `content` is the literal text between the delimiters; `body` prints it,
/// unescaping `\/` when asked to.
fn regex(
    f: &mut Formatter<'_>,
    opening_loc: &Location<'_>,
    closing_loc: &Location<'_>,
    content: &str,
    body: impl FnOnce(&mut Formatter<'_>, bool),
) {
    let opening = f.slice(opening_loc).to_owned();
    let closing = f.slice(closing_loc).to_owned();
    let flags = &closing[1..];
    let braces = content.contains(['{', '}']);
    if opening == "/" {
        if content.contains("\\/") && !braces {
            f.b.text("%r{");
            body(f, true);
            f.b.text("}");
        } else {
            f.b.text("/");
            body(f, false);
            f.b.text("/");
        }
    } else if content.contains('/') {
        if braces {
            f.b.text(opening);
            body(f, false);
            f.b.text_ref(&closing[..1]);
        } else {
            f.b.text("%r{");
            body(f, false);
            f.b.text("}");
        }
    } else {
        f.b.text("/");
        body(f, false);
        f.b.text("/");
    }
    f.b.text_ref(flags);
}

pub fn xstring_node(f: &mut Formatter<'_>, node: &XStringNode<'_>) {
    let opening = node.opening_loc();
    if is_heredoc(f, &opening) {
        let body = node.content_loc();
        heredoc(f, &opening, &node.closing_loc(), |f| {
            f.source_lines(body.start_offset(), body.end_offset(), return_line)
        });
        return;
    }
    f.b.text("`");
    f.text_of(&node.content_loc());
    f.b.text("`");
}

pub fn interpolated_xstring_node(f: &mut Formatter<'_>, node: &InterpolatedXStringNode<'_>) {
    let opening = node.opening_loc();
    let parts = node.parts();
    if is_heredoc(f, &opening) {
        heredoc(f, &opening, &node.closing_loc(), |f| {
            heredoc_parts(f, &parts, &node.closing_loc())
        });
        return;
    }
    f.b.text("`");
    for part in parts.iter() {
        match part.as_string_node() {
            Some(s) => f.text_of(&s.content_loc()),
            None => f.node(&part),
        }
    }
    f.b.text("`");
}

fn is_heredoc(f: &Formatter<'_>, opening: &Location<'_>) -> bool {
    f.slice(opening).starts_with("<<")
}

/// The opening prints in place; the body and closing identifier are deferred
/// to the end of the current line, after any trailing comment, and never
/// force the enclosing groups to break.
fn heredoc(
    f: &mut Formatter<'_>,
    opening: &Location<'_>,
    closing: &Location<'_>,
    body: impl FnOnce(&mut Formatter<'_>),
) {
    f.text_of(opening);
    let closing_start = closing.start_offset();
    let closing_end =
        closing.end_offset() - usize::from(f.source[closing.start_offset()..closing.end_offset()].ends_with(b"\n"));
    f.line_suffix(HEREDOC_PRIORITY, |f| {
        return_line(f);
        body(f);
        f.b.source(closing_start, closing_end);
    });
}

/// Prism excludes the whitespace a `<<~` heredoc dedents from its parts, so
/// the body is walked from the source with each part printed in place.
fn heredoc_parts(f: &mut Formatter<'_>, parts: &NodeList<'_>, closing: &Location<'_>) {
    let end = closing.start_offset();
    let first = parts.iter().next().map_or(end, |part| part.location().start_offset());
    let mut cursor = f.source[..first].iter().rposition(|&b| b == b'\n').map_or(0, |i| i + 1);
    for part in parts.iter() {
        let (start, part_end) = (part.location().start_offset(), part.location().end_offset());
        if part.as_string_node().is_some() {
            f.source_lines(cursor, part_end, return_line);
        } else {
            f.source_lines(cursor, start, return_line);
            f.node(&part);
        }
        cursor = part_end;
    }
    f.source_lines(cursor, end, return_line);
}

/// End offset of the last heredoc closing identifier inside `node`, for
/// statement spacing. Heredocs under an array, hash, or parentheses literal
/// do not extend the containing statement past its closing bracket.
pub fn heredoc_end_offset(source: &[u8], node: &Node<'_>) -> Option<usize> {
    struct Finder<'s> {
        source: &'s [u8],
        end: Option<usize>,
    }
    impl<'s> Finder<'s> {
        fn record(&mut self, opening: &Location<'_>, closing: &Location<'_>) {
            if self.source[opening.start_offset()..opening.end_offset()].starts_with(b"<<") {
                let text = &self.source[closing.start_offset()..closing.end_offset()];
                let end = closing.end_offset() - usize::from(text.ends_with(b"\n"));
                self.end = self.end.max(Some(end));
            }
        }
    }
    impl<'pr> Visit<'pr> for Finder<'_> {
        fn visit_array_node(&mut self, _: &ruby_prism::ArrayNode<'pr>) {}
        fn visit_hash_node(&mut self, _: &ruby_prism::HashNode<'pr>) {}
        fn visit_parentheses_node(&mut self, _: &ruby_prism::ParenthesesNode<'pr>) {}
        fn visit_string_node(&mut self, node: &StringNode<'pr>) {
            if let (Some(opening), Some(closing)) = (node.opening_loc(), node.closing_loc()) {
                self.record(&opening, &closing);
            }
        }
        fn visit_x_string_node(&mut self, node: &XStringNode<'pr>) {
            self.record(&node.opening_loc(), &node.closing_loc());
        }
        fn visit_interpolated_string_node(&mut self, node: &InterpolatedStringNode<'pr>) {
            if let (Some(opening), Some(closing)) = (node.opening_loc(), node.closing_loc()) {
                self.record(&opening, &closing);
            }
            ruby_prism::visit_interpolated_string_node(self, node);
        }
        fn visit_interpolated_x_string_node(&mut self, node: &InterpolatedXStringNode<'pr>) {
            self.record(&node.opening_loc(), &node.closing_loc());
            ruby_prism::visit_interpolated_x_string_node(self, node);
        }
    }
    let mut finder = Finder { source, end: None };
    finder.visit(node);
    finder.end
}

/// Verbatim text whose newlines restart the column at zero without
/// breaking any group.
/// A `RETURN` line that does not propagate a break to enclosing groups, so
/// a heredoc argument leaves its call flat. Both branches are identical, so
/// the printer emits the line in either mode.
fn return_line(f: &mut Formatter<'_>) {
    let line = f.b.line_fragment(RETURN);
    f.b.close_if_break(line, line);
}
