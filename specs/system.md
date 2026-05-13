# System

The papur compiler turns a `.papur` source file into a target-agnostic AST, then dispatches the AST to one or more target emitters. The framework architecture is shared across every feature spec under `specs/`.

## Compile Pipeline

```text
source (.papur) ──▶ parser ──▶ AST ──▶ target emitter ──▶ output (HTML/CSS/text/...)
                                  │
                                  ├──▶ web emitter   ──▶ HTML + CSS
                                  ├──▶ print emitter ──▶ HTML + print-CSS (or PDF)
                                  ├──▶ email emitter ──▶ table-based HTML, inlined CSS
                                  └──▶ plain emitter ──▶ plain text
```

1. The parser consumes a `.papur` source file and produces a single AST.
2. The AST is target-agnostic — it preserves every fenced region (content, `::: theme`, `::: css`, `::: script`, `::: html`, `::: meta`) and every role attribute exactly as authored.
3. A per-target emitter walks the AST and produces output for that target. Multiple emitters can run from the same AST without re-parsing.
4. No runtime is shipped. Behaviors compile to vanilla JS at build time (see [006-behavior-layer](006-behavior-layer/spec.md)).

## Three Parallel Layers, Keyed by Role

papur separates a document into three layers that attach to the same role names:

| Layer | Block | Purpose |
| --- | --- | --- |
| **Structure** | content (prose) | What the document is |
| **Style** | `::: css` | How it looks |
| **Behavior** | `::: script` | How it acts |

A role like `.btn.primary` is the join key. The CSS block targets it, the script block attaches handlers to it, and the structure block applies it to elements. The compiler does not require the three layers to live in the same file — `@include` (see [008-includes](008-includes/spec.md)) lets each layer live in its own `.papur` file when that fits the project.

## File Inputs

Every input file uses the `.papur` extension. Filename middle segments (`styles.css.papur`, `mytheme.theme.papur`) are author signage; the parser only inspects fences. See [001-file-format](001-file-format/spec.md).

## Target Emitters

Each target emitter is its own spec:

- [010-target-web](010-target-web/spec.md) — HTML + CSS
- [011-target-print](011-target-print/spec.md) — print-CSS or PDF
- [012-target-email](012-target-email/spec.md) — table-based HTML, inlined CSS
- [013-target-plain](013-target-plain/spec.md) — plain text

The dispatch architecture, target-qualified content (`::: @web` blocks, `@print` qualifiers), and the behavior-drop rule for non-web targets are governed by [009-multi-target](009-multi-target/spec.md).

## Cross-Cutting Concerns

- **Accessibility** ([014-accessibility](014-accessibility/spec.md)) — strict-mode lint rules, bundled accessible patterns, and auto-emitted defaults that every target emitter respects.
- **Attribute syntax** ([002-attribute-syntax](002-attribute-syntax/spec.md)) — the role grammar all three layers share.
- **Semantic element registry** ([003-semantic-elements](003-semantic-elements/spec.md)) — the canonical mapping from role keywords to HTML elements.

## Open Questions

- **AST shape** — mdast-derived (with papur extensions) or a custom tree? The choice affects parser implementation but not the language surface. Resolution belongs at the start of implementation, not at design time.
