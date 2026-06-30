//! Document structure (spec 002).
//!
//! Consumes a raw `Block::Content` span and produces a provisional
//! [`StructureTree`] — the role/scope skeleton the web emitter (spec 010) walks
//! and that spec 016 will subsume into the canonical AST. It recognizes the
//! three role constructs — `:::` fenced divs (whose header is an attribute
//! group), ATX headings (with pre-text and post-text attribute groups), and
//! `[text]{attrs}` inline spans —
//! and assembles them into a nested scope tree. Fence depth and the heading
//! scope rule are tracked together on one frame stack; an unbalanced or dangling
//! `:::` marker is reported as `PAPUR-P002`. Prose is held verbatim for the
//! downstream Markdown/AST pass.

use crate::attr::{Attributes, parse_attributes};
use crate::block::{Block, BlockStream, ParseMode};
use crate::diagnostic::{Diagnostic, DiagnosticCode};
use crate::span::Span;

/// The literal opening/closing content-fence marker.
const FENCE_MARKER: &str = ":::";
/// Byte length of [`FENCE_MARKER`].
const FENCE_MARKER_LEN: usize = FENCE_MARKER.len();
/// The maximum ATX heading level (`#` through `######`).
const MAX_HEADING_LEVEL: usize = 6;

/// The role/scope skeleton parsed from one content span.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructureTree {
    /// Top-level nodes in document order.
    pub nodes: Vec<Node>,
    /// The mode the span was parsed under.
    pub mode: ParseMode,
}

/// One node in the structure tree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Node {
    /// An ATX heading. A pre-text group attaches to the heading element
    /// (`opens_scope == false`); a post-text group opens a section scope
    /// (`opens_scope == true`) whose contents are `children` and whose class is
    /// carried by `attrs`.
    Heading {
        /// Heading level, 1–6.
        level: u8,
        /// The heading's text content.
        text: String,
        /// Attributes: on the heading element when `!opens_scope`, on the
        /// wrapping section when `opens_scope`.
        attrs: Attributes,
        /// Whether this heading opens a section scope.
        opens_scope: bool,
        /// Fence nesting depth (0 at the top level).
        fence_depth: u32,
        /// Section contents when `opens_scope`; empty otherwise.
        children: Vec<Node>,
        /// The span of the heading line.
        span: Span,
    },
    /// A `:::` fenced div. The header parses as an attribute group — the same
    /// grammar as a heading's `{…}`, minus the braces: a bare word names the
    /// element (`attrs.element`), `.class` adds a class, and `#id`/`key=value`
    /// apply to the element; there is no implicit primary-class "name". With no
    /// element bareword the block defaults to `<div>`. Element resolution is
    /// owned by spec 003. Nested fences are `children`.
    FencedDiv {
        /// The header's parsed attributes: element bareword, classes, id, pairs.
        attrs: Attributes,
        /// Nesting depth (0 for a top-level fence).
        fence_depth: u32,
        /// Nodes nested inside this fence.
        children: Vec<Node>,
        /// The span of the opening fence line.
        span: Span,
    },
    /// An inline `[text]{attrs}` span. Attaches to the bracketed text and never
    /// opens a scope.
    InlineSpan {
        /// The bracketed text.
        text: String,
        /// The attribute group following the bracket.
        attrs: Attributes,
        /// The span covering `[text]{attrs}`.
        span: Span,
    },
    /// A run of prose held verbatim for the downstream Markdown/AST pass.
    Prose {
        /// The raw prose text (consecutive non-construct lines, newline-joined).
        text: String,
        /// The span covering the run.
        span: Span,
    },
}

/// Parse one content span into a [`StructureTree`].
///
/// An unbalanced or dangling `:::` marker is `PAPUR-P002` in strict mode; in
/// lenient mode a dangling closer is kept as literal prose and an unterminated
/// opener is closed implicitly at end of input.
pub fn parse_structure(text: &str, mode: ParseMode) -> (StructureTree, Vec<Diagnostic>) {
    let mut diags = Vec::new();
    let mut stack: Vec<Frame> = vec![Frame::root()];
    let mut fence_count: u32 = 0;
    let mut prose = ProseBuffer::default();

    for line in lines_with_offsets(text) {
        let trimmed = line.text.trim_start();
        if let Some(after_colons) = trimmed.strip_prefix(FENCE_MARKER) {
            flush_prose(&mut prose, &mut stack, mode, &mut diags);
            if after_colons.trim().is_empty() {
                close_fence(&mut stack, &mut fence_count, &line, mode, &mut diags);
            } else {
                open_fence(&mut stack, &mut fence_count, &line, mode, &mut diags);
            }
        } else if let Some(heading) = parse_heading(&line, mode, &mut diags) {
            flush_prose(&mut prose, &mut stack, mode, &mut diags);
            handle_heading(&mut stack, fence_count, heading);
        } else {
            prose.push(&line);
        }
    }
    flush_prose(&mut prose, &mut stack, mode, &mut diags);

    // Close everything still open. An unterminated fence is `P002` in strict
    // mode; an open section just closes at end of input.
    while stack.len() > 1 {
        let frame = stack.pop().expect("non-root frame present");
        if let FrameKind::Fence(open) = &frame.kind
            && mode == ParseMode::Strict
        {
            diags.push(dangling(open.span));
        }
        if let Some(node) = finish_frame(frame) {
            current(&mut stack).push(node);
        }
    }

    let root = stack.pop().expect("root frame present");
    (
        StructureTree {
            nodes: root.children,
            mode,
        },
        diags,
    )
}

/// A parsed document: one [`StructureTree`] per content block, in source order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Document {
    /// The structure trees, one per `Block::Content` span.
    pub trees: Vec<StructureTree>,
}

/// Parse every content block of a segmented document into a structure tree and
/// run the whole-file duplicate-`id` lint across them.
///
/// Per-block diagnostics are offset from content-relative spans into
/// file-absolute spans using each content block's own span.
pub fn parse_document(stream: &BlockStream) -> (Document, Vec<Diagnostic>) {
    let mut trees = Vec::new();
    let mut diags = Vec::new();
    let mut ids: Vec<(String, Span)> = Vec::new();

    for block in &stream.blocks {
        if let Block::Content { text, span } = block {
            let (tree, block_diags) = parse_structure(text, stream.mode);
            for d in block_diags {
                diags.push(offset_diagnostic(d, *span));
            }
            collect_ids(&tree.nodes, *span, &mut ids);
            trees.push(tree);
        }
    }

    check_duplicate_ids(&ids, stream.mode, &mut diags);
    (Document { trees }, diags)
}

/// Collect every element id with its file-absolute span, walking the tree in
/// pre-order. Uses an explicit work stack rather than recursion so a
/// pathologically deep document cannot overflow the call stack.
fn collect_ids(nodes: &[Node], base: Span, out: &mut Vec<(String, Span)>) {
    let mut work: Vec<&Node> = nodes.iter().rev().collect();
    while let Some(node) = work.pop() {
        match node {
            Node::Heading {
                attrs,
                span,
                children,
                ..
            }
            | Node::FencedDiv {
                attrs,
                span,
                children,
                ..
            } => {
                push_id(attrs, *span, base, out);
                work.extend(children.iter().rev());
            }
            Node::InlineSpan { attrs, span, .. } => push_id(attrs, *span, base, out),
            Node::Prose { .. } => {}
        }
    }
}

/// Record a node's id (if any) at its file-absolute span.
fn push_id(attrs: &Attributes, span: Span, base: Span, out: &mut Vec<(String, Span)>) {
    if let Some(id) = &attrs.id {
        out.push((id.clone(), offset_span(span, base)));
    }
}

/// Emit `PAPUR-P020` for every id used more than once (strict mode only; lenient
/// keeps both).
fn check_duplicate_ids(ids: &[(String, Span)], mode: ParseMode, diags: &mut Vec<Diagnostic>) {
    if mode != ParseMode::Strict {
        return;
    }
    let mut seen = std::collections::HashSet::new();
    for (id, span) in ids {
        if !seen.insert(id.as_str()) {
            diags.push(Diagnostic::new(
                DiagnosticCode::DuplicateId,
                format!("duplicate id `{id}`"),
                *span,
            ));
        }
    }
}

/// Offset a content-relative span into file-absolute coordinates using the
/// content block's span.
fn offset_span(span: Span, base: Span) -> Span {
    Span {
        start_line: base.start_line + span.start_line - 1,
        start_col: span.start_col,
        start_byte: base.start_byte + span.start_byte,
        end_byte: base.start_byte + span.end_byte,
    }
}

/// Offset a diagnostic's span into file-absolute coordinates.
fn offset_diagnostic(d: Diagnostic, base: Span) -> Diagnostic {
    Diagnostic::new(d.code, d.message, offset_span(d.span, base))
}

/// A node being assembled, with its accumulating children.
struct Frame {
    kind: FrameKind,
    children: Vec<Node>,
}

impl Frame {
    fn root() -> Self {
        Frame {
            kind: FrameKind::Root,
            children: Vec::new(),
        }
    }
}

/// What a [`Frame`] represents.
enum FrameKind {
    /// The document root.
    Root,
    /// An open `:::` fenced div.
    Fence(OpenDiv),
    /// An open post-text-roled heading section.
    Section(OpenHeading),
}

/// A fence still open on the stack.
struct OpenDiv {
    attrs: Attributes,
    fence_depth: u32,
    span: Span,
}

/// A heading section still open on the stack.
struct OpenHeading {
    level: u8,
    text: String,
    attrs: Attributes,
    fence_depth: u32,
    span: Span,
}

/// Turn a finished frame into its node (the root yields nothing).
fn finish_frame(frame: Frame) -> Option<Node> {
    match frame.kind {
        FrameKind::Root => None,
        FrameKind::Fence(open) => Some(Node::FencedDiv {
            attrs: open.attrs,
            fence_depth: open.fence_depth,
            children: frame.children,
            span: open.span,
        }),
        FrameKind::Section(open) => Some(Node::Heading {
            level: open.level,
            text: open.text,
            attrs: open.attrs,
            opens_scope: true,
            fence_depth: open.fence_depth,
            children: frame.children,
            span: open.span,
        }),
    }
}

/// The child list of the frame currently on top of the stack.
fn current(stack: &mut [Frame]) -> &mut Vec<Node> {
    &mut stack
        .last_mut()
        .expect("stack always has the root frame")
        .children
}

/// Open a fenced div: parse its header and push a new frame. `fence_count`
/// tracks the current fence depth in O(1) and is bumped as the fence opens.
fn open_fence(
    stack: &mut Vec<Frame>,
    fence_count: &mut u32,
    line: &LineInfo,
    mode: ParseMode,
    diags: &mut Vec<Diagnostic>,
) {
    let depth = *fence_count;
    let attrs = parse_fence_header(line, mode, diags);
    stack.push(Frame {
        kind: FrameKind::Fence(OpenDiv {
            attrs,
            fence_depth: depth,
            span: line_span(line),
        }),
        children: Vec::new(),
    });
    *fence_count += 1;
}

/// Close the top fence — also closing any heading sections opened inside it
/// (rule 2: the fence closer closes scopes opened within it) — or report a
/// dangling closer when no fence is open.
fn close_fence(
    stack: &mut Vec<Frame>,
    fence_count: &mut u32,
    line: &LineInfo,
    mode: ParseMode,
    diags: &mut Vec<Diagnostic>,
) {
    if *fence_count > 0 {
        loop {
            let frame = stack.pop().expect("a fence is open");
            let is_fence = matches!(frame.kind, FrameKind::Fence(_));
            if let Some(node) = finish_frame(frame) {
                current(stack).push(node);
            }
            if is_fence {
                *fence_count -= 1;
                break;
            }
        }
    } else {
        let span = line_span(line);
        if mode == ParseMode::Strict {
            diags.push(dangling(span));
        } else {
            current(stack).push(Node::Prose {
                text: line.text.to_string(),
                span,
            });
        }
    }
}

/// A parsed heading line, before it is placed in the tree.
struct ParsedHeading {
    level: u8,
    text: String,
    attrs: Attributes,
    post_text: bool,
    span: Span,
}

/// Apply the heading scope rule, then either open a section (post-text role) or
/// emit a plain heading element (pre-text role or no role).
fn handle_heading(stack: &mut Vec<Frame>, fence_count: u32, heading: ParsedHeading) {
    // Close sections at the current fence depth whose level is equal or higher
    // (level number ≤). The loop stops at a shallower section (an ancestor) or
    // at a fence — never crossing into a different fence depth.
    while matches!(
        stack.last(),
        Some(Frame { kind: FrameKind::Section(open), .. }) if open.level >= heading.level
    ) {
        let frame = stack.pop().expect("checked Section on top");
        if let Some(node) = finish_frame(frame) {
            current(stack).push(node);
        }
    }

    let depth = fence_count;
    if heading.post_text {
        stack.push(Frame {
            kind: FrameKind::Section(OpenHeading {
                level: heading.level,
                text: heading.text,
                attrs: heading.attrs,
                fence_depth: depth,
                span: heading.span,
            }),
            children: Vec::new(),
        });
    } else {
        current(stack).push(Node::Heading {
            level: heading.level,
            text: heading.text,
            attrs: heading.attrs,
            opens_scope: false,
            fence_depth: depth,
            children: Vec::new(),
            span: heading.span,
        });
    }
}

/// Parse an ATX heading line, or `None` when the line is not a heading.
fn parse_heading(
    line: &LineInfo,
    mode: ParseMode,
    diags: &mut Vec<Diagnostic>,
) -> Option<ParsedHeading> {
    let trimmed = line.text.trim_start();
    let leading_ws = line.text.len() - trimmed.len();
    let hashes = trimmed.bytes().take_while(|b| *b == b'#').count();
    if hashes == 0 || hashes > MAX_HEADING_LEVEL {
        return None;
    }
    let after = &trimmed[hashes..];
    // ATX headings require a space after the `#` run (or an empty body).
    if !after.is_empty() && !after.starts_with(' ') {
        return None;
    }
    let after_ws = after.len() - after.trim_start().len();
    let body = after.trim();
    let body_byte = leading_ws + hashes + after_ws;

    let (text, attrs, post_text) = parse_heading_role(body, body_byte, line, mode, diags);
    Some(ParsedHeading {
        level: hashes as u8,
        text,
        attrs,
        post_text,
        span: line_span(line),
    })
}

/// Split a heading body into its text and role group, classifying the role as
/// pre-text (attaches to the element) or post-text (opens a section scope).
fn parse_heading_role(
    body: &str,
    body_byte: usize,
    line: &LineInfo,
    mode: ParseMode,
    diags: &mut Vec<Diagnostic>,
) -> (String, Attributes, bool) {
    if let Some(rest) = body.strip_prefix('{')
        && let Some(close) = rest.find('}')
    {
        let group = &rest[..close];
        let text = rest[close + 1..].trim().to_string();
        let col = body_byte + 1;
        let attrs = parse_group(
            group,
            line.start_byte + col,
            line.line_no,
            col as u32,
            mode,
            diags,
        );
        return (text, attrs, false);
    }
    if body.ends_with('}')
        && let Some(open) = body.rfind('{')
    {
        let group = &body[open + 1..body.len() - 1];
        let text = body[..open].trim().to_string();
        let col = body_byte + open + 1;
        let attrs = parse_group(
            group,
            line.start_byte + col,
            line.line_no,
            col as u32,
            mode,
            diags,
        );
        return (text, attrs, true);
    }
    (body.to_string(), Attributes::default(), false)
}

/// Parse a `:::` fence header as an attribute group — the same grammar a heading
/// uses inside `{…}`, minus the braces. A bare word names the element, a
/// `.class` adds a class, and `#id`/`key=value` apply to the element; there is no
/// implicit primary-class "name".
fn parse_fence_header(line: &LineInfo, mode: ParseMode, diags: &mut Vec<Diagnostic>) -> Attributes {
    let leading_ws = line.text.len() - line.text.trim_start().len();
    let after_colons = &line.text[leading_ws + FENCE_MARKER_LEN..];
    let header_ws = after_colons.len() - after_colons.trim_start().len();
    let header = after_colons.trim();
    let col = leading_ws + FENCE_MARKER_LEN + header_ws;
    parse_group(
        header,
        line.start_byte + col,
        line.line_no,
        col as u32,
        mode,
        diags,
    )
}

/// Parse a brace-group's inner text, offsetting its diagnostics from the group
/// position (`byte_base` / `col_base`, both relative to the source) into source
/// coordinates.
fn parse_group(
    group: &str,
    byte_base: usize,
    line_no: u32,
    col_base: u32,
    mode: ParseMode,
    diags: &mut Vec<Diagnostic>,
) -> Attributes {
    let (attrs, group_diags) = parse_attributes(group, mode);
    for d in group_diags {
        diags.push(Diagnostic::new(
            d.code,
            d.message,
            Span {
                start_line: line_no,
                start_col: col_base + d.span.start_col,
                start_byte: byte_base + d.span.start_byte,
                end_byte: byte_base + d.span.end_byte,
            },
        ));
    }
    attrs
}

/// Build a `PAPUR-P002` dangling-content-fence diagnostic.
fn dangling(span: Span) -> Diagnostic {
    Diagnostic::new(
        DiagnosticCode::DanglingContentFence,
        "unbalanced or dangling `:::` content fence",
        span,
    )
}

/// A whole-line span.
fn line_span(line: &LineInfo) -> Span {
    Span {
        start_line: line.line_no,
        start_col: 1,
        start_byte: line.start_byte,
        end_byte: line.end_byte,
    }
}

/// Flush any accumulated prose into the current frame, splitting inline spans
/// out of the run.
fn flush_prose(
    prose: &mut ProseBuffer,
    stack: &mut [Frame],
    mode: ParseMode,
    diags: &mut Vec<Diagnostic>,
) {
    if let Some((run, span)) = prose.take() {
        emit_run(&run, span, mode, current(stack), diags);
    }
}

/// Emit a prose run as `Prose` and `InlineSpan` nodes.
fn emit_run(
    run: &str,
    span: Span,
    mode: ParseMode,
    into: &mut Vec<Node>,
    diags: &mut Vec<Diagnostic>,
) {
    for seg in split_inline(run) {
        match seg {
            InlineSeg::Text { start, text } => {
                if !text.is_empty() {
                    into.push(Node::Prose {
                        text: text.to_string(),
                        span: run_span(span, start, text.len()),
                    });
                }
            }
            InlineSeg::Span {
                start,
                text,
                group_start,
                attrs,
            } => {
                let parsed = parse_group(
                    attrs,
                    span.start_byte + group_start,
                    span.start_line,
                    group_start as u32,
                    mode,
                    diags,
                );
                let end = group_start + attrs.len() + 1;
                into.push(Node::InlineSpan {
                    text: text.to_string(),
                    attrs: parsed,
                    span: Span {
                        start_line: span.start_line,
                        start_col: start as u32 + 1,
                        start_byte: span.start_byte + start,
                        end_byte: span.start_byte + end,
                    },
                });
            }
        }
    }
}

/// A span for a sub-slice of a prose run at byte offset `start`.
fn run_span(run: Span, start: usize, len: usize) -> Span {
    Span {
        start_line: run.start_line,
        start_col: start as u32 + 1,
        start_byte: run.start_byte + start,
        end_byte: run.start_byte + start + len,
    }
}

/// A segment of a prose run: plain text or an inline span.
enum InlineSeg<'a> {
    Text {
        start: usize,
        text: &'a str,
    },
    Span {
        start: usize,
        text: &'a str,
        group_start: usize,
        attrs: &'a str,
    },
}

/// Split a run into text and `[text]{attrs}` inline-span segments.
///
/// Linear in the run length: each candidate `[` consults [`try_inline`] once,
/// and on a non-match scanning resumes just past the `]` that was examined.
/// Every `[` before that `]` shares it and would fail identically, so the
/// `]`-search never revisits a byte (avoiding an O(n^2) rescan); a `[` with no
/// following `]` ends the search outright.
fn split_inline(run: &str) -> Vec<InlineSeg<'_>> {
    let mut segs = Vec::new();
    let bytes = run.as_bytes();
    let mut i = 0;
    let mut text_start = 0;
    while i < bytes.len() {
        if bytes[i] != b'[' {
            i += 1;
            continue;
        }
        match try_inline(run, i) {
            InlineMatch::Span {
                text,
                group_start,
                attrs,
                end,
            } => {
                if i > text_start {
                    segs.push(InlineSeg::Text {
                        start: text_start,
                        text: &run[text_start..i],
                    });
                }
                segs.push(InlineSeg::Span {
                    start: i,
                    text,
                    group_start,
                    attrs,
                });
                i = end;
                text_start = end;
            }
            // No `]` follows, so no span can start at `i` or any later `[`.
            InlineMatch::NoClose => break,
            // A `]` was found but no `{…}` group followed; resume past it.
            InlineMatch::NoMatch { resume } => i = resume,
        }
    }
    if text_start < run.len() {
        segs.push(InlineSeg::Text {
            start: text_start,
            text: &run[text_start..],
        });
    }
    segs
}

/// The outcome of attempting to read a `[text]{attrs}` inline span at a `[`.
enum InlineMatch<'a> {
    /// A complete span: the label, the attr group's start offset, the attr
    /// text, and the byte index just past the closing `}`.
    Span {
        text: &'a str,
        group_start: usize,
        attrs: &'a str,
        end: usize,
    },
    /// No `]` follows the `[` — no inline span can start here or later.
    NoClose,
    /// A `]` was found but no valid `{…}` group followed. `resume` is the byte
    /// just past that `]`; every `[` between the candidate and `resume` shares
    /// the same `]` and fails identically, so scanning continues from there.
    NoMatch { resume: usize },
}

/// Try to read a `[text]{attrs}` inline span at the `[` at byte `lb`.
fn try_inline(run: &str, lb: usize) -> InlineMatch<'_> {
    let after = &run[lb + 1..];
    let Some(rel) = after.find(']') else {
        return InlineMatch::NoClose;
    };
    let rb = lb + 1 + rel;
    let label = &run[lb + 1..rb];
    let rest = &run[rb + 1..];
    let resume = rb + 1;
    if !rest.starts_with('{') {
        return InlineMatch::NoMatch { resume };
    }
    let Some(close) = rest.find('}') else {
        return InlineMatch::NoMatch { resume };
    };
    let attrs = &rest[1..close];
    let group_start = rb + 2;
    let end = rb + 1 + close + 1;
    InlineMatch::Span {
        text: label,
        group_start,
        attrs,
        end,
    }
}

/// A line of source with its byte offsets, the trailing newline excluded.
struct LineInfo<'a> {
    text: &'a str,
    start_byte: usize,
    end_byte: usize,
    line_no: u32,
}

/// Split `text` into lines with byte offsets, tolerating CRLF.
fn lines_with_offsets(text: &str) -> Vec<LineInfo<'_>> {
    let mut out = Vec::new();
    let mut start = 0;
    let mut line_no = 1;
    for (i, ch) in text.char_indices() {
        if ch == '\n' {
            let raw = &text[start..i];
            let line = raw.strip_suffix('\r').unwrap_or(raw);
            out.push(LineInfo {
                text: line,
                start_byte: start,
                end_byte: start + line.len(),
                line_no,
            });
            start = i + 1;
            line_no += 1;
        }
    }
    if start < text.len() {
        let raw = &text[start..];
        let line = raw.strip_suffix('\r').unwrap_or(raw);
        out.push(LineInfo {
            text: line,
            start_byte: start,
            end_byte: start + line.len(),
            line_no,
        });
    }
    out
}

/// Accumulates consecutive non-construct lines into a single prose run.
#[derive(Default)]
struct ProseBuffer {
    text: String,
    start_byte: usize,
    end_byte: usize,
    start_line: u32,
    active: bool,
}

impl ProseBuffer {
    fn push(&mut self, line: &LineInfo) {
        if self.active {
            self.text.push('\n');
        } else {
            self.active = true;
            self.text.clear();
            self.start_byte = line.start_byte;
            self.start_line = line.line_no;
        }
        self.text.push_str(line.text);
        self.end_byte = line.end_byte;
    }

    fn take(&mut self) -> Option<(String, Span)> {
        if !self.active {
            return None;
        }
        self.active = false;
        Some((
            std::mem::take(&mut self.text),
            Span {
                start_line: self.start_line,
                start_col: 1,
                start_byte: self.start_byte,
                end_byte: self.end_byte,
            },
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::attr::Namespace;

    fn parse(text: &str) -> (StructureTree, Vec<Diagnostic>) {
        parse_structure(text, ParseMode::Strict)
    }

    fn codes(diags: &[Diagnostic]) -> Vec<&'static str> {
        diags.iter().map(|d| d.code.code()).collect()
    }

    fn as_div(node: &Node) -> (&Attributes, u32, &[Node]) {
        match node {
            Node::FencedDiv {
                attrs,
                fence_depth,
                children,
                ..
            } => (attrs, *fence_depth, children),
            other => panic!("expected FencedDiv, got {other:?}"),
        }
    }

    #[allow(clippy::type_complexity)]
    fn as_heading(node: &Node) -> (u8, &str, &Attributes, bool, &[Node]) {
        match node {
            Node::Heading {
                level,
                text,
                attrs,
                opens_scope,
                children,
                ..
            } => (*level, text, attrs, *opens_scope, children),
            other => panic!("expected Heading, got {other:?}"),
        }
    }

    fn prose_contains(nodes: &[Node], needle: &str) -> bool {
        nodes
            .iter()
            .any(|n| matches!(n, Node::Prose { text, .. } if text.contains(needle)))
    }

    // --- fenced divs (task 5) ---

    #[test]
    fn balanced_fence_nests_content() {
        let (tree, diags) = parse("::: hero\ncontent\n:::");
        assert!(diags.is_empty());
        assert_eq!(tree.nodes.len(), 1);
        let (attrs, depth, children) = as_div(&tree.nodes[0]);
        assert_eq!(attrs.element.as_deref(), Some("hero"));
        assert_eq!(depth, 0);
        assert!(prose_contains(children, "content"));
    }

    #[test]
    fn fence_header_is_an_attribute_group() {
        // Bare word → element, `.class` → class, plus `#id` and `key=value`.
        let (tree, diags) = parse("::: nav .fancy #top cols=2\n:::");
        assert!(diags.is_empty());
        let (attrs, _, _) = as_div(&tree.nodes[0]);
        assert_eq!(attrs.element.as_deref(), Some("nav"));
        assert_eq!(attrs.roles.len(), 1);
        assert_eq!(attrs.roles[0].name, "fancy");
        assert_eq!(attrs.roles[0].namespace, Namespace::Auto);
        assert_eq!(attrs.id.as_deref(), Some("top"));
        assert_eq!(attrs.attrs.get("cols").map(String::as_str), Some("2"));
    }

    #[test]
    fn fence_with_only_a_class_has_no_element() {
        // `::: .grid cols=3` → class + data-cols, no element bareword (→ <div>).
        let (tree, diags) = parse("::: .grid cols=3\n:::");
        assert!(diags.is_empty());
        let (attrs, _, _) = as_div(&tree.nodes[0]);
        assert_eq!(attrs.element, None);
        assert_eq!(attrs.roles[0].name, "grid");
        assert_eq!(attrs.attrs.get("cols").map(String::as_str), Some("3"));
    }

    #[test]
    fn nested_fences_record_depth() {
        let text = "::: grid cols=3\ninner\n  ::: card\n  deep\n  :::\n:::";
        let (tree, diags) = parse(text);
        assert!(diags.is_empty());
        let (oattrs, odepth, ochildren) = as_div(&tree.nodes[0]);
        assert_eq!(oattrs.element.as_deref(), Some("grid"));
        assert_eq!(odepth, 0);
        assert_eq!(oattrs.attrs.get("cols").map(String::as_str), Some("3"));
        let inner = ochildren
            .iter()
            .find(|n| matches!(n, Node::FencedDiv { .. }))
            .expect("nested div present");
        let (iattrs, idepth, ichildren) = as_div(inner);
        assert_eq!(iattrs.element.as_deref(), Some("card"));
        assert_eq!(idepth, 1);
        assert!(prose_contains(ichildren, "deep"));
    }

    #[test]
    fn dangling_closer_strict_is_p002() {
        let (_, diags) = parse(":::");
        assert_eq!(codes(&diags), vec!["PAPUR-P002"]);
    }

    #[test]
    fn dangling_closer_lenient_is_literal() {
        let (tree, diags) = parse_structure(":::", ParseMode::Lenient);
        assert!(diags.is_empty());
        assert!(matches!(&tree.nodes[0], Node::Prose { text, .. } if text == ":::"));
    }

    #[test]
    fn unterminated_opener_strict_is_p002() {
        let (tree, diags) = parse("::: hero\ncontent");
        assert_eq!(codes(&diags), vec!["PAPUR-P002"]);
        let (attrs, _, children) = as_div(&tree.nodes[0]);
        assert_eq!(attrs.element.as_deref(), Some("hero"));
        assert!(prose_contains(children, "content"));
    }

    // --- headings, scopes, inline spans (task 6) ---

    #[test]
    fn pre_text_role_attaches_to_heading_element() {
        let (tree, diags) = parse("### {.hero} Welcome");
        assert!(diags.is_empty());
        let (level, text, attrs, opens, _) = as_heading(&tree.nodes[0]);
        assert_eq!(level, 3);
        assert_eq!(text, "Welcome");
        assert!(!opens);
        assert_eq!(attrs.roles[0].name, "hero");
    }

    #[test]
    fn post_text_role_opens_section_scope() {
        let (tree, diags) = parse("### Welcome {.hero}\ncontent");
        assert!(diags.is_empty());
        let (level, text, attrs, opens, children) = as_heading(&tree.nodes[0]);
        assert_eq!(level, 3);
        assert_eq!(text, "Welcome");
        assert!(opens);
        assert_eq!(attrs.roles[0].name, "hero");
        assert!(prose_contains(children, "content"));
    }

    #[test]
    fn inline_span_attaches_and_opens_no_scope() {
        let (tree, diags) = parse("Click [here]{.btn} now");
        assert!(diags.is_empty());
        assert!(
            tree.nodes
                .iter()
                .all(|n| !matches!(n, Node::Heading { .. }))
        );
        let span = tree
            .nodes
            .iter()
            .find_map(|n| match n {
                Node::InlineSpan { text, attrs, .. } => Some((text.clone(), attrs.clone())),
                _ => None,
            })
            .expect("inline span present");
        assert_eq!(span.0, "here");
        assert_eq!(span.1.roles[0].name, "btn");
    }

    #[test]
    fn bracket_runs_stay_prose_and_later_span_still_matches() {
        // Unmatched `[` runs (the former O(n^2) path) stay prose, and a valid
        // span after a bracket-with-no-group is still recognized.
        let (tree, diags) = parse("[[[[ no closer here");
        assert!(diags.is_empty());
        assert_eq!(tree.nodes.len(), 1);
        assert!(matches!(&tree.nodes[0], Node::Prose { text, .. } if text == "[[[[ no closer here"));

        let (tree, diags) = parse("see [a] then [here]{.btn}");
        assert!(diags.is_empty());
        let span = tree
            .nodes
            .iter()
            .find_map(|n| match n {
                Node::InlineSpan { text, attrs, .. } => Some((text.clone(), attrs.clone())),
                _ => None,
            })
            .expect("inline span present");
        assert_eq!(span.0, "here");
        assert_eq!(span.1.roles[0].name, "btn");
        // The `[a]` (no attribute group) stays prose, not a span.
        assert!(prose_contains(&tree.nodes, "[a]"));
    }

    #[test]
    fn sibling_heading_closes_scope() {
        let (tree, _) = parse("## A {.s1}\ncontent\n## B {.s2}\nmore");
        assert_eq!(tree.nodes.len(), 2);
        let (_, a_text, _, a_opens, a_children) = as_heading(&tree.nodes[0]);
        assert_eq!(a_text, "A");
        assert!(a_opens);
        assert!(prose_contains(a_children, "content"));
        let (_, b_text, ..) = as_heading(&tree.nodes[1]);
        assert_eq!(b_text, "B");
    }

    #[test]
    fn deeper_heading_nests_scope() {
        let (tree, _) = parse("## A {.s1}\n### B {.s2}\nx");
        assert_eq!(tree.nodes.len(), 1);
        let (.., a_children) = as_heading(&tree.nodes[0]);
        let nested = a_children
            .iter()
            .find(|n| matches!(n, Node::Heading { .. }))
            .expect("nested heading");
        let (level, b_text, ..) = as_heading(nested);
        assert_eq!(level, 3);
        assert_eq!(b_text, "B");
    }

    #[test]
    fn inner_fence_does_not_close_outer_heading_scope() {
        // The multi-role nesting example from the spec.
        let text = "::: grid cols=3\n\
                    ### Fast {.carda}\n\
                    Content.\n\
                    \n\
                    \u{20}\u{20}::: grid cols=2\n\
                    \u{20}\u{20}Still in .carda.\n\
                    \n\
                    \u{20}\u{20}#### Smaller {.card1}\n\
                    \u{20}\u{20}In .carda > .card1.\n\
                    \u{20}\u{20}:::\n\
                    :::";
        let (tree, diags) = parse(text);
        assert!(diags.is_empty());

        let (gattrs, _, gchildren) = as_div(&tree.nodes[0]);
        assert_eq!(gattrs.element.as_deref(), Some("grid"));
        assert_eq!(gattrs.attrs.get("cols").map(String::as_str), Some("3"));

        let carda = gchildren
            .iter()
            .find(|n| {
                matches!(
                    n,
                    Node::Heading {
                        opens_scope: true,
                        ..
                    }
                )
            })
            .expect("carda section");
        let (clevel, ctext, cattrs, _, cchildren) = as_heading(carda);
        assert_eq!(clevel, 3);
        assert_eq!(ctext, "Fast");
        assert_eq!(cattrs.roles[0].name, "carda");

        // The nested grid sits inside carda — proving the deeper fence did not
        // close the outer heading scope.
        let inner_grid = cchildren
            .iter()
            .find(|n| matches!(n, Node::FencedDiv { .. }))
            .expect("nested grid inside carda");
        let (igattrs, _, igchildren) = as_div(inner_grid);
        assert_eq!(igattrs.element.as_deref(), Some("grid"));
        assert_eq!(igattrs.attrs.get("cols").map(String::as_str), Some("2"));

        let card1 = igchildren
            .iter()
            .find(|n| {
                matches!(
                    n,
                    Node::Heading {
                        opens_scope: true,
                        ..
                    }
                )
            })
            .expect("card1 section");
        let (cl, ct, ca, ..) = as_heading(card1);
        assert_eq!(cl, 4);
        assert_eq!(ct, "Smaller");
        assert_eq!(ca.roles[0].name, "card1");
    }

    // --- whole-file duplicate-id lint (task 7) ---

    #[test]
    fn duplicate_id_across_file_is_p020() {
        let stream =
            crate::block::segment("::: card #dup\n:::\n::: note #dup\n:::", ParseMode::Strict)
                .expect("segments");
        let (_, diags) = parse_document(&stream);
        assert_eq!(codes(&diags), vec!["PAPUR-P020"]);
    }

    #[test]
    fn distinct_ids_are_clean() {
        let stream = crate::block::segment("::: card #a\n:::\n::: note #b\n:::", ParseMode::Strict)
            .expect("segments");
        let (_, diags) = parse_document(&stream);
        assert!(diags.is_empty());
    }

    #[test]
    fn duplicate_id_lenient_is_silent() {
        let stream =
            crate::block::segment("::: card #dup\n:::\n::: note #dup\n:::", ParseMode::Lenient)
                .expect("segments");
        let (_, diags) = parse_document(&stream);
        assert!(diags.is_empty());
    }
}
