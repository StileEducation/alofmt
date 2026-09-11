//! Per-call facts for the parentheses and `self.` policies (see
//! [`crate::MethodCallParentheses`]): which calls name a member of the
//! enclosing class or module body, which sit in macro position, which share
//! a name with a local variable in scope, and which sit in a tail position
//! where a command call parses the same as a parenthesised one.
//!
//! Members are matched lexically: a `def`, an `alias` or a member-macro call
//! in the same body, at the same `self` level as the call. Definitions in
//! superclasses, included modules and other reopenings of the class are not
//! visible. A misclassified call is a style difference only: `self.foo` and
//! `foo` call the same method on Ruby 2.7 and later.

use ruby_prism::{
    AliasMethodNode, BlockNode, BreakNode, CallNode, ClassNode, ClassVariableAndWriteNode,
    ClassVariableOperatorWriteNode, ClassVariableOrWriteNode, ClassVariableWriteNode, ConstantAndWriteNode,
    ConstantOperatorWriteNode, ConstantOrWriteNode, ConstantPathAndWriteNode, ConstantPathOperatorWriteNode,
    ConstantPathOrWriteNode, ConstantPathWriteNode, ConstantWriteNode, DefNode, EmbeddedStatementsNode,
    GlobalVariableAndWriteNode, GlobalVariableOperatorWriteNode, GlobalVariableOrWriteNode, GlobalVariableWriteNode,
    IfNode, InstanceVariableAndWriteNode, InstanceVariableOperatorWriteNode, InstanceVariableOrWriteNode,
    InstanceVariableWriteNode, LambdaNode, LocalVariableAndWriteNode, LocalVariableOperatorWriteNode,
    LocalVariableOrWriteNode, LocalVariableWriteNode, ModuleNode, MultiWriteNode, NextNode, Node, ParenthesesNode,
    ProgramNode, RescueModifierNode, ReturnNode, SingletonClassNode, StatementsNode, SuperNode, Visit, YieldNode,
};
use rustc_hash::FxHashSet as HashSet;

use crate::FormatOptions;
use crate::options::{MethodCallParentheses, RedundantSelf};

#[derive(Default)]
pub struct State {
    member_calls: HashSet<(usize, usize)>,
    macro_calls: HashSet<(usize, usize)>,
    shadowed_calls: HashSet<(usize, usize)>,
    tail_nodes: HashSet<(usize, usize)>,
    arguments_with_blocks: HashSet<(usize, usize)>,
}

impl State {
    pub fn analyze(root: &Node<'_>, options: &FormatOptions) -> Self {
        let active = options.method_call_with_args_parentheses != MethodCallParentheses::Preserve
            || options.method_call_without_args_parentheses != MethodCallParentheses::Preserve
            || options.redundant_self != RedundantSelf::Preserve;
        if !active {
            return Self::default();
        }
        let requires = |policy: MethodCallParentheses| policy == MethodCallParentheses::RequireParentheses;
        let omits = |policy: MethodCallParentheses| policy == MethodCallParentheses::OmitParentheses;
        let mut walk = Walk {
            options,
            track: Track {
                members: requires(options.method_call_without_args_parentheses)
                    || options.redundant_self == RedundantSelf::RequireSelf,
                macros: requires(options.method_call_with_args_parentheses)
                    || requires(options.method_call_without_args_parentheses)
                    || options.redundant_self == RedundantSelf::RequireSelf,
                scopes: omits(options.method_call_with_args_parentheses)
                    || omits(options.method_call_without_args_parentheses)
                    || options.redundant_self == RedundantSelf::OmitSelf,
                tails: omits(options.method_call_with_args_parentheses),
            },
            frames: Vec::new(),
            scopes: Vec::new(),
            position: Position::OPAQUE,
            macro_statements: false,
            expression_statements: false,
            blocks_seen: 0,
            state: Self::default(),
        };
        walk.visit(root);
        walk.state
    }

    /// A call with no receiver, or `self` as the receiver, to a method
    /// defined in the enclosing body at the same `self` level as the call.
    pub fn is_member_call(&self, node: &CallNode<'_>) -> bool {
        is_member_candidate(node) && self.member_calls.contains(&span_of(&node.as_node()))
    }

    /// A receiverless call that is a statement of a class, module,
    /// singleton-class or top-level body, or of a block attached to a macro.
    pub fn in_macro_position(&self, node: &CallNode<'_>) -> bool {
        node.receiver().is_none() && self.macro_calls.contains(&span_of(&node.as_node()))
    }

    /// A call with no receiver, or `self` as the receiver, whose name is
    /// also a local variable in scope, or the implicit block parameter `it`:
    /// spelled bare, it would read as the variable.
    pub fn is_shadowed(&self, node: &CallNode<'_>) -> bool {
        is_member_candidate(node) && self.shadowed_calls.contains(&span_of(&node.as_node()))
    }

    /// A node whose command form parses the same as its parenthesised form:
    /// a statement; the value of an assignment, or the sole value of
    /// `return`, `break` or `next`, that is itself in tail position; or the
    /// expression of a rescue modifier.
    pub fn in_tail_position(&self, node: &Node<'_>) -> bool {
        self.tail_nodes.contains(&span_of(node))
    }

    /// A call, `super` or `yield` with a block somewhere in its arguments.
    /// Printed flat without parentheses, a `do` block there would bind to
    /// the outer call, and a brace block may become one when it breaks.
    pub fn arguments_contain_block(&self, node: &Node<'_>) -> bool {
        self.arguments_with_blocks.contains(&span_of(node))
    }
}

/// Which of a body's two member sets a definition or call belongs to.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Level {
    Instance,
    Singleton,
}

/// The members of one class, module or top-level body. Calls are matched
/// after the whole body has been visited, so a call may precede the
/// definition.
#[derive(Default)]
struct Frame<'pr> {
    instance: HashSet<&'pr [u8]>,
    singleton: HashSet<&'pr [u8]>,
    pending: Vec<((usize, usize), Level, &'pr [u8])>,
}

impl<'pr> Frame<'pr> {
    fn names(&mut self, level: Level) -> &mut HashSet<&'pr [u8]> {
        match level {
            Level::Instance => &mut self.instance,
            Level::Singleton => &mut self.singleton,
        }
    }
}

/// The local variables of one Ruby scope. A block or lambda scope also sees
/// the scopes outside it, up to the nearest `def`, class body or program.
struct Scope<'pr> {
    closed: bool,
    locals: HashSet<&'pr [u8]>,
}

/// What the current lexical position means for definitions and calls;
/// `None` where the level is unknown (an unusual receiver, a nested
/// `class << self`).
#[derive(Clone, Copy, PartialEq, Eq)]
struct Position {
    /// The level at which a bare `def` in this position is defined.
    def_target: Option<Level>,
    /// The level at which a call in this position is matched.
    call_level: Option<Level>,
}

impl Position {
    const OPAQUE: Self = Self {
        def_target: None,
        call_level: None,
    };
    /// A class or module body: `def` defines instance methods and `self` is
    /// the class.
    const CLASS_BODY: Self = Self {
        def_target: Some(Level::Instance),
        call_level: Some(Level::Singleton),
    };
}

/// Which facts the enabled policies read; the rest are not recorded.
struct Track {
    /// Member calls: `require_parentheses` without arguments, `require_self`.
    members: bool,
    /// Macro position: every `require` value.
    macros: bool,
    /// Local variables in scope: every `omit` value.
    scopes: bool,
    /// Tail position and blocks inside arguments: `omit_parentheses` with
    /// arguments.
    tails: bool,
}

struct Walk<'o, 'pr> {
    options: &'o FormatOptions,
    track: Track,
    frames: Vec<Frame<'pr>>,
    scopes: Vec<Scope<'pr>>,
    position: Position,
    /// Set immediately before visiting a statement list whose statements
    /// are in macro position; cleared by that visit.
    macro_statements: bool,
    /// Set immediately before visiting a statement list whose statements
    /// are not in tail position (inside parentheses, string interpolation,
    /// a ternary branch or an endless `def`); cleared by that visit.
    expression_statements: bool,
    /// Blocks visited so far, compared before and after an argument list.
    blocks_seen: usize,
    state: State,
}

/// `x = foo a` is a statement-level form: as a predicate, an argument or
/// an operand, the assignment's value has to keep its parentheses.
macro_rules! tail_value {
    ($($method:ident($node:ident)),* $(,)?) => {
        $(
            fn $method(&mut self, node: &$node<'pr>) {
                if self.state.in_tail_position(&node.as_node()) {
                    self.mark_tail(&node.value());
                }
                ruby_prism::$method(self, node);
            }
        )*
    };
}

/// `return foo a` likewise: only where the jump itself is a statement.
macro_rules! tail_sole_argument {
    ($($method:ident($node:ident)),* $(,)?) => {
        $(
            fn $method(&mut self, node: &$node<'pr>) {
                if self.state.in_tail_position(&node.as_node())
                    && let Some(arguments) = node.arguments()
                    && arguments.arguments().len() == 1
                    && let Some(argument) = arguments.arguments().first()
                {
                    self.mark_tail(&argument);
                }
                ruby_prism::$method(self, node);
            }
        )*
    };
}

impl<'pr> Visit<'pr> for Walk<'_, 'pr> {
    fn visit_program_node(&mut self, node: &ProgramNode<'pr>) {
        let position = Position {
            def_target: Some(Level::Instance),
            call_level: Some(Level::Instance),
        };
        self.with_scope(true, node.locals().iter(), |walk| {
            walk.with_frame(position, |walk| {
                walk.macro_statements = true;
                walk.visit_statements_node(&node.statements());
            });
        });
    }

    fn visit_class_node(&mut self, node: &ClassNode<'pr>) {
        self.visit(&node.constant_path());
        if let Some(superclass) = node.superclass() {
            self.visit(&superclass);
        }
        self.with_scope(true, node.locals().iter(), |walk| {
            walk.with_frame(Position::CLASS_BODY, |walk| walk.body(node.body(), true));
        });
    }

    fn visit_module_node(&mut self, node: &ModuleNode<'pr>) {
        self.visit(&node.constant_path());
        self.with_scope(true, node.locals().iter(), |walk| {
            walk.with_frame(Position::CLASS_BODY, |walk| walk.body(node.body(), true));
        });
    }

    fn visit_singleton_class_node(&mut self, node: &SingletonClassNode<'pr>) {
        self.visit(&node.expression());
        self.with_scope(true, node.locals().iter(), |walk| {
            if matches!(node.expression(), Node::SelfNode { .. }) && walk.position == Position::CLASS_BODY {
                // Definitions inside `class << self` are singleton members of the
                // enclosing body; inside it, `self` is the singleton class.
                let position = Position {
                    def_target: Some(Level::Singleton),
                    call_level: None,
                };
                walk.with_position(position, |walk| walk.body(node.body(), true));
            } else {
                walk.with_frame(Position::OPAQUE, |walk| walk.body(node.body(), true));
            }
        });
    }

    fn visit_def_node(&mut self, node: &DefNode<'pr>) {
        // Calls in a method body are matched at the level the method is defined at.
        let level = match node.receiver() {
            None => self.position.def_target,
            Some(Node::SelfNode { .. }) if self.position.def_target == Some(Level::Instance) => Some(Level::Singleton),
            Some(_) => None,
        };
        if let Some(level) = level {
            self.define(level, node.name().as_slice());
        }
        if let Some(receiver) = node.receiver() {
            self.visit(&receiver);
        }
        let position = Position {
            def_target: self.position.def_target,
            call_level: level,
        };
        self.with_scope(true, node.locals().iter(), |walk| {
            walk.with_position(position, |walk| {
                if let Some(parameters) = node.parameters() {
                    walk.visit_parameters_node(&parameters);
                }
                if node.equal_loc().is_some() {
                    walk.expression_statements = true;
                }
                walk.body(node.body(), false);
            });
        });
    }

    fn visit_alias_method_node(&mut self, node: &AliasMethodNode<'pr>) {
        if let Some(level) = self.position.def_target
            && let Some(name) = node.new_name().as_symbol_node()
            && let Some(value) = name.value_loc()
        {
            self.define(level, value.as_slice());
        }
    }

    fn visit_call_node(&mut self, node: &CallNode<'pr>) {
        let span = span_of(&node.as_node());
        let name = node.name().as_slice();
        if is_member_candidate(node) {
            if self.track.members
                && let Some(level) = self.position.call_level
                && let Some(frame) = self.frames.last_mut()
            {
                frame.pending.push((span, level, name));
            }
            if self.track.scopes && (name == b"it" || self.local_in_scope(name)) {
                self.state.shadowed_calls.insert(span);
            }
        }
        let member_macro = node.receiver().is_none() && self.options.member_macros.iter().any(|m| m.as_bytes() == name);
        if self.track.members
            && member_macro
            && let Some(level) = self.position.def_target
        {
            // The source text of the symbol or string: a name with escapes
            // could not be called bare anyway.
            for argument in node.arguments().iter().flat_map(|a| a.arguments().iter()) {
                if let Some(symbol) = argument.as_symbol_node() {
                    if let Some(value) = symbol.value_loc() {
                        self.define(level, value.as_slice());
                    }
                } else if let Some(string) = argument.as_string_node() {
                    self.define(level, string.content_loc().as_slice());
                }
            }
        }
        if let Some(receiver) = node.receiver() {
            self.visit(&receiver);
        }
        self.arguments(&node.as_node(), node.arguments());
        match node.block() {
            Some(Node::BlockNode { .. }) => {
                let block = node.block().and_then(|b| b.as_block_node()).expect("kind");
                // The block of `define_method(:x) { ... }` is the method body,
                // so it is matched at the definition level.
                let position = Position {
                    def_target: self.position.def_target,
                    call_level: if member_macro {
                        self.position.def_target
                    } else {
                        self.position.call_level
                    },
                };
                let in_macro = !member_macro && self.state.macro_calls.contains(&span);
                self.with_position(position, |walk| walk.block(&block, in_macro));
            }
            Some(block) => self.visit(&block),
            None => {}
        }
    }

    fn visit_block_node(&mut self, node: &BlockNode<'pr>) {
        self.block(node, false);
    }

    fn visit_super_node(&mut self, node: &SuperNode<'pr>) {
        self.arguments(&node.as_node(), node.arguments());
        if let Some(block) = node.block() {
            self.visit(&block);
        }
    }

    fn visit_yield_node(&mut self, node: &YieldNode<'pr>) {
        self.arguments(&node.as_node(), node.arguments());
    }

    fn visit_lambda_node(&mut self, node: &LambdaNode<'pr>) {
        self.with_scope(false, node.locals().iter(), |walk| {
            if let Some(parameters) = node.parameters() {
                walk.visit(&parameters);
            }
            walk.body(node.body(), false);
        });
    }

    fn visit_statements_node(&mut self, node: &StatementsNode<'pr>) {
        let in_macro = std::mem::take(&mut self.macro_statements);
        let in_tail = !std::mem::take(&mut self.expression_statements);
        for statement in node.body().iter() {
            if in_macro
                && self.track.macros
                && let Some(call) = statement.as_call_node()
                && call.receiver().is_none()
            {
                self.state.macro_calls.insert(span_of(&statement));
            }
            if in_tail {
                self.mark_tail(&statement);
            }
        }
        ruby_prism::visit_statements_node(self, node);
    }

    fn visit_parentheses_node(&mut self, node: &ParenthesesNode<'pr>) {
        self.expression_statements = node.body().is_some();
        ruby_prism::visit_parentheses_node(self, node);
    }

    fn visit_embedded_statements_node(&mut self, node: &EmbeddedStatementsNode<'pr>) {
        self.expression_statements = node.statements().is_some();
        ruby_prism::visit_embedded_statements_node(self, node);
    }

    fn visit_if_node(&mut self, node: &IfNode<'pr>) {
        if node.if_keyword_loc().is_some() {
            ruby_prism::visit_if_node(self, node);
            return;
        }
        // A ternary: its branches sit between `?` and `:`, where a command
        // call would swallow the rest of the expression.
        self.visit(&node.predicate());
        if let Some(statements) = node.statements() {
            self.expression_statements = true;
            self.visit_statements_node(&statements);
        }
        if let Some(else_clause) = node.subsequent().and_then(|s| s.as_else_node())
            && let Some(statements) = else_clause.statements()
        {
            self.expression_statements = true;
            self.visit_statements_node(&statements);
        }
    }

    fn visit_rescue_modifier_node(&mut self, node: &RescueModifierNode<'pr>) {
        // `x rescue y` prints as a `begin` block, where `x` is a statement.
        self.mark_tail(&node.expression());
        ruby_prism::visit_rescue_modifier_node(self, node);
    }

    tail_value!(
        visit_local_variable_write_node(LocalVariableWriteNode),
        visit_local_variable_operator_write_node(LocalVariableOperatorWriteNode),
        visit_local_variable_and_write_node(LocalVariableAndWriteNode),
        visit_local_variable_or_write_node(LocalVariableOrWriteNode),
        visit_instance_variable_write_node(InstanceVariableWriteNode),
        visit_instance_variable_operator_write_node(InstanceVariableOperatorWriteNode),
        visit_instance_variable_and_write_node(InstanceVariableAndWriteNode),
        visit_instance_variable_or_write_node(InstanceVariableOrWriteNode),
        visit_class_variable_write_node(ClassVariableWriteNode),
        visit_class_variable_operator_write_node(ClassVariableOperatorWriteNode),
        visit_class_variable_and_write_node(ClassVariableAndWriteNode),
        visit_class_variable_or_write_node(ClassVariableOrWriteNode),
        visit_global_variable_write_node(GlobalVariableWriteNode),
        visit_global_variable_operator_write_node(GlobalVariableOperatorWriteNode),
        visit_global_variable_and_write_node(GlobalVariableAndWriteNode),
        visit_global_variable_or_write_node(GlobalVariableOrWriteNode),
        visit_constant_write_node(ConstantWriteNode),
        visit_constant_operator_write_node(ConstantOperatorWriteNode),
        visit_constant_and_write_node(ConstantAndWriteNode),
        visit_constant_or_write_node(ConstantOrWriteNode),
        visit_constant_path_write_node(ConstantPathWriteNode),
        visit_constant_path_operator_write_node(ConstantPathOperatorWriteNode),
        visit_constant_path_and_write_node(ConstantPathAndWriteNode),
        visit_constant_path_or_write_node(ConstantPathOrWriteNode),
        visit_multi_write_node(MultiWriteNode),
    );

    tail_sole_argument!(
        visit_return_node(ReturnNode),
        visit_break_node(BreakNode),
        visit_next_node(NextNode),
    );
}

impl<'pr> Walk<'_, 'pr> {
    fn with_frame(&mut self, position: Position, body: impl FnOnce(&mut Self)) {
        let previous = std::mem::replace(&mut self.position, position);
        self.frames.push(Frame::default());
        body(self);
        let mut frame = self.frames.pop().expect("pushed above");
        self.position = previous;
        for (span, level, name) in std::mem::take(&mut frame.pending) {
            if frame.names(level).contains(name) {
                self.state.member_calls.insert(span);
            }
        }
    }

    fn with_position(&mut self, position: Position, body: impl FnOnce(&mut Self)) {
        let previous = std::mem::replace(&mut self.position, position);
        body(self);
        self.position = previous;
    }

    fn with_scope(
        &mut self,
        closed: bool,
        locals: impl Iterator<Item = ruby_prism::ConstantId<'pr>>,
        body: impl FnOnce(&mut Self),
    ) {
        if !self.track.scopes {
            body(self);
            return;
        }
        self.scopes.push(Scope {
            closed,
            locals: locals.map(|local| local.as_slice()).collect(),
        });
        body(self);
        self.scopes.pop();
    }

    fn local_in_scope(&self, name: &[u8]) -> bool {
        for scope in self.scopes.iter().rev() {
            if scope.locals.contains(name) {
                return true;
            }
            if scope.closed {
                return false;
            }
        }
        false
    }

    fn define(&mut self, level: Level, name: &'pr [u8]) {
        if self.track.members
            && let Some(frame) = self.frames.last_mut()
        {
            frame.names(level).insert(name);
        }
    }

    /// Records the node kinds whose tail position is ever asked about: the
    /// calls themselves, and the assignments and jumps that pass it on.
    fn mark_tail(&mut self, node: &Node<'_>) {
        if !self.track.tails {
            return;
        }
        if matches!(
            node,
            Node::CallNode { .. }
                | Node::SuperNode { .. }
                | Node::YieldNode { .. }
                | Node::ReturnNode { .. }
                | Node::BreakNode { .. }
                | Node::NextNode { .. }
        ) || super::control::is_assignment(node)
        {
            self.state.tail_nodes.insert(span_of(node));
        }
    }

    /// Visits an argument list, recording whether a block sits inside it.
    fn arguments(&mut self, owner: &Node<'_>, arguments: Option<ruby_prism::ArgumentsNode<'pr>>) {
        let Some(arguments) = arguments else {
            return;
        };
        let before = self.blocks_seen;
        self.visit_arguments_node(&arguments);
        if self.track.tails && self.blocks_seen != before {
            self.state.arguments_with_blocks.insert(span_of(owner));
        }
    }

    fn block(&mut self, node: &BlockNode<'pr>, in_macro: bool) {
        self.blocks_seen += 1;
        self.with_scope(false, node.locals().iter(), |walk| {
            if let Some(parameters) = node.parameters() {
                walk.visit(&parameters);
            }
            walk.body(node.body(), in_macro);
        });
    }

    /// Visits a `class`, `module`, `def` or block body. A body with
    /// `rescue`/`ensure` clauses is a `BeginNode`; only the main statements
    /// count as the body for macro position.
    fn body(&mut self, body: Option<Node<'pr>>, macro_statements: bool) {
        let Some(body) = body else {
            self.expression_statements = false;
            return;
        };
        if !macro_statements {
            self.visit(&body);
            return;
        }
        if let Some(statements) = body.as_statements_node() {
            self.macro_statements = true;
            self.visit_statements_node(&statements);
        } else if let Some(begin) = body.as_begin_node() {
            if let Some(statements) = begin.statements() {
                self.macro_statements = true;
                self.visit_statements_node(&statements);
            }
            if let Some(rescue) = begin.rescue_clause() {
                self.visit_rescue_node(&rescue);
            }
            if let Some(else_clause) = begin.else_clause() {
                self.visit_else_node(&else_clause);
            }
            if let Some(ensure) = begin.ensure_clause() {
                self.visit_ensure_node(&ensure);
            }
        } else {
            self.visit(&body);
        }
    }
}

/// A plain method call on an implicit or literal `self` receiver.
fn is_member_candidate(node: &CallNode<'_>) -> bool {
    let receiver_is_self = match node.receiver() {
        None => true,
        Some(Node::SelfNode { .. }) => node.call_operator_loc().is_some(),
        Some(_) => false,
    };
    receiver_is_self
        && node.message_loc().is_some()
        && !node.is_attribute_write()
        && node
            .name()
            .as_slice()
            .first()
            .is_some_and(|byte| byte.is_ascii_lowercase() || *byte == b'_' || !byte.is_ascii())
}

fn span_of(node: &Node<'_>) -> (usize, usize) {
    let location = node.location();
    (location.start_offset(), location.end_offset())
}
