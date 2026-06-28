# 001 — File Format Plan

Implements [001 — File Format](spec.md).

## Overview

001 stands up the Rust workspace and implements **block segmentation** — the
pre-AST pass that splits a `.papur` source into an ordered stream of typed
*layer* blocks (`meta` / `theme` / `css` / `script` / `html`) and raw *content*
spans. It normalizes leading YAML frontmatter into an implicit `::: meta` block,
applies the multi-block merge semantics resolved during clarify, and enforces
the `.papur` extension and strict/lenient mode rules.

Parsing block *bodies* (Markdown → AST, CSS, script, attributes) is explicitly
**out of scope** — 001 captures raw block bodies and document order only. The
node-level AST shape is deferred to spec 016 per `system.md`; 001 emits the
block-stream precursor that 016 and 002 build the full AST on top of.

## Technical Decisions

### Implementation language and distribution

**Rust, shipped as a single self-contained binary** per platform
(macOS / Windows / Linux). The driving constraint: an end user installs nothing
extra — no Node, no interpreter, no runtime. Distribution is
download-and-run, `brew` / `scoop`, or `cargo install`.

- Aligns with the project ethos (CLI tools like `fd` / `ripgrep`) and the
  existing `gvrn` runtime, which is already Rust.
- Honors `system.md`'s `mdast` hint: `markdown-rs` (a CommonMark parser by the
  unified/remark author) yields an mdast tree *without* a Node dependency.
  `lightningcss` covers the `::: css` layer. Both are downstream (specs that use
  them), not 001 — recorded as project-level choices in `AGENTS.md`.

### Workspace layout

A Cargo workspace with the library and CLI split:

```text
Cargo.toml                      # workspace manifest
crates/papur-core/              # library: parser, blocks, AST (later), emitters (later)
  src/lib.rs
  src/block/mod.rs              # BlockStream, Block, ParseMode, segment()
  src/block/scanner.rs          # line scanner
  src/block/frontmatter.rs      # YAML frontmatter -> meta
  src/block/merge.rs            # merged_meta / merged_theme
  src/diagnostic.rs             # Diagnostic, DiagnosticCode
crates/papur/                   # binary (package: papur): arg parsing, file load, extension guard
  src/main.rs
```

Splitting the core library from the CLI lets later specs (emitters, a future
WASM playground) depend on `papur-core` without the CLI surface — the same shape
ripgrep uses (the `grep` crates vs. the `rg` binary). 001 creates the workspace
and the `block` module.

### Block segmentation as a pre-AST pass

001 emits a `BlockStream` (ordered `Block` list plus derived merged meta/theme).
The **reserved layer-fence keywords are exactly five**: `meta`, `theme`, `css`,
`script`, `html`. A `:::` fence whose keyword is reserved becomes a `Layer`
block with a raw body; **any other** `:::` fence (`::: grid`, `::: @web`, …) is
content/structure and stays untouched inside the surrounding `Content` span for
downstream specs (002/003/009) to parse. This keeps 001 from front-running
002/003/016. See [data-model.md](data-model.md).

### Scanning algorithm (line-oriented)

- Walk the source line by line, accumulating a current `Content` span.
- A line at **column 0** matching `:::` + a single reserved keyword (no
  trailing args) **opens a layer block**. Its body accumulates until a line that
  is exactly `:::` (column 0, optional trailing whitespace) **closes** it.
- Everything else accumulates into `Content` — including non-reserved content
  fences and their `:::` markers, which pass through verbatim.
- A reserved opener with trailing arguments (`::: css foo`) is treated as a
  *content* fence, not a reserved layer block — reserved openers take no args.

### Strict vs lenient mode

`ParseMode { Strict, Lenient }`, **default Strict** (matches the project's
"strict mode is the default" principle). The spec's "typed content outside a
fence" maps to two strict-mode errors:

- An **unterminated** reserved fence (EOF before the closing `:::`) →
  `PAPUR-P001`.
- A reserved-layer construct that is otherwise malformed / a dangling typed
  marker → `PAPUR-P002`.

In **Lenient** mode both degrade to `Content` prose with no error (AC3). The CLI
exposes `--lenient`; Strict is the default.

### Extension and filename handling

The `.papur` extension guard (AC1) lives at the **CLI / file-load boundary**: a
path whose extension is not exactly `.papur` is rejected before segmentation.
The library `segment()` entry accepts arbitrary text so tests and downstream
tooling can drive it. The parser never branches on the filename (AC2) — middle
segments like `.css.papur` are human signage only.

### Frontmatter → meta normalization

If the source begins (byte 0) with a `---` line, scan to the next `---`, parse
the enclosed text as YAML into an insertion-ordered `KeyMap`, and treat it as an
implicit `::: meta` block at the top, merged with any explicit `::: meta` under
the key-value rule (AC4). A `---` that is **not** the leading block is a
Markdown thematic break and stays `Content`. Malformed frontmatter YAML →
`PAPUR-P010`. YAML is parsed with a **maintained** crate (`saphyr` /
`yaml-rust2`) — **not** the archived `serde_yaml`.

### Merge semantics

The ordered `Block` list is canonical; merge views are derived:

- `merged_meta()` / `merged_theme()` fold all matching blocks left to right,
  later keys win; empty blocks contribute nothing.
- `css_blocks()` / `script_blocks()` yield bodies in document order
  (concatenation is an emit-time concern; 001 only preserves order).

### Diagnostics

001 emits the project's first compiler diagnostics, rendered with `miette`
(source-highlighted, labeled) plus `thiserror` — friendly errors matter for the
non-technical authors papur targets. 001 also establishes the
compiler-diagnostic convention in `specs/errors.md` (the `PAPUR-Pxxx` code
family); the template's web/JSON envelope does not apply to a CLI compiler.

### Crate dependencies

`clap` (CLI), a maintained YAML crate (`saphyr` / `yaml-rust2`), `miette` +
`thiserror` (diagnostics), `indexmap` (`KeyMap`), and `insta` (dev, snapshot
tests). Per **Boundaries**, verify each crate's license is MIT / BSD / ISC /
Apache-2.0 before adding, and record any Apache-2.0 `NOTICE` in
`THIRD_PARTY_NOTICES`.

## Affected Files

| File | Action | Purpose |
| --- | --- | --- |
| `Cargo.toml` | Create | Workspace manifest (members: `papur-core`, `papur`) |
| `crates/papur-core/Cargo.toml` | Create | Core library manifest + deps |
| `crates/papur-core/src/lib.rs` | Create | Core crate root and re-exports |
| `crates/papur-core/src/block/mod.rs` | Create | `BlockStream`, `Block`, `ParseMode`, `segment()` |
| `crates/papur-core/src/block/scanner.rs` | Create | Line scanner / reserved-fence segmentation |
| `crates/papur-core/src/block/frontmatter.rs` | Create | YAML frontmatter → meta normalization |
| `crates/papur-core/src/block/merge.rs` | Create | `merged_meta` / `merged_theme`, ordered accessors |
| `crates/papur-core/src/diagnostic.rs` | Create | `Diagnostic`, `DiagnosticCode`, `PAPUR-P` codes |
| `crates/papur/Cargo.toml` | Create | Binary crate manifest (package `papur`) |
| `crates/papur/src/main.rs` | Create | `clap` CLI, extension guard, `--lenient`, diagnostic rendering |
| `crates/papur-core/tests/segmentation.rs` | Create | Acceptance-criteria + snapshot tests |
| `AGENTS.md` | Modify | Fill Tech Stack and Commands sections |
| `specs/errors.md` | Modify | Establish compiler-diagnostic convention + `PAPUR-P` registry |
| `.gitignore` | Modify | Ignore `/target` |
| `README.md` | Modify | Note build / run commands (optional) |

## Trade-offs

- **Segmentation separate from the AST** — a second pass over the source rather
  than one-shot parsing. Chosen for the clean 001/016 spec boundary and the
  target-agnostic block preservation `system.md` requires; the extra traversal
  cost is negligible.
- **Raw block bodies (no body parsing in 001)** — 001 cannot validate
  CSS/script/meta contents, so malformed bodies surface later. Accepted: those
  are specs 002/004/005/006; 001 stays minimal and stable.
- **YAML crate** — `serde_yaml` is archived; `saphyr` / `yaml-rust2` is
  maintained but lower-level (manual map building). Accepted for maintenance
  safety over convenience.
- **Reserved layer-keyword set hardcoded (five)** — adding a sixth layer type
  later touches 001. Accepted: the layer set is a deliberately closed,
  foundational decision, whereas content fences are open-ended and need no 001
  change.
- **`miette` dependency** — adds weight, but source-highlighted errors are a
  product requirement for non-technical authors. Plain `eprintln!` rejected.
- **Known limitation** — precise strict/lenient lexing edge cases (nested
  reserved fences, CRLF line endings, tabs in fence bodies) are refined during
  implement; the data-model defines the contract, the tests pin the behavior.

## Data Model

See [data-model.md](data-model.md) — 001 introduces the block-segmentation
types (`BlockStream`, `Block`, `LayerKind`, `ParseMode`, `Span`, `KeyMap`) and
the `Diagnostic` model.
