//! Formatting for calls: method calls and operators, argument lists, blocks,
//! hashes, arrays, `super` and `yield`.
//!
//! Widths and indents here are in doc units (see `doc.rs`); a command call's
//! continuation lines align under its first argument by measuring the text
//! after the last possible break in the prefix.

use ruby_prism::{
    ArgumentsNode, ArrayNode, AssocNode, AssocSplatNode, BlockArgumentNode, BlockNode, CallNode,
    ForwardingArgumentsNode, ForwardingSuperNode, HashNode, KeywordHashNode, Location, Node, SplatNode, SuperNode,
    YieldNode,
};

use super::{AttachedSlot, Formatter, assign, strings};
use crate::comments::Comment;
use crate::doc::{Builder, Doc, Fragment, HARD, SOFT, SPACE};

/// Layout context inherited from enclosing nodes; lives on the [`Formatter`].
#[derive(Default)]
pub struct State {
    /// Set while printing the receiver and arguments of a paren-less command
    /// call, where a brace block must keep its braces: `do`/`end` there would
    /// bind to the command instead. Parentheses and block bodies clear it.
    in_command: bool,
    /// Set while printing the predicate of `if`/`unless`/`while`/`until`,
    /// where a block keeps its braces however it breaks: `do`/`end` there
    /// would read as the keyword's own body. Block bodies clear it.
    in_predicate: bool,
    /// Set while printing the arguments of `return`, whose braceless
    /// keyword hash keeps each key in its source form (`:a => b, c: d`).
    source_keys: bool,
    /// Whether the assocs of the hash being printed use the `key:` form.
    /// A hash uses labels only when every key can be one.
    labels: bool,
    /// Start offsets of statements directly inside a configured compact-chain
    /// block.
    compact_chain_statements: Vec<usize>,
}

fn in_command<R>(f: &mut Formatter<'_>, inside: bool, build: impl FnOnce(&mut Formatter<'_>) -> R) -> R {
    let previous = std::mem::replace(&mut f.calls.in_command, inside);
    let result = build(f);
    f.calls.in_command = previous;
    result
}

/// Prints a conditional's predicate (see [`State::in_predicate`]).
pub fn in_predicate(f: &mut Formatter<'_>, build: impl FnOnce(&mut Formatter<'_>)) {
    let previous = std::mem::replace(&mut f.calls.in_predicate, true);
    build(f);
    f.calls.in_predicate = previous;
}

/// Prints a parenthesised expression, where blocks are free again even
/// inside a predicate.
pub fn outside_predicate(f: &mut Formatter<'_>, build: impl FnOnce(&mut Formatter<'_>)) {
    let previous = std::mem::replace(&mut f.calls.in_predicate, false);
    build(f);
    f.calls.in_predicate = previous;
}

/// Prints the arguments of `return` (see [`State::source_keys`]).
pub fn with_source_keys(f: &mut Formatter<'_>, build: impl FnOnce(&mut Formatter<'_>)) {
    let previous = std::mem::replace(&mut f.calls.source_keys, true);
    build(f);
    f.calls.source_keys = previous;
}

/// A block body starts afresh: neither the enclosing command nor predicate
/// constrains the blocks inside it.
fn in_body<R>(f: &mut Formatter<'_>, build: impl FnOnce(&mut Formatter<'_>) -> R) -> R {
    let previous = (
        std::mem::replace(&mut f.calls.in_command, false),
        std::mem::replace(&mut f.calls.in_predicate, false),
    );
    let result = build(f);
    (f.calls.in_command, f.calls.in_predicate) = previous;
    result
}

pub fn arguments_node(f: &mut Formatter<'_>, node: &ArgumentsNode<'_>) {
    argument_list(f, Some(node), None);
}

pub fn array_node(f: &mut Formatter<'_>, node: &ArrayNode<'_>) {
    let Some(opening) = node.opening_loc() else {
        // Bare `1, 2` on the right of an assignment or `return` prints in the
        // caller's group: once that breaks, the elements go one per line.
        f.comma_separated(node.elements().iter());
        return;
    };
    let opening = f.slice(&opening);
    if opening.starts_with('%') {
        percent_array(f, node, opening);
        return;
    }
    if node.elements().is_empty() {
        empty_container(f, "[", "]", &node.as_node());
        return;
    }
    if f.options.prefer_percent_arrays
        && let Some(kind) = percent_kind(f, node)
    {
        percent_array(f, node, kind);
        return;
    }
    f.group(|f| {
        f.b.text("[");
        f.indent(|f| {
            f.b.line(SOFT);
            f.comma_separated(node.elements().iter());
            trailing_comma(f);
            dangling_lines(f, &node.as_node());
        });
        f.b.line(SOFT);
        f.b.text("]");
    });
}

/// `[]` or `{}`, which opens across two lines when its line is too wide;
/// own-line comments inside keep it open.
fn empty_container(f: &mut Formatter<'_>, open: &'static str, close: &'static str, node: &Node<'_>) {
    let comments = f.dangling(node);
    f.group(|f| {
        f.b.text(open);
        if comments.is_empty() {
            f.b.line(SOFT);
        } else {
            f.indent(|f| {
                for comment in &comments {
                    f.b.line(HARD);
                    f.comment(comment);
                }
            });
            f.b.line(HARD);
        }
        f.b.text(close);
    });
}

/// The `%w`/`%i` form a bracketed array collapses to: two or more plain
/// string literals (or bare symbols) whose source text has no whitespace,
/// backslash or square bracket, and no comments among the elements.
fn percent_kind(f: &Formatter<'_>, node: &ArrayNode<'_>) -> Option<&'static str> {
    let elements = node.elements();
    if elements.len() < 2 || elements.iter().any(|e| f.comments.get(&e).is_some()) {
        return None;
    }
    let word = |loc: Option<Location<'_>>| {
        loc.is_some_and(|loc| {
            let text = f.slice(&loc);
            !text.is_empty() && !text.contains(|c: char| c.is_whitespace() || matches!(c, '\\' | '[' | ']'))
        })
    };
    if elements.iter().all(|e| {
        e.as_string_node().is_some_and(|s| {
            word(Some(s.content_loc())) && s.opening_loc().is_some_and(|o| !f.slice(&o).starts_with("<<"))
        })
    }) {
        return Some("%w");
    }
    if elements.iter().all(|e| {
        e.as_symbol_node()
            .is_some_and(|s| s.opening_loc().is_none_or(|o| f.slice(&o) == ":") && word(s.value_loc()))
    }) {
        return Some("%i");
    }
    None
}

/// `%w[]`-style arrays: one word per line when broken, never a comma, and
/// always square brackets whatever the source used.
fn percent_array(f: &mut Formatter<'_>, node: &ArrayNode<'_>, opening: &str) {
    let kind: String = opening.chars().take(2).collect();
    if node.elements().is_empty() {
        f.b.text(format!("{kind}[]"));
        return;
    }
    f.group(|f| {
        f.b.text(format!("{kind}["));
        f.indent(|f| {
            f.b.line(SOFT);
            let mut first = true;
            for element in node.elements().iter() {
                if !first {
                    f.b.line(SPACE);
                }
                first = false;
                match &element {
                    Node::StringNode { .. } => f.text_of(&element.as_string_node().expect("kind").content_loc()),
                    Node::SymbolNode { .. } => f.text_of(
                        &element
                            .as_symbol_node()
                            .expect("kind")
                            .value_loc()
                            .expect("bare symbol has a value"),
                    ),
                    Node::InterpolatedStringNode { .. } => {
                        strings::interpolated_word(f, &element.as_interpolated_string_node().expect("kind").parts())
                    }
                    Node::InterpolatedSymbolNode { .. } => {
                        strings::interpolated_word(f, &element.as_interpolated_symbol_node().expect("kind").parts())
                    }
                    _ => f.text_of(&element.location()),
                }
            }
        });
        f.b.line(SOFT);
        f.b.text("]");
    });
}

/// `key: value` or `key => value`, whichever the enclosing hash chose (see
/// [`State::labels`]). The pair is one group: a value that does not
/// [`assign::stays_inline`] moves to an indented line when the pair, key
/// included, does not fit. A braced hash value has no group of its own and
/// breaks exactly when the enclosing hash or argument list does.
pub fn assoc_node(f: &mut Formatter<'_>, node: &AssocNode<'_>) {
    let key = node.key();
    let value = node.value();
    let labels = if f.calls.source_keys {
        node.operator_loc().is_none()
    } else {
        f.calls.labels
    };
    let key = |f: &mut Formatter<'_>| match (&key, labels) {
        (Node::SymbolNode { .. }, true) => strings::symbol_as_label(f, &key.as_symbol_node().expect("kind")),
        (Node::InterpolatedSymbolNode { .. }, true) => {
            strings::interpolated_symbol_as_label(f, &key.as_interpolated_symbol_node().expect("kind"))
        }
        (Node::SymbolNode { .. }, false) if node.operator_loc().is_none() => {
            // A `c:` label in a hash that prints rockets becomes `:c =>`.
            strings::symbol_as_rocket_key(f, &key.as_symbol_node().expect("kind"));
            f.b.text(" =>");
        }
        _ => {
            f.node(&key);
            f.b.text(" =>");
        }
    };
    match &value {
        Node::ImplicitNode { .. } => key(f),
        Node::HashNode { .. } => {
            key(f);
            f.b.text(" ");
            hash_contents(f, &value.as_hash_node().expect("kind"));
        }
        _ if assign::stays_inline(f, &value) => {
            key(f);
            f.b.text(" ");
            f.node(&value);
        }
        _ => f.group(|f| {
            key(f);
            f.indent(|f| {
                f.b.line(SPACE);
                f.node(&value);
            });
        }),
    }
}

/// Whether every key of a hash can print as a label: bare symbols that are
/// identifiers, and any quoted or interpolated symbol. Splats do not count.
fn all_labels(f: &Formatter<'_>, elements: &[Node<'_>]) -> bool {
    elements
        .iter()
        .all(|element| match element.as_assoc_node().map(|a| a.key()) {
            None => true,
            Some(Node::SymbolNode { .. }) => strings::symbol_can_be_label(
                f,
                &element
                    .as_assoc_node()
                    .expect("kind")
                    .key()
                    .as_symbol_node()
                    .expect("kind"),
            ),
            Some(Node::InterpolatedSymbolNode { .. }) => true,
            Some(_) => false,
        })
}

/// Prints hash elements with [`State::labels`] decided for this hash.
fn hash_elements(f: &mut Formatter<'_>, elements: Vec<Node<'_>>) {
    let labels = all_labels(f, &elements);
    let previous = std::mem::replace(&mut f.calls.labels, labels);
    f.comma_separated(elements.into_iter());
    f.calls.labels = previous;
}

pub fn assoc_splat_node(f: &mut Formatter<'_>, node: &AssocSplatNode<'_>) {
    f.b.text("**");
    if let Some(value) = node.value() {
        f.node(&value);
    }
}

pub fn block_argument_node(f: &mut Formatter<'_>, node: &BlockArgumentNode<'_>) {
    f.b.text("&");
    if let Some(expression) = node.expression() {
        f.node(&expression);
    }
}

pub fn block_node(f: &mut Formatter<'_>, node: &BlockNode<'_>) {
    block(f, node, BlockStyle::Braces);
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum BlockStyle {
    /// `{ }` when it fits on one line, `do`/`end` otherwise.
    Braces,
    /// `do`/`end` even on one line (a block on `super` with arguments).
    Keywords,
    /// Always broken `do`/`end` (a block on a paren-less command call).
    ForcedDo,
    /// Always broken, with the source's delimiters (a block on a command
    /// whose first argument is parenthesised: `let (:x) { ... }`).
    ForcedSource,
    /// Whatever the source used, flat or broken (a block inside a paren-less
    /// command call, where `do`/`end` would rebind); see [`State::in_command`].
    Source,
    /// Braces, flat or broken (inside a conditional's predicate); see
    /// [`State::in_predicate`].
    Predicate,
}

fn block(f: &mut Formatter<'_>, node: &BlockNode<'_>, style: BlockStyle) {
    let style = match style {
        BlockStyle::Braces if f.calls.in_predicate => BlockStyle::Predicate,
        BlockStyle::Braces if f.calls.in_command => BlockStyle::Source,
        style => style,
    };
    let keywords = match style {
        BlockStyle::Keywords => true,
        BlockStyle::Source | BlockStyle::ForcedSource => f.slice(&node.opening_loc()) == "do",
        BlockStyle::Braces | BlockStyle::ForcedDo | BlockStyle::Predicate => false,
    };
    let fixed = matches!(
        style,
        BlockStyle::Keywords | BlockStyle::Source | BlockStyle::ForcedSource | BlockStyle::Predicate
    );
    f.b.text(" ");
    f.group(|f| {
        if matches!(style, BlockStyle::ForcedDo | BlockStyle::ForcedSource) {
            f.b.break_parent();
        }
        match (fixed, keywords) {
            (true, true) => f.b.text("do"),
            (true, false) => f.b.text("{"),
            (false, _) => f.if_break(|f| f.b.text("do"), |f| f.b.text("{")),
        }
        let parameters = node.parameters();
        if let Some(parameters) = &parameters
            && matches!(parameters, Node::BlockParametersNode { .. })
        {
            f.b.text(" ");
            // `|a, b = {}|` never breaks inside: an empty container
            // default stays `{}` and the call's arguments break instead.
            f.b.push_target();
            f.node(parameters);
            let docs = f.b.pop_target();
            let docs = f.b.flatten(docs);
            f.b.append(docs);
        }
        // A comment on the opener line of an otherwise empty block heads or
        // trails the block node itself; either way it prints after the opener.
        f.trailing_attached(&node.as_node(), AttachedSlot::Header);
        f.trailing_attached(&node.as_node(), AttachedSlot::Trailing);
        let dangling = f.dangling(&node.as_node());
        match node.body() {
            // A body with `rescue`/`ensure` lays itself out, clauses at the
            // block's own column.
            Some(body) if matches!(body, Node::BeginNode { .. }) => {
                in_body(f, |f| f.node(&body));
                f.b.line(SPACE);
            }
            Some(body) => {
                f.indent(|f| {
                    f.b.line(SPACE);
                    in_body(f, |f| f.node(&body));
                });
                f.b.line(SPACE);
            }
            None if !dangling.is_empty() => {
                // A block holding only comments prints them as its body.
                f.indent(|f| {
                    f.b.line(HARD);
                    comment_lines(f, &dangling);
                });
                f.b.line(HARD);
            }
            None if parameters.is_some() || keywords => f.b.line(SPACE),
            None => f.b.line(SOFT),
        }
        match (fixed, keywords) {
            (true, true) => f.b.text("end"),
            (true, false) => f.b.text("}"),
            (false, _) => f.if_break(|f| f.b.text("end"), |f| f.b.text("}")),
        }
    });
}

pub fn call_node(f: &mut Formatter<'_>, node: &CallNode<'_>) {
    match shape(f, node) {
        Shape::Unary => unary(f, node),
        Shape::Binary => binary(f, node),
        Shape::Index => index(f, node, None),
        Shape::IndexWrite => {
            let value = node
                .arguments()
                .expect("index write has a value")
                .arguments()
                .last()
                .expect("index write has a value");
            if f.has_header_comments(&node.as_node()) {
                // `x[1] = # c` keeps the brackets flat and moves the value down.
                f.header_break = false;
                index(f, node, Some(&value));
                f.b.text(" =");
                f.indent(|f| {
                    f.b.line(SPACE);
                    f.node(&value);
                });
            } else {
                index(f, node, Some(&value));
                f.b.text(" = ");
                // A bare `a, b` value keeps its elements together.
                f.group(|f| f.node(&value));
            }
        }
        Shape::AttributeWrite => attribute_write(f, node),
        Shape::Method => method_call(f, node),
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Shape {
    Unary,
    Binary,
    Index,
    IndexWrite,
    AttributeWrite,
    Method,
}

pub(super) fn shape(f: &Formatter<'_>, node: &CallNode<'_>) -> Shape {
    let name = node.name().as_slice();
    let argument_count = node.arguments().map_or(0, |a| a.arguments().len());
    if node.receiver().is_some() && node.call_operator_loc().is_none() {
        if name == b"[]" {
            return Shape::Index;
        }
        if name == b"[]=" {
            return Shape::IndexWrite;
        }
        let message = node
            .message_loc()
            .map(|location| &f.source[location.start_offset()..location.end_offset()]);
        if argument_count == 0
            && node.block().is_none()
            && message.is_some_and(|message| [b"!".as_slice(), b"not", b"-", b"+", b"~"].contains(&message))
        {
            return Shape::Unary;
        }
        if argument_count == 1 && node.opening_loc().is_none() && node.block().is_none() && is_operator(name) {
            return Shape::Binary;
        }
    }
    if node.is_attribute_write() {
        return Shape::AttributeWrite;
    }
    Shape::Method
}

fn is_operator(name: &[u8]) -> bool {
    name.first()
        .is_some_and(|byte| byte.is_ascii() && !byte.is_ascii_alphanumeric() && *byte != b'_' && *byte != b'[')
}

fn unary(f: &mut Formatter<'_>, node: &CallNode<'_>) {
    let receiver = node.receiver().expect("unary call has a receiver");
    let message = node.message_loc().expect("unary call has a message");
    f.text_of(&message);
    match node.opening_loc() {
        Some(_) => {
            f.b.text("(");
            f.node(&receiver);
            f.b.text(")");
        }
        None => {
            if f.slice(&message) == "not" {
                f.b.text(" ");
            }
            f.node(&receiver);
        }
    }
}

/// `a + b` breaks after the operator with the right operand indented; `**`
/// binds tightly enough to print without spaces.
fn binary(f: &mut Formatter<'_>, node: &CallNode<'_>) {
    let receiver = node.receiver().expect("binary call has a receiver");
    let operator = f.slice(&node.message_loc().expect("binary call has a message"));
    let argument = node
        .arguments()
        .expect("binary call has an argument")
        .arguments()
        .first()
        .expect("binary call has an argument");
    let power = operator == "**";
    let left_is_binary = receiver
        .as_call_node()
        .is_some_and(|call| matches!(shape(f, &call), Shape::Binary));
    if operator == "<<" && !left_is_binary {
        // Appending never breaks after `<<`; the operands break inside
        // themselves instead.
        f.node(&receiver);
        f.b.text(" << ");
        f.node(&argument);
        return;
    }
    f.group(|f| {
        f.node(&receiver);
        if !power {
            f.b.text(" ");
        }
        f.b.text_ref(operator);
        f.group(|f| {
            f.indent(|f| {
                f.b.line(if power { SOFT } else { SPACE });
                f.node(&argument);
            })
        });
    });
}

/// `recv.name = value`; a multi-value `recv.name = a, b` arrives as one bare
/// array argument.
fn attribute_write(f: &mut Formatter<'_>, node: &CallNode<'_>) {
    let arguments = node.arguments().expect("attribute write has a value").arguments();
    let value = arguments.iter().last().expect("attribute write has a value");
    let message = node.message_loc().expect("attribute write has a message");
    let name_end =
        message.end_offset() - usize::from(f.source[message.start_offset()..message.end_offset()].ends_with(b"="));
    assign::assignment(
        f,
        |f| {
            f.node(&node.receiver().expect("attribute write has a receiver"));
            call_operator(f, node);
            f.b.source(message.start_offset(), name_end);
        },
        "=",
        &value,
    );
}

/// Own-line comments printed one per line at the current indent.
fn comment_lines(f: &mut Formatter<'_>, comments: &[Comment]) {
    for (i, comment) in comments.iter().enumerate() {
        if i > 0 {
            f.b.line(HARD);
        }
        f.comment(comment);
    }
}

/// Own-line comments dangling after the last element of a container, each
/// on its own line; they force the container open.
fn dangling_lines(f: &mut Formatter<'_>, owner: &Node<'_>) {
    let len = f.dangling_len(owner);
    if len > 0 {
        f.b.line(HARD);
        for index in 0..len {
            if index > 0 {
                f.b.line(HARD);
            }
            let comment = f.dangling_comment(owner, index);
            f.comment(&comment);
        }
    }
}

/// Prints a node through a custom layout, keeping the comments attached to
/// it (and, when `dangling` is set, the own-line comments dangling on it,
/// which lead a chain member). Local stand-in for `Formatter::with_comments`.
fn with_comments(f: &mut Formatter<'_>, node: &Node<'_>, dangling: bool, build: impl FnOnce(&mut Formatter<'_>)) {
    for index in 0..f.attached_len(node, AttachedSlot::Leading) {
        let comment = f.attached_comment(node, AttachedSlot::Leading, index);
        f.comment(&comment);
        f.b.line(HARD);
    }
    if dangling {
        print_leading_dangling(f, node);
    }
    build(f);
    print_chain_trailing(f, node);
}

/// Own-line comments dangling on a call that sit before its name: those
/// after it belong to its argument list (see [`paren_args`]).
fn print_leading_dangling(f: &mut Formatter<'_>, node: &Node<'_>) {
    let name_start = node
        .as_call_node()
        .and_then(|c| c.message_loc())
        .map_or(usize::MAX, |l| l.start_offset());
    for index in 0..f.dangling_len(node) {
        let comment = f.dangling_comment(node, index);
        if comment.start < name_start {
            f.comment(&comment);
            f.b.line(HARD);
        }
    }
}

/// The last member of a chain is the node the caller printed through
/// `f.node`, which already emits its leading and trailing comments; only
/// the own-line comments dangling on it remain.
fn with_dangling_only(f: &mut Formatter<'_>, node: &Node<'_>, build: impl FnOnce(&mut Formatter<'_>)) {
    print_leading_dangling(f, node);
    build(f);
}

fn print_chain_trailing(f: &mut Formatter<'_>, node: &Node<'_>) {
    // A same-line comment counts towards the member's width, so wide ones
    // break the member's arguments or block; the chain breaks regardless.
    let len = f.attached_len(node, AttachedSlot::Trailing);
    for index in 0..len {
        let comment = f.attached_comment(node, AttachedSlot::Trailing, index);
        if !comment.own_line {
            f.b.text(" ");
            f.comment(&comment);
            f.b.break_parent();
        }
    }
    for index in 0..len {
        let comment = f.attached_comment(node, AttachedSlot::Trailing, index);
        if comment.own_line {
            f.trailing_comments(std::slice::from_ref(&comment));
        }
    }
}

fn call_operator(f: &mut Formatter<'_>, node: &CallNode<'_>) {
    if let Some(operator) = node.call_operator_loc() {
        let text = if f.slice(&operator) == "&." { "&." } else { "." };
        f.b.text(text);
    }
}

/// A member of a dot chain: `.name`, `.name(args)` or `.name { block }`.
/// Command-style arguments, index and attribute writes end a chain instead.
fn is_chain_member(node: &CallNode<'_>) -> bool {
    node.receiver().is_some()
        && node.call_operator_loc().is_some()
        && node.message_loc().is_some()
        && !node.is_attribute_write()
        && !(node.arguments().is_some() && node.opening_loc().is_none())
}

fn method_call(f: &mut Formatter<'_>, node: &CallNode<'_>) {
    if is_chain_member(node) {
        let mut inner: Vec<CallNode<'_>> = Vec::new();
        let mut receiver = node.receiver();
        while let Some(call) = receiver.as_ref().and_then(Node::as_call_node).filter(is_chain_member) {
            receiver = call.receiver();
            inner.push(call);
        }
        let members: Vec<&CallNode<'_>> = inner.iter().rev().chain(std::iter::once(node)).collect();
        let weight = members.len() + members.iter().filter(|m| has_block(m)).count();
        let threshold = if f
            .calls
            .compact_chain_statements
            .contains(&node.location().start_offset())
        {
            f.options.compact_chain_break_threshold
        } else {
            f.options.chain_break_threshold
        };
        if weight >= threshold {
            chain(f, &members);
            return;
        }
        // A trailing comment on the receiver ends its line; the `.call`s
        // after it take indented lines of their own.
        let receiver = node.receiver().expect("chain member has a receiver");
        if has_trailing_comments(f, &receiver) {
            f.group(|f| {
                f.node(&receiver);
                // Only the break is indented: the arguments align with the
                // receiver's line.
                f.indent(|f| f.b.line(SOFT));
                with_dangling_only(f, &node.as_node(), |f| chain_member(f, node, MemberLayout::Full));
            });
            return;
        }
    }
    let (block_node, block_argument) = split_block(node.block());
    let arguments = node.arguments();
    let command = node.opening_loc().is_none() && (arguments.is_some() || block_argument.is_some());
    let sole_conditional = command
        && block_argument.is_none()
        && arguments.as_ref().is_some_and(|a| {
            a.arguments().len() == 1
                && matches!(
                    a.arguments().first(),
                    Some(Node::IfNode { .. } | Node::UnlessNode { .. })
                )
        });
    if sole_conditional {
        // `foo a ? b : c` gains parentheses when the ternary has to break
        // into its keyword form, which prints there without its own.
        let argument = arguments
            .as_ref()
            .and_then(|a| a.arguments().first())
            .expect("checked above");
        f.group(|f| {
            receiver_and_name(f, node);
            f.if_break(|f| f.b.text("("), |f| f.b.text(" "));
            f.indent(|f| {
                f.b.line(SOFT);
                in_command(f, true, |f| f.node(&argument));
            });
            f.b.line(SOFT);
            f.if_break(|f| f.b.text(")"), |_| {});
        });
    } else if command {
        // The receiver shares the arguments' group: a receiver forced onto
        // several lines breaks the argument list with it.
        f.group(|f| {
            in_command(f, true, |f| {
                let prefix = measured(f, |f| {
                    receiver_and_name(f, node);
                    f.b.text(" ");
                });
                // A receiver's continuation lines align twice the prefix in;
                // past the line width the arguments fall back to the call's
                // own column.
                let prefix = if unaligned_command(f, node)
                    || f.options.max_command_alignment == 0
                    || (node.receiver().is_some() && prefix > f.options.max_command_alignment)
                    || consistent_sole_argument(f, arguments.as_ref(), block_argument.as_ref())
                {
                    0
                } else {
                    prefix
                };
                command_args(f, arguments.as_ref(), block_argument.as_ref(), prefix);
            })
        });
    } else {
        receiver_and_name(f, node);
        if node.opening_loc().is_some() {
            paren_args(f, &node.as_node(), arguments.as_ref(), block_argument.as_ref());
        }
    }
    if let Some(block_node) = block_node {
        let compact_chain_statements = if f
            .options
            .compact_chain_blocks
            .iter()
            .any(|name| node.name().as_slice() == name.as_bytes())
        {
            statement_starts(block_node.body())
        } else {
            Vec::new()
        };
        let previous = std::mem::replace(&mut f.calls.compact_chain_statements, compact_chain_statements);
        let style = match (command, arguments.as_ref().map(|a| a.arguments().first())) {
            (true, Some(Some(Node::ParenthesesNode { .. }))) => BlockStyle::ForcedSource,
            (true, _) => BlockStyle::ForcedDo,
            (false, _) => BlockStyle::Braces,
        };
        block(f, &block_node, style);
        f.calls.compact_chain_statements = previous;
    }
}

fn statement_starts(body: Option<Node<'_>>) -> Vec<usize> {
    body.and_then(|b| {
        b.as_statements_node()
            .map(|s| s.body().iter().map(|n| n.location().start_offset()).collect())
    })
    .unwrap_or_default()
}

/// Under [`crate::DelimitedArgumentAlignment::Consistent`], a command whose
/// sole argument opens a bracket breaks that bracket at the enclosing indent
/// instead of the argument column, so the closing delimiter lines up with the
/// start of the command.
fn consistent_sole_argument(
    f: &Formatter<'_>,
    arguments: Option<&ArgumentsNode<'_>>,
    block_argument: Option<&Node<'_>>,
) -> bool {
    if f.options.delimited_argument_alignment == crate::options::DelimitedArgumentAlignment::Aligned
        || block_argument.is_some()
    {
        return false;
    }
    let Some(arguments) = arguments else {
        return false;
    };
    if arguments.arguments().len() != 1 {
        return false;
    }
    match arguments.arguments().first() {
        Some(Node::ArrayNode { .. }) => arguments
            .arguments()
            .first()
            .and_then(|a| a.as_array_node())
            .is_some_and(|a| a.opening_loc().is_some()),
        Some(Node::HashNode { .. } | Node::LambdaNode { .. }) => true,
        Some(Node::CallNode { .. }) => arguments
            .arguments()
            .first()
            .and_then(|a| a.as_call_node())
            .is_some_and(|c| c.opening_loc().is_some() && c.arguments().is_some()),
        _ => false,
    }
}

/// Command calls whose continuation lines sit at the enclosing indent rather
/// than under the first argument: calls configured by name, `private def ...`,
/// and calls on a receiver that is itself a command call with a `do` block.
fn unaligned_command(f: &Formatter<'_>, node: &CallNode<'_>) -> bool {
    if node
        .arguments()
        .is_some_and(|a| matches!(a.arguments().first(), Some(Node::DefNode { .. })))
    {
        return true;
    }
    let Some(receiver) = node.receiver() else {
        return false;
    };
    if f.options
        .unaligned_command_calls
        .iter()
        .any(|name| node.name().as_slice() == name.as_bytes())
    {
        return true;
    }
    let Some(receiver) = receiver.as_call_node() else {
        return false;
    };
    matches!(shape(f, &receiver), Shape::Method)
        && receiver.opening_loc().is_none()
        && receiver.arguments().is_some()
        && has_block(&receiver)
}

/// A call's `block` is either a real block or a `&blk` argument.
fn split_block<'pr>(block: Option<Node<'pr>>) -> (Option<BlockNode<'pr>>, Option<Node<'pr>>) {
    match block {
        Some(node) => match node.as_block_node() {
            Some(block_node) => (Some(block_node), None),
            None => (None, Some(node)),
        },
        None => (None, None),
    }
}

fn has_trailing_comments(f: &Formatter<'_>, node: &Node<'_>) -> bool {
    f.has_trailing_comments(node)
}

fn has_block(node: &CallNode<'_>) -> bool {
    node.block().is_some_and(|b| matches!(b, Node::BlockNode { .. }))
}

fn receiver_and_name(f: &mut Formatter<'_>, node: &CallNode<'_>) {
    if let Some(receiver) = node.receiver() {
        f.node(&receiver);
        call_operator(f, node);
    }
    if let Some(message) = node.message_loc() {
        f.text_of(&message);
    }
}

/// Builds `build`'s docs, keeps them, and reports the width of the text after
/// their last possible line break: the column a command call's arguments
/// align to.
fn measured(f: &mut Formatter<'_>, build: impl FnOnce(&mut Formatter<'_>)) -> usize {
    f.b.push_target();
    build(f);
    let docs = f.b.pop_target();
    let width = trailing_width(&f.b, docs, f.source);
    f.b.append(docs);
    width
}

fn trailing_width(builder: &Builder, docs: Fragment, source: &[u8]) -> usize {
    let mut width = 0;
    walk_trailing_width(builder, docs, source, &mut width);
    width
}

fn walk_trailing_width(builder: &Builder, docs: Fragment, source: &[u8], width: &mut usize) {
    for doc in builder.iter(docs) {
        match doc {
            Doc::Text(span) => {
                let text = builder.text_contents(*span);
                *width += if text.is_ascii() {
                    text.len()
                } else {
                    text.chars().count()
                };
            }
            Doc::Source(span) => {
                let text = std::str::from_utf8(&source[span.range()]).expect("the source was validated as UTF-8");
                *width += if text.is_ascii() {
                    text.len()
                } else {
                    text.chars().count()
                };
            }
            Doc::Line(_) => *width = 0,
            Doc::Group(group) => walk_trailing_width(builder, group.contents, source, width),
            Doc::Indent(contents) | Doc::Align(_, contents) => walk_trailing_width(builder, *contents, source, width),
            Doc::IfBreak { flat, .. } => walk_trailing_width(builder, *flat, source, width),
            Doc::LineSuffix { .. } | Doc::BreakParent | Doc::Trim => {}
        }
    }
}

/// Three or more chained calls (a block counts as one more) print with every
/// `.call` on its own indented line once they no longer fit. When the only
/// arguments in the chain belong to its last call, the dots stay together and
/// those arguments break first.
fn chain(f: &mut Formatter<'_>, members: &[&CallNode<'_>]) {
    let head = members
        .first()
        .expect("chain has members")
        .receiver()
        .expect("chain member has a receiver");
    f.node(&head);
    let last_index = members.len() - 1;
    let inner = members.iter().enumerate().any(|(i, m)| {
        let last = i == last_index;
        (has_block(m) && !last) || (m.opening_loc().is_some() && (!last || has_block(m)))
    });
    let last = members.last().expect("chain has members");
    let member_with_comments = |f: &mut Formatter<'_>, member: &CallNode<'_>, layout: MemberLayout| {
        if std::ptr::eq(member, *last) {
            with_dangling_only(f, &member.as_node(), |f| chain_member(f, member, layout));
        } else {
            with_comments(f, &member.as_node(), true, |f| chain_member(f, member, layout));
        }
    };
    // A bracketed array, hash or backtick head keeps the first `.call(args)`
    // on its line; that member's block alone sits inside the chain's indent,
    // so its body indents one level deeper than the `.call`s after it.
    let literal = literal_head(f, &head);
    let members_after_head = if literal { &members[1..] } else { members };
    // The head member shares the group with the dots: its arguments
    // breaking puts every later `.call` on its own line.
    let with_head_member = |f: &mut Formatter<'_>, layout: MemberLayout, rest: &dyn Fn(&mut Formatter<'_>)| {
        f.group(|f| {
            if literal {
                member_with_comments(f, members[0], layout);
            }
            rest(f);
        });
    };
    if inner {
        let first_block = if literal {
            split_block(members[0].block()).0
        } else {
            None
        };
        with_head_member(f, MemberLayout::Arguments, &|f| {
            f.indent(|f| {
                if let Some(block_node) = &first_block {
                    block(f, block_node, BlockStyle::Braces);
                }
                for member in members_after_head {
                    f.b.line(SOFT);
                    member_with_comments(f, member, MemberLayout::Full);
                }
            })
        });
        return;
    }
    let dots = |f: &mut Formatter<'_>| {
        with_head_member(f, MemberLayout::Name, &|f| {
            f.indent(|f| {
                for member in members_after_head {
                    f.b.line(SOFT);
                    member_with_comments(f, member, MemberLayout::Name);
                }
            })
        });
    };
    let (block_node, block_argument) = split_block(last.block());
    if last.opening_loc().is_some() {
        f.group(|f| {
            dots(f);
            paren_args(f, &last.as_node(), last.arguments().as_ref(), block_argument.as_ref());
        });
    } else {
        dots(f);
    }
    if let Some(block_node) = block_node {
        block(f, &block_node, BlockStyle::Braces);
    }
}

fn literal_head(f: &Formatter<'_>, head: &Node<'_>) -> bool {
    match head {
        Node::HashNode { .. } | Node::XStringNode { .. } | Node::InterpolatedXStringNode { .. } => true,
        Node::ArrayNode { .. } => head
            .as_array_node()
            .expect("kind")
            .opening_loc()
            .is_some_and(|o| f.slice(&o) == "["),
        _ => false,
    }
}

/// How much of a chain member prints in place; the rest is the caller's.
#[derive(Clone, Copy)]
enum MemberLayout {
    /// `.name` only.
    Name,
    /// `.name(args)`, the block left to the caller.
    Arguments,
    /// `.name(args) { block }`.
    Full,
}

fn chain_member(f: &mut Formatter<'_>, member: &CallNode<'_>, layout: MemberLayout) {
    call_operator(f, member);
    f.text_of(&member.message_loc().expect("chain member has a message"));
    if matches!(layout, MemberLayout::Name) {
        return;
    }
    let (block_node, block_argument) = split_block(member.block());
    if member.opening_loc().is_some() {
        paren_args(
            f,
            &member.as_node(),
            member.arguments().as_ref(),
            block_argument.as_ref(),
        );
    }
    if let (MemberLayout::Full, Some(block_node)) = (layout, block_node) {
        block(f, &block_node, BlockStyle::Braces);
    }
}

/// `(a, b)`, one argument per line with a trailing comma when broken. Ruby
/// forbids the comma after a block pass or `...`.
fn paren_args(
    f: &mut Formatter<'_>,
    owner: &Node<'_>,
    arguments: Option<&ArgumentsNode<'_>>,
    block_argument: Option<&Node<'_>>,
) {
    if arguments.is_none() && block_argument.is_none() {
        f.b.text("()");
        return;
    }
    f.group(|f| {
        f.b.text("(");
        f.indent(|f| {
            f.b.line(SOFT);
            in_command(f, false, |f| argument_list(f, arguments, block_argument));
            if block_argument.is_none() && arguments.is_some_and(|a| allows_trailing_comma(f, a)) {
                trailing_comma(f);
            }
            // Own-line comments after the last argument dangle on the call.
            let opening = owner
                .as_call_node()
                .and_then(|c| c.opening_loc())
                .map_or(0, |l| l.start_offset());
            let comments: Vec<Comment> = f.dangling(owner).into_iter().filter(|c| c.start > opening).collect();
            if !comments.is_empty() {
                f.b.line(HARD);
                comment_lines(f, &comments);
            }
        });
        f.b.line(SOFT);
        f.b.text(")");
    });
}

/// Omits a trailing comma after `...` or a command call (`foo(bar baz)`).
fn allows_trailing_comma(f: &Formatter<'_>, arguments: &ArgumentsNode<'_>) -> bool {
    match arguments.arguments().last() {
        Some(Node::ForwardingArgumentsNode { .. }) => false,
        Some(Node::CallNode { .. }) => {
            let call = arguments
                .arguments()
                .last()
                .and_then(|n| n.as_call_node())
                .expect("kind");
            !is_command(f, &call)
        }
        _ => true,
    }
}

/// A method call whose arguments (or block pass) have no parentheses.
pub(super) fn is_command(f: &Formatter<'_>, node: &CallNode<'_>) -> bool {
    matches!(shape(f, node), Shape::Method)
        && node.opening_loc().is_none()
        && (node.arguments().is_some() || matches!(node.block(), Some(Node::BlockArgumentNode { .. })))
}

/// `recv[args]`. The receiver shares the group, so a receiver that had to
/// break leaves the brackets broken too.
fn index(f: &mut Formatter<'_>, node: &CallNode<'_>, exclude: Option<&Node<'_>>) {
    let exclude_offset = exclude.map(|n| n.location().start_offset());
    let arguments = node.arguments();
    let indexes: Vec<Node<'_>> = arguments
        .iter()
        .flat_map(|a| a.arguments().iter())
        .filter(|n| Some(n.location().start_offset()) != exclude_offset)
        .collect();
    f.group(|f| {
        f.node(&node.receiver().expect("index call has a receiver"));
        if indexes.is_empty() {
            f.b.text("[]");
            return;
        }
        f.b.text("[");
        f.indent(|f| {
            f.b.line(SOFT);
            in_command(f, false, |f| f.comma_separated(indexes.into_iter()));
        });
        f.b.line(SOFT);
        f.b.text("]");
    });
}

/// Arguments without parentheses: `foo a, b` continues under the first
/// argument, `prefix` units in. The caller supplies the group.
fn command_args(
    f: &mut Formatter<'_>,
    arguments: Option<&ArgumentsNode<'_>>,
    block_argument: Option<&Node<'_>>,
    prefix: usize,
) {
    f.align(prefix, |f| argument_list(f, arguments, block_argument));
}

fn argument_list(f: &mut Formatter<'_>, arguments: Option<&ArgumentsNode<'_>>, block_argument: Option<&Node<'_>>) {
    let positional = arguments.map_or(0, |a| a.arguments().len());
    if let Some(arguments) = arguments {
        // An own-line comment before the first argument leads the list.
        with_comments(f, &arguments.as_node(), false, |f| {
            f.comma_separated(arguments.arguments().iter())
        });
    }
    if let Some(block_argument) = block_argument {
        if positional > 0 {
            f.b.text(",");
            f.b.line(SPACE);
        }
        f.node(block_argument);
    }
}

fn trailing_comma(f: &mut Formatter<'_>) {
    if f.options.trailing_commas {
        f.if_break(|f| f.b.text(","), |_| {});
    }
}

pub fn forwarding_arguments_node(f: &mut Formatter<'_>, _node: &ForwardingArgumentsNode<'_>) {
    f.b.text("...");
}

pub fn forwarding_super_node(f: &mut Formatter<'_>, node: &ForwardingSuperNode<'_>) {
    f.b.text("super");
    if let Some(block_node) = node.block() {
        block(f, &block_node, BlockStyle::Braces);
    }
}

pub fn hash_node(f: &mut Formatter<'_>, node: &HashNode<'_>) {
    if node.elements().is_empty() {
        empty_container(f, "{", "}", &node.as_node());
        return;
    }
    f.group(|f| hash_contents(f, node));
}

/// `{ a: 1 }` without a group of its own, so the caller decides when it
/// breaks.
fn hash_contents(f: &mut Formatter<'_>, node: &HashNode<'_>) {
    if node.elements().is_empty() {
        // An empty hash value opens across lines with its parent: `{\n}`.
        f.b.text("{");
        if !f.has_dangling(&node.as_node()) {
            f.b.line(SOFT);
        } else {
            f.indent(|f| dangling_lines(f, &node.as_node()));
            f.b.line(HARD);
        }
        f.b.text("}");
        return;
    }
    f.b.text("{");
    f.indent(|f| {
        f.b.line(SPACE);
        hash_elements(f, node.elements().iter().collect());
        trailing_comma(f);
        dangling_lines(f, &node.as_node());
    });
    f.b.line(SPACE);
    f.b.text("}");
}

pub fn keyword_hash_node(f: &mut Formatter<'_>, node: &KeywordHashNode<'_>) {
    hash_elements(f, node.elements().iter().collect());
}

pub fn splat_node(f: &mut Formatter<'_>, node: &SplatNode<'_>) {
    f.b.text("*");
    if let Some(expression) = node.expression() {
        f.node(&expression);
    }
}

pub fn super_node(f: &mut Formatter<'_>, node: &SuperNode<'_>) {
    f.b.text("super");
    let parenthesised = node.lparen_loc().is_some();
    let (block_node, block_argument) = split_block(node.block());
    let arguments = node.arguments();
    if parenthesised {
        paren_args(f, &node.as_node(), arguments.as_ref(), block_argument.as_ref());
    } else if arguments.is_some() || block_argument.is_some() {
        f.group(|f| {
            f.b.text(" ");
            command_args(f, arguments.as_ref(), block_argument.as_ref(), "super ".len());
        });
    }
    if let Some(block_node) = block_node {
        let style = if parenthesised || arguments.is_some() {
            BlockStyle::Keywords
        } else {
            BlockStyle::Braces
        };
        block(f, &block_node, style);
    }
}

/// `yield a, b` gains parentheses when it breaks; `yield(a, b)` keeps them.
/// Neither takes a trailing comma.
pub fn yield_node(f: &mut Formatter<'_>, node: &YieldNode<'_>) {
    f.b.text("yield");
    let Some(arguments) = node.arguments() else {
        if node.lparen_loc().is_some() {
            f.b.text("()");
        }
        return;
    };
    let parenthesised = node.lparen_loc().is_some();
    f.group(|f| {
        if parenthesised {
            f.b.text("(");
        } else {
            f.if_break(|f| f.b.text("("), |f| f.b.text(" "));
        }
        f.indent(|f| {
            f.b.line(SOFT);
            argument_list(f, Some(&arguments), None);
        });
        f.b.line(SOFT);
        if parenthesised {
            f.b.text(")");
        } else {
            f.if_break(|f| f.b.text(")"), |_| {});
        }
    });
}
