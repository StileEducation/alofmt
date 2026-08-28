//! Wadler-style document algebra and its printer.
//!
//! Widths are counted in fit units. A [`Doc::Indent`] may occupy a different
//! number of columns when measured than the number of spaces it emits; this
//! is an explicit printer option rather than a property of the document.

#[derive(Clone, Debug)]
pub enum Doc {
    Text(Span),
    Source(Span),
    Group(Group),
    Indent(Fragment),
    Align(u32, Fragment),
    Line(Line),
    IfBreak { broken: Fragment, flat: Fragment },
    LineSuffix { priority: u8, contents: Fragment },
    BreakParent,
    Trim,
}

#[derive(Clone, Copy, Debug)]
pub struct Group {
    pub broken: bool,
    pub contents: Fragment,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Span {
    start: u32,
    end: u32,
}

impl Span {
    fn new(start: usize, end: usize) -> Self {
        Self {
            start: u32::try_from(start).expect("a document contains fewer than 2^32 bytes"),
            end: u32::try_from(end).expect("a document contains fewer than 2^32 bytes"),
        }
    }

    pub fn range(self) -> std::ops::Range<usize> {
        self.start as usize..self.end as usize
    }
}

/// A sequence of document node IDs in [`Document::children`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Fragment {
    start: u32,
    end: u32,
}

impl Fragment {
    pub const EMPTY: Self = Self { start: 0, end: 0 };

    fn new(start: usize, end: usize) -> Self {
        Self {
            start: u32::try_from(start).expect("a document contains fewer than 2^32 edges"),
            end: u32::try_from(end).expect("a document contains fewer than 2^32 edges"),
        }
    }

    fn range(self) -> std::ops::Range<usize> {
        self.start as usize..self.end as usize
    }

    pub fn is_empty(self) -> bool {
        self.start == self.end
    }
}

/// A document tree stored in flat node and edge arenas. Container nodes refer
/// to ranges in `children`, so dropping a large document is constant-depth and
/// does not free one allocation per group.
#[derive(Debug)]
pub struct Document {
    docs: Vec<Doc>,
    children: Vec<u32>,
    text: String,
    root: Fragment,
}

impl Document {
    fn doc(&self, id: u32) -> &Doc {
        &self.docs[id as usize]
    }

    fn ids(&self, fragment: Fragment) -> &[u32] {
        &self.children[fragment.range()]
    }

    fn text(&self, span: Span) -> &str {
        &self.text[span.range()]
    }

    #[cfg(test)]
    fn iter(&self, fragment: Fragment) -> impl DoubleEndedIterator<Item = &Doc> {
        self.ids(fragment).iter().map(|&id| self.doc(id))
    }

    #[cfg(test)]
    fn root(&self) -> Fragment {
        self.root
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Separator {
    Empty,
    Space,
}

impl Separator {
    fn as_str(self) -> &'static str {
        match self {
            Self::Empty => "",
            Self::Space => " ",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Line {
    /// Emitted instead of a newline when flat.
    separator: Separator,
    /// A forced line always breaks and breaks every enclosing group.
    pub force: bool,
    /// A non-indenting line restarts at column zero (heredoc bodies, `__END__`).
    pub indent: bool,
}

pub const SOFT: Line = Line {
    separator: Separator::Empty,
    force: false,
    indent: true,
};
pub const SPACE: Line = Line {
    separator: Separator::Space,
    force: false,
    indent: true,
};
pub const HARD: Line = Line {
    separator: Separator::Space,
    force: true,
    indent: true,
};
pub const RETURN: Line = Line {
    separator: Separator::Space,
    force: true,
    indent: false,
};

pub const COMMENT_PRIORITY: u8 = 1;
pub const HEREDOC_PRIORITY: u8 = 2;

/// Builds a document tree in flat arenas. Groups are marked broken eagerly
/// (`BreakParent` propagation) so [`Builder::if_break`] can decide layouts
/// while building.
pub struct Builder {
    docs: Vec<Doc>,
    children: Vec<u32>,
    text: String,
    /// IDs in every open target. `target_starts` delimits the active suffix.
    working: Vec<u32>,
    /// Stack of open groups; each entry is that group's contents-in-progress
    /// paired with its broken flag. Index 0 is the root.
    groups: Vec<OpenGroup>,
    /// Start in `working` of each open target. Index 0 is the root.
    target_starts: Vec<usize>,
}

struct OpenGroup {
    broken: bool,
    /// Depth of `targets` when this group was opened; used to restore.
    target_depth: usize,
}

impl Default for Builder {
    fn default() -> Self {
        Self::new()
    }
}

impl Builder {
    pub fn new() -> Self {
        Self::with_capacity(0)
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            docs: Vec::with_capacity(capacity),
            children: Vec::with_capacity(capacity),
            text: String::new(),
            working: Vec::new(),
            groups: vec![OpenGroup {
                broken: false,
                target_depth: 0,
            }],
            target_starts: vec![0],
        }
    }

    fn push_doc(&mut self, doc: Doc) {
        let id = u32::try_from(self.docs.len()).expect("a document contains fewer than 2^32 nodes");
        self.docs.push(doc);
        self.working.push(id);
    }

    fn target_last(&self) -> Option<u32> {
        let start = *self.target_starts.last().expect("builder always has a target");
        (self.working.len() > start).then(|| *self.working.last().expect("non-empty target"))
    }

    fn seal_target(&mut self) -> Fragment {
        let start = self.target_starts.pop().expect("builder always has a target");
        let fragment_start = self.children.len();
        self.children.extend_from_slice(&self.working[start..]);
        self.working.truncate(start);
        Fragment::new(fragment_start, self.children.len())
    }

    /// Appends already-built docs, e.g. contents measured in a probe.
    pub fn append(&mut self, docs: Fragment) {
        self.working.extend_from_slice(&self.children[docs.range()]);
    }

    pub fn text(&mut self, s: impl AsRef<str>) {
        let s = s.as_ref();
        if s.is_empty() {
            return;
        }
        let start = self.text.len();
        self.text.push_str(s);
        let span = Span::new(start, self.text.len());
        if let Some(id) = self.target_last()
            && let Doc::Text(previous) = &mut self.docs[id as usize]
            && previous.end == span.start
        {
            previous.end = span.end;
        } else {
            self.push_doc(Doc::Text(span));
        }
    }

    pub fn text_ref(&mut self, s: &str) {
        self.text(s);
    }

    pub fn text_contents(&self, span: Span) -> &str {
        &self.text[span.range()]
    }

    pub fn source(&mut self, start: usize, end: usize) {
        if start == end {
            return;
        }
        let span = Span::new(start, end);
        if let Some(id) = self.target_last()
            && let Doc::Source(previous) = &mut self.docs[id as usize]
            && previous.end == span.start
        {
            previous.end = span.end;
        } else {
            self.push_doc(Doc::Source(span));
        }
    }

    pub fn line(&mut self, line: Line) {
        self.push_doc(Doc::Line(line));
        if line.force {
            self.break_parent();
        }
    }

    #[cfg(test)]
    pub fn space(&mut self) {
        self.line(SPACE);
    }

    pub fn soft(&mut self) {
        self.line(SOFT);
    }

    #[cfg(test)]
    pub fn hard(&mut self) {
        self.line(HARD);
    }

    pub fn trim(&mut self) {
        self.push_doc(Doc::Trim);
    }

    pub fn break_parent(&mut self) {
        self.push_doc(Doc::BreakParent);
        for group in self.groups.iter_mut().rev() {
            if group.broken {
                break;
            }
            group.broken = true;
        }
    }

    pub fn current_group_broken(&self) -> bool {
        self.groups.last().expect("root group").broken
    }

    pub fn push_target(&mut self) {
        self.target_starts.push(self.working.len());
    }

    pub fn pop_target(&mut self) -> Fragment {
        assert!(self.target_starts.len() > 1, "popped the root target");
        self.seal_target()
    }

    pub fn open_group(&mut self) {
        self.groups.push(OpenGroup {
            broken: false,
            target_depth: self.target_starts.len(),
        });
        self.push_target();
    }

    pub fn close_group(&mut self) {
        let contents = self.pop_target();
        let group = self.groups.pop().expect("open group");
        assert_eq!(
            self.target_starts.len(),
            group.target_depth,
            "group closed with containers still open"
        );
        self.push_doc(Doc::Group(Group {
            broken: group.broken,
            contents,
        }));
    }

    pub fn close_indent(&mut self) {
        let contents = self.pop_target();
        self.push_doc(Doc::Indent(contents));
    }

    pub fn close_align(&mut self, units: usize) {
        let contents = self.pop_target();
        self.push_doc(Doc::Align(
            u32::try_from(units).expect("alignment fits in 32 bits"),
            contents,
        ));
    }

    pub fn close_line_suffix(&mut self, priority: u8) {
        let contents = self.pop_target();
        self.push_doc(Doc::LineSuffix { priority, contents });
    }

    /// Snapshot of every open group's broken flag, restored after building a
    /// flat branch that can never print (see [`Builder::close_if_break`]).
    pub fn broken_flags(&self) -> Vec<bool> {
        self.groups.iter().map(|g| g.broken).collect()
    }

    pub fn restore_broken_flags(&mut self, flags: Vec<bool>) {
        for (group, was) in self.groups.iter_mut().zip(flags) {
            group.broken = was;
        }
    }

    pub fn close_if_break(&mut self, broken: Fragment, flat: Fragment) {
        self.push_doc(Doc::IfBreak { broken, flat });
    }

    /// Creates a line fragment without propagating a forced break to the open
    /// groups. This is used for a line emitted identically by both branches of
    /// an `IfBreak`.
    pub fn line_fragment(&mut self, line: Line) -> Fragment {
        let id = u32::try_from(self.docs.len()).expect("a document contains fewer than 2^32 nodes");
        self.docs.push(Doc::Line(line));
        let start = self.children.len();
        self.children.push(id);
        Fragment::new(start, self.children.len())
    }

    pub fn iter(&self, fragment: Fragment) -> impl DoubleEndedIterator<Item = &Doc> {
        self.children[fragment.range()]
            .iter()
            .map(|&id| &self.docs[id as usize])
    }

    pub fn any(&self, fragment: Fragment, predicate: impl Fn(&Doc) -> bool) -> bool {
        self.iter(fragment).any(predicate)
    }

    /// Stably partitions a consumed fragment without allocating temporary
    /// vectors. The predicate is evaluated twice.
    pub fn partition(&mut self, fragment: Fragment, predicate: impl Fn(&Doc) -> bool) -> (Fragment, Fragment) {
        let matching_start = self.children.len();
        for offset in fragment.range() {
            let id = self.children[offset];
            if predicate(&self.docs[id as usize]) {
                self.children.push(id);
            }
        }
        let matching = Fragment::new(matching_start, self.children.len());
        let other_start = self.children.len();
        for offset in fragment.range() {
            let id = self.children[offset];
            if !predicate(&self.docs[id as usize]) {
                self.children.push(id);
            }
        }
        let other = Fragment::new(other_start, self.children.len());
        (matching, other)
    }

    /// Dissolves groups, indents and aligns; keeps the flat `IfBreak` branch.
    pub fn flatten(&mut self, docs: Fragment) -> Fragment {
        self.push_target();
        self.flatten_into(docs);
        self.pop_target()
    }

    fn flatten_into(&mut self, docs: Fragment) {
        for offset in docs.range() {
            let id = self.children[offset];
            match &self.docs[id as usize] {
                Doc::Group(group) => {
                    let contents = group.contents;
                    self.flatten_into(contents);
                }
                Doc::Indent(contents) | Doc::Align(_, contents) => {
                    let contents = *contents;
                    self.flatten_into(contents);
                }
                Doc::IfBreak { flat, .. } => {
                    let flat = *flat;
                    self.flatten_into(flat);
                }
                Doc::Line(line) if !line.force => {
                    let separator = line.separator.as_str();
                    self.text(separator);
                }
                Doc::Text(_)
                | Doc::Source(_)
                | Doc::Line(_)
                | Doc::LineSuffix { .. }
                | Doc::BreakParent
                | Doc::Trim => self.working.push(id),
            }
        }
    }

    #[cfg(test)]
    pub fn group(&mut self, f: impl FnOnce(&mut Self)) {
        self.open_group();
        f(self);
        self.close_group();
    }

    #[cfg(test)]
    pub fn indent(&mut self, f: impl FnOnce(&mut Self)) {
        self.push_target();
        f(self);
        self.close_indent();
    }

    #[cfg(test)]
    /// Both branches are built; the printer picks one. A forced break inside
    /// the flat branch of an already-broken group is discarded, matching the
    /// eager-break semantics: that branch can never print.
    pub fn if_break(&mut self, broken: impl FnOnce(&mut Self), flat: impl FnOnce(&mut Self)) {
        self.push_target();
        broken(self);
        let broken = self.pop_target();
        let flat = if self.current_group_broken() {
            let flags = self.broken_flags();
            self.push_target();
            flat(self);
            self.pop_target();
            self.restore_broken_flags(flags);
            Fragment::EMPTY
        } else {
            self.push_target();
            flat(self);
            self.pop_target()
        };
        self.close_if_break(broken, flat);
    }

    #[cfg(test)]
    pub fn line_suffix(&mut self, priority: u8, f: impl FnOnce(&mut Self)) {
        self.push_target();
        f(self);
        self.close_line_suffix(priority);
    }

    pub fn finish(mut self) -> Document {
        assert_eq!(self.groups.len(), 1, "unclosed group");
        assert_eq!(self.target_starts.len(), 1, "unclosed container");
        let root = self.seal_target();
        Document {
            docs: self.docs,
            children: self.children,
            text: self.text,
            root,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode {
    Break,
    Flat,
}

#[derive(Clone, Copy)]
struct Command {
    indent: usize,
    mode: Mode,
    id: u32,
}

struct Suffix {
    order: usize,
    priority: u8,
    indent: usize,
    mode: Mode,
    contents: Fragment,
}

#[derive(Clone, Copy, Debug)]
pub struct PrintOptions {
    pub line_width: usize,
    pub indent_width: usize,
    pub fit_indent_width: usize,
    pub output_capacity: usize,
}

pub fn print(document: &Document, source: &str, options: PrintOptions) -> String {
    render(
        document,
        source,
        options,
        String::with_capacity(options.output_capacity),
    )
}

pub fn matches_source(document: &Document, source: &str, options: PrintOptions) -> bool {
    render(document, source, options, MatchOutput::new(source.as_bytes())).matches()
}

trait Output {
    fn write_str(&mut self, text: &str);
    fn write_spaces(&mut self, count: usize);
    fn trim_trailing(&mut self) -> usize;
}

impl Output for String {
    fn write_str(&mut self, text: &str) {
        self.push_str(text);
    }

    fn write_spaces(&mut self, count: usize) {
        self.extend(std::iter::repeat_n(' ', count));
    }

    fn trim_trailing(&mut self) -> usize {
        let trimmed = self.trim_end_matches([' ', '\t']).len();
        let removed = self.len() - trimmed;
        self.truncate(trimmed);
        removed
    }
}

struct MatchOutput<'a> {
    expected: &'a [u8],
    cursor: usize,
    matches: bool,
    trailing_cursor: usize,
    trailing_matches: bool,
    trailing_len: usize,
}

impl<'a> MatchOutput<'a> {
    fn new(expected: &'a [u8]) -> Self {
        Self {
            expected,
            cursor: 0,
            matches: true,
            trailing_cursor: 0,
            trailing_matches: true,
            trailing_len: 0,
        }
    }

    fn compare(&mut self, bytes: &[u8]) {
        let available = self.expected.len().saturating_sub(self.cursor).min(bytes.len());
        self.matches &= available == bytes.len()
            && self.expected.get(self.cursor..self.cursor + available) == Some(&bytes[..available]);
        self.cursor = self.cursor.saturating_add(bytes.len());
    }

    fn begin_trailing(&mut self) {
        if self.trailing_len == 0 {
            self.trailing_cursor = self.cursor;
            self.trailing_matches = self.matches;
        }
    }

    fn matches(self) -> bool {
        self.matches && self.cursor == self.expected.len()
    }
}

impl Output for MatchOutput<'_> {
    fn write_str(&mut self, text: &str) {
        let bytes = text.as_bytes();
        let prefix_len = bytes
            .iter()
            .rposition(|byte| !matches!(byte, b' ' | b'\t'))
            .map_or(0, |index| index + 1);
        if prefix_len > 0 {
            self.compare(&bytes[..prefix_len]);
            self.trailing_len = 0;
        }
        if prefix_len < bytes.len() {
            self.begin_trailing();
            self.compare(&bytes[prefix_len..]);
            self.trailing_len += bytes.len() - prefix_len;
        }
    }

    fn write_spaces(&mut self, count: usize) {
        if count == 0 {
            return;
        }
        self.begin_trailing();
        let available = self.expected.len().saturating_sub(self.cursor).min(count);
        self.matches &= available == count
            && self
                .expected
                .get(self.cursor..self.cursor.saturating_add(available))
                .is_some_and(|bytes| bytes.iter().all(|&byte| byte == b' '));
        self.cursor = self.cursor.saturating_add(count);
        self.trailing_len += count;
    }

    fn trim_trailing(&mut self) -> usize {
        let removed = self.trailing_len;
        if removed > 0 {
            self.cursor = self.trailing_cursor;
            self.matches = self.trailing_matches;
            self.trailing_len = 0;
        }
        removed
    }
}

fn render<T: Output>(document: &Document, source: &str, options: PrintOptions, mut out: T) -> T {
    debug_assert!(options.line_width > 0);
    debug_assert!(options.indent_width > 0);
    debug_assert!(options.fit_indent_width > 0);

    let mut position = 0usize;
    let mut commands = Vec::new();
    push_all(&mut commands, document, document.root, 0, Mode::Break);
    let mut remeasure = false;
    let mut suffixes: Vec<Suffix> = Vec::new();
    let mut suffix_order = 0usize;
    let mut fit_commands = Vec::new();

    while let Some(Command { indent, mode, id }) = commands.pop() {
        let doc = document.doc(id);
        match doc {
            Doc::Text(span) => {
                let text = document.text(*span);
                out.write_str(text);
                position += text_width(text);
            }
            Doc::Source(span) => {
                let text = &source[span.range()];
                out.write_str(text);
                position += text_width(text);
            }
            Doc::Group(group) => {
                if mode == Mode::Flat && !remeasure {
                    let next = if group.broken { Mode::Break } else { Mode::Flat };
                    push_all(&mut commands, document, group.contents, indent, next);
                } else {
                    remeasure = false;
                    if group.broken {
                        push_all(&mut commands, document, group.contents, indent, Mode::Break);
                    } else {
                        let mode = if fits(
                            document,
                            group.contents,
                            &commands,
                            source,
                            &mut fit_commands,
                            FitOptions {
                                indent,
                                remaining: options.line_width.saturating_sub(position),
                                within_width: position <= options.line_width,
                                indent_width: options.fit_indent_width,
                            },
                        ) {
                            Mode::Flat
                        } else {
                            Mode::Break
                        };
                        push_all(&mut commands, document, group.contents, indent, mode);
                    }
                }
            }
            Doc::Line(line) => {
                if mode == Mode::Flat {
                    if line.force {
                        remeasure = true;
                    } else {
                        let separator = line.separator.as_str();
                        out.write_str(separator);
                        position += separator.len();
                        continue;
                    }
                }
                if !suffixes.is_empty() {
                    commands.push(Command { indent, mode, id });
                    flush_suffixes(&mut commands, document, &mut suffixes);
                    continue;
                }
                if line.indent {
                    out.trim_trailing();
                    out.write_str("\n");
                    let output_indent = indent.saturating_mul(options.indent_width) / options.fit_indent_width;
                    out.write_spaces(output_indent);
                    position = indent;
                } else {
                    out.write_str("\n");
                    position = 0;
                }
            }
            Doc::Indent(contents) => push_all(
                &mut commands,
                document,
                *contents,
                indent.saturating_add(options.fit_indent_width),
                mode,
            ),
            Doc::Align(units, contents) => push_all(&mut commands, document, *contents, indent + *units as usize, mode),
            // Trimmed spaces outnumber the units they were counted as; nothing
            // reads `position` again before the next line resets it.
            Doc::Trim => position = position.saturating_sub(out.trim_trailing()),
            Doc::IfBreak { broken, flat } => match mode {
                Mode::Break if !broken.is_empty() => push_all(&mut commands, document, *broken, indent, mode),
                Mode::Flat if !flat.is_empty() => push_all(&mut commands, document, *flat, indent, mode),
                _ => {}
            },
            Doc::LineSuffix { priority, contents } => {
                suffixes.push(Suffix {
                    order: suffix_order,
                    priority: *priority,
                    indent,
                    mode,
                    contents: *contents,
                });
                suffix_order += 1;
            }
            Doc::BreakParent => {}
        }
        if commands.is_empty() && !suffixes.is_empty() {
            flush_suffixes(&mut commands, document, &mut suffixes);
        }
    }
    out
}

fn push_all(commands: &mut Vec<Command>, document: &Document, docs: Fragment, indent: usize, mode: Mode) {
    commands.extend(document.ids(docs).iter().rev().map(|&id| Command { indent, mode, id }));
}

/// Higher priority first; among equals, most recently queued first.
fn flush_suffixes(commands: &mut Vec<Command>, document: &Document, suffixes: &mut Vec<Suffix>) {
    suffixes.sort_by_key(|s| (std::cmp::Reverse(s.priority), std::cmp::Reverse(s.order)));
    for suffix in suffixes.drain(..) {
        push_all(commands, document, suffix.contents, suffix.indent, suffix.mode);
    }
}

/// Whether `contents` printed flat, followed by whatever of `rest` is needed
/// to reach the next line break, fits in `remaining` columns.
struct FitOptions {
    indent: usize,
    remaining: usize,
    within_width: bool,
    indent_width: usize,
}

fn fits(
    document: &Document,
    contents: Fragment,
    rest: &[Command],
    source: &str,
    commands: &mut Vec<Command>,
    options: FitOptions,
) -> bool {
    if !options.within_width {
        return false;
    }
    let mut remaining = options.remaining as isize;
    let mut rest_index = rest.len();
    commands.clear();
    push_all(commands, document, contents, options.indent, Mode::Flat);
    let mut trailing_whitespace = 0usize;
    while remaining >= 0 {
        let Command { indent, mode, id } = match commands.pop() {
            Some(command) => command,
            None => {
                if rest_index == 0 {
                    return true;
                }
                rest_index -= 1;
                commands.push(rest[rest_index]);
                continue;
            }
        };
        let doc = document.doc(id);
        match doc {
            Doc::Text(span) => {
                let text = document.text(*span);
                let width = text_width(text);
                trailing_whitespace = appended_trailing_whitespace(trailing_whitespace, text, width);
                remaining -= width as isize;
            }
            Doc::Source(span) => {
                let text = &source[span.range()];
                let width = text_width(text);
                trailing_whitespace = appended_trailing_whitespace(trailing_whitespace, text, width);
                remaining -= width as isize;
            }
            Doc::Group(group) => {
                let next = if group.broken { Mode::Break } else { mode };
                push_all(commands, document, group.contents, indent, next);
            }
            Doc::Line(line) => {
                if mode == Mode::Flat && !line.force {
                    let separator = line.separator.as_str();
                    let width = separator.len();
                    trailing_whitespace = appended_trailing_whitespace(trailing_whitespace, separator, width);
                    remaining -= width as isize;
                    continue;
                }
                return true;
            }
            Doc::Indent(contents) => push_all(
                commands,
                document,
                *contents,
                indent.saturating_add(options.indent_width),
                mode,
            ),
            Doc::Align(units, contents) => push_all(commands, document, *contents, indent + *units as usize, mode),
            Doc::Trim => {
                remaining += trailing_whitespace as isize;
                trailing_whitespace = 0;
            }
            Doc::IfBreak { broken, flat } => match mode {
                Mode::Break if !broken.is_empty() => push_all(commands, document, *broken, indent, mode),
                Mode::Flat if !flat.is_empty() => push_all(commands, document, *flat, indent, mode),
                _ => {}
            },
            Doc::LineSuffix { .. } | Doc::BreakParent => {}
        }
    }
    false
}

fn appended_trailing_whitespace(previous: usize, text: &str, width: usize) -> usize {
    let trailing = text
        .as_bytes()
        .iter()
        .rev()
        .take_while(|&&byte| matches!(byte, b' ' | b'\t'))
        .count();
    if trailing == width {
        previous.saturating_add(trailing)
    } else {
        trailing
    }
}

fn text_width(text: &str) -> usize {
    if text.is_ascii() {
        text.len()
    } else {
        text.chars().count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn render(f: impl FnOnce(&mut Builder), width: usize) -> String {
        let mut b = Builder::new();
        f(&mut b);
        print(
            &b.finish(),
            "",
            PrintOptions {
                line_width: width,
                indent_width: 4,
                fit_indent_width: 2,
                output_capacity: 0,
            },
        )
    }

    #[test]
    fn flat_when_it_fits() {
        let s = render(
            |b| {
                b.group(|b| {
                    b.text("[");
                    b.indent(|b| {
                        b.soft();
                        b.text("1,");
                        b.space();
                        b.text("2");
                    });
                    b.soft();
                    b.text("]");
                })
            },
            80,
        );
        assert_eq!(s, "[1, 2]");
    }

    #[test]
    fn breaks_with_four_space_indent_counted_as_two() {
        let s = render(
            |b| {
                b.group(|b| {
                    b.text("[");
                    b.indent(|b| {
                        b.soft();
                        b.text("aaaa,");
                        b.space();
                        b.text("bbbb");
                    });
                    b.soft();
                    b.text("]");
                })
            },
            8,
        );
        assert_eq!(s, "[\n    aaaa,\n    bbbb\n]");
    }

    #[test]
    fn forced_break_propagates_to_enclosing_groups() {
        let s = render(
            |b| {
                b.group(|b| {
                    b.text("a");
                    b.space();
                    b.group(|b| {
                        b.text("b");
                        b.hard();
                        b.text("c");
                    });
                })
            },
            80,
        );
        assert_eq!(s, "a\nb\nc");
    }

    #[test]
    fn line_suffix_defers_to_end_of_line() {
        let s = render(
            |b| {
                b.text("x = 1");
                b.line_suffix(COMMENT_PRIORITY, |b| b.text(" # hi"));
                b.hard();
                b.text("y");
            },
            80,
        );
        assert_eq!(s, "x = 1 # hi\ny");
    }

    #[test]
    fn if_break_selects_by_mode() {
        let doc = |b: &mut Builder| {
            b.group(|b| {
                b.text("f(");
                b.indent(|b| {
                    b.soft();
                    b.text("arg");
                    b.if_break(|b| b.text(","), |_| {});
                });
                b.soft();
                b.text(")");
            })
        };
        assert_eq!(render(doc, 80), "f(arg)");
        assert_eq!(render(doc, 4), "f(\n    arg,\n)");
    }

    #[test]
    fn trim_refunds_only_trailing_whitespace_while_fitting() {
        let s = render(
            |b| {
                b.group(|b| {
                    b.text("a  ");
                    b.trim();
                    b.space();
                    b.text("b");
                });
            },
            3,
        );

        assert_eq!(s, "a b");
    }

    #[test]
    fn output_and_fit_indentation_are_independent() {
        let mut b = Builder::new();
        b.group(|b| {
            b.text("[");
            b.indent(|b| {
                b.soft();
                b.text("item");
            });
            b.soft();
            b.text("]");
        });

        assert_eq!(
            print(
                &b.finish(),
                "",
                PrintOptions {
                    line_width: 4,
                    indent_width: 2,
                    fit_indent_width: 2,
                    output_capacity: 0,
                },
            ),
            "[\n  item\n]"
        );
    }

    #[test]
    fn compares_rendered_output_without_materialising_it() {
        let mut b = Builder::new();
        b.text("a  ");
        b.trim();
        b.push_target();
        b.line(HARD);
        b.text("b");
        b.close_indent();
        let docs = b.finish();
        let options = PrintOptions {
            line_width: 80,
            indent_width: 4,
            fit_indent_width: 2,
            output_capacity: 0,
        };

        assert_eq!(print(&docs, "", options), "a\n    b");
        assert!(matches_source(&docs, "a\n    b", options));
        assert!(!matches_source(&docs, "a  \n    b", options));
        assert!(!matches_source(&docs, "a\n  b", options));
    }

    #[test]
    fn source_spans_are_rendered_without_owned_text() {
        let mut b = Builder::new();
        b.source(0, 1);
        b.source(1, 3);

        let docs = b.finish();
        let mut root = docs.iter(docs.root());
        assert!(matches!(root.next(), Some(Doc::Source(Span { start: 0, end: 3 }))));
        assert!(root.next().is_none());
        assert_eq!(
            print(
                &docs,
                "abc",
                PrintOptions {
                    line_width: 80,
                    indent_width: 2,
                    fit_indent_width: 2,
                    output_capacity: 3,
                },
            ),
            "abc"
        );
    }
}
