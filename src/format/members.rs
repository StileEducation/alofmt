//! Per-call facts for the parentheses and `self.` policies (see
//! [`crate::MethodCallParentheses`]): which calls name a member of the
//! enclosing class or module body, which sit in macro position, which share
//! a name with a local variable in scope, and which sit in a tail position
//! where a command call parses the same as a parenthesised one. The
//! analysis walk in `analysis.rs` gathers them; this module holds the state
//! and the bookkeeping the walk drives.
//!
//! Members are matched lexically: a `def`, an `alias` or a member-macro call
//! in the same body, at the same `self` level as the call. Definitions in
//! superclasses, included modules and other reopenings of the class are not
//! visible. A misclassified call is a style difference only: `self.foo` and
//! `foo` call the same method on Ruby 2.7 and later.

use ruby_prism::{CallNode, Node};
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
pub(super) enum Level {
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
pub(super) struct Position {
    /// The level at which a bare `def` in this position is defined.
    pub(super) def_target: Option<Level>,
    /// The level at which a call in this position is matched.
    pub(super) call_level: Option<Level>,
}

impl Position {
    pub(super) const OPAQUE: Self = Self {
        def_target: None,
        call_level: None,
    };
    /// A class or module body: `def` defines instance methods and `self` is
    /// the class.
    pub(super) const CLASS_BODY: Self = Self {
        def_target: Some(Level::Instance),
        call_level: Some(Level::Singleton),
    };
}

/// Which facts the enabled policies read; the rest are not recorded, and
/// with every policy on `preserve` nothing is.
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

/// The bookkeeping the analysis walk keeps while it visits the tree.
pub(super) struct MembersWalk<'o, 'pr> {
    options: &'o FormatOptions,
    track: Track,
    frames: Vec<Frame<'pr>>,
    scopes: Vec<Scope<'pr>>,
    pub(super) position: Position,
    /// Set immediately before visiting a statement list whose statements
    /// are in macro position; cleared by that visit.
    pub(super) macro_statements: bool,
    /// Set immediately before visiting a statement list whose statements
    /// are not in tail position (inside parentheses, string interpolation,
    /// a ternary branch or an endless `def`); cleared by that visit.
    pub(super) expression_statements: bool,
    /// Blocks visited so far, compared before and after an argument list.
    pub(super) blocks_seen: usize,
    pub(super) state: State,
}

impl<'o, 'pr> MembersWalk<'o, 'pr> {
    pub(super) fn new(options: &'o FormatOptions) -> Self {
        let requires = |policy: MethodCallParentheses| policy == MethodCallParentheses::RequireParentheses;
        let omits = |policy: MethodCallParentheses| policy == MethodCallParentheses::OmitParentheses;
        Self {
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
            state: State::default(),
        }
    }

    pub(super) fn finish(self) -> State {
        self.state
    }

    /// Records a call for member matching and local-variable shadowing.
    pub(super) fn candidate(&mut self, node: &CallNode<'pr>) {
        if !is_member_candidate(node) {
            return;
        }
        let span = span_of(&node.as_node());
        let name = node.name().as_slice();
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

    /// Whether a call is a configured member macro, defining the members
    /// named by its positional symbol and string arguments.
    pub(super) fn member_macro(&mut self, node: &CallNode<'pr>) -> bool {
        let name = node.name().as_slice();
        let member_macro = node.receiver().is_none() && self.options.member_macros.iter().any(|m| m.as_bytes() == name);
        if member_macro
            && self.track.members
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
        member_macro
    }

    pub(super) fn define(&mut self, level: Level, name: &'pr [u8]) {
        if self.track.members
            && let Some(frame) = self.frames.last_mut()
        {
            frame.names(level).insert(name);
        }
    }

    pub(super) fn mark_macro(&mut self, statement: &Node<'_>) {
        if self.track.macros
            && let Some(call) = statement.as_call_node()
            && call.receiver().is_none()
        {
            self.state.macro_calls.insert(span_of(statement));
        }
    }

    /// Records the node kinds whose tail position is ever asked about: the
    /// calls themselves, and the assignments and jumps that pass it on.
    pub(super) fn mark_tail(&mut self, node: &Node<'_>) {
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

    /// Called after an argument list, with the block count from before it.
    pub(super) fn arguments_visited(&mut self, owner: &Node<'_>, blocks_before: usize) {
        if self.track.tails && self.blocks_seen != blocks_before {
            self.state.arguments_with_blocks.insert(span_of(owner));
        }
    }

    pub(super) fn push_frame(&mut self) {
        self.frames.push(Frame::default());
    }

    /// Matches the calls recorded in the frame against its definitions.
    pub(super) fn pop_frame(&mut self) {
        let mut frame = self.frames.pop().expect("a frame was pushed");
        for (span, level, name) in std::mem::take(&mut frame.pending) {
            if frame.names(level).contains(name) {
                self.state.member_calls.insert(span);
            }
        }
    }

    /// Returns whether a scope was pushed; none is when no policy reads
    /// local variables.
    pub(super) fn push_scope(
        &mut self,
        closed: bool,
        locals: impl Iterator<Item = ruby_prism::ConstantId<'pr>>,
    ) -> bool {
        if !self.track.scopes {
            return false;
        }
        self.scopes.push(Scope {
            closed,
            locals: locals.map(|local| local.as_slice()).collect(),
        });
        true
    }

    pub(super) fn pop_scope(&mut self) {
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
