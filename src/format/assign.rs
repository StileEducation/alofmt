//! Formatting for assignments: single writes (`=`, `||=`, `&&=`, `+=`, ...)
//! to variables, constants, attributes and indexes; assignment targets; and
//! multiple assignment.
//!
//! The one rule that matters: `lhs = value` stays on a line when it fits and
//! otherwise moves the value to an indented line, unless the value "opens a
//! bracket" (see [`stays_inline`]), in which case it stays beside the `=`
//! and breaks its own contents. Index writes (`a[i] = v`) never move the
//! value: the brackets break instead.

use ruby_prism::{
    ArgumentsNode, CallAndWriteNode, CallOperatorWriteNode, CallOrWriteNode, CallTargetNode, ClassVariableAndWriteNode,
    ClassVariableOperatorWriteNode, ClassVariableOrWriteNode, ClassVariableTargetNode, ClassVariableWriteNode,
    ConstantAndWriteNode, ConstantOperatorWriteNode, ConstantOrWriteNode, ConstantPathAndWriteNode,
    ConstantPathOperatorWriteNode, ConstantPathOrWriteNode, ConstantPathTargetNode, ConstantPathWriteNode,
    ConstantTargetNode, ConstantWriteNode, GlobalVariableAndWriteNode, GlobalVariableOperatorWriteNode,
    GlobalVariableOrWriteNode, GlobalVariableTargetNode, GlobalVariableWriteNode, IndexAndWriteNode,
    IndexOperatorWriteNode, IndexOrWriteNode, IndexTargetNode, InstanceVariableAndWriteNode,
    InstanceVariableOperatorWriteNode, InstanceVariableOrWriteNode, InstanceVariableTargetNode,
    InstanceVariableWriteNode, LocalVariableAndWriteNode, LocalVariableOperatorWriteNode, LocalVariableOrWriteNode,
    LocalVariableTargetNode, LocalVariableWriteNode, Location, MultiTargetNode, MultiWriteNode, Node,
};

use super::Formatter;
use crate::doc::{SOFT, SPACE};

/// `lhs <operator> value` for every assignment operator (`=`, `||=`, `&&=`,
/// `+=`, ...). The whole assignment is one group: when it does not fit, the
/// value moves to an indented line of its own — unless it [`stays_inline`],
/// in which case it follows the operator and breaks its own contents.
///
/// `lhs` prints the target; it may break internally (a long receiver chain),
/// which also forces the value down. Attribute writes in `calls.rs` are the
/// same layout and can be switched to this function.
pub fn assignment(
    f: &mut Formatter<'_>,
    lhs: impl FnOnce(&mut Formatter<'_>),
    operator: &'static str,
    value: &Node<'_>,
) {
    assignment_with(f, lhs, operator == "=", |f| f.b.text(operator), value);
}

fn located_assignment(
    f: &mut Formatter<'_>,
    lhs: impl FnOnce(&mut Formatter<'_>),
    operator: &Location<'_>,
    value: &Node<'_>,
) {
    let plain = f.slice(operator) == "=";
    assignment_with(f, lhs, plain, |f| f.text_of(operator), value);
}

fn assignment_with(
    f: &mut Formatter<'_>,
    lhs: impl FnOnce(&mut Formatter<'_>),
    plain: bool,
    operator: impl FnOnce(&mut Formatter<'_>),
    value: &Node<'_>,
) {
    // `x = # c` moves even a bracketed value down, the comment ending the
    // line; `x ||= # c` keeps it inline and the comment trails the value.
    let headed = std::mem::take(&mut f.header_break);
    let inline = stays_inline(f, value) && !(headed && plain);
    f.header_break = headed && !inline;
    f.group(|f| {
        lhs(f);
        f.b.text(" ");
        operator(f);
        assignment_value(f, value, inline);
    });
}

/// The value part of [`assignment`], after the operator. Values that stay
/// inline are preceded by a space; the rest sit in an indented line.
fn assignment_value(f: &mut Formatter<'_>, value: &Node<'_>, inline: bool) {
    if inline {
        f.b.text(" ");
        f.node(value);
    } else {
        f.indent(|f| {
            f.b.line(SPACE);
            f.node(value);
        });
    }
}

/// Values that stay beside the operator when the assignment breaks: bracketed
/// array and hash literals, heredocs, `->` lambdas, and dot chains rooted in
/// one of those whose members all have parenthesised (or no) arguments and no
/// block. Everything else — calls, blocks, operators, strings, constants —
/// moves to the next line.
pub fn stays_inline(f: &Formatter<'_>, value: &Node<'_>) -> bool {
    match value {
        Node::ArrayNode { .. } => value.as_array_node().expect("kind").opening_loc().is_some(),
        Node::HashNode { .. } | Node::LambdaNode { .. } => true,
        Node::StringNode { .. } => value
            .as_string_node()
            .expect("kind")
            .opening_loc()
            .is_some_and(|l| is_heredoc(f, &l)),
        Node::InterpolatedStringNode { .. } => value
            .as_interpolated_string_node()
            .expect("kind")
            .opening_loc()
            .is_some_and(|l| is_heredoc(f, &l)),
        Node::XStringNode { .. } => is_heredoc(f, &value.as_x_string_node().expect("kind").opening_loc()),
        Node::InterpolatedXStringNode { .. } => {
            is_heredoc(f, &value.as_interpolated_x_string_node().expect("kind").opening_loc())
        }
        Node::CallNode { .. } => {
            let call = value.as_call_node().expect("kind");
            // The same-line layout keeps any call or chain beside the
            // operator, except bare command calls, whose unbracketed
            // arguments have nowhere good to break.
            if f.options.multiline_assignment_layout == crate::options::MultilineAssignmentLayout::SameLine {
                // Something after the operator must be able to break: a
                // bracket, a block, or a dot chain. A bare identifier call
                // still moves down whole, like any other unbreakable value.
                return !call.is_attribute_write()
                    && !(call.arguments().is_some() && call.opening_loc().is_none())
                    && (call.opening_loc().is_some() || call.block().is_some() || call.receiver().is_some());
            }
            let Some(receiver) = call.receiver() else {
                return false;
            };
            call.call_operator_loc().is_some()
                && call.message_loc().is_some()
                && !call.is_attribute_write()
                && !(call.arguments().is_some() && call.opening_loc().is_none())
                && !matches!(call.block(), Some(Node::BlockNode { .. }))
                && stays_inline(f, &receiver)
        }
        _ => false,
    }
}

fn is_heredoc(f: &Formatter<'_>, opening: &Location<'_>) -> bool {
    f.slice(opening).starts_with("<<")
}

/// `receiver[args]` as the target of a write. The receiver shares the group,
/// so a receiver that had to break leaves the brackets broken too.
fn index_target(f: &mut Formatter<'_>, receiver: &Node<'_>, arguments: Option<ArgumentsNode<'_>>) {
    f.group(|f| {
        f.node(receiver);
        match arguments {
            None => f.b.text("[]"),
            Some(arguments) => {
                f.b.text("[");
                f.indent(|f| {
                    f.b.line(SOFT);
                    f.comma_separated(arguments.arguments().iter());
                });
                f.b.line(SOFT);
                f.b.text("]");
            }
        }
    });
}

/// `receiver[args] <operator> value`: the value never moves down; when the
/// statement does not fit, the brackets break around the index arguments.
fn index_write(
    f: &mut Formatter<'_>,
    receiver: &Node<'_>,
    arguments: Option<ArgumentsNode<'_>>,
    operator: &Location<'_>,
    value: &Node<'_>,
) {
    // `a[i] ||= # c` stays on one line, the comment trailing the value.
    f.header_break = false;
    index_target(f, receiver, arguments);
    f.b.text(" ");
    f.text_of(operator);
    f.b.text(" ");
    f.node(value);
}

/// `.` or `&.` between a receiver and its attribute; `::` prints as `.`.
fn call_operator(f: &mut Formatter<'_>, operator: Option<Location<'_>>) {
    if let Some(operator) = operator {
        let text = if f.slice(&operator) == "&." { "&." } else { "." };
        f.b.text(text);
    }
}

fn attribute_target(
    f: &mut Formatter<'_>,
    receiver: Option<Node<'_>>,
    operator: Option<Location<'_>>,
    message: Option<Location<'_>>,
) {
    if let Some(receiver) = receiver {
        f.node(&receiver);
    }
    call_operator(f, operator);
    if let Some(message) = message {
        f.text_of(&message);
    }
}

fn constant_path_target(f: &mut Formatter<'_>, parent: Option<Node<'_>>, name: &Location<'_>) {
    if let Some(parent) = parent {
        f.node(&parent);
    }
    f.b.text("::");
    f.text_of(name);
}

pub fn call_and_write_node(f: &mut Formatter<'_>, node: &CallAndWriteNode<'_>) {
    located_assignment(
        f,
        |f| attribute_target(f, node.receiver(), node.call_operator_loc(), node.message_loc()),
        &node.operator_loc(),
        &node.value(),
    );
}

pub fn call_operator_write_node(f: &mut Formatter<'_>, node: &CallOperatorWriteNode<'_>) {
    located_assignment(
        f,
        |f| attribute_target(f, node.receiver(), node.call_operator_loc(), node.message_loc()),
        &node.binary_operator_loc(),
        &node.value(),
    );
}

pub fn call_or_write_node(f: &mut Formatter<'_>, node: &CallOrWriteNode<'_>) {
    located_assignment(
        f,
        |f| attribute_target(f, node.receiver(), node.call_operator_loc(), node.message_loc()),
        &node.operator_loc(),
        &node.value(),
    );
}

pub fn call_target_node(f: &mut Formatter<'_>, node: &CallTargetNode<'_>) {
    attribute_target(
        f,
        Some(node.receiver()),
        Some(node.call_operator_loc()),
        Some(node.message_loc()),
    );
}

macro_rules! variable_write_family {
    (
        $and_fn:ident: $and_node:ident,
        $operator_fn:ident: $operator_node:ident,
        $or_fn:ident: $or_node:ident,
        $target_fn:ident: $target_node:ident,
        $write_fn:ident: $write_node:ident
    ) => {
        pub fn $and_fn(f: &mut Formatter<'_>, node: &$and_node<'_>) {
            located_assignment(
                f,
                |f| f.text_of(&node.name_loc()),
                &node.operator_loc(),
                &node.value(),
            );
        }

        pub fn $operator_fn(f: &mut Formatter<'_>, node: &$operator_node<'_>) {
            located_assignment(
                f,
                |f| f.text_of(&node.name_loc()),
                &node.binary_operator_loc(),
                &node.value(),
            );
        }

        pub fn $or_fn(f: &mut Formatter<'_>, node: &$or_node<'_>) {
            located_assignment(
                f,
                |f| f.text_of(&node.name_loc()),
                &node.operator_loc(),
                &node.value(),
            );
        }

        pub fn $target_fn(f: &mut Formatter<'_>, node: &$target_node<'_>) {
            f.text_of(&node.location());
        }

        pub fn $write_fn(f: &mut Formatter<'_>, node: &$write_node<'_>) {
            assignment(f, |f| f.text_of(&node.name_loc()), "=", &node.value());
        }
    };
}

variable_write_family!(
    class_variable_and_write_node: ClassVariableAndWriteNode,
    class_variable_operator_write_node: ClassVariableOperatorWriteNode,
    class_variable_or_write_node: ClassVariableOrWriteNode,
    class_variable_target_node: ClassVariableTargetNode,
    class_variable_write_node: ClassVariableWriteNode
);

variable_write_family!(
    constant_and_write_node: ConstantAndWriteNode,
    constant_operator_write_node: ConstantOperatorWriteNode,
    constant_or_write_node: ConstantOrWriteNode,
    constant_target_node: ConstantTargetNode,
    constant_write_node: ConstantWriteNode
);

pub fn constant_path_and_write_node(f: &mut Formatter<'_>, node: &ConstantPathAndWriteNode<'_>) {
    located_assignment(
        f,
        |f| f.node(&node.target().as_node()),
        &node.operator_loc(),
        &node.value(),
    );
}

pub fn constant_path_operator_write_node(f: &mut Formatter<'_>, node: &ConstantPathOperatorWriteNode<'_>) {
    located_assignment(
        f,
        |f| f.node(&node.target().as_node()),
        &node.binary_operator_loc(),
        &node.value(),
    );
}

pub fn constant_path_or_write_node(f: &mut Formatter<'_>, node: &ConstantPathOrWriteNode<'_>) {
    located_assignment(
        f,
        |f| f.node(&node.target().as_node()),
        &node.operator_loc(),
        &node.value(),
    );
}

pub fn constant_path_target_node(f: &mut Formatter<'_>, node: &ConstantPathTargetNode<'_>) {
    constant_path_target(f, node.parent(), &node.name_loc());
}

pub fn constant_path_write_node(f: &mut Formatter<'_>, node: &ConstantPathWriteNode<'_>) {
    assignment(f, |f| f.node(&node.target().as_node()), "=", &node.value());
}

variable_write_family!(
    global_variable_and_write_node: GlobalVariableAndWriteNode,
    global_variable_operator_write_node: GlobalVariableOperatorWriteNode,
    global_variable_or_write_node: GlobalVariableOrWriteNode,
    global_variable_target_node: GlobalVariableTargetNode,
    global_variable_write_node: GlobalVariableWriteNode
);

pub fn index_and_write_node(f: &mut Formatter<'_>, node: &IndexAndWriteNode<'_>) {
    let Some(receiver) = node.receiver() else {
        f.unsupported("IndexAndWriteNode without receiver", &node.location());
        return;
    };
    if node.block().is_some() || node.call_operator_loc().is_some() {
        f.unsupported("IndexAndWriteNode with block or call operator", &node.location());
        return;
    }
    index_write(f, &receiver, node.arguments(), &node.operator_loc(), &node.value());
}

pub fn index_operator_write_node(f: &mut Formatter<'_>, node: &IndexOperatorWriteNode<'_>) {
    let Some(receiver) = node.receiver() else {
        f.unsupported("IndexOperatorWriteNode without receiver", &node.location());
        return;
    };
    if node.block().is_some() || node.call_operator_loc().is_some() {
        f.unsupported("IndexOperatorWriteNode with block or call operator", &node.location());
        return;
    }
    index_write(
        f,
        &receiver,
        node.arguments(),
        &node.binary_operator_loc(),
        &node.value(),
    );
}

pub fn index_or_write_node(f: &mut Formatter<'_>, node: &IndexOrWriteNode<'_>) {
    let Some(receiver) = node.receiver() else {
        f.unsupported("IndexOrWriteNode without receiver", &node.location());
        return;
    };
    if node.block().is_some() || node.call_operator_loc().is_some() {
        f.unsupported("IndexOrWriteNode with block or call operator", &node.location());
        return;
    }
    index_write(f, &receiver, node.arguments(), &node.operator_loc(), &node.value());
}

pub fn index_target_node(f: &mut Formatter<'_>, node: &IndexTargetNode<'_>) {
    if node.block().is_some() {
        f.unsupported("IndexTargetNode with block", &node.location());
        return;
    }
    index_target(f, &node.receiver(), node.arguments());
}

variable_write_family!(
    instance_variable_and_write_node: InstanceVariableAndWriteNode,
    instance_variable_operator_write_node: InstanceVariableOperatorWriteNode,
    instance_variable_or_write_node: InstanceVariableOrWriteNode,
    instance_variable_target_node: InstanceVariableTargetNode,
    instance_variable_write_node: InstanceVariableWriteNode
);

variable_write_family!(
    local_variable_and_write_node: LocalVariableAndWriteNode,
    local_variable_operator_write_node: LocalVariableOperatorWriteNode,
    local_variable_or_write_node: LocalVariableOrWriteNode,
    local_variable_target_node: LocalVariableTargetNode,
    local_variable_write_node: LocalVariableWriteNode
);

/// A comma-separated target list: `a, *b, c`, or `a,` for an implicit rest.
/// Breakable lists break one target per line; a flat one never does.
fn targets<'pr>(
    f: &mut Formatter<'_>,
    lefts: impl Iterator<Item = Node<'pr>>,
    rest: Option<Node<'pr>>,
    rights: impl Iterator<Item = Node<'pr>>,
) {
    let mut nodes: Vec<Node<'_>> = lefts.collect();
    let implicit_rest = matches!(rest, Some(Node::ImplicitRestNode { .. }));
    if let Some(rest) = rest.filter(|_| !implicit_rest) {
        nodes.push(rest);
    }
    nodes.extend(rights);
    f.comma_separated(nodes.into_iter());
    if implicit_rest {
        f.b.text(",");
    }
}

/// `(a, b)` nested in a target list or block parameters; without its parens
/// (a `for` index) the targets print bare and stay on the `for` line.
pub fn multi_target_node(f: &mut Formatter<'_>, node: &MultiTargetNode<'_>) {
    if node.lparen_loc().is_none() {
        targets(f, node.lefts().iter(), node.rest(), node.rights().iter());
        return;
    }
    f.group(|f| {
        f.b.text("(");
        f.indent(|f| {
            f.b.line(SOFT);
            targets(f, node.lefts().iter(), node.rest(), node.rights().iter());
        });
        f.b.line(SOFT);
        f.b.text(")");
    });
}

/// `a, b = 1, 2`. Targets and values each break one per line; the value
/// always moves to an indented line when the statement does not fit, even
/// an array literal. Parentheses around the whole target list are dropped.
pub fn multi_write_node(f: &mut Formatter<'_>, node: &MultiWriteNode<'_>) {
    f.group(|f| {
        f.group(|f| {
            let mut lefts: Vec<Node<'_>> = node.lefts().iter().collect();
            let mut rest = node.rest();
            let mut rights: Vec<Node<'_>> = node.rights().iter().collect();
            while let [only] = lefts.as_slice() {
                let Some(inner) = only
                    .as_multi_target_node()
                    .filter(|_| rest.is_none() && rights.is_empty())
                else {
                    break;
                };
                rest = inner.rest();
                rights = inner.rights().iter().collect();
                lefts = inner.lefts().iter().collect();
            }
            targets(f, lefts.into_iter(), rest, rights.into_iter());
        });
        f.b.text(" =");
        // The reference always moves a multi-write value down; only the
        // same-line layout keeps it beside the operator, and a header comment
        // (`a, b = # c`) still forces the break.
        let headed = std::mem::take(&mut f.header_break);
        let inline = f.options.multiline_assignment_layout == crate::options::MultilineAssignmentLayout::SameLine
            && stays_inline(f, &node.value())
            && !headed;
        f.header_break = headed && !inline;
        assignment_value(f, &node.value(), inline);
    });
}
