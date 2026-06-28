//! The line-oriented block scanner (spec 001, task 3).
//!
//! Splits a `.papur` source into an ordered list of [`Block`]s: reserved layer
//! fences (`::: meta|theme|css|script|html`) become [`Block::Layer`]; everything
//! else — prose and non-reserved content fences (`::: grid`, `::: @web`) — is
//! captured verbatim as [`Block::Content`]. Block bodies are not parsed here.

use crate::diagnostic::{Diagnostic, DiagnosticCode};
use crate::span::Span;

use super::{Block, LayerKind, ParseMode};

/// A single source line, without its trailing newline.
struct Line<'a> {
    /// 1-based line number.
    num: u32,
    /// Byte offset of the line's first character in the source.
    start: usize,
    /// The line text, excluding the trailing `\n` (a trailing `\r` is kept).
    text: &'a str,
}

fn split_lines(source: &str) -> Vec<Line<'_>> {
    let mut lines = Vec::new();
    let mut offset = 0usize;
    for (idx, chunk) in source.split_inclusive('\n').enumerate() {
        let text = chunk.strip_suffix('\n').unwrap_or(chunk);
        lines.push(Line {
            num: idx as u32 + 1,
            start: offset,
            text,
        });
        offset += chunk.len();
    }
    lines
}

/// A reserved layer opener: `:::` at column 0, whitespace, then exactly one
/// reserved keyword and nothing else. `::: css foo` and `::: grid` are *not*
/// reserved openers — they are content fences and pass through as content.
fn reserved_opener(text: &str) -> Option<LayerKind> {
    let rest = text.strip_prefix(":::")?;
    if !rest.starts_with([' ', '\t']) {
        return None;
    }
    let keyword = rest.trim_matches([' ', '\t', '\r']);
    if keyword.is_empty() || keyword.contains([' ', '\t']) {
        return None;
    }
    LayerKind::from_keyword(keyword)
}

/// A bare close marker: `:::` at column 0, nothing after it but whitespace.
fn is_close(text: &str) -> bool {
    match text.strip_prefix(":::") {
        Some(rest) => rest.trim_matches([' ', '\t', '\r']).is_empty(),
        None => false,
    }
}

/// Flush the pending content run `[start, end)` as a [`Block::Content`], unless
/// it is empty or all-whitespace (separators between layer blocks are dropped).
fn flush_content(
    source: &str,
    pending: &mut Option<(usize, u32)>,
    end: usize,
    blocks: &mut Vec<Block>,
) {
    if let Some((start, start_line)) = pending.take() {
        let raw = &source[start..end];
        if !raw.trim().is_empty() {
            blocks.push(Block::Content {
                text: raw.to_string(),
                span: Span {
                    start_line,
                    start_col: 1,
                    start_byte: start,
                    end_byte: end,
                },
            });
        }
    }
}

/// Segment `source` into blocks, collecting diagnostics. An unterminated
/// reserved fence records a `P001` in strict mode; both modes then degrade the
/// opener to content so scanning can continue.
pub(super) fn scan(source: &str, mode: ParseMode) -> (Vec<Block>, Vec<Diagnostic>) {
    let lines = split_lines(source);
    let n = lines.len();
    let mut blocks = Vec::new();
    let mut diags = Vec::new();
    let mut pending: Option<(usize, u32)> = None;

    let mut i = 0usize;
    while i < n {
        let line = &lines[i];

        let Some(kind) = reserved_opener(line.text) else {
            // ordinary content line: prose or a non-reserved content fence
            pending.get_or_insert((line.start, line.num));
            i += 1;
            continue;
        };

        // a reserved opener — scan forward for its closing `:::`
        let mut j = i + 1;
        while j < n && !is_close(lines[j].text) {
            j += 1;
        }

        if j >= n {
            // unterminated fence
            if matches!(mode, ParseMode::Strict) {
                diags.push(Diagnostic::new(
                    DiagnosticCode::UnterminatedFence,
                    format!("unterminated `::: {}` fence", kind.keyword()),
                    Span {
                        start_line: line.num,
                        start_col: 1,
                        start_byte: line.start,
                        end_byte: line.start + line.text.len(),
                    },
                ));
            }
            // both modes: degrade the opener to content and keep scanning
            pending.get_or_insert((line.start, line.num));
            i += 1;
            continue;
        }

        // a well-formed layer block: flush content before it, then emit it
        flush_content(source, &mut pending, line.start, &mut blocks);

        let close = &lines[j];
        let body = if i + 1 < j {
            let raw = &source[lines[i + 1].start..close.start];
            raw.strip_suffix('\n').unwrap_or(raw).to_string()
        } else {
            String::new()
        };
        blocks.push(Block::Layer {
            kind,
            body,
            span: Span {
                start_line: line.num,
                start_col: 1,
                start_byte: line.start,
                end_byte: close.start + close.text.len(),
            },
        });
        i = j + 1;
    }

    flush_content(source, &mut pending, source.len(), &mut blocks);
    (blocks, diags)
}

#[cfg(test)]
mod tests {
    use crate::block::{segment, Block, LayerKind, ParseMode};

    fn kinds(blocks: &[Block]) -> Vec<&'static str> {
        blocks
            .iter()
            .map(|b| match b {
                Block::Content { .. } => "content",
                Block::Layer { kind, .. } => kind.keyword(),
            })
            .collect()
    }

    #[test]
    fn segments_mixed_content_in_order() {
        let src = "::: meta\ntitle: x\n:::\n\n# Heading\n\n::: css\n.a\n  color: red\n:::\n\nBody text.\n\n::: script\n.a\n  on click\n:::\n";
        let stream = segment(src, ParseMode::Strict).unwrap();
        assert_eq!(kinds(&stream.blocks), ["meta", "content", "css", "content", "script"]);
        match &stream.blocks[0] {
            Block::Layer { kind, body, .. } => {
                assert_eq!(*kind, LayerKind::Meta);
                assert_eq!(body, "title: x");
            }
            other => panic!("expected meta layer, got {other:?}"),
        }
        match &stream.blocks[2] {
            Block::Layer { body, .. } => assert_eq!(body, ".a\n  color: red"),
            other => panic!("expected css layer, got {other:?}"),
        }
    }

    #[test]
    fn content_fences_pass_through_untouched() {
        let src = "::: grid cols=3\n### Card {.card}\nContent.\n:::\n";
        let stream = segment(src, ParseMode::Strict).unwrap();
        assert_eq!(stream.blocks.len(), 1);
        match &stream.blocks[0] {
            Block::Content { text, .. } => {
                assert!(text.contains("::: grid cols=3"));
                assert!(text.contains(":::"));
            }
            other => panic!("expected a single content block, got {other:?}"),
        }
    }

    #[test]
    fn prose_only_file_is_one_content_block() {
        let stream = segment("Just some prose.\n\nMore prose.\n", ParseMode::Strict).unwrap();
        assert_eq!(kinds(&stream.blocks), ["content"]);
    }

    #[test]
    fn empty_source_yields_no_blocks() {
        let stream = segment("", ParseMode::Strict).unwrap();
        assert!(stream.blocks.is_empty());
    }

    #[test]
    fn empty_layer_block_has_empty_body() {
        let stream = segment("::: css\n:::\n", ParseMode::Strict).unwrap();
        assert_eq!(stream.blocks.len(), 1);
        match &stream.blocks[0] {
            Block::Layer { kind, body, .. } => {
                assert_eq!(*kind, LayerKind::Css);
                assert_eq!(body, "");
            }
            other => panic!("expected empty css layer, got {other:?}"),
        }
    }

    #[test]
    fn span_tracks_line_and_byte() {
        let src = "# Title\n\n::: css\n.a\n:::\n";
        let stream = segment(src, ParseMode::Strict).unwrap();
        let css = stream
            .blocks
            .iter()
            .find_map(|b| match b {
                Block::Layer {
                    kind: LayerKind::Css,
                    span,
                    ..
                } => Some(*span),
                _ => None,
            })
            .unwrap();
        assert_eq!(css.start_line, 3);
        assert_eq!(css.start_col, 1);
        assert_eq!(&src[css.start_byte..css.start_byte + 3], ":::");
    }

    #[test]
    fn strict_unterminated_fence_is_p001() {
        let src = "# Intro\n\n::: css\n.a\n  color: red\n"; // no closing :::
        let err = segment(src, ParseMode::Strict).unwrap_err();
        assert_eq!(err.len(), 1);
        assert_eq!(err[0].code.code(), "PAPUR-P001");
        assert_eq!(err[0].span.start_line, 3);
    }

    #[test]
    fn lenient_unterminated_fence_degrades_to_content() {
        let src = "# Intro\n\n::: css\n.a\n  color: red\n";
        let stream = segment(src, ParseMode::Lenient).unwrap();
        assert_eq!(kinds(&stream.blocks), ["content"]);
        match &stream.blocks[0] {
            Block::Content { text, .. } => assert!(text.contains("::: css")),
            other => panic!("expected content, got {other:?}"),
        }
    }

    #[test]
    fn default_mode_is_strict() {
        assert_eq!(ParseMode::default(), ParseMode::Strict);
    }
}
