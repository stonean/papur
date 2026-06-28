# 001 — File Format Data Model

The data structures produced by **block segmentation** — the pre-AST pass 001
owns. Segmentation splits a `.papur` source into an ordered stream of typed
*layer* blocks and raw *content* spans. It does **not** parse block bodies
(Markdown, CSS, script, attributes) or build the node-level AST — the AST shape
is deferred to spec 016, and body parsing belongs to specs 002/003/004/005/006.
All types live in the `papur-core` crate's `block` module.

## Types / Structs

```rust
/// Parsing strictness. Strict is the default — the project's stated principle
/// is "strict mode is the default" (README guiding principles).
pub enum ParseMode {
    Strict,
    Lenient,
}

/// The five reserved layer-fence keywords 001 recognizes. Every *other* `:::`
/// fence (e.g. `::: grid`, `::: @web`) is content/structure, owned by specs
/// 002/003/009, and is left untouched inside a `Content` span.
pub enum LayerKind {
    Meta,
    Theme,
    Css,
    Script,
    Html,
}

/// One segmented region of a `.papur` source, in document (source) order.
pub enum Block {
    /// A raw content/prose span (Markdown, including non-reserved `:::`
    /// content fences). Captured verbatim; parsed downstream (002/003/016).
    Content { text: String, span: Span },
    /// A reserved typed layer block with its raw, unparsed body.
    Layer { kind: LayerKind, body: String, span: Span },
}

/// The ordered segmentation result for one source file. The block list is the
/// canonical record — merge views are derived from it on demand so source
/// order (needed for CSS cascade / script order and prose interleaving) is
/// never lost.
pub struct BlockStream {
    pub blocks: Vec<Block>,
    pub mode: ParseMode,
}

impl BlockStream {
    /// Fold every Meta block (including normalized frontmatter) left to right;
    /// later keys win. Empty blocks contribute nothing.
    pub fn merged_meta(&self) -> KeyMap;

    /// Fold every Theme block left to right; later keys win.
    pub fn merged_theme(&self) -> KeyMap;

    /// Css block bodies in document (source) order.
    pub fn css_blocks(&self) -> impl Iterator<Item = &str>;

    /// Script block bodies in document (source) order.
    pub fn script_blocks(&self) -> impl Iterator<Item = &str>;
}

/// Insertion-ordered key map for meta/theme merges: preserves document order
/// while letting a later insertion overwrite an earlier value for the same key
/// (last-wins). Backed by `indexmap::IndexMap`.
pub type KeyMap = IndexMap<String, YamlValue>;

/// A source location, for human-facing diagnostics.
pub struct Span {
    pub start_line: u32,   // 1-based
    pub start_col: u32,    // 1-based
    pub start_byte: usize, // 0-based offset
    pub end_byte: usize,
}
```

## Diagnostics

```rust
/// A compiler diagnostic. papur is a CLI compiler, so a diagnostic is a
/// code + message + source span — not the web/JSON envelope in the
/// `specs/errors.md` template (that template targets web APIs and does not
/// apply here). 001 establishes the compiler-diagnostic convention; see the
/// errors.md task.
pub struct Diagnostic {
    pub code: DiagnosticCode,
    pub message: String,
    pub span: Span,
}

pub enum DiagnosticCode {
    UnterminatedFence,    // PAPUR-P001 (strict mode)
    MalformedFrontmatter, // PAPUR-P010
}

/// The public library entry point. The `.papur` extension guard (AC1) is
/// enforced by the CLI *before* this is called; the library itself accepts
/// arbitrary text so tests and downstream tooling can drive it directly.
pub fn segment(source: &str, mode: ParseMode) -> Result<BlockStream, Vec<Diagnostic>>;
```

## Notes

- **`KeyMap` is insertion-ordered** (`IndexMap`) so the last-wins merge and the
  document-order authoring convention are both honored in one structure.
- **`LayerKind` is a closed set of five** — that closed set *is* the 001 scope
  boundary. Content fences (`::: grid`, target qualifiers like `::: @web`) are
  deliberately excluded; recognizing them is downstream work and needs no 001
  change.
- **`Span` carries both 1-based line/col and byte offsets** — line/col for the
  human-readable message, byte offsets for `miette`'s source highlighting.
- **Raw bodies only.** `Block::Layer.body` and `Block::Content.text` are
  unparsed source. Validating CSS/script/meta contents is specs 002/004/005/006;
  001 guarantees only correct *segmentation*, *merge*, and *mode* behavior.
