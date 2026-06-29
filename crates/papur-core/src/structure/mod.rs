//! Document structure (spec 002).
//!
//! Consumes a raw `Block::Content` span and produces a provisional
//! [`StructureTree`] — the role/scope skeleton the web emitter (spec 010) walks
//! and that spec 016 will subsume into the canonical AST. This module builds the
//! fenced-div layer: a stack-based `:::` scan that nests `::: name [attrs]`
//! blocks and reports an unbalanced or dangling marker as `PAPUR-P002`.
//! Headings, inline spans, and the heading scope rule arrive in a later task.

use crate::attr::{Attributes, parse_attributes};
use crate::block::ParseMode;
use crate::diagnostic::{Diagnostic, DiagnosticCode};
use crate::span::Span;

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
    /// A `::: name [attrs]` fenced div. `name` is the primary class; trailing
    /// attributes apply to the same element. Nested fences are `children`.
    FencedDiv {
        /// The fence name — the div's primary class.
        name: String,
        /// Trailing `.class`/`#id`/`key=value` attributes on the same element.
        attrs: Attributes,
        /// Nesting depth (0 for a top-level fence).
        fence_depth: u32,
        /// Nodes nested inside this fence.
        children: Vec<Node>,
        /// The span of the opening fence line.
        span: Span,
    },
    /// A run of prose held verbatim for the downstream Markdown/AST pass.
    Prose {
        /// The raw prose text (consecutive non-fence lines, newline-joined).
        text: String,
        /// The span covering the run.
        span: Span,
    },
}

/// Parse one content span into a [`StructureTree`], tracking fenced-div depth.
///
/// An unbalanced or dangling `:::` marker is `PAPUR-P002` in strict mode; in
/// lenient mode a dangling closer is kept as literal prose and an unterminated
/// opener is closed implicitly at end of input.
pub fn parse_structure(text: &str, mode: ParseMode) -> (StructureTree, Vec<Diagnostic>) {
    let mut diags = Vec::new();
    let mut stack: Vec<Frame> = vec![Frame::root()];
    let mut prose = ProseBuffer::default();

    for line in lines_with_offsets(text) {
        let trimmed = line.text.trim_start();
        if let Some(after_colons) = trimmed.strip_prefix(":::") {
            // A fence marker breaks any accumulating prose run.
            prose.flush(current(&mut stack));
            if after_colons.trim().is_empty() {
                close_fence(&mut stack, &line, mode, &mut diags);
            } else {
                open_fence(&mut stack, &line, mode, &mut diags);
            }
        } else {
            prose.push(&line);
        }
    }
    prose.flush(current(&mut stack));

    // Any fences still open at end of input are unterminated openers.
    while stack.len() > 1 {
        let frame = stack.pop().expect("non-root frame present");
        let open = frame.open.expect("non-root frame has an open div");
        if mode == ParseMode::Strict {
            diags.push(dangling(open.span));
        }
        let node = open.into_node(frame.children);
        current(&mut stack).push(node);
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

/// A node being assembled: the root (`open == None`) or an open fenced div.
struct Frame {
    open: Option<OpenDiv>,
    children: Vec<Node>,
}

impl Frame {
    fn root() -> Self {
        Frame {
            open: None,
            children: Vec::new(),
        }
    }
}

/// The header of a fence that is still open.
struct OpenDiv {
    name: String,
    attrs: Attributes,
    fence_depth: u32,
    span: Span,
}

impl OpenDiv {
    fn into_node(self, children: Vec<Node>) -> Node {
        Node::FencedDiv {
            name: self.name,
            attrs: self.attrs,
            fence_depth: self.fence_depth,
            children,
            span: self.span,
        }
    }
}

/// The child list of the frame currently on top of the stack.
fn current(stack: &mut [Frame]) -> &mut Vec<Node> {
    &mut stack
        .last_mut()
        .expect("stack always has the root frame")
        .children
}

/// Open a fenced div: parse its header and push a new frame.
fn open_fence(
    stack: &mut Vec<Frame>,
    line: &LineInfo,
    mode: ParseMode,
    diags: &mut Vec<Diagnostic>,
) {
    let depth = (stack.len() - 1) as u32;
    let (name, attrs) = parse_fence_header(line, mode, diags);
    stack.push(Frame {
        open: Some(OpenDiv {
            name,
            attrs,
            fence_depth: depth,
            span: line_span(line),
        }),
        children: Vec::new(),
    });
}

/// Close the top fenced div, or report a dangling closer when none is open.
fn close_fence(
    stack: &mut Vec<Frame>,
    line: &LineInfo,
    mode: ParseMode,
    diags: &mut Vec<Diagnostic>,
) {
    if stack.len() > 1 {
        let frame = stack.pop().expect("non-root frame present");
        let open = frame.open.expect("non-root frame has an open div");
        let node = open.into_node(frame.children);
        current(stack).push(node);
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

/// Parse a `name [attrs]` fence header. Trailing attributes reuse the brace-
/// group parser; their diagnostics are offset to the source position.
fn parse_fence_header(
    line: &LineInfo,
    mode: ParseMode,
    diags: &mut Vec<Diagnostic>,
) -> (String, Attributes) {
    let leading_ws = line.text.len() - line.text.trim_start().len();
    let after_colons = &line.text[leading_ws + 3..];
    let name_ws = after_colons.len() - after_colons.trim_start().len();
    let header = after_colons.trim_start();
    let name_end = header.find(char::is_whitespace).unwrap_or(header.len());
    let name = header[..name_end].to_string();

    let attr_raw = &header[name_end..];
    let attr_ws = attr_raw.len() - attr_raw.trim_start().len();
    let attr_text = attr_raw.trim();
    let (attrs, attr_diags) = parse_attributes(attr_text, mode);

    let byte_base = line.start_byte + leading_ws + 3 + name_ws + name_end + attr_ws;
    let col_base = (leading_ws + 3 + name_ws + name_end + attr_ws) as u32;
    for d in attr_diags {
        diags.push(Diagnostic::new(
            d.code,
            d.message,
            Span {
                start_line: line.line_no,
                start_col: col_base + d.span.start_col,
                start_byte: byte_base + d.span.start_byte,
                end_byte: byte_base + d.span.end_byte,
            },
        ));
    }

    (name, attrs)
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

/// Accumulates consecutive non-fence lines into a single [`Node::Prose`].
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

    fn flush(&mut self, into: &mut Vec<Node>) {
        if !self.active {
            return;
        }
        into.push(Node::Prose {
            text: std::mem::take(&mut self.text),
            span: Span {
                start_line: self.start_line,
                start_col: 1,
                start_byte: self.start_byte,
                end_byte: self.end_byte,
            },
        });
        self.active = false;
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

    #[allow(clippy::type_complexity)]
    fn as_div(node: &Node) -> (&str, &Attributes, u32, &[Node]) {
        match node {
            Node::FencedDiv {
                name,
                attrs,
                fence_depth,
                children,
                ..
            } => (name, attrs, *fence_depth, children),
            other => panic!("expected FencedDiv, got {other:?}"),
        }
    }

    #[test]
    fn balanced_fence_nests_content() {
        let (tree, diags) = parse("::: hero\ncontent\n:::");
        assert!(diags.is_empty());
        assert_eq!(tree.nodes.len(), 1);
        let (name, _, depth, children) = as_div(&tree.nodes[0]);
        assert_eq!(name, "hero");
        assert_eq!(depth, 0);
        assert!(matches!(&children[0], Node::Prose { text, .. } if text == "content"));
    }

    #[test]
    fn fence_name_and_trailing_attributes() {
        let (tree, diags) = parse("::: hero .fancy #top cols=2\n:::");
        assert!(diags.is_empty());
        let (name, attrs, _, _) = as_div(&tree.nodes[0]);
        assert_eq!(name, "hero");
        assert_eq!(attrs.roles.len(), 1);
        assert_eq!(attrs.roles[0].name, "fancy");
        assert_eq!(attrs.roles[0].namespace, Namespace::Auto);
        assert_eq!(attrs.id.as_deref(), Some("top"));
        assert_eq!(attrs.attrs.get("cols").map(String::as_str), Some("2"));
    }

    #[test]
    fn nested_fences_record_depth() {
        let text = "::: grid cols=3\ninner\n  ::: card\n  deep\n  :::\n:::";
        let (tree, diags) = parse(text);
        assert!(diags.is_empty());
        let (outer, oattrs, odepth, ochildren) = as_div(&tree.nodes[0]);
        assert_eq!(outer, "grid");
        assert_eq!(odepth, 0);
        assert_eq!(oattrs.attrs.get("cols").map(String::as_str), Some("3"));
        let inner = ochildren
            .iter()
            .find(|n| matches!(n, Node::FencedDiv { .. }))
            .expect("nested div present");
        let (iname, _, idepth, ichildren) = as_div(inner);
        assert_eq!(iname, "card");
        assert_eq!(idepth, 1);
        assert!(matches!(&ichildren[0], Node::Prose { text, .. } if text == "  deep"));
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
        let (name, _, _, children) = as_div(&tree.nodes[0]);
        assert_eq!(name, "hero");
        assert!(matches!(&children[0], Node::Prose { text, .. } if text == "content"));
    }

    #[test]
    fn unterminated_opener_lenient_has_no_diag() {
        let (tree, diags) = parse_structure("::: hero\ncontent", ParseMode::Lenient);
        assert!(diags.is_empty());
        let (name, ..) = as_div(&tree.nodes[0]);
        assert_eq!(name, "hero");
    }
}
