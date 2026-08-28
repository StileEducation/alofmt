//! Formatting for pattern matching: `case`/`in`, the pattern node kinds, and
//! the `=>` / `in` one-line matches.
//!
//! Inside an `in` clause the pattern sits in an `align(3)` (the width of
//! `in `), which the printer renders as six spaces; that is why broken
//! patterns there indent by ten and close at six.

use ruby_prism::{
    AlternationPatternNode, ArrayPatternNode, CapturePatternNode, CaseMatchNode, FindPatternNode, HashPatternNode,
    ImplicitNode, ImplicitRestNode, InNode, MatchPredicateNode, MatchRequiredNode, MatchWriteNode, Node,
    PinnedExpressionNode, PinnedVariableNode,
};

use super::Formatter;
use crate::comments::Comment;
use crate::doc::{HARD, SOFT, SPACE};

/// `a | b | c` prints as a fill: after each `|` the next alternative stays on
/// the line when it fits and drops to the next otherwise, all continuation
/// lines one level in. Prism nests the chain to the left; it is flattened
/// so no alternative indents deeper than the second.
pub fn alternation_pattern_node(f: &mut Formatter<'_>, node: &AlternationPatternNode<'_>) {
    let mut alternatives = vec![node.right()];
    let mut left = node.left();
    while let Node::AlternationPatternNode { .. } = &left {
        let inner = left.as_alternation_pattern_node().expect("kind");
        alternatives.push(inner.right());
        left = inner.left();
    }
    alternatives.push(left);
    alternatives.reverse();
    let mut alternatives = alternatives.into_iter();
    let first = alternatives.next().expect("alternation has a left side");
    pattern(f, &first);
    f.b.text(" |");
    f.indent(|f| {
        let mut first = true;
        for alternative in alternatives {
            if !first {
                f.b.text(" |");
            }
            first = false;
            // A group holding only the line measures up to the next `|` line.
            f.group(|f| f.b.line(SPACE));
            pattern(f, &alternative);
        }
    });
}

/// A sub-pattern. Splats print here so their inner target is a pattern too.
fn pattern(f: &mut Formatter<'_>, node: &Node<'_>) {
    match node {
        Node::SplatNode { .. } => {
            f.b.text("*");
            if let Some(expression) = node.as_splat_node().expect("kind").expression() {
                pattern(f, &expression);
            }
        }
        Node::AssocSplatNode { .. } => {
            f.b.text("**");
            if let Some(value) = node.as_assoc_splat_node().expect("kind").value() {
                pattern(f, &value);
            }
        }
        _ => f.node(node),
    }
}

pub fn array_pattern_node(f: &mut Formatter<'_>, node: &ArrayPatternNode<'_>) {
    let mut elements: Vec<Node<'_>> = node.requireds().iter().collect();
    // `[a,]`: the implicit rest has no binding and prints as `[a]`.
    if let Some(rest) = node
        .rest()
        .filter(|rest| !matches!(rest, Node::ImplicitRestNode { .. }))
    {
        elements.push(rest);
    }
    elements.extend(node.posts().iter());
    bracketed(f, node.constant(), &elements);
}

pub fn find_pattern_node(f: &mut Formatter<'_>, node: &FindPatternNode<'_>) {
    let mut elements = vec![node.left().as_node()];
    elements.extend(node.requireds().iter());
    elements.push(node.right());
    bracketed(f, node.constant(), &elements);
}

/// `[a, b]` or `Const[a, b]`, whatever the source's brackets (`Const(a, b)`
/// becomes `Const[a, b]`); also the form a bare `in a, b` takes.
fn bracketed(f: &mut Formatter<'_>, constant: Option<Node<'_>>, elements: &[Node<'_>]) {
    if let Some(constant) = constant {
        f.node(&constant);
    }
    if elements.is_empty() {
        f.b.text("[]");
        return;
    }
    f.group(|f| {
        f.b.text("[");
        f.indent(|f| {
            f.b.line(SOFT);
            separated(f, elements);
        });
        f.b.line(SOFT);
        f.b.text("]");
    });
}

/// `value => name`. The name follows a bracketed value's closing bracket
/// and otherwise may drop to an indented line.
pub fn capture_pattern_node(f: &mut Formatter<'_>, node: &CapturePatternNode<'_>) {
    let value = node.value();
    let target = node.target().location();
    f.group(|f| {
        pattern(f, &value);
        f.b.text(" =>");
        if is_bracketed(&value) {
            f.b.text(" ");
            f.text_of(&target);
        } else {
            f.indent(|f| {
                f.b.line(SPACE);
                f.text_of(&target);
            });
        }
    });
}

/// Comments extending a clause body keep a blank line where the source had
/// one after `previous_end`; those at the `case` column never do.
fn clause_end_comments(f: &mut Formatter<'_>, comments: &[&Comment], at_case_column: bool, previous_end: usize) {
    if comments.is_empty() {
        return;
    }
    if at_case_column {
        for comment in comments {
            f.b.line(HARD);
            f.comment(comment);
        }
    } else {
        f.indent(|f| {
            let mut previous_end = previous_end;
            for comment in comments {
                f.b.line(HARD);
                if f.line_of(comment.start) - f.line_of(previous_end) > 1 {
                    f.b.line(HARD);
                }
                f.comment(comment);
                previous_end = comment.end;
            }
        });
    }
}

fn is_bracketed(pattern: &Node<'_>) -> bool {
    matches!(
        pattern,
        Node::ArrayPatternNode { .. } | Node::HashPatternNode { .. } | Node::FindPatternNode { .. }
    )
}

/// Own-line comments between clauses dangle on the `case` node: those before
/// the first `in` print at its column, any later ones extend the body of the
/// clause they follow.
pub fn case_match_node(f: &mut Formatter<'_>, node: &CaseMatchNode<'_>) {
    f.b.text("case ");
    let predicate = node.predicate().expect("case/in always has a predicate");
    f.node(&predicate);
    let dangling = f.dangling(&node.as_node());
    let mut dangling = dangling.iter().peekable();
    let mut first = true;
    let mut previous_end = predicate.location().end_offset();
    for condition in node.conditions().iter() {
        let comments: Vec<_> =
            std::iter::from_fn(|| dangling.next_if(|c| c.start < condition.location().start_offset())).collect();
        clause_end_comments(f, &comments, first, previous_end);
        first = false;
        previous_end = condition.location().end_offset();
        f.b.line(HARD);
        f.node(&condition);
    }
    if let Some(else_clause) = node.else_clause() {
        let comments: Vec<_> =
            std::iter::from_fn(|| dangling.next_if(|c| c.start < else_clause.location().start_offset())).collect();
        clause_end_comments(f, &comments, first, previous_end);
        f.b.line(HARD);
        f.b.text("else");
        f.body_of(else_clause.statements(), &else_clause.as_node());
    }
    let comments: Vec<_> = dangling.collect();
    clause_end_comments(f, &comments, first, previous_end);
    f.b.line(HARD);
    f.b.text("end");
}

/// A `**nil` rest is a `NoKeywordsParameterNode`; a `**rest` one is an
/// `AssocSplatNode` printed by the calls family.
pub fn hash_pattern_node(f: &mut Formatter<'_>, node: &HashPatternNode<'_>) {
    hash_pattern(f, node, true);
}

fn hash_pattern(f: &mut Formatter<'_>, node: &HashPatternNode<'_>, braces: bool) {
    let mut elements: Vec<Node<'_>> = node.elements().iter().collect();
    if let Some(rest) = node.rest() {
        elements.push(rest);
    }
    if let Some(constant) = node.constant() {
        f.node(&constant);
        if elements.is_empty() {
            f.b.text("[]");
            return;
        }
        f.group(|f| {
            f.b.text("[");
            f.indent(|f| {
                f.b.line(SOFT);
                hash_elements(f, &elements);
            });
            f.b.line(SOFT);
            f.b.text("]");
        });
        return;
    }
    if elements.is_empty() {
        f.b.text("{}");
        return;
    }
    if !braces {
        f.group(|f| hash_elements(f, &elements));
        return;
    }
    f.group(|f| {
        f.b.text("{");
        f.indent(|f| {
            f.b.line(SPACE);
            hash_elements(f, &elements);
        });
        f.b.line(SPACE);
        f.b.text("}");
    });
}

fn hash_elements(f: &mut Formatter<'_>, elements: &[Node<'_>]) {
    let mut first = true;
    for element in elements {
        if !first {
            f.b.text(",");
            f.b.line(SPACE);
        }
        first = false;
        match element {
            Node::AssocNode { .. } => f.with_comments(element, |f| {
                let assoc = element.as_assoc_node().expect("kind");
                f.node(&assoc.key());
                let value = assoc.value();
                // There is no break opportunity between the label and value.
                if !matches!(value, Node::ImplicitNode { .. }) {
                    f.b.text(" ");
                    pattern(f, &value);
                }
            }),
            Node::NoKeywordsParameterNode { .. } => f.b.text("**nil"),
            _ => pattern(f, element),
        }
    }
}

/// The value of a shorthand `a:` pair; the key already says it all.
pub fn implicit_node(_f: &mut Formatter<'_>, _node: &ImplicitNode<'_>) {}

pub fn implicit_rest_node(f: &mut Formatter<'_>, node: &ImplicitRestNode<'_>) {
    f.unsupported("ImplicitRestNode outside an array pattern", &node.location());
}

/// `in pattern [if guard]` and its body; `then` is never printed. A single
/// `key: value` pair loses its braces here (and only here).
pub fn in_node(f: &mut Formatter<'_>, node: &InNode<'_>) {
    f.b.text("in ");
    f.align(3, |f| {
        let (pattern, guard) = split_guard(node.pattern());
        top_level_pattern(f, &pattern);
        if let Some((keyword, predicate)) = guard {
            f.b.text(format!(" {keyword} "));
            f.node(&predicate);
        }
    });
    f.body_of(node.statements(), &node.as_node());
}

/// Prism wraps a guarded pattern in the guard's `if`/`unless` node, with the
/// pattern as its only statement.
fn split_guard<'pr>(pattern: Node<'pr>) -> (Node<'pr>, Option<(&'static str, Node<'pr>)>) {
    let (keyword, predicate, statements) = match &pattern {
        Node::IfNode { .. } => {
            let guard = pattern.as_if_node().expect("kind");
            ("if", guard.predicate(), guard.statements())
        }
        Node::UnlessNode { .. } => {
            let guard = pattern.as_unless_node().expect("kind");
            ("unless", guard.predicate(), guard.statements())
        }
        _ => return (pattern, None),
    };
    let statements = statements.expect("guard wraps the pattern");
    let mut body = statements.body().iter();
    let inner = body.next().expect("guard wraps the pattern");
    assert!(body.next().is_none(), "guard wraps exactly one pattern");
    (inner, Some((keyword, predicate)))
}

fn top_level_pattern(f: &mut Formatter<'_>, node: &Node<'_>) {
    match node {
        Node::HashPatternNode { .. } => {
            let hash = node.as_hash_pattern_node().expect("kind");
            let count = hash.elements().len() + usize::from(hash.rest().is_some());
            let braces = hash.constant().is_some() || count != 1;
            hash_pattern(f, &hash, braces);
        }
        _ => pattern(f, node),
    }
}

pub fn match_predicate_node(f: &mut Formatter<'_>, node: &MatchPredicateNode<'_>) {
    one_line_match(f, &node.value(), "in", &node.pattern());
}

pub fn match_required_node(f: &mut Formatter<'_>, node: &MatchRequiredNode<'_>) {
    one_line_match(f, &node.value(), "=>", &node.pattern());
}

/// `value => pattern` / `value in pattern`. A bracketed pattern opens on the
/// same line and breaks inside; anything else may drop to an indented line.
fn one_line_match(f: &mut Formatter<'_>, value: &Node<'_>, operator: &str, pattern: &Node<'_>) {
    f.group(|f| {
        f.node(value);
        f.b.text(format!(" {operator}"));
        if matches!(
            pattern,
            Node::ArrayPatternNode { .. } | Node::HashPatternNode { .. } | Node::FindPatternNode { .. }
        ) {
            f.b.text(" ");
            f.node(pattern);
        } else {
            f.indent(|f| {
                f.b.line(SPACE);
                self::pattern(f, pattern);
            });
        }
    });
}

/// `/(?<name>.)/ =~ value` is the call; the captures it writes are implicit.
pub fn match_write_node(f: &mut Formatter<'_>, node: &MatchWriteNode<'_>) {
    f.node(&node.call().as_node());
}

/// `^(expr)`: the parentheses align one column past the caret.
pub fn pinned_expression_node(f: &mut Formatter<'_>, node: &PinnedExpressionNode<'_>) {
    f.b.text("^");
    f.align(1, |f| {
        f.group(|f| {
            f.b.text("(");
            f.indent(|f| {
                f.b.line(SOFT);
                f.node(&node.expression());
            });
            f.b.line(SOFT);
            f.b.text(")");
        });
    });
}

pub fn pinned_variable_node(f: &mut Formatter<'_>, node: &PinnedVariableNode<'_>) {
    f.b.text("^");
    f.node(&node.variable());
}

fn separated(f: &mut Formatter<'_>, nodes: &[Node<'_>]) {
    let mut first = true;
    for node in nodes {
        if !first {
            f.b.text(",");
            f.b.line(SPACE);
        }
        first = false;
        pattern(f, node);
    }
}
