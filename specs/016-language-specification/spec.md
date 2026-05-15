---
status: draft
dependencies: []
review:
  last-run: null
  reviewed-against: null
  must-violations: 0
  should-violations: 0
  low-confidence: 0
  blocking: false
---

# 016 — Language Specification

Produce the formal language specification for papur — the artifact a parser implementor needs to build a conforming compiler. This spec consolidates the design specs (001–015) into a precise grammar + semantics document, fills the lexical and static-semantics gaps the design specs leave open, and pins down the AST shape.

The design specs define *what* the language does. The formal specification is the parser-implementable rendering of those decisions: grammar productions, AST shape, static and dynamic semantics, error catalog, and a conformance corpus.

## Deliverable Shape

The formal specification ships as a set of markdown documents under `spec/` (singular, to distinguish from govern's `specs/` directory):

- `spec/grammar.md` — EBNF (or PEG) for lexical and syntactic productions, including reserved words and identifier rules.
- `spec/ast.md` — the canonical AST shape: node types, field types, structural invariants. Resolves the AST-shape open question in `system.md`.
- `spec/static-semantics.md` — name resolution, namespace lookup, qualifier precedence, scope rules, computed-token evaluation, expressed as algorithms rather than prose.
- `spec/dynamic-semantics.md` — per-AST-node emission rules indexed by target (web / pdf / email / plain).
- `spec/css-subset.md` — the supported CSS and Sass feature set, with per-target compatibility notes.
- `spec/errors.md` — error catalog with stable codes, message templates, and severity (strict / lenient / lint-only).
- `spec/conformance/` — reference test corpus (input file + expected output per target).

The design specs under `specs/NNN-*/` remain the source of truth for *what* the language does. The formal spec under `spec/` is the precise, implementable rendering.

## Inputs

> The formal specification consolidates the following design specs:
>
> - 001-file-format, 002-attribute-syntax, 003-semantic-elements
> - 004-theming, 005-css-layer, 006-behavior-layer, 007-raw-html
> - 008-includes, 009-multi-target
> - 010-target-web, 011-target-pdf, 012-target-email, 013-target-plain
> - 014-accessibility, 015-forms (deferred — see Open Questions)

## Gaps to Close

The design specs deliberately stop at design level. The formal spec MUST close every gap below before it can claim parser-implementability:

- **AST shape** — currently an open question in `system.md`.
- **Lexical rules** for identifiers, role names, block names, theme keys, and reserved words.
- **Indentation algorithm** in `::: css`, `::: theme`, `::: script` — tabs vs. spaces, significant-whitespace handling, nesting termination.
- **Pre-text vs. post-text** detection on roled headings — the exact tokenizer condition that distinguishes inline-attach from scope-open.
- **Frontmatter merge** — same-key conflict rules beyond "later wins" when `---` YAML and `::: meta` coexist.
- **Namespace resolution algorithm** — the precise local-first → global fallback procedure, including what counts as a "local" scope boundary.
- **Variant qualifier precedence** — what wins when multiple variants apply simultaneously (e.g., `@dark @pdf`, `@reduced-motion @email`).
- **Computed-token arithmetic** — operator set, operator precedence, mixed-unit semantics, division-by-zero behavior.
- **CSS subset boundary** — which Sass features are in scope (mixins, functions, control flow), which CSS features are blocked per target.
- **Multiple blocks of same type** — resolution of the open question in 001 (single block per type, or merged across multiple).
- **Cycle detection** — exact algorithm for cyclic `@include` detection.
- **Error codes** — stable identifiers, message templates, severity per mode.

## Acceptance Criteria

- [ ] `spec/grammar.md` defines a complete EBNF (or equivalent) covering every fence type, role syntax, theme syntax, CSS subset, behavior DSL, and include directive.
- [ ] `spec/ast.md` defines every AST node type with field types and structural invariants; the AST-shape open question in `system.md` is closed.
- [ ] `spec/static-semantics.md` defines name resolution, namespace lookup, qualifier precedence, scope rules, and computed-token evaluation as algorithms.
- [ ] `spec/dynamic-semantics.md` defines the emission rule for every AST node on every target (web, pdf, email, plain).
- [ ] `spec/css-subset.md` enumerates the supported CSS / Sass feature set with per-target compatibility.
- [ ] `spec/errors.md` lists every error condition with a stable code, message template, and severity.
- [ ] `spec/conformance/` contains at least one passing test per acceptance criterion across specs 001–014.
- [ ] Every open question in the design specs (001–015) is either resolved in the formal spec or explicitly carried forward as a known limitation in `spec/grammar.md`.
- [ ] Two independent parser implementations following only the formal spec produce byte-identical output on the conformance corpus.

## Open Questions

- **Spec format** — split per layer (the directory layout above) or bundled into a single `spec.md`? Split has cleaner change diffs and lets reviewers focus; bundled is easier to grep and version.
- **Conformance test format** — directory of fixture pairs (`input.papur` + `expected.html`), a TAP-style harness, or a custom DSL? Affects contributor ergonomics and tooling.
- **Reserved words** — does papur reserve any names authors cannot use as roles (e.g., `nav`, `html`, `theme`, `script`)? The semantic-element registry implies a soft reservation; the formal spec must make it explicit.
- **Behavior DSL readiness** — spec 006 still has open questions about the DSL grammar. Does the formal spec block on those, or ship without 006 and add behavior in a follow-up edition?
- **Forms** — does the v1 formal spec include forms (currently 015 deferred), or does forms wait for its own design pass and a later edition?
- **Versioning policy** — does the formal spec adopt semver, an edition system (à la Rust), or no compatibility commitment until v1.0?
