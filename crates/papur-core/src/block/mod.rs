//! Block segmentation — the pre-AST pass (spec 001).
//!
//! Segmentation splits a `.papur` source into an ordered [`BlockStream`] of
//! typed layer blocks and raw content spans. It does not parse block bodies or
//! build the node-level AST (deferred to spec 016); each block holds its raw
//! source text.

use indexmap::IndexMap;

use crate::span::Span;

/// A YAML value, as parsed from `::: meta` / `::: theme` blocks and frontmatter.
/// Aliased here so the YAML backend stays a single swap-point for consumers.
pub type YamlValue = yaml_rust2::Yaml;

/// An insertion-ordered key map for meta/theme merges: preserves document order
/// while a later insertion overwrites an earlier value for the same key.
pub type KeyMap = IndexMap<String, YamlValue>;

/// Parsing strictness. [`Strict`](ParseMode::Strict) is the default — the
/// project's stated principle is "strict mode is the default".
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ParseMode {
    /// Malformed typed content is a hard parse error.
    #[default]
    Strict,
    /// Malformed typed content degrades to content prose.
    Lenient,
}

/// The five reserved layer-fence keywords 001 recognizes. Every *other* `:::`
/// fence is content/structure and is left untouched inside a [`Block::Content`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayerKind {
    /// `::: meta` — document metadata (key-value).
    Meta,
    /// `::: theme` — design tokens (key-value).
    Theme,
    /// `::: css` — the style layer (ordered).
    Css,
    /// `::: script` — the behavior layer (ordered).
    Script,
    /// `::: html` — raw HTML passthrough.
    Html,
}

impl LayerKind {
    /// Match a fence keyword against the reserved layer set; `None` for any
    /// non-reserved keyword (which is content/structure, not a layer block).
    pub fn from_keyword(keyword: &str) -> Option<Self> {
        match keyword {
            "meta" => Some(Self::Meta),
            "theme" => Some(Self::Theme),
            "css" => Some(Self::Css),
            "script" => Some(Self::Script),
            "html" => Some(Self::Html),
            _ => None,
        }
    }

    /// The reserved keyword for this layer kind.
    pub fn keyword(self) -> &'static str {
        match self {
            Self::Meta => "meta",
            Self::Theme => "theme",
            Self::Css => "css",
            Self::Script => "script",
            Self::Html => "html",
        }
    }
}

/// One segmented region of a `.papur` source, in document (source) order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Block {
    /// A raw content/prose span (Markdown, including non-reserved content
    /// fences). Captured verbatim; parsed downstream (specs 002/003/016).
    Content {
        /// The raw source text of the span.
        text: String,
        /// Where the span sits in the source.
        span: Span,
    },
    /// A reserved typed layer block with its raw, unparsed body.
    Layer {
        /// Which reserved layer this block is.
        kind: LayerKind,
        /// The raw body between the opening and closing fences.
        body: String,
        /// Where the whole block (fences included) sits in the source.
        span: Span,
    },
}

/// The ordered segmentation result for one source file. The block list is the
/// canonical record; merge views (added in a later task) are derived from it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockStream {
    /// Every block in document (source) order.
    pub blocks: Vec<Block>,
    /// The mode the source was parsed under.
    pub mode: ParseMode,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostic::{Diagnostic, DiagnosticCode};

    fn zero_span() -> Span {
        Span {
            start_line: 1,
            start_col: 1,
            start_byte: 0,
            end_byte: 0,
        }
    }

    #[test]
    fn types_are_constructible() {
        assert_eq!(ParseMode::default(), ParseMode::Strict);
        assert_eq!(LayerKind::from_keyword("css"), Some(LayerKind::Css));
        assert_eq!(LayerKind::from_keyword("grid"), None);
        assert_eq!(LayerKind::Meta.keyword(), "meta");

        let stream = BlockStream {
            blocks: vec![
                Block::Content {
                    text: "# Hi".into(),
                    span: zero_span(),
                },
                Block::Layer {
                    kind: LayerKind::Css,
                    body: ".x\n  color: red".into(),
                    span: zero_span(),
                },
            ],
            mode: ParseMode::Strict,
        };
        assert_eq!(stream.blocks.len(), 2);

        let map: KeyMap = KeyMap::new();
        assert!(map.is_empty());

        let diag = Diagnostic::new(
            DiagnosticCode::UnterminatedFence,
            "unterminated `::: css` fence",
            zero_span(),
        );
        assert_eq!(diag.code.code(), "PAPUR-P001");
    }
}
