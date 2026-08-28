//! Attaches every comment to the node it decorates.
//!
//! A comment sharing a line with code trails the sibling that ends before
//! it. With nothing before it, it sits on the parent's *header* (`x = # c`,
//! `def foo # c`, `{ # c`): it stays on that line and the parent's first
//! group breaks so the children move down. The exceptions are a call's `(`,
//! an explicit `begin` and `#{`, after which the comment moves to its own line
//! before the first argument or statement. After the
//! operator of a write or assoc whose target is a child (`x.y = # c`,
//! `x[i] = # c`, `key: # c`), the comment heads the assignment rather than
//! trailing the target.
//!
//! A comment on its own line leads the next sibling, except that:
//!
//! - directly inside a statement list, or next to a `StatementsNode`
//!   sibling, it becomes a *body entry* of that list, so the statements
//!   formatter can keep blank lines around it;
//! - when the following sibling opens a clause (`else`, `elsif`, `rescue`,
//!   `ensure`, `when`, `in`), or there is none and the parent has a body
//!   (`def`, `do`/`end`, `case`, ...), it is *dangling* on the parent: the
//!   comment sits in an empty body or before a closing keyword, and the
//!   parent's formatter prints it there;
//! - after the last element of anything else (`[1,\n  # c\n]`), it trails
//!   that element on its own line — a block after the arguments' `)` is not
//!   the next element;
//! - after a `when`, `in` or `rescue` body, it joins that body, so blank
//!   lines around it survive;
//! - after an `else` body that `ensure` follows, it trails the `else`
//!   clause, which prints it at the keyword's column with blank lines
//!   dropped.
//!
//! A heredoc node ends at its opener, so for placement every node's range
//! extends over the heredoc bodies it contains: a comment inside a body's
//! `#{}` reaches that interpolation's statements.
//!
//! A `def` or block body with `rescue` is a keyword-less `BeginNode` that
//! Prism locates from its parent's opener, so it spans its own siblings;
//! before its first statement only an own-line comment that no sibling
//! follows is inside it.
//!
//! `StatementsNode`, `ArgumentsNode` and `ParametersNode` are never printed
//! as nodes, so a comment aimed at one is handed to its first or last entry.

use std::num::NonZeroUsize;

use ruby_prism::{
    ArgumentsNode, BlockArgumentNode, BlockNode, BlockParameterNode, CallNode, CommentType, ConstantPathNode, ElseNode,
    EnsureNode, LocalVariableTargetNode, Node, ParametersNode, RescueNode, SplatNode, StatementsNode, Visit,
};
use rustc_hash::FxHashMap as HashMap;

/// Identity of a node: byte range plus discriminant, since Prism nodes aren't
/// hashable or clonable.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Key {
    pub start: usize,
    pub end: usize,
    pub kind: std::mem::Discriminant<Node<'static>>,
}

impl Key {
    pub fn of(node: &Node<'_>) -> Self {
        let loc = node.location();
        // SAFETY: the discriminant does not depend on the lifetime parameter.
        let kind = unsafe {
            std::mem::transmute::<std::mem::Discriminant<Node<'_>>, std::mem::Discriminant<Node<'static>>>(
                std::mem::discriminant(node),
            )
        };
        Self {
            start: loc.start_offset(),
            end: loc.end_offset(),
            kind,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Comment {
    pub start: usize,
    /// Excludes trailing whitespace, so `=end` is the last thing before it.
    pub end: usize,
    /// Starts a line (only whitespace precedes it).
    pub own_line: bool,
    /// `=begin`/`=end` block, printed verbatim from column zero.
    pub embdoc: bool,
}

#[derive(Default, Debug)]
pub struct Attached {
    comments: Vec<Comment>,
    leading_end: usize,
    header_end: usize,
}

impl Attached {
    pub fn leading(&self) -> &[Comment] {
        &self.comments[..self.leading_end]
    }

    /// Same-line comments after the node's opening token but before its
    /// first child; printed on that line, forcing the node's first group.
    pub fn header(&self) -> &[Comment] {
        &self.comments[self.leading_end..self.header_end]
    }

    /// Comments after the node; `own_line` ones print on the next line.
    pub fn trailing(&self) -> &[Comment] {
        &self.comments[self.header_end..]
    }

    fn push(&mut self, slot: Slot, comment: Comment) {
        let index = match slot {
            Slot::Leading => {
                let index = self.leading_end;
                self.leading_end += 1;
                self.header_end += 1;
                index
            }
            Slot::Header => {
                let index = self.header_end;
                self.header_end += 1;
                index
            }
            Slot::Trailing => self.comments.len(),
        };
        if index == self.comments.len() {
            self.comments.push(comment);
        } else {
            self.comments.insert(index, comment);
        }
    }
}

#[derive(Default, Debug)]
pub struct CommentMap {
    attached: HashMap<Key, Attached>,
    /// Own-line comments inside a `StatementsNode`, keyed by that node.
    body: HashMap<Key, Vec<Comment>>,
    /// Own-line comments in a node that no child claims, keyed by the node.
    dangling: HashMap<Key, Vec<Comment>>,
    /// Start offsets of every heredoc opener, in source order.
    heredoc_openings: Vec<usize>,
}

impl CommentMap {
    pub fn get(&self, node: &Node<'_>) -> Option<&Attached> {
        self.get_key(&Key::of(node))
    }

    pub fn get_key(&self, key: &Key) -> Option<&Attached> {
        self.attached.get(key)
    }

    pub fn body(&self, statements: &Node<'_>) -> &[Comment] {
        self.body.get(&Key::of(statements)).map_or(&[], Vec::as_slice)
    }

    pub fn dangling(&self, node: &Node<'_>) -> &[Comment] {
        self.dangling.get(&Key::of(node)).map_or(&[], Vec::as_slice)
    }

    pub fn heredoc_openings(&self) -> &[usize] {
        &self.heredoc_openings
    }
}

/// An optional source offset using zero as the absent value.
#[derive(Clone, Copy, Default)]
struct Offset(Option<NonZeroUsize>);

impl Offset {
    fn new(offset: Option<usize>) -> Self {
        Self(offset.map(|offset| {
            NonZeroUsize::new(offset.checked_add(1).expect("source offset fits in usize"))
                .expect("offset plus one is nonzero")
        }))
    }

    fn get(self) -> Option<usize> {
        self.0.map(|offset| offset.get() - 1)
    }
}

/// What a node's kind means for comment placement.
#[derive(Clone, Copy, Default)]
struct Role {
    statements: bool,
    /// Never printed as a node; comments go to its first or last entry.
    list: bool,
    /// Has a body or clauses, so own-line comments before its closing
    /// keyword dangle on it.
    bodied: bool,
    /// Starts a clause of its parent (`else`, `elsif`, `rescue`, `ensure`,
    /// `when`, `in`): an own-line comment before one belongs to the body it
    /// closes, not to the clause.
    clause: bool,
    else_clause: bool,
    /// A call other than `recv.attr = value`, whose `=` line is the
    /// assignment's header like any other write.
    call: bool,
    /// `begin` with the keyword or `#{`: like a call's `(`, a comment after
    /// it moves to its own line before the first statement.
    opens_statements: bool,
    /// End of the last heredoc body under this node, which lies past the
    /// node's own range (a heredoc node stops at its opener).
    heredoc_end: Offset,
    /// Start of the gap between an `else` body's last statement and the
    /// `ensure` that follows it. An own-line comment there trails the
    /// `else` clause, which prints it at the keyword's column, blank lines
    /// dropped.
    else_tail: Offset,
    /// Start of a call's message when it has a receiver: an own-line comment
    /// between the two dangles on the call and leads it in a broken chain.
    chained_message: Offset,
    /// Prints its own parentheses, so a header comment after `(` is its own.
    parameters: bool,
    /// End of an assignment's or assoc's operator, which a child (receiver,
    /// index, key) precedes: a same-line comment after the operator is the
    /// node's header, not that child's trailer.
    operator_end: Offset,
    /// Start of a `def`'s or call's `)`: a comment before it is inside the
    /// parameters or arguments, whatever follows the parenthesis.
    rparen: Offset,
    /// Where a keyword-less `BeginNode` (a `def` or block body with
    /// `rescue`) starts its own content. Prism locates it from its parent's
    /// opener, so it spans its own siblings: before this offset, only an
    /// own-line comment that no sibling follows is inside it.
    content_start: Offset,
}

struct TreeNode {
    key: Key,
    role: Role,
    /// During traversal, the first child; afterwards, its index in
    /// `Tree::children`.
    child_start: usize,
    /// During traversal, the next sibling; afterwards, the child count.
    child_count: usize,
}

const NO_INDEX: usize = usize::MAX;

struct Tree<'a> {
    source: &'a [u8],
    nodes: Vec<TreeNode>,
    /// Every node's children in source order, grouped by parent.
    children: Vec<usize>,
    /// Maximum effective end through each child prefix. This lets comment
    /// placement reject a prefix that cannot contain the comment in O(1).
    child_max_ends: Vec<usize>,
    /// `else` clauses followed by `ensure`, with the end of their body.
    else_tails: Vec<(Key, usize)>,
    stack: Vec<usize>,
    heredoc_openings: Vec<usize>,
}

impl<'pr> Visit<'pr> for Tree<'_> {
    fn visit_branch_node_enter(&mut self, node: Node<'pr>) {
        self.enter(&node);
    }

    fn visit_branch_node_leave(&mut self) {
        self.leave();
    }

    fn visit_leaf_node_enter(&mut self, node: Node<'pr>) {
        self.enter(&node);
    }

    fn visit_leaf_node_leave(&mut self) {
        self.leave();
    }

    // The default visitors reach statically typed children through these
    // typed methods, skipping the enter/leave hooks above.

    fn visit_statements_node(&mut self, node: &StatementsNode<'pr>) {
        self.typed(&node.as_node(), |tree| ruby_prism::visit_statements_node(tree, node));
    }

    fn visit_arguments_node(&mut self, node: &ArgumentsNode<'pr>) {
        self.typed(&node.as_node(), |tree| ruby_prism::visit_arguments_node(tree, node));
    }

    fn visit_else_node(&mut self, node: &ElseNode<'pr>) {
        self.typed(&node.as_node(), |tree| ruby_prism::visit_else_node(tree, node));
    }

    fn visit_constant_path_node(&mut self, node: &ConstantPathNode<'pr>) {
        self.typed(&node.as_node(), |tree| ruby_prism::visit_constant_path_node(tree, node));
    }

    fn visit_block_argument_node(&mut self, node: &BlockArgumentNode<'pr>) {
        self.typed(&node.as_node(), |tree| {
            ruby_prism::visit_block_argument_node(tree, node)
        });
    }

    fn visit_rescue_node(&mut self, node: &RescueNode<'pr>) {
        self.typed(&node.as_node(), |tree| ruby_prism::visit_rescue_node(tree, node));
    }

    fn visit_parameters_node(&mut self, node: &ParametersNode<'pr>) {
        self.typed(&node.as_node(), |tree| ruby_prism::visit_parameters_node(tree, node));
    }

    fn visit_splat_node(&mut self, node: &SplatNode<'pr>) {
        self.typed(&node.as_node(), |tree| ruby_prism::visit_splat_node(tree, node));
    }

    fn visit_local_variable_target_node(&mut self, node: &LocalVariableTargetNode<'pr>) {
        self.typed(&node.as_node(), |tree| {
            ruby_prism::visit_local_variable_target_node(tree, node)
        });
    }

    fn visit_ensure_node(&mut self, node: &EnsureNode<'pr>) {
        self.typed(&node.as_node(), |tree| ruby_prism::visit_ensure_node(tree, node));
    }

    fn visit_call_node(&mut self, node: &CallNode<'pr>) {
        self.typed(&node.as_node(), |tree| ruby_prism::visit_call_node(tree, node));
    }

    fn visit_block_parameter_node(&mut self, node: &BlockParameterNode<'pr>) {
        self.typed(&node.as_node(), |tree| {
            ruby_prism::visit_block_parameter_node(tree, node)
        });
    }

    fn visit_block_node(&mut self, node: &BlockNode<'pr>) {
        self.typed(&node.as_node(), |tree| ruby_prism::visit_block_node(tree, node));
    }
}

impl Tree<'_> {
    /// Visits a typed child, entering it unless [`Visit::visit`] already did.
    fn typed(&mut self, node: &Node<'_>, descend: impl FnOnce(&mut Self)) {
        let key = Key::of(node);
        let entered = self.stack.last().is_none_or(|&index| self.nodes[index].key != key);
        if entered {
            self.enter(node);
        }
        descend(self);
        if entered {
            self.leave();
        }
    }

    /// Pops the current node, extending its parent's range over any heredoc
    /// body it contains.
    fn leave(&mut self) {
        let index = self.stack.pop().expect("leave matches an enter");
        let end = self.nodes[index].role.heredoc_end.get();
        if let (Some(end), Some(&parent)) = (end, self.stack.last()) {
            let role = &mut self.nodes[parent].role;
            role.heredoc_end = Offset::new(role.heredoc_end.get().max(Some(end)));
        }
    }

    fn enter(&mut self, node: &Node<'_>) {
        let key = Key::of(node);
        let heredoc = heredoc_range(self.source, node);
        if let Some((opening, _)) = heredoc {
            self.heredoc_openings.push(opening);
        }
        let statements = matches!(node, Node::StatementsNode { .. });
        let role = Role {
            statements,
            list: statements || matches!(node, Node::ArgumentsNode { .. } | Node::ParametersNode { .. }),
            bodied: matches!(
                node,
                Node::ProgramNode { .. }
                    | Node::StatementsNode { .. }
                    | Node::ParenthesesNode { .. }
                    | Node::DefNode { .. }
                    | Node::ClassNode { .. }
                    | Node::ModuleNode { .. }
                    | Node::SingletonClassNode { .. }
                    | Node::BlockNode { .. }
                    | Node::LambdaNode { .. }
                    | Node::BeginNode { .. }
                    | Node::IfNode { .. }
                    | Node::UnlessNode { .. }
                    | Node::WhileNode { .. }
                    | Node::UntilNode { .. }
                    | Node::ForNode { .. }
                    | Node::CaseNode { .. }
                    | Node::CaseMatchNode { .. }
                    | Node::ElseNode { .. }
                    | Node::RescueNode { .. }
                    | Node::EnsureNode { .. }
                    | Node::WhenNode { .. }
                    | Node::InNode { .. }
            ),
            clause: match node {
                Node::ElseNode { .. }
                | Node::RescueNode { .. }
                | Node::EnsureNode { .. }
                | Node::WhenNode { .. }
                | Node::InNode { .. } => true,
                Node::IfNode { .. } => self.source[key.start..].starts_with(b"elsif"),
                _ => false,
            },
            else_clause: matches!(node, Node::ElseNode { .. }),
            opens_statements: matches!(node, Node::EmbeddedStatementsNode { .. })
                || node
                    .as_begin_node()
                    .is_some_and(|begin| begin.begin_keyword_loc().is_some()),
            heredoc_end: Offset::new(heredoc.map(|(_, end)| end)),
            else_tail: Offset::new(
                self.else_tails
                    .iter()
                    .rev()
                    .find_map(|(candidate, tail)| (*candidate == key).then_some(*tail)),
            ),
            call: node.as_call_node().is_some_and(|call| !call.is_attribute_write()),
            chained_message: Offset::new(
                node.as_call_node()
                    .filter(|call| call.receiver().is_some())
                    .and_then(|call| call.message_loc())
                    .map(|loc| loc.start_offset()),
            ),
            parameters: matches!(node, Node::ParametersNode { .. }),
            operator_end: Offset::new(operator_end(node)),
            rparen: Offset::new(
                match node {
                    Node::DefNode { .. } => node.as_def_node().and_then(|def| def.rparen_loc()),
                    Node::CallNode { .. } => node
                        .as_call_node()
                        .and_then(|call| call.closing_loc())
                        .filter(|loc| &self.source[loc.start_offset()..loc.end_offset()] == b")"),
                    _ => None,
                }
                .map(|loc| loc.start_offset()),
            ),
            content_start: Offset::new(
                node.as_begin_node()
                    .filter(|begin| begin.begin_keyword_loc().is_none())
                    .and_then(|begin| {
                        [
                            begin.statements().map(|s| s.location().start_offset()),
                            begin.rescue_clause().map(|r| r.location().start_offset()),
                            begin.else_clause().map(|e| e.location().start_offset()),
                            begin.ensure_clause().map(|e| e.location().start_offset()),
                        ]
                        .into_iter()
                        .flatten()
                        .min()
                    }),
            ),
        };
        if let Some(begin) = node.as_begin_node()
            && let (Some(else_clause), Some(_)) = (begin.else_clause(), begin.ensure_clause())
        {
            let tail = else_clause
                .statements()
                .map_or(else_clause.else_keyword_loc().end_offset(), |s| {
                    s.location().end_offset()
                });
            self.else_tails.push((Key::of(&else_clause.as_node()), tail));
        }
        let index = self.nodes.len();
        let next_sibling = self.stack.last().map_or(NO_INDEX, |&parent| {
            let next_sibling = self.nodes[parent].child_start;
            self.nodes[parent].child_start = index;
            next_sibling
        });
        self.nodes.push(TreeNode {
            key,
            role,
            child_start: NO_INDEX,
            child_count: next_sibling,
        });
        self.stack.push(index);
    }

    /// Replaces the allocation per child list with one packed adjacency
    /// array. Traversal prepends siblings, so reverse before the stable source
    /// sort to retain visitation order among equal ranges.
    fn index_children(&mut self) {
        self.children.reserve(self.nodes.len().saturating_sub(1));
        self.child_max_ends.reserve(self.nodes.len().saturating_sub(1));
        let mut siblings = Vec::new();
        for parent in 0..self.nodes.len() {
            siblings.clear();
            let mut child = self.nodes[parent].child_start;
            while child != NO_INDEX {
                siblings.push(child);
                child = self.nodes[child].child_count;
            }
            siblings.reverse();
            siblings.sort_by_key(|&child| {
                let key = self.key(child);
                (key.start, key.end)
            });
            self.nodes[parent].child_start = self.children.len();
            self.nodes[parent].child_count = siblings.len();
            let mut max_end = 0;
            for child in siblings.iter().copied() {
                let node = &self.nodes[child];
                max_end = max_end.max(node.key.end.max(node.role.heredoc_end.get().unwrap_or(0)));
                self.children.push(child);
                self.child_max_ends.push(max_end);
            }
        }
    }

    fn key(&self, index: usize) -> Key {
        self.nodes[index].key
    }

    fn role(&self, index: usize) -> &Role {
        &self.nodes[index].role
    }

    fn children(&self, index: usize) -> &[usize] {
        let node = &self.nodes[index];
        &self.children[node.child_start..node.child_start + node.child_count]
    }

    fn containing_child(&self, node: usize, at: usize, comment: &Comment, after_operator: bool) -> Option<usize> {
        let parent = &self.nodes[node];
        let mut relative = at;
        while relative > 0 {
            if self.child_max_ends[parent.child_start + relative - 1] < comment.end {
                return None;
            }
            relative -= 1;
            let child = self.children[parent.child_start + relative];
            let role = self.role(child);
            let effective_end = self.key(child).end.max(role.heredoc_end.get().unwrap_or(0));
            if comment.end > effective_end {
                continue;
            }
            if after_operator
                && role.list
                && !self
                    .children(child)
                    .iter()
                    .any(|&entry| self.key(entry).start <= comment.start && comment.end <= self.key(entry).end)
            {
                continue;
            }
            if role
                .content_start
                .get()
                .is_some_and(|start| comment.start < start && (!comment.own_line || at < self.children(node).len()))
            {
                continue;
            }
            return Some(child);
        }
        None
    }

    /// The nearest child ending before `offset`, and the nearest one
    /// starting at or after `end`.
    fn neighbours(&self, node: usize, at: usize, start: usize, end: usize) -> (Option<usize>, Option<usize>) {
        let children = self.children(node);
        let preceding = children[..at]
            .iter()
            .rev()
            .copied()
            .find(|&child| self.key(child).end <= start);
        let following = children[at..]
            .iter()
            .copied()
            .find(|&child| self.key(child).start >= end);
        (preceding, following)
    }
}

pub fn attach(source: &[u8], result: &ruby_prism::ParseResult<'_>) -> CommentMap {
    // A heredoc opener is the only formatting state collected here when the
    // parser found no comments.
    if result.comments().next().is_none() && memchr::memmem::find(source, b"<<").is_none() {
        return CommentMap::default();
    }

    let root = result.node();
    let mut tree = Tree {
        source,
        nodes: Vec::new(),
        children: Vec::new(),
        child_max_ends: Vec::new(),
        else_tails: Vec::new(),
        stack: Vec::new(),
        heredoc_openings: Vec::new(),
    };
    tree.visit(&root);
    // Prism stores some children by kind (`ParametersNode`), not by position.
    tree.index_children();
    let root_index = 0;
    debug_assert_eq!(tree.key(root_index), Key::of(&root));

    let mut map = CommentMap::default();
    let mut child_cursors = vec![0; tree.nodes.len()];
    for raw in result.comments() {
        let loc = raw.location();
        let start = loc.start_offset();
        let end = start + trim_end(&source[start..loc.end_offset()]).len();
        let comment = Comment {
            start,
            end,
            own_line: source[memchr::memrchr(b'\n', &source[..start]).map_or(0, |newline| newline + 1)..start]
                .iter()
                .all(|b| b.is_ascii_whitespace()),
            embdoc: raw.type_() == CommentType::EmbDocComment,
        };
        place(&tree, &mut child_cursors, root_index, comment, &mut map);
    }
    tree.heredoc_openings.sort_unstable();
    tree.heredoc_openings.dedup();
    map.heredoc_openings = tree.heredoc_openings;
    map
}

fn trim_end(bytes: &[u8]) -> &[u8] {
    let len = bytes
        .iter()
        .rposition(|b| !b.is_ascii_whitespace())
        .map_or(0, |i| i + 1);
    &bytes[..len]
}

fn place(tree: &Tree<'_>, child_cursors: &mut [usize], node: usize, comment: Comment, map: &mut CommentMap) {
    let children = tree.children(node);
    let cursor = &mut child_cursors[node];
    while *cursor < children.len() && tree.key(children[*cursor]).start <= comment.start {
        *cursor += 1;
    }
    let at = *cursor;
    let role = tree.role(node);
    if comment.own_line && role.else_tail.get().is_some_and(|tail| tail <= comment.start) {
        map.attached
            .entry(tree.key(node))
            .or_default()
            .push(Slot::Trailing, comment);
        return;
    }
    // `x[i] = # c`: the arguments span the index and the value, but a
    // comment after `=` that no argument contains belongs to the write.
    let after_operator = role.operator_end.get().is_some_and(|end| end <= comment.start);
    if let Some(inner) = tree.containing_child(node, at, &comment, after_operator) {
        return place(tree, child_cursors, inner, comment, map);
    }
    if comment.own_line && role.statements {
        map.body.entry(tree.key(node)).or_default().push(comment);
        return;
    }
    let (preceding, following) = tree.neighbours(node, at, comment.start, comment.end);
    // Inside the parentheses, a block after them is not a neighbour.
    let inside_parens = role.rparen.get().is_some_and(|rparen| comment.start < rparen);
    let following = following
        .filter(|&child| !inside_parens || role.rparen.get().is_some_and(|rparen| tree.key(child).start < rparen));
    if comment.own_line {
        if let Some(statements) = following.filter(|&child| tree.role(child).statements) {
            map.body.entry(tree.key(statements)).or_default().push(comment);
            return;
        }
        if let Some(statements) = preceding.filter(|&child| tree.role(child).statements) {
            map.body.entry(tree.key(statements)).or_default().push(comment);
            return;
        }
        // After a clause whose range ends with its body (`when`, `rescue`,
        // `in`), the comment belongs to that body, blank lines and all. Not
        // `else`: a comment after its body belongs beside `ensure` or `end`.
        if let Some(statements) = preceding
            .filter(|&child| tree.role(child).clause && !tree.role(child).else_clause)
            .and_then(|child| tree.children(child).last().copied())
            .filter(|&child| tree.role(child).statements)
        {
            map.body.entry(tree.key(statements)).or_default().push(comment);
            return;
        }
    }
    if comment.own_line
        && role
            .chained_message
            .get()
            .is_some_and(|message| comment.start < message)
    {
        map.dangling.entry(tree.key(node)).or_default().push(comment);
        return;
    }
    let (mut target, slot) = match (comment.own_line, preceding, following) {
        (true, Some(previous), None) if inside_parens && tree.role(previous).parameters => (previous, Slot::Trailing),
        (true, _, Some(next)) if !tree.role(next).clause => (next, Slot::Leading),
        (false, Some(_), _) if after_operator => (node, Slot::Header),
        (true, Some(prev), None) if !role.bodied => (prev, Slot::Trailing),
        (true, _, _) => {
            map.dangling.entry(tree.key(node)).or_default().push(comment);
            return;
        }
        (false, Some(prev), _) => (prev, Slot::Trailing),
        (false, None, Some(next)) if tree.role(next).parameters => (next, Slot::Header),
        (false, None, Some(next)) if role.call || (role.opens_statements && tree.role(next).statements) => {
            (next, Slot::Leading)
        }
        (false, None, _) => (node, Slot::Header),
    };
    // A list is never printed as a node; hand the comment to the entry it
    // actually sits beside. Parameters print their own `)`, so a comment
    // after it stays theirs.
    while tree.role(target).list
        && match slot {
            Slot::Leading => true,
            Slot::Trailing => !tree.role(target).parameters || inside_parens,
            Slot::Header => false,
        }
    {
        let children = tree.children(target);
        match if slot == Slot::Leading {
            children.first()
        } else {
            children.last()
        } {
            Some(&child) => target = child,
            None => break,
        }
    }
    map.attached.entry(tree.key(target)).or_default().push(slot, comment);
}

/// End of the operator of a write whose target is a child node, or of an
/// assoc (`=>`, or the label's colon). A multiple assignment is left out:
/// its comment trails the last target and breaks the target list.
fn operator_end(node: &Node<'_>) -> Option<usize> {
    let loc = match node {
        Node::AssocNode { .. } => {
            let assoc = node.as_assoc_node()?;
            Some(assoc.operator_loc().unwrap_or_else(|| assoc.key().location()))
        }
        Node::CallNode { .. } => {
            let call = node.as_call_node()?;
            // `x[i] = v` has no `=` location; the comment after `=` follows
            // the `]`.
            if call.name().as_slice() == b"[]=" {
                call.closing_loc()
            } else if call.is_attribute_write() {
                call.message_loc()
            } else {
                None
            }
        }
        Node::CallAndWriteNode { .. } => Some(node.as_call_and_write_node()?.operator_loc()),
        Node::CallOrWriteNode { .. } => Some(node.as_call_or_write_node()?.operator_loc()),
        Node::CallOperatorWriteNode { .. } => Some(node.as_call_operator_write_node()?.binary_operator_loc()),
        Node::ConstantPathWriteNode { .. } => Some(node.as_constant_path_write_node()?.operator_loc()),
        Node::ConstantPathAndWriteNode { .. } => Some(node.as_constant_path_and_write_node()?.operator_loc()),
        Node::ConstantPathOrWriteNode { .. } => Some(node.as_constant_path_or_write_node()?.operator_loc()),
        Node::ConstantPathOperatorWriteNode { .. } => {
            Some(node.as_constant_path_operator_write_node()?.binary_operator_loc())
        }
        Node::IndexAndWriteNode { .. } => Some(node.as_index_and_write_node()?.operator_loc()),
        Node::IndexOrWriteNode { .. } => Some(node.as_index_or_write_node()?.operator_loc()),
        Node::IndexOperatorWriteNode { .. } => Some(node.as_index_operator_write_node()?.binary_operator_loc()),
        _ => None,
    };
    loc.map(|loc| loc.end_offset())
}

/// Opening and closing offsets for a string node opened by `<<`.
fn heredoc_range(source: &[u8], node: &Node<'_>) -> Option<(usize, usize)> {
    let (opening, closing) = match node {
        Node::StringNode { .. } => {
            let string = node.as_string_node()?;
            (string.opening_loc()?, string.closing_loc()?)
        }
        Node::InterpolatedStringNode { .. } => {
            let string = node.as_interpolated_string_node()?;
            (string.opening_loc()?, string.closing_loc()?)
        }
        Node::XStringNode { .. } => {
            let string = node.as_x_string_node()?;
            (string.opening_loc(), string.closing_loc())
        }
        Node::InterpolatedXStringNode { .. } => {
            let string = node.as_interpolated_x_string_node()?;
            (string.opening_loc(), string.closing_loc())
        }
        _ => return None,
    };
    source[opening.start_offset()..opening.end_offset()]
        .starts_with(b"<<")
        .then(|| (opening.start_offset(), closing.end_offset()))
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Slot {
    Leading,
    Header,
    Trailing,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn placements(source: &[u8]) -> CommentMap {
        let result = ruby_prism::parse(source);
        assert!(result.errors().next().is_none());
        attach(source, &result)
    }

    /// The sole comment in a parameterised block dangles on the block, not
    /// on its parameters, so the block prints it as its body.
    #[test]
    fn sole_block_comment_dangles_on_the_block() {
        let source = b"foo do |x|\n    # todo\nend\n";
        let map = placements(source);
        let (block, comments) = map.dangling.iter().next().expect("a dangling comment");
        assert_eq!(&source[block.start..block.end], b"do |x|\n    # todo\nend");
        assert_eq!(comments.len(), 1);
        assert!(map.attached.is_empty(), "comment attached to a child: {map:?}");
    }

    #[test]
    fn indexes_heredoc_openings_during_comment_analysis() {
        let source = b"x = <<~TEXT\n  body\nTEXT\n";
        let map = placements(source);

        assert_eq!(map.heredoc_openings(), [4]);
    }
}
