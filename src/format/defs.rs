//! Formatting for definitions: `def`, `class`, `module`, `class << self`,
//! lambdas, parameter lists (method, block, and lambda), `undef` and `alias`.

use ruby_prism::{
    AliasGlobalVariableNode, AliasMethodNode, BlockLocalVariableNode, BlockParameterNode, BlockParametersNode,
    ClassNode, DefNode, ForwardingParameterNode, ItParametersNode, KeywordRestParameterNode, LambdaNode, ModuleNode,
    MultiTargetNode, NoKeywordsParameterNode, Node, NumberedParametersNode, OptionalKeywordParameterNode,
    OptionalParameterNode, ParametersNode, RequiredKeywordParameterNode, RequiredParameterNode, RestParameterNode,
    SingletonClassNode, UndefNode,
};

use super::Formatter;
use crate::doc::{HARD, SOFT, SPACE};

/// How a parameter list separates its entries.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Separator {
    /// `, ` with a possible break: method and lambda parameters, which break
    /// one per line when the list overflows.
    Breakable,
    /// Literal `, `: block parameters between pipes never break, whatever
    /// the enclosing block does.
    Flat,
}

fn separator(f: &mut Formatter<'_>, separator: Separator) {
    f.b.text(",");
    match separator {
        Separator::Breakable => f.b.line(SPACE),
        Separator::Flat => f.b.text(" "),
    }
}

/// `(` entries `)` that break one per line, indented, when they overflow.
fn breakable_parens(f: &mut Formatter<'_>, entries: impl FnOnce(&mut Formatter<'_>)) {
    f.group(|f| {
        f.b.text("(");
        f.indent(|f| {
            f.b.line(SOFT);
            entries(f);
        });
        f.b.line(SOFT);
        f.b.text(")");
    });
}

pub fn alias_global_variable_node(f: &mut Formatter<'_>, node: &AliasGlobalVariableNode<'_>) {
    f.b.text("alias ");
    f.node(&node.new_name());
    f.b.text(" ");
    f.node(&node.old_name());
}

pub fn alias_method_node(f: &mut Formatter<'_>, node: &AliasMethodNode<'_>) {
    f.b.text("alias ");
    method_name(f, &node.new_name());
    f.b.text(" ");
    method_name(f, &node.old_name());
}

/// A method name in `alias`/`undef`: a bare or `:`-prefixed symbol prints
/// as the bare name; anything quoted or interpolated prints as a symbol.
fn method_name(f: &mut Formatter<'_>, name: &Node<'_>) {
    if let Some(symbol) = name.as_symbol_node() {
        let bare = symbol.opening_loc().is_none_or(|opening| f.slice(&opening) == ":");
        if bare {
            let value = symbol.value_loc().expect("bare symbol has a value");
            f.text_of(&value);
            return;
        }
    }
    f.node(name);
}

pub fn block_local_variable_node(f: &mut Formatter<'_>, node: &BlockLocalVariableNode<'_>) {
    f.text_of(&node.location());
}

pub fn block_parameter_node(f: &mut Formatter<'_>, node: &BlockParameterNode<'_>) {
    f.b.text("&");
    if let Some(name) = node.name_loc() {
        f.text_of(&name);
    }
}

/// `|a, b; c|` on a block, or `(a, b; c)` on a lambda, told apart by the
/// opening delimiter. A lambda's parameters break like a method's; a
/// block's stay on the block's opening line.
pub fn block_parameters_node(f: &mut Formatter<'_>, node: &BlockParametersNode<'_>) {
    let pipes = node.opening_loc().is_some_and(|opening| f.slice(&opening) == "|");
    if pipes {
        f.b.text("|");
        block_parameters_contents(f, node, Separator::Flat);
        f.b.text("|");
    } else {
        breakable_parens(f, |f| block_parameters_contents(f, node, Separator::Breakable));
    }
}

fn block_parameters_contents(f: &mut Formatter<'_>, node: &BlockParametersNode<'_>, separator: Separator) {
    if let Some(parameters) = node.parameters() {
        parameter_list(f, &parameters, separator);
    }
    // Block locals stay on one line even when a lambda's parameters break.
    let locals = node.locals();
    if !locals.is_empty() {
        f.b.text("; ");
        separated(f, locals.iter(), Separator::Flat);
    }
}

fn has_parameters_or_locals(node: &BlockParametersNode<'_>) -> bool {
    node.parameters().is_some() || !node.locals().is_empty()
}

pub fn class_node(f: &mut Formatter<'_>, node: &ClassNode<'_>) {
    f.b.text("class ");
    f.node(&node.constant_path());
    if let Some(superclass) = node.superclass() {
        f.b.text(" < ");
        f.node(&superclass);
    }
    definition_body(f, node.body(), &node.as_node());
    f.b.line(HARD);
    f.b.text("end");
}

pub fn def_node(f: &mut Formatter<'_>, node: &DefNode<'_>) {
    f.b.text("def ");
    if let Some(receiver) = node.receiver() {
        f.node(&receiver);
        f.b.text(".");
    }
    f.text_of(&node.name_loc());
    match node.parameters() {
        Some(parameters) => f.node(&parameters.as_node()),
        // `def foo()` keeps its empty parens; `def foo` stays bare.
        None if node.lparen_loc().is_some() => f.b.text("()"),
        None => {}
    }
    if node.equal_loc().is_some() {
        f.b.text(" =");
        f.group(|f| {
            f.indent(|f| {
                f.b.line(SPACE);
                if let Some(body) = node.body() {
                    f.node(&body);
                }
            });
        });
        return;
    }
    definition_body(f, node.body(), &node.as_node());
    f.b.line(HARD);
    f.b.text("end");
}

/// The body of a `def`/`class`/`module`: plain statements are indented on
/// their own lines (own-line comments in an empty body included); a body
/// with `rescue`/`ensure` clauses is a `BeginNode` that lays itself out,
/// clauses at the keyword's own indentation.
fn definition_body(f: &mut Formatter<'_>, body: Option<Node<'_>>, parent: &Node<'_>) {
    match body {
        Some(body) => match body.as_statements_node() {
            Some(statements) => f.body_of(Some(statements), parent),
            None => f.node(&body),
        },
        None => f.body_of(None, parent),
    }
}

pub fn forwarding_parameter_node(f: &mut Formatter<'_>, _node: &ForwardingParameterNode<'_>) {
    f.b.text("...");
}

pub fn it_parameters_node(_f: &mut Formatter<'_>, _node: &ItParametersNode<'_>) {}

pub fn keyword_rest_parameter_node(f: &mut Formatter<'_>, node: &KeywordRestParameterNode<'_>) {
    f.b.text("**");
    if let Some(name) = node.name_loc() {
        f.text_of(&name);
    }
}

/// `-> { }` / `->(a) { }`, with `do`/`end` whenever the group breaks. The
/// source's own choice of delimiters is ignored, as is an empty `()`.
pub fn lambda_node(f: &mut Formatter<'_>, node: &LambdaNode<'_>) {
    f.group(|f| {
        f.b.text("->");
        // A comment on the opener line (`-> do # c`, `->(a) { # c`) heads
        // the lambda or trails its parameters; either way the closing
        // delimiter needs a line of its own.
        let mut commented = f.has_header_comments(&node.as_node());
        if let Some(parameters) = node.parameters() {
            match parameters.as_block_parameters_node() {
                Some(block_parameters) if has_parameters_or_locals(&block_parameters) => {
                    f.node(&parameters);
                    commented |= f.has_trailing_comments(&parameters);
                }
                // `it` and `_1` are implicit; an empty `()` is dropped.
                _ => {}
            }
        }
        f.b.text(" ");
        let dangling = f.dangling(&node.as_node());
        let bare = dangling.is_empty() && !commented;
        match node.body() {
            None if bare => f.b.text("{}"),
            body => {
                f.if_break(|f| f.b.text("do"), |f| f.b.text("{"));
                match body {
                    Some(body) => match body.as_statements_node() {
                        Some(statements) => {
                            f.indent(|f| {
                                f.b.line(SPACE);
                                f.statements(&statements);
                            });
                            f.b.line(SPACE);
                        }
                        None => {
                            f.b.break_parent();
                            f.node(&body);
                            f.b.line(HARD);
                        }
                    },
                    None => {
                        f.body_of(None, &node.as_node());
                        f.b.line(HARD);
                    }
                }
                f.if_break(|f| f.b.text("end"), |f| f.b.text("}"));
            }
        }
    });
}

pub fn module_node(f: &mut Formatter<'_>, node: &ModuleNode<'_>) {
    f.b.text("module ");
    f.node(&node.constant_path());
    definition_body(f, node.body(), &node.as_node());
    f.b.line(HARD);
    f.b.text("end");
}

pub fn no_keywords_parameter_node(f: &mut Formatter<'_>, _node: &NoKeywordsParameterNode<'_>) {
    f.b.text("**nil");
}

pub fn numbered_parameters_node(_f: &mut Formatter<'_>, _node: &NumberedParametersNode<'_>) {}

pub fn optional_keyword_parameter_node(f: &mut Formatter<'_>, node: &OptionalKeywordParameterNode<'_>) {
    f.text_of(&node.name_loc());
    f.b.text(" ");
    f.node(&node.value());
}

pub fn optional_parameter_node(f: &mut Formatter<'_>, node: &OptionalParameterNode<'_>) {
    f.text_of(&node.name_loc());
    f.b.text(" = ");
    f.node(&node.value());
}

/// Method parameters, always parenthesised, one per line when they overflow
/// and never with a trailing comma. Block and lambda parameter lists reuse
/// the entries through [`BlockParametersNode`].
pub fn parameters_node(f: &mut Formatter<'_>, node: &ParametersNode<'_>) {
    breakable_parens(f, |f| parameter_list(f, node, Separator::Breakable));
}

/// Every parameter in source order. Prism stores them by kind, so the
/// order is recovered from their offsets. A bare trailing comma (`|a,|`)
/// is an implicit rest and prints as just the comma.
fn parameter_list(f: &mut Formatter<'_>, node: &ParametersNode<'_>, separator: Separator) {
    let mut entries: Vec<Node<'_>> = Vec::new();
    entries.extend(node.requireds().iter());
    entries.extend(node.optionals().iter());
    let mut implicit_rest = false;
    if let Some(rest) = node.rest() {
        if matches!(rest, Node::ImplicitRestNode { .. }) {
            implicit_rest = true;
        } else {
            entries.push(rest);
        }
    }
    entries.extend(node.posts().iter());
    entries.extend(node.keywords().iter());
    entries.extend(node.keyword_rest());
    entries.extend(node.block().map(|block| block.as_node()));
    entries.sort_by_key(|entry| entry.location().start_offset());
    separated(f, entries.into_iter(), separator);
    if implicit_rest {
        f.b.text(",");
    }
}

fn separated<'pr>(f: &mut Formatter<'_>, entries: impl Iterator<Item = Node<'pr>>, separator_kind: Separator) {
    for (index, entry) in entries.enumerate() {
        if index > 0 {
            separator(f, separator_kind);
        }
        parameter(f, &entry, separator_kind);
    }
}

/// A destructuring parameter `(a, (b, *c))` is a `MultiTargetNode`, whose
/// own formatter is the assignment family's; as a parameter it prints here
/// so its entries break like the list around them.
fn parameter(f: &mut Formatter<'_>, entry: &Node<'_>, separator_kind: Separator) {
    match entry.as_multi_target_node() {
        Some(target) => destructured_parameter(f, &target, separator_kind),
        None => f.node(entry),
    }
}

fn destructured_parameter(f: &mut Formatter<'_>, node: &MultiTargetNode<'_>, separator_kind: Separator) {
    let entries = || node.lefts().iter().chain(node.rest()).chain(node.rights().iter());
    match separator_kind {
        Separator::Breakable => breakable_parens(f, |f| separated(f, entries(), separator_kind)),
        Separator::Flat => {
            f.b.text("(");
            separated(f, entries(), separator_kind);
            f.b.text(")");
        }
    }
}

pub fn required_keyword_parameter_node(f: &mut Formatter<'_>, node: &RequiredKeywordParameterNode<'_>) {
    f.text_of(&node.name_loc());
}

pub fn required_parameter_node(f: &mut Formatter<'_>, node: &RequiredParameterNode<'_>) {
    f.text_of(&node.location());
}

pub fn rest_parameter_node(f: &mut Formatter<'_>, node: &RestParameterNode<'_>) {
    f.b.text("*");
    if let Some(name) = node.name_loc() {
        f.text_of(&name);
    }
}

/// `class << expr`. An empty body contains a blank line; own-line comments
/// take its place.
pub fn singleton_class_node(f: &mut Formatter<'_>, node: &SingletonClassNode<'_>) {
    f.b.text("class << ");
    f.node(&node.expression());
    let empty = node.body().is_none() && !f.has_dangling(&node.as_node());
    if empty {
        f.b.line(HARD);
    } else {
        definition_body(f, node.body(), &node.as_node());
    }
    f.b.line(HARD);
    f.b.text("end");
}

pub fn undef_node(f: &mut Formatter<'_>, node: &UndefNode<'_>) {
    f.b.text("undef ");
    for (index, name) in node.names().iter().enumerate() {
        if index > 0 {
            f.b.text(", ");
        }
        method_name(f, &name);
    }
}
