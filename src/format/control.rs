//! Formatting for control flow: conditionals, loops, `case`, `begin`/`rescue`,
//! jumps, boolean operators, parentheses and the odd keywords.
//!
//! A one-statement `if`/`unless`/`while`/`until` is a group whose flat branch
//! is the modifier (or ternary) form and whose broken branch is the block
//! form; comments and multi-line children break the group, so the block form
//! falls out without special cases. The parent decides whether a converted
//! form needs parentheses (`x = (b if a)`) and whether an `if`/`else` may
//! become a ternary at all (never directly inside parentheses); there are no
//! parent pointers, so one analysis walk records each node's context.

use ruby_prism::{
    AndNode, ArgumentsNode, BeginNode, BreakNode, CallNode, CaseNode, DefinedNode, ElseNode, EnsureNode, FlipFlopNode,
    ForNode, IfNode, Location, NextNode, Node, OrNode, ParenthesesNode, PostExecutionNode, PreExecutionNode, RedoNode,
    RescueModifierNode, RescueNode, RetryNode, ReturnNode, StatementsNode, UnlessNode, UntilNode, Visit, WhenNode,
    WhileNode,
};
use rustc_hash::FxHashMap as HashMap;

use super::{Formatter, calls};
use crate::comments::Comment;
use crate::doc::{COMMENT_PRIORITY, Doc, HARD, SOFT, SPACE};

/// Where a node sits relative to its parent.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Context {
    /// An entry of a statement list (a body, the program).
    Statement,
    /// The contents of a `ParenthesesNode`, possibly via its statement list.
    Paren,
    /// Anywhere else: an argument, an operand, an assigned value.
    Expression,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Statements,
    Parentheses,
    /// `x rescue y` expands to a `begin` block, so `x` is a statement.
    RescueModifier,
    Arguments,
    /// A paren-less call with exactly one argument.
    Command,
    Other,
}

pub struct State {
    contexts: HashMap<(usize, usize), Context>,
    assignment_starts: Vec<usize>,
    node_count: usize,
    /// A flip-flop directly under an `if`/`unless` predicate prints `a .. b`;
    /// anywhere else (`while`, `elsif`, inside parentheses) it prints `a..b`.
    spaced_flip_flop: bool,
}

struct ContextWalk {
    stack: Vec<(Kind, (usize, usize))>,
    contexts: HashMap<(usize, usize), Context>,
    assignment_starts: Vec<usize>,
    node_count: usize,
}

impl<'pr> Visit<'pr> for ContextWalk {
    fn visit_branch_node_enter(&mut self, node: Node<'pr>) {
        self.enter(&node);
    }

    fn visit_branch_node_leave(&mut self) {
        self.stack.pop();
    }

    fn visit_leaf_node_enter(&mut self, node: Node<'pr>) {
        self.enter(&node);
    }

    fn visit_leaf_node_leave(&mut self) {
        self.stack.pop();
    }

    // Statement lists are statically typed children of most parents, which
    // the default visitor reaches without the enter/leave hooks.
    fn visit_statements_node(&mut self, node: &StatementsNode<'pr>) {
        self.typed(&node.as_node(), |walk| ruby_prism::visit_statements_node(walk, node));
    }

    fn visit_arguments_node(&mut self, node: &ArgumentsNode<'pr>) {
        self.typed(&node.as_node(), |walk| ruby_prism::visit_arguments_node(walk, node));
    }

    fn visit_call_node(&mut self, node: &CallNode<'pr>) {
        self.typed(&node.as_node(), |walk| ruby_prism::visit_call_node(walk, node));
    }
}

impl ContextWalk {
    fn typed(&mut self, node: &Node<'_>, descend: impl FnOnce(&mut Self)) {
        let span = span_of(node);
        let entered = self.stack.last() != Some(&(kind_of(node), span));
        if entered {
            self.enter(node);
        }
        descend(self);
        if entered {
            self.stack.pop();
        }
    }

    fn enter(&mut self, node: &Node<'_>) {
        self.node_count += 1;
        if is_assignment(node) {
            self.assignment_starts.push(node.location().start_offset());
        }
        let kind = kind_of(node);
        let span = span_of(node);
        if matches!(
            node,
            Node::IfNode { .. } | Node::UnlessNode { .. } | Node::WhileNode { .. } | Node::UntilNode { .. }
        ) {
            let parent = self.stack.last().map(|(k, _)| *k);
            let grandparent = self.stack.len().checked_sub(2).map(|i| self.stack[i].0);
            let context = match (parent, grandparent) {
                (Some(Kind::Parentheses), _) | (Some(Kind::Statements), Some(Kind::Parentheses)) => Context::Paren,
                // The sole argument of a paren-less command breaks inside the
                // parentheses the command then gains: `foo(\n if ... end\n)`.
                (Some(Kind::Arguments), Some(Kind::Command)) => Context::Paren,
                (Some(Kind::Statements | Kind::RescueModifier), _) => Context::Statement,
                _ => Context::Expression,
            };
            self.contexts.insert(span, context);
        }
        self.stack.push((kind, span));
    }
}

fn kind_of(node: &Node<'_>) -> Kind {
    match node {
        Node::StatementsNode { .. } => Kind::Statements,
        Node::ParenthesesNode { .. } => Kind::Parentheses,
        Node::RescueModifierNode { .. } => Kind::RescueModifier,
        Node::ArgumentsNode { .. } => Kind::Arguments,
        Node::CallNode { .. }
            if node.as_call_node().is_some_and(|call| {
                call.opening_loc().is_none()
                    && call
                        .arguments()
                        .is_some_and(|arguments| arguments.arguments().len() == 1)
            }) =>
        {
            Kind::Command
        }
        _ => Kind::Other,
    }
}

fn span_of(node: &Node<'_>) -> (usize, usize) {
    let loc = node.location();
    (loc.start_offset(), loc.end_offset())
}

impl State {
    pub fn analyze(root: &Node<'_>) -> Self {
        let mut walk = ContextWalk {
            stack: Vec::new(),
            contexts: HashMap::default(),
            assignment_starts: Vec::new(),
            node_count: 0,
        };
        walk.visit(root);
        walk.assignment_starts.sort_unstable();
        walk.assignment_starts.dedup();
        Self {
            contexts: walk.contexts,
            assignment_starts: walk.assignment_starts,
            node_count: walk.node_count,
            spaced_flip_flop: false,
        }
    }

    pub(super) fn node_count(&self) -> usize {
        self.node_count
    }
}

fn context_of(f: &Formatter<'_>, node: &Node<'_>) -> Context {
    *f.control
        .contexts
        .get(&span_of(node))
        .unwrap_or_else(|| panic!("no context recorded for node at {:?}", span_of(node)))
}

/// Prints a predicate `align` units in from the keyword's column.
fn predicate(f: &mut Formatter<'_>, node: &Node<'_>, align: usize, spaced_flip_flop: bool) {
    let previous = std::mem::replace(&mut f.control.spaced_flip_flop, spaced_flip_flop);
    calls::in_predicate(f, |f| {
        if align == 0 {
            f.node(node);
        } else {
            f.align(align, |f| f.node(node));
        }
    });
    f.control.spaced_flip_flop = previous;
}

/// The single statement of a body that has no comments of its own, which is
/// what the modifier and ternary forms can carry.
fn single_statement<'pr>(
    f: &Formatter<'_>,
    statements: &Option<StatementsNode<'pr>>,
    parent: &Node<'_>,
) -> Option<Node<'pr>> {
    let statements = statements.as_ref()?;
    if statements.body().len() != 1 || f.has_dangling_body(statements) || f.has_dangling(parent) {
        return None;
    }
    statements.body().iter().next()
}

/// A fresh handle on a statement list, which Prism does not let us clone.
fn refetch<'pr>(statements: &Option<StatementsNode<'pr>>) -> Option<StatementsNode<'pr>> {
    statements
        .as_ref()
        .map(|s| s.as_node().as_statements_node().expect("kind"))
}

fn parenthesized(f: &mut Formatter<'_>, needed: bool, inner: impl FnOnce(&mut Formatter<'_>)) {
    if needed {
        f.b.text("(");
    }
    inner(f);
    if needed {
        f.b.text(")");
    }
}

fn is_assignment(node: &Node<'_>) -> bool {
    match node {
        Node::LocalVariableWriteNode { .. }
        | Node::LocalVariableOperatorWriteNode { .. }
        | Node::LocalVariableAndWriteNode { .. }
        | Node::LocalVariableOrWriteNode { .. }
        | Node::InstanceVariableWriteNode { .. }
        | Node::InstanceVariableOperatorWriteNode { .. }
        | Node::InstanceVariableAndWriteNode { .. }
        | Node::InstanceVariableOrWriteNode { .. }
        | Node::ClassVariableWriteNode { .. }
        | Node::ClassVariableOperatorWriteNode { .. }
        | Node::ClassVariableAndWriteNode { .. }
        | Node::ClassVariableOrWriteNode { .. }
        | Node::GlobalVariableWriteNode { .. }
        | Node::GlobalVariableOperatorWriteNode { .. }
        | Node::GlobalVariableAndWriteNode { .. }
        | Node::GlobalVariableOrWriteNode { .. }
        | Node::ConstantWriteNode { .. }
        | Node::ConstantOperatorWriteNode { .. }
        | Node::ConstantAndWriteNode { .. }
        | Node::ConstantOrWriteNode { .. }
        | Node::ConstantPathWriteNode { .. }
        | Node::ConstantPathOperatorWriteNode { .. }
        | Node::ConstantPathAndWriteNode { .. }
        | Node::ConstantPathOrWriteNode { .. }
        | Node::CallOperatorWriteNode { .. }
        | Node::CallAndWriteNode { .. }
        | Node::CallOrWriteNode { .. }
        | Node::IndexOperatorWriteNode { .. }
        | Node::IndexAndWriteNode { .. }
        | Node::IndexOrWriteNode { .. }
        | Node::MultiWriteNode { .. } => true,
        Node::CallNode { .. } => {
            let call = node.as_call_node().expect("kind");
            call.is_attribute_write() || call.name().as_slice() == b"[]="
        }
        _ => false,
    }
}

/// An assignment anywhere in a predicate keeps the statement in block form.
fn contains_assignment(f: &Formatter<'_>, node: &Node<'_>) -> bool {
    let location = node.location();
    let first = f
        .control
        .assignment_starts
        .partition_point(|&start| start < location.start_offset());
    f.control
        .assignment_starts
        .get(first)
        .is_some_and(|&start| start < location.end_offset())
}

enum CallShape {
    Command,
    Binary,
    Not,
    Other,
}

fn call_shape(f: &Formatter<'_>, call: &CallNode<'_>) -> CallShape {
    let shape = calls::shape(f, call);
    if matches!(shape, calls::Shape::Unary)
        && call
            .message_loc()
            .is_some_and(|location| &f.source[location.start_offset()..location.end_offset()] == b"not")
    {
        return CallShape::Not;
    }
    match shape {
        calls::Shape::Binary => CallShape::Binary,
        _ if calls::is_command(f, call) => CallShape::Command,
        _ => CallShape::Other,
    }
}

/// Whether `if p ... else ... end` may print as `p ? a : b`: the predicate
/// must bind tighter than `?`.
fn ternary_predicate_ok(f: &Formatter<'_>, node: &Node<'_>) -> bool {
    if is_assignment(node) {
        return false;
    }
    match node {
        // Ruby does not parse a bare pattern predicate in `a in b ? c : d`.
        Node::AndNode { .. }
        | Node::OrNode { .. }
        | Node::RescueModifierNode { .. }
        | Node::MatchPredicateNode { .. }
        | Node::MatchRequiredNode { .. } => false,
        Node::CallNode { .. } => matches!(call_shape(f, &node.as_call_node().expect("kind")), CallShape::Other),
        _ => true,
    }
}

/// The expressions that survive as a ternary branch: no keywords, no
/// assignments, no paren-less arguments.
fn ternary_branch_ok(f: &Formatter<'_>, node: &Node<'_>) -> bool {
    if f.contains_heredoc(node) {
        return false;
    }
    match node {
        Node::CallNode { .. } => {
            let call = node.as_call_node().expect("kind");
            !is_assignment(node) && !matches!(call_shape(f, &call), CallShape::Command)
        }
        Node::AndNode { .. } => f.slice(&node.as_and_node().expect("kind").operator_loc()) == "&&",
        Node::OrNode { .. } => f.slice(&node.as_or_node().expect("kind").operator_loc()) == "||",
        Node::IntegerNode { .. }
        | Node::FloatNode { .. }
        | Node::RationalNode { .. }
        | Node::ImaginaryNode { .. }
        | Node::StringNode { .. }
        | Node::InterpolatedStringNode { .. }
        | Node::XStringNode { .. }
        | Node::InterpolatedXStringNode { .. }
        | Node::SymbolNode { .. }
        | Node::InterpolatedSymbolNode { .. }
        | Node::RegularExpressionNode { .. }
        | Node::InterpolatedRegularExpressionNode { .. }
        | Node::ArrayNode { .. }
        | Node::HashNode { .. }
        | Node::RangeNode { .. }
        | Node::NilNode { .. }
        | Node::TrueNode { .. }
        | Node::FalseNode { .. }
        | Node::SelfNode { .. }
        | Node::SourceFileNode { .. }
        | Node::SourceLineNode { .. }
        | Node::SourceEncodingNode { .. }
        | Node::LocalVariableReadNode { .. }
        | Node::InstanceVariableReadNode { .. }
        | Node::ClassVariableReadNode { .. }
        | Node::GlobalVariableReadNode { .. }
        | Node::ConstantReadNode { .. }
        | Node::ConstantPathNode { .. }
        | Node::BackReferenceReadNode { .. }
        | Node::NumberedReferenceReadNode { .. }
        | Node::ItLocalVariableReadNode { .. }
        | Node::ParenthesesNode { .. } => true,
        _ => false,
    }
}

/// A branch that keeps a too-wide ternary a ternary, broken after `?` and
/// `:`, instead of expanding it to `if`/`else`: a bare nested ternary, an
/// assignment or a lambda.
fn ternary_only(node: &Node<'_>) -> bool {
    match node {
        Node::IfNode { .. } => node.as_if_node().expect("kind").if_keyword_loc().is_none(),
        Node::LambdaNode { .. } => true,
        _ => is_assignment(node),
    }
}

/// A ternary branch; `not x` needs its parentheses there.
fn ternary_branch(f: &mut Formatter<'_>, node: &Node<'_>) {
    if let Some(call) = node.as_call_node()
        && matches!(call_shape(f, &call), CallShape::Not)
        && call.opening_loc().is_none()
    {
        f.b.text("not(");
        f.node(&call.receiver().expect("not has a receiver"));
        f.b.text(")");
        return;
    }
    f.node(node);
}

/// Whether `if p; s; end` may print as `s if p`. A conditional written as
/// a modifier stays one whatever its body; a block form keeps a body that is
/// itself a conditional.
fn modifier_ok(f: &Formatter<'_>, form: SourceForm, predicate: &Node<'_>, statement: &Node<'_>) -> bool {
    (form == SourceForm::Modifier || !matches!(statement, Node::IfNode { .. } | Node::UnlessNode { .. }))
        && !contains_assignment(f, predicate)
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum SourceForm {
    Block,
    Modifier,
    Ternary,
}

/// Everything shared by `if` and `unless`.
struct Conditional<'pr> {
    node: Node<'pr>,
    keyword: &'static str,
    predicate: Node<'pr>,
    statements: Option<StatementsNode<'pr>>,
    /// An `ElseNode`, or the `elsif` `IfNode`.
    subsequent: Option<Node<'pr>>,
    form: SourceForm,
}

const IF_ALIGN: usize = "if ".len();
const UNLESS_ALIGN: usize = "unless ".len();
// The predicate aligns with the predicate of the corresponding `if`.
const ELSIF_ALIGN: usize = "elsif ".len() - 2;

pub fn if_node(f: &mut Formatter<'_>, node: &IfNode<'_>) {
    if let Some(loc) = node.if_keyword_loc()
        && f.slice(&loc) == "elsif"
    {
        elsif_clause(f, node);
        return;
    }
    let form = if node.if_keyword_loc().is_none() {
        SourceForm::Ternary
    } else if node.end_keyword_loc().is_none() {
        SourceForm::Modifier
    } else {
        SourceForm::Block
    };
    conditional(
        f,
        Conditional {
            node: node.as_node(),
            keyword: "if",
            predicate: node.predicate(),
            statements: node.statements(),
            subsequent: node.subsequent(),
            form,
        },
    );
}

pub fn unless_node(f: &mut Formatter<'_>, node: &UnlessNode<'_>) {
    let form = if node.end_keyword_loc().is_none() {
        SourceForm::Modifier
    } else {
        SourceForm::Block
    };
    conditional(
        f,
        Conditional {
            node: node.as_node(),
            keyword: "unless",
            predicate: node.predicate(),
            statements: node.statements(),
            subsequent: node.else_clause().map(|e| e.as_node()),
            form,
        },
    );
}

fn conditional(f: &mut Formatter<'_>, c: Conditional<'_>) {
    let context = context_of(f, &c.node);
    let needs_parens = context == Context::Expression;
    let statement = single_statement(f, &c.statements, &c.node);
    let else_clause = c.subsequent.as_ref().and_then(Node::as_else_node);
    let else_statement = else_clause
        .as_ref()
        .and_then(|e| single_statement(f, &e.statements(), &e.as_node()));

    if c.form == SourceForm::Ternary {
        // Only its own layout can be too wide: the branches are expressions.
        let (statement, else_statement) = (
            statement.expect("ternary has a branch"),
            else_statement.expect("ternary has an else branch"),
        );
        if ternary_only(&statement) || ternary_only(&else_statement) {
            f.group(|f| {
                predicate(f, &c.predicate, 0, true);
                f.b.text(" ?");
                f.indent(|f| {
                    f.b.line(SPACE);
                    ternary_branch(f, &statement);
                    f.b.text(" :");
                    f.b.line(SPACE);
                    ternary_branch(f, &else_statement);
                });
            });
            return;
        }
        f.group(|f| {
            f.if_break(
                |f| {
                    if needs_parens {
                        f.b.text("(");
                        f.indent(|f| {
                            f.b.line(SPACE);
                            block_form(f, &c, Some(&statement), Some(&else_statement));
                        });
                        f.b.line(SPACE);
                        f.b.text(")");
                    } else {
                        block_form(f, &c, Some(&statement), Some(&else_statement));
                    }
                },
                |f| ternary(f, &c, &statement, &else_statement, false),
            );
        });
        return;
    }

    match (&c.subsequent, &statement) {
        // A modifier whose statement assigns anywhere stays one whatever its
        // width: the statement lays out its own breaks and the predicate follows.
        (None, Some(statement)) if c.form == SourceForm::Modifier && contains_assignment(f, statement) => {
            modifier_form(f, c.keyword, statement, &c.predicate, needs_parens, true);
        }
        (None, Some(statement)) if modifier_ok(f, c.form, &c.predicate, statement) => {
            f.group(|f| {
                f.if_break(
                    |f| block_form(f, &c, Some(statement), None),
                    |f| modifier_form(f, c.keyword, statement, &c.predicate, needs_parens, true),
                );
            });
        }
        (Some(_), Some(statement))
            if else_clause
                .as_ref()
                .is_some_and(|e| !f.has_header_comments(&e.as_node()))
                && context != Context::Paren
                && else_statement
                    .as_ref()
                    .is_some_and(|e| ternary_branch_ok(f, statement) && ternary_branch_ok(f, e))
                && ternary_predicate_ok(f, &c.predicate) =>
        {
            let else_statement = else_statement.as_ref().expect("checked above");
            f.group(|f| {
                f.if_break(
                    |f| block_form(f, &c, Some(statement), Some(else_statement)),
                    |f| ternary(f, &c, statement, else_statement, needs_parens),
                );
            });
        }
        _ => block_form(f, &c, None, None),
    }
}

/// `s if p`.
fn modifier_form(
    f: &mut Formatter<'_>,
    keyword: &'static str,
    statement: &Node<'_>,
    predicate_node: &Node<'_>,
    needs_parens: bool,
    spaced_flip_flop: bool,
) {
    parenthesized(f, needs_parens, |f| {
        f.node(statement);
        f.b.text(" ");
        f.b.text(keyword);
        f.b.text(" ");
        predicate(f, predicate_node, 0, spaced_flip_flop);
    });
}

/// `p ? a : b`, with the branches swapped for `unless`.
fn ternary(
    f: &mut Formatter<'_>,
    c: &Conditional<'_>,
    statement: &Node<'_>,
    else_statement: &Node<'_>,
    needs_parens: bool,
) {
    let (first, second) = if c.keyword == "unless" {
        (else_statement, statement)
    } else {
        (statement, else_statement)
    };
    parenthesized(f, needs_parens, |f| {
        predicate(f, &c.predicate, 0, true);
        f.b.text(" ? ");
        ternary_branch(f, first);
        f.b.text(" : ");
        ternary_branch(f, second);
    });
}

/// The keyword form. With `statement` (and `else_statement`) given, the
/// bodies are those single statements and the newlines are soft, so the
/// enclosing group can still choose the flat form; otherwise the real bodies
/// print with forced newlines.
fn block_form(
    f: &mut Formatter<'_>,
    c: &Conditional<'_>,
    statement: Option<&Node<'_>>,
    else_statement: Option<&Node<'_>>,
) {
    let soft = statement.is_some();
    let line = if soft { SPACE } else { HARD };
    let align = if c.keyword == "unless" { UNLESS_ALIGN } else { IF_ALIGN };
    f.b.text(c.keyword);
    f.b.text(" ");
    predicate(f, &c.predicate, align, true);
    match statement {
        Some(statement) => f.indent(|f| {
            f.b.line(SPACE);
            f.node(statement);
        }),
        None => f.body_of(refetch(&c.statements), &c.node),
    }
    if let Some(subsequent) = &c.subsequent {
        f.b.line(line);
        match else_statement {
            Some(else_statement) => {
                f.b.text("else");
                f.indent(|f| {
                    f.b.line(SPACE);
                    f.node(else_statement);
                });
            }
            None => f.node(subsequent),
        }
    }
    f.b.line(line);
    f.b.text("end");
}

fn elsif_clause(f: &mut Formatter<'_>, node: &IfNode<'_>) {
    f.b.text("elsif ");
    // An `elsif` predicate is free to use `do`/`end`: nothing else can claim it.
    let previous = std::mem::replace(&mut f.control.spaced_flip_flop, false);
    f.align(ELSIF_ALIGN, |f| f.node(&node.predicate()));
    f.control.spaced_flip_flop = previous;
    f.body_of(node.statements(), &node.as_node());
    if let Some(subsequent) = node.subsequent() {
        f.b.line(HARD);
        f.node(&subsequent);
    }
}

pub fn else_node(f: &mut Formatter<'_>, node: &ElseNode<'_>) {
    f.b.text("else");
    f.body_of(node.statements(), &node.as_node());
}

pub fn while_node(f: &mut Formatter<'_>, node: &WhileNode<'_>) {
    let form = loop_form(node.is_begin_modifier(), node.closing_loc().is_some());
    loop_node(f, &node.as_node(), "while", &node.predicate(), &node.statements(), form);
}

pub fn until_node(f: &mut Formatter<'_>, node: &UntilNode<'_>) {
    let form = loop_form(node.is_begin_modifier(), node.closing_loc().is_some());
    loop_node(f, &node.as_node(), "until", &node.predicate(), &node.statements(), form);
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum LoopForm {
    Block,
    Modifier,
    /// `begin ... end while p`.
    BeginModifier,
}

fn loop_form(begin_modifier: bool, closed: bool) -> LoopForm {
    if begin_modifier {
        LoopForm::BeginModifier
    } else if closed {
        LoopForm::Block
    } else {
        LoopForm::Modifier
    }
}

const LOOP_ALIGN: usize = "while ".len();

fn loop_node(
    f: &mut Formatter<'_>,
    node: &Node<'_>,
    keyword: &'static str,
    predicate_node: &Node<'_>,
    statements: &Option<StatementsNode<'_>>,
    form: LoopForm,
) {
    if form == LoopForm::BeginModifier {
        // `begin ... end while p` runs the body first and always keeps its shape.
        let statements = statements.as_ref().expect("begin modifier has a body");
        let body: Vec<Node<'_>> = statements.body().iter().collect();
        let [begin] = body.as_slice() else {
            panic!("begin modifier body is not a single begin block")
        };
        f.node(begin);
        f.b.text(" ");
        f.b.text(keyword);
        f.b.text(" ");
        predicate(f, predicate_node, 0, false);
        return;
    }
    let block = |f: &mut Formatter<'_>, statement: Option<&Node<'_>>| {
        f.b.text(keyword);
        f.b.text(" ");
        predicate(f, predicate_node, LOOP_ALIGN, false);
        match statement {
            Some(statement) => {
                f.indent(|f| {
                    f.b.line(SPACE);
                    f.node(statement);
                });
                f.b.line(SPACE);
            }
            None => {
                f.body_of(refetch(statements), node);
                f.b.line(HARD);
            }
        }
        f.b.text("end");
    };
    match single_statement(f, statements, node) {
        Some(statement) if form == LoopForm::Modifier && contains_assignment(f, &statement) => {
            let needs_parens = context_of(f, node) == Context::Expression;
            modifier_form(f, keyword, &statement, predicate_node, needs_parens, false);
        }
        Some(statement) if !contains_assignment(f, predicate_node) => {
            let needs_parens = context_of(f, node) == Context::Expression;
            f.group(|f| {
                f.if_break(
                    |f| block(f, Some(&statement)),
                    |f| modifier_form(f, keyword, &statement, predicate_node, needs_parens, false),
                );
            });
        }
        _ => block(f, None),
    }
}

pub fn for_node(f: &mut Formatter<'_>, node: &ForNode<'_>) {
    f.b.text("for ");
    // `for a, b in c`: the targets fill the line and break without indent.
    f.group(|f| f.node(&node.index()));
    f.b.text(" in ");
    f.node(&node.collection());
    f.body_of(node.statements(), &node.as_node());
    f.b.line(HARD);
    f.b.text("end");
}

/// Own-line comments a clause-bearing node holds between its clauses, in
/// source order, split by the clause they precede.
fn comments_before(dangling: &[Comment], offset: usize, printed: &mut usize) -> Vec<Comment> {
    let mut out = Vec::new();
    while *printed < dangling.len() && dangling[*printed].start < offset {
        out.push(dangling[*printed]);
        *printed += 1;
    }
    out
}

/// Comments after a clause's body, indented like it; a blank line in the
/// source before any of them survives, as it would inside the body.
fn indented_comments(f: &mut Formatter<'_>, comments: &[Comment], mut previous_end: usize) {
    if comments.is_empty() {
        return;
    }
    f.indent(|f| {
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

pub fn case_node(f: &mut Formatter<'_>, node: &CaseNode<'_>) {
    f.b.text("case");
    if let Some(predicate) = node.predicate() {
        f.b.text(" ");
        f.node(&predicate);
    }
    let dangling = f.dangling(&node.as_node());
    let mut printed = 0;
    let mut previous_end = None;
    let clauses: Vec<Node<'_>> = node
        .conditions()
        .iter()
        .chain(node.else_clause().map(|e| e.as_node()))
        .collect();
    for clause in &clauses {
        let comments = comments_before(&dangling, clause.location().start_offset(), &mut printed);
        match previous_end {
            // Before the first `when` there is no body to belong to.
            None => {
                for comment in &comments {
                    f.b.line(HARD);
                    f.comment(comment);
                }
            }
            Some(previous_end) => indented_comments(f, &comments, previous_end),
        }
        previous_end = Some(clause.location().end_offset());
        f.b.line(HARD);
        f.node(clause);
    }
    if let Some(previous_end) = previous_end {
        indented_comments(f, &dangling[printed..], previous_end);
    }
    f.b.line(HARD);
    f.b.text("end");
}

const WHEN_ALIGN: usize = "when ".len();

/// `when a, b, c`: as many conditions per line as fit, except that a comment
/// trailing the last one puts every condition on its own line.
pub fn when_node(f: &mut Formatter<'_>, node: &WhenNode<'_>) {
    f.b.text("when ");
    let conditions: Vec<Node<'_>> = node.conditions().iter().collect();
    let (last, rest) = conditions.split_last().expect("when has a condition");
    f.group(|f| {
        f.align(WHEN_ALIGN, |f| {
            f.b.push_target();
            f.node(last);
            let last_docs = f.b.pop_target();
            let commented = f.b.any(
                last_docs,
                |d| matches!(d, Doc::LineSuffix { priority, .. } if *priority == COMMENT_PRIORITY),
            );
            for condition in rest {
                f.node(condition);
                f.b.text(",");
                if commented {
                    f.b.line(HARD);
                } else {
                    f.group(|f| f.b.line(SPACE));
                }
            }
            f.b.append(last_docs);
        });
    });
    f.body_of(node.statements(), &node.as_node());
}

/// `begin`/`rescue`/`else`/`ensure`/`end`. Without the `begin` keyword this
/// is a block or method body: the statements print indented and the clause
/// keywords at the caller's column, and the caller prints `end`.
pub fn begin_node(f: &mut Formatter<'_>, node: &BeginNode<'_>) {
    let explicit = node.begin_keyword_loc().is_some();
    let dangling = f.dangling(&node.as_node());
    let mut printed = 0;
    let rescue = node.rescue_clause();
    let else_clause = node.else_clause();
    let ensure = node.ensure_clause();
    let end_offset = node
        .end_keyword_loc()
        .map_or(node.location().end_offset(), |l| l.start_offset());
    let first_clause = [
        rescue.as_ref().map(|r| r.location().start_offset()),
        else_clause.as_ref().map(|e| e.location().start_offset()),
        ensure.as_ref().map(|e| e.location().start_offset()),
    ]
    .into_iter()
    .flatten()
    .min()
    .unwrap_or(end_offset);

    if explicit {
        f.b.text("begin");
    }
    let body_comments = comments_before(&dangling, first_clause, &mut printed);
    let body_end = node
        .statements()
        .map_or(node.location().start_offset(), |s| s.location().end_offset());
    match node.statements() {
        Some(statements) => {
            f.body(Some(statements));
            indented_comments(f, &body_comments, body_end);
        }
        None if !body_comments.is_empty() => indented_comments(f, &body_comments, body_end),
        None if !explicit => f.indent(|f| f.b.line(HARD)),
        None => {}
    }
    let mut previous_end = body_end;
    if let Some(rescue) = &rescue {
        f.b.line(HARD);
        f.node(&rescue.as_node());
        previous_end = rescue.location().end_offset();
        let next = else_clause
            .as_ref()
            .map(|e| e.location().start_offset())
            .or(ensure.as_ref().map(|e| e.location().start_offset()))
            .unwrap_or(end_offset);
        indented_comments(f, &comments_before(&dangling, next, &mut printed), previous_end);
    }
    if let Some(else_clause) = &else_clause {
        f.b.line(HARD);
        f.node(&else_clause.as_node());
        previous_end = else_clause.location().end_offset();
        let next = ensure.as_ref().map_or(end_offset, |e| e.location().start_offset());
        indented_comments(f, &comments_before(&dangling, next, &mut printed), previous_end);
    }
    if let Some(ensure) = &ensure {
        f.b.line(HARD);
        f.node(&ensure.as_node());
        previous_end = ensure.location().end_offset();
    }
    indented_comments(f, &dangling[printed..], previous_end);
    if explicit {
        f.b.line(HARD);
        f.b.text("end");
    }
}

const RESCUE_ALIGN: usize = "rescue ".len();

pub fn rescue_node(f: &mut Formatter<'_>, node: &RescueNode<'_>) {
    f.b.text("rescue");
    let exceptions: Vec<Node<'_>> = node.exceptions().iter().collect();
    let reference = node.reference();
    if exceptions.is_empty() && reference.is_none() && f.options.explicit_standard_error {
        f.b.text(" StandardError");
    }
    if let Some((last, rest)) = exceptions.split_last() {
        // A comment trailing the last exception stays on the keyword line
        // instead of breaking the list, so it is printed outside the group.
        f.b.push_target();
        f.node(last);
        let docs = f.b.pop_target();
        let (comments, last_docs) = f.b.partition(
            docs,
            |d| matches!(d, Doc::LineSuffix { priority, .. } if *priority == COMMENT_PRIORITY),
        );
        f.b.text(" ");
        f.group(|f| {
            f.align(RESCUE_ALIGN, |f| {
                for exception in rest {
                    f.node(exception);
                    f.b.text(",");
                    f.b.line(SPACE);
                }
                f.b.append(last_docs);
            });
        });
        f.b.append(comments);
    }
    if let Some(reference) = reference {
        f.b.text(" => ");
        f.node(&reference);
    }
    f.body_of(node.statements(), &node.as_node());
    if let Some(subsequent) = node.subsequent() {
        f.b.line(HARD);
        f.node(&subsequent.as_node());
    }
}

pub fn ensure_node(f: &mut Formatter<'_>, node: &EnsureNode<'_>) {
    f.b.text("ensure");
    f.body_of(node.statements(), &node.as_node());
}

/// `x rescue y` always expands to the block form.
pub fn rescue_modifier_node(f: &mut Formatter<'_>, node: &RescueModifierNode<'_>) {
    f.b.text("begin");
    f.indent(|f| {
        f.b.line(HARD);
        f.node(&node.expression());
    });
    f.b.line(HARD);
    f.b.text("rescue");
    if f.options.explicit_standard_error {
        f.b.text(" StandardError");
    }
    f.indent(|f| {
        f.b.line(HARD);
        f.node(&node.rescue_expression());
    });
    f.b.line(HARD);
    f.b.text("end");
}

pub fn return_node(f: &mut Formatter<'_>, node: &ReturnNode<'_>) {
    jump(f, "return", node.arguments());
}

pub fn break_node(f: &mut Formatter<'_>, node: &BreakNode<'_>) {
    jump(f, "break", node.arguments());
}

pub fn next_node(f: &mut Formatter<'_>, node: &NextNode<'_>) {
    jump(f, "next", node.arguments());
}

/// `return a` gains parentheses when it breaks; `return a, b` (or a literal
/// array of several elements) becomes a bracketed list without a trailing
/// comma.
fn jump(f: &mut Formatter<'_>, keyword: &'static str, arguments: Option<ArgumentsNode<'_>>) {
    f.b.text(keyword);
    let Some(arguments) = arguments else {
        return;
    };
    let arguments: Vec<Node<'_>> = arguments.arguments().iter().collect();
    calls::with_source_keys(f, |f| jump_arguments(f, &arguments));
}

fn jump_arguments(f: &mut Formatter<'_>, arguments: &[Node<'_>]) {
    let bracketed = |f: &mut Formatter<'_>, elements: &[Node<'_>]| {
        f.group(|f| {
            f.if_break(|f| f.b.text(" ["), |f| f.b.text(" "));
            f.indent(|f| {
                f.b.line(SOFT);
                let mut first = true;
                for element in elements {
                    if !first {
                        f.b.text(",");
                        f.b.line(SPACE);
                    }
                    first = false;
                    f.node(element);
                }
            });
            f.b.line(SOFT);
            f.if_break(|f| f.b.text("]"), |_| {});
        });
    };
    match arguments {
        [single] if matches!(single, Node::ParenthesesNode { .. }) => f.node(single),
        [single]
            if single
                .as_array_node()
                .is_some_and(|a| super::calls::prints_bracketed(f, &a)) =>
        {
            let elements: Vec<Node<'_>> = single.as_array_node().expect("kind").elements().iter().collect();
            if elements.len() >= 2 {
                bracketed(f, &elements);
            } else {
                f.b.text(" ");
                f.node(single);
            }
        }
        [single] => f.group(|f| {
            f.if_break(|f| f.b.text("("), |f| f.b.text(" "));
            f.indent(|f| {
                f.b.line(SOFT);
                f.node(single);
            });
            f.b.line(SOFT);
            f.if_break(|f| f.b.text(")"), |_| {});
        }),
        many => bracketed(f, many),
    }
}

pub fn redo_node(f: &mut Formatter<'_>, _node: &RedoNode<'_>) {
    f.b.text("redo");
}

pub fn retry_node(f: &mut Formatter<'_>, _node: &RetryNode<'_>) {
    f.b.text("retry");
}

/// `a && b`: the right operand breaks onto an indented line by itself, with
/// no group around the pair, so a chain fills each line before breaking.
fn boolean(f: &mut Formatter<'_>, left: &Node<'_>, operator: &Location<'_>, right: &Node<'_>) {
    f.node(left);
    f.b.text(" ");
    f.text_of(operator);
    f.group(|f| {
        f.indent(|f| {
            f.b.line(SPACE);
            f.node(right);
        })
    });
}

pub fn and_node(f: &mut Formatter<'_>, node: &AndNode<'_>) {
    boolean(f, &node.left(), &node.operator_loc(), &node.right());
}

pub fn or_node(f: &mut Formatter<'_>, node: &OrNode<'_>) {
    boolean(f, &node.left(), &node.operator_loc(), &node.right());
}

pub fn parentheses_node(f: &mut Formatter<'_>, node: &ParenthesesNode<'_>) {
    let previous = std::mem::replace(&mut f.control.spaced_flip_flop, false);
    calls::outside_predicate(f, |f| parentheses_contents(f, node));
    f.control.spaced_flip_flop = previous;
}

fn parentheses_contents(f: &mut Formatter<'_>, node: &ParenthesesNode<'_>) {
    match node.body() {
        None => {
            let dangling = f.dangling(&node.as_node());
            if dangling.is_empty() {
                f.b.text("()");
            } else {
                // Empty parentheses keep a blank line after body comments.
                f.b.text("(");
                f.indent(|f| {
                    for comment in &dangling {
                        f.b.line(HARD);
                        f.comment(comment);
                    }
                    f.b.line(HARD);
                });
                f.b.line(HARD);
                f.b.text(")");
            }
        }
        Some(body) => f.group(|f| {
            f.b.text("(");
            f.indent(|f| {
                f.b.line(SOFT);
                f.node(&body);
            });
            f.b.line(SOFT);
            f.b.text(")");
        }),
    }
}

pub fn flip_flop_node(f: &mut Formatter<'_>, node: &FlipFlopNode<'_>) {
    let spaced = f.control.spaced_flip_flop;
    if let Some(left) = node.left() {
        f.node(&left);
        if spaced {
            f.b.text(" ");
        }
    }
    f.text_of(&node.operator_loc());
    if let Some(right) = node.right() {
        if spaced {
            f.b.text(" ");
        }
        f.node(&right);
    }
}

pub fn defined_node(f: &mut Formatter<'_>, node: &DefinedNode<'_>) {
    f.b.text("defined?(");
    f.group(|f| {
        f.indent(|f| {
            f.b.line(SOFT);
            f.node(&node.value());
        });
        f.b.line(SOFT);
    });
    f.b.text(")");
}

fn execution_hook(
    f: &mut Formatter<'_>,
    keyword: &'static str,
    statements: Option<StatementsNode<'_>>,
    node: &Node<'_>,
) {
    f.b.text(keyword);
    f.b.text(" {");
    f.group(|f| {
        f.indent(|f| {
            f.b.line(SPACE);
            match statements {
                Some(statements) => {
                    f.statements(&statements);
                }
                None => {
                    let dangling = f.dangling(node);
                    let mut first = true;
                    for comment in &dangling {
                        if !first {
                            f.b.line(HARD);
                        }
                        first = false;
                        f.comment(comment);
                    }
                    if !dangling.is_empty() {
                        f.b.break_parent();
                    }
                }
            }
        });
        f.b.line(SPACE);
        f.b.text("}");
    });
}

pub fn pre_execution_node(f: &mut Formatter<'_>, node: &PreExecutionNode<'_>) {
    execution_hook(f, "BEGIN", node.statements(), &node.as_node());
}

pub fn post_execution_node(f: &mut Formatter<'_>, node: &PostExecutionNode<'_>) {
    execution_hook(f, "END", node.statements(), &node.as_node());
}
