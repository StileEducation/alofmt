//! The one analysis walk over the tree before layout. It records the
//! control-flow contexts (see [`super::control`]) and, when a call policy
//! is enabled, the per-call facts in [`super::members`]. Both families need
//! parent information, which Prism nodes do not carry, so one traversal
//! collects everything.
//!
//! The walk visits every child through [`Visit::visit`] so the enter and
//! leave hooks keep the context stack accurate; statement and argument
//! lists, which some parents reach as typed children without the hooks,
//! go through [`Walk::typed`].

use ruby_prism::{
    AliasMethodNode, ArgumentsNode, BlockNode, BreakNode, CallNode, ClassNode, ClassVariableAndWriteNode,
    ClassVariableOperatorWriteNode, ClassVariableOrWriteNode, ClassVariableWriteNode, ConstantAndWriteNode,
    ConstantOperatorWriteNode, ConstantOrWriteNode, ConstantPathAndWriteNode, ConstantPathOperatorWriteNode,
    ConstantPathOrWriteNode, ConstantPathWriteNode, ConstantWriteNode, DefNode, EmbeddedStatementsNode,
    GlobalVariableAndWriteNode, GlobalVariableOperatorWriteNode, GlobalVariableOrWriteNode, GlobalVariableWriteNode,
    IfNode, InstanceVariableAndWriteNode, InstanceVariableOperatorWriteNode, InstanceVariableOrWriteNode,
    InstanceVariableWriteNode, LambdaNode, LocalVariableAndWriteNode, LocalVariableOperatorWriteNode,
    LocalVariableOrWriteNode, LocalVariableWriteNode, ModuleNode, MultiWriteNode, NextNode, Node, ParenthesesNode,
    ProgramNode, RescueModifierNode, ReturnNode, SingletonClassNode, StatementsNode, SuperNode, Visit, YieldNode,
};

use super::control::ContextWalk;
use super::members::{Level, MembersWalk, Position};
use super::{control, members};
use crate::FormatOptions;

pub fn analyze(root: &Node<'_>, options: &FormatOptions) -> (control::State, members::State) {
    let mut walk = Walk {
        context: ContextWalk::new(),
        members: MembersWalk::new(options),
    };
    walk.visit(root);
    (walk.context.finish(), walk.members.finish())
}

struct Walk<'o, 'pr> {
    context: ContextWalk,
    members: MembersWalk<'o, 'pr>,
}

/// `x = foo a` is a statement-level form: as a predicate, an argument or
/// an operand, the assignment's value has to keep its parentheses.
macro_rules! tail_value {
    ($($method:ident($node:ident)),* $(,)?) => {
        $(
            fn $method(&mut self, node: &$node<'pr>) {
                if self.members.state.in_tail_position(&node.as_node()) {
                    self.members.mark_tail(&node.value());
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
                if self.members.state.in_tail_position(&node.as_node())
                    && let Some(arguments) = node.arguments()
                    && arguments.arguments().len() == 1
                    && let Some(argument) = arguments.arguments().first()
                {
                    self.members.mark_tail(&argument);
                }
                ruby_prism::$method(self, node);
            }
        )*
    };
}

impl<'pr> Visit<'pr> for Walk<'_, 'pr> {
    fn visit_branch_node_enter(&mut self, node: Node<'pr>) {
        self.context.enter(&node);
    }

    fn visit_branch_node_leave(&mut self) {
        self.context.leave();
    }

    fn visit_leaf_node_enter(&mut self, node: Node<'pr>) {
        self.context.enter(&node);
    }

    fn visit_leaf_node_leave(&mut self) {
        self.context.leave();
    }

    fn visit_statements_node(&mut self, node: &StatementsNode<'pr>) {
        self.typed(&node.as_node(), |walk| {
            let in_macro = std::mem::take(&mut walk.members.macro_statements);
            let in_tail = !std::mem::take(&mut walk.members.expression_statements);
            for statement in node.body().iter() {
                if in_macro {
                    walk.members.mark_macro(&statement);
                }
                if in_tail {
                    walk.members.mark_tail(&statement);
                }
            }
            ruby_prism::visit_statements_node(walk, node);
        });
    }

    fn visit_arguments_node(&mut self, node: &ArgumentsNode<'pr>) {
        self.typed(&node.as_node(), |walk| ruby_prism::visit_arguments_node(walk, node));
    }

    fn visit_program_node(&mut self, node: &ProgramNode<'pr>) {
        let position = Position {
            def_target: Some(Level::Instance),
            call_level: Some(Level::Instance),
        };
        self.with_scope(true, node.locals().iter(), |walk| {
            walk.with_frame(position, |walk| {
                walk.members.macro_statements = true;
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
            if matches!(node.expression(), Node::SelfNode { .. }) && walk.members.position == Position::CLASS_BODY {
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
            None => self.members.position.def_target,
            Some(Node::SelfNode { .. }) if self.members.position.def_target == Some(Level::Instance) => {
                Some(Level::Singleton)
            }
            Some(_) => None,
        };
        if let Some(level) = level {
            self.members.define(level, node.name().as_slice());
        }
        if let Some(receiver) = node.receiver() {
            self.visit(&receiver);
        }
        let position = Position {
            def_target: self.members.position.def_target,
            call_level: level,
        };
        self.with_scope(true, node.locals().iter(), |walk| {
            walk.with_position(position, |walk| {
                if let Some(parameters) = node.parameters() {
                    walk.visit(&parameters.as_node());
                }
                // The value of an endless `def` is an expression, not a statement.
                if node.equal_loc().is_some() {
                    walk.members.expression_statements = true;
                }
                walk.body(node.body(), false);
            });
        });
    }

    fn visit_alias_method_node(&mut self, node: &AliasMethodNode<'pr>) {
        if let Some(level) = self.members.position.def_target
            && let Some(name) = node.new_name().as_symbol_node()
            && let Some(value) = name.value_loc()
        {
            self.members.define(level, value.as_slice());
        }
        ruby_prism::visit_alias_method_node(self, node);
    }

    fn visit_call_node(&mut self, node: &CallNode<'pr>) {
        self.typed(&node.as_node(), |walk| walk.call(node));
    }

    fn visit_block_node(&mut self, node: &BlockNode<'pr>) {
        self.block(node, false);
    }

    fn visit_lambda_node(&mut self, node: &LambdaNode<'pr>) {
        self.with_scope(false, node.locals().iter(), |walk| {
            if let Some(parameters) = node.parameters() {
                walk.visit(&parameters);
            }
            walk.body(node.body(), false);
        });
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

    fn visit_parentheses_node(&mut self, node: &ParenthesesNode<'pr>) {
        self.members.expression_statements = node.body().is_some();
        ruby_prism::visit_parentheses_node(self, node);
    }

    fn visit_embedded_statements_node(&mut self, node: &EmbeddedStatementsNode<'pr>) {
        self.members.expression_statements = node.statements().is_some();
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
            self.members.expression_statements = true;
            self.visit_statements_node(&statements);
        }
        if let Some(subsequent) = node.subsequent() {
            self.members.expression_statements = true;
            self.visit(&subsequent);
        }
    }

    fn visit_rescue_modifier_node(&mut self, node: &RescueModifierNode<'pr>) {
        // `x rescue y` prints as a `begin` block, where `x` is a statement.
        self.members.mark_tail(&node.expression());
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
    /// Visits a statically typed child, which reaches its visit method
    /// without the enter and leave hooks, keeping the context stack in step.
    fn typed(&mut self, node: &Node<'_>, descend: impl FnOnce(&mut Self)) {
        let entered = self.context.enter_typed(node);
        descend(self);
        if entered {
            self.context.leave();
        }
    }

    fn call(&mut self, node: &CallNode<'pr>) {
        self.members.candidate(node);
        let member_macro = self.members.member_macro(node);
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
                    def_target: self.members.position.def_target,
                    call_level: if member_macro {
                        self.members.position.def_target
                    } else {
                        self.members.position.call_level
                    },
                };
                let in_macro = !member_macro && self.members.state.in_macro_position(node);
                self.with_position(position, |walk| walk.block(&block, in_macro));
            }
            Some(block) => self.visit(&block),
            None => {}
        }
    }

    /// Visits an argument list, recording whether a block sits inside it.
    fn arguments(&mut self, owner: &Node<'_>, arguments: Option<ArgumentsNode<'pr>>) {
        let Some(arguments) = arguments else {
            return;
        };
        let before = self.members.blocks_seen;
        self.visit_arguments_node(&arguments);
        self.members.arguments_visited(owner, before);
    }

    fn block(&mut self, node: &BlockNode<'pr>, in_macro: bool) {
        self.members.blocks_seen += 1;
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
            self.members.expression_statements = false;
            return;
        };
        if !macro_statements {
            self.visit(&body);
            return;
        }
        if let Some(statements) = body.as_statements_node() {
            self.members.macro_statements = true;
            self.visit_statements_node(&statements);
        } else if let Some(begin) = body.as_begin_node() {
            self.context.enter(&body);
            if let Some(statements) = begin.statements() {
                self.members.macro_statements = true;
                self.visit_statements_node(&statements);
            }
            if let Some(rescue) = begin.rescue_clause() {
                self.visit(&rescue.as_node());
            }
            if let Some(else_clause) = begin.else_clause() {
                self.visit(&else_clause.as_node());
            }
            if let Some(ensure) = begin.ensure_clause() {
                self.visit(&ensure.as_node());
            }
            self.context.leave();
        } else {
            self.visit(&body);
        }
    }

    fn with_frame(&mut self, position: Position, body: impl FnOnce(&mut Self)) {
        let previous = std::mem::replace(&mut self.members.position, position);
        self.members.push_frame();
        body(self);
        self.members.pop_frame();
        self.members.position = previous;
    }

    fn with_position(&mut self, position: Position, body: impl FnOnce(&mut Self)) {
        let previous = std::mem::replace(&mut self.members.position, position);
        body(self);
        self.members.position = previous;
    }

    fn with_scope(
        &mut self,
        closed: bool,
        locals: impl Iterator<Item = ruby_prism::ConstantId<'pr>>,
        body: impl FnOnce(&mut Self),
    ) {
        let pushed = self.members.push_scope(closed, locals);
        body(self);
        if pushed {
            self.members.pop_scope();
        }
    }
}
