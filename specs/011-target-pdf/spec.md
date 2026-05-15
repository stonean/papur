---
status: draft
dependencies: [003-semantic-elements, 006-behavior-layer, 007-raw-html, 009-multi-target, 010-target-web, 014-accessibility]
review:
  last-run: null
  reviewed-against: null
  must-violations: 0
  should-violations: 0
  low-confidence: 0
  blocking: false
---

# 011 — PDF Target

Emission rules for the PDF target. The PDF emitter produces a tagged PDF document directly from the papur AST, without driving an external HTML rendering engine. Dispatch architecture lives in [009-multi-target](../009-multi-target/spec.md).

This is the canonical path for a paginated, frozen, no-JS artifact. The web target's browser save-as-PDF is a lower-fidelity fallback covered as a scenario under [010-target-web](../010-target-web/spec.md).

## Output Shape

- **PDF bytes** — a single PDF document conformant with Tagged PDF (PDF/UA target). Consumable by Adobe Acrobat, `pdf.js`, and macOS Preview.
- **Structure tree** — headings, lists, table structure, reading order, and image alt text are tagged from the AST, not inferred from glyph positions.
- **No JS** — the behavior layer ([006-behavior-layer](../006-behavior-layer/spec.md)) is silently dropped.
- **No HTML pipeline** — the emitter does not go through HTML and a browser engine. See "Rejected approach" below.

## Theme Tokens on PDF

Theme tokens are inlined as their resolved values. `$brand` emits the value from the `@pdf` variant block if present, otherwise the default. CSS custom properties (`var()`) are not used — the output is a frozen artifact.

## Layout Pipeline

The emitter is shaped around one new intermediate representation between the AST and the PDF bytes — a **positioned box tree** carrying resolved geometry and shaped glyph runs. Layout, line breaking, and page breaking operate on this tree. The emitter that writes PDF bytes walks the positioned box tree and does no layout decisions of its own.

The box tree is testable without emitting a single PDF byte — layout correctness is asserted on box positions in unit tests, independent of the byte format. Implementation details (box-tree fields, page-break algorithm, font-subsetting library choice) belong to `plan.md`.

## Accessibility

PDF/UA is the conformance target. The structure tree is built from the AST's semantic elements ([003-semantic-elements](../003-semantic-elements/spec.md)) and the accessibility commitments in [014-accessibility](../014-accessibility/spec.md). Alt text, heading levels, list structure, table structure, and reading order are propagated from the AST into PDF structure-tree roles in a single traversal.

## Fonts

- Standard-14 fonts are supported with no embedding (metrics only).
- Author-supplied TrueType / OpenType fonts MUST embed as subsets containing only the glyphs the document uses.
- Missing fonts, unresolvable font references, and unsupported glyph requests are compile-time diagnostics. The emitter MUST NOT silently fall back to a default font.

## Security

The emitter MUST NOT perform network access at any point during compilation. Asset references (images, fonts) resolve against a caller-provided, explicitly-scoped resource root. The compiler is responsible for every dereference; nothing in the PDF pipeline can be tricked into fetching an arbitrary URL.

This commitment is what makes the direct-emission path defensible against the SSRF surface a browser-based pipeline would expose. See `BE-INPUT-007` in `specs/security-backend.md`.

## What Drops

- `::: script` blocks — silently dropped.
- `::: html` blocks — dropped unless explicitly qualified `::: @pdf html` ([007-raw-html](../007-raw-html/spec.md)).
- `::: @web` and `::: @email` qualified blocks — dropped.
- Auto-emitted web defaults (skip link, viewport meta tag) — dropped (not meaningful in PDF).

## What Stays

- Content prose and roles, with semantic-element mapping preserved as PDF structure-tree roles.
- `@pdf`-qualified blocks across `::: theme`, `::: css`, and content.
- `::: @pdf` content blocks.
- `prefers-color-scheme` and other media-condition variants — collapsed at compile time (PDF is not interactive).

## Page Geometry

Page size and margins are taken from the `@page`-equivalent settings in the `::: theme` and `::: meta` regions. A document with no explicit page geometry uses the project default; the default size is a configuration choice resolved at `/clarify`.

## Page Breaking

- Widow / orphan control on body text.
- Keep-with-next on headings (a heading MUST NOT be the last laid-out line on a page).
- `break-inside: avoid` honored on roles that set it.
- Table-row splitting repeats the header row at the top of each continuation page.

## Rejected Approach: Headless Chrome

Rendering papur's HTML output via headless Chrome (e.g. `chromedp` + `Page.printToPDF`) was considered and rejected on two grounds:

1. **Distribution.** Chrome cannot be linked into the Go binary and cannot be cross-compiled. Adopting it ends papur's single-static-binary distribution story and replaces it with a bundle-or-download model.
2. **Security.** HTML-to-PDF via a real browser engine inherits the browser's full network and parsing attack surface. The prominent risk is **SSRF**: a `.papur` document containing an image or stylesheet URL would cause the rendering host (often CI or a build server) to dereference attacker-controlled URLs, including cloud metadata endpoints and internal services. The emitter would also inherit the browser engine's CVE and sandbox-escape surface.

Direct emission keeps control of all dereferencing inside papur. The residual attack surface narrows to font and image **parsing** of author-supplied assets — real, but small, auditable, and substantially de-fanged by Go's memory safety.

## Out of Scope for v1

- Complex-script shaping, bidi, vertical writing modes. Simple-script shaping (rune → glyph with kerning) is the v1 ceiling.
- Floats and arbitrary absolute positioning. papur's block model does not require them; the emitter is not a general CSS layout engine.
- Form fields, annotations, embedded multimedia, JavaScript actions.
- PDF/A archival conformance (revisit after PDF/UA lands).
- Encryption / digital signatures.

## Acceptance Criteria

- [ ] A `.papur` document compiles to a valid, Tagged PDF via the `pdf` target with no network access performed at any point during compilation.
- [ ] Headings, lists, tables, and images appear in the PDF structure tree with correct reading order and propagated alt text.
- [ ] The layout pass produces a serializable / inspectable box tree, with unit tests asserting box positions independent of PDF byte output.
- [ ] A document longer than one page breaks correctly: no stranded headings, table headers repeat across page boundaries, `break-inside: avoid` is honored.
- [ ] Standard-14 fonts emit with no embedding; author-supplied TrueType / OpenType fonts embed as subsets containing only used glyphs.
- [ ] Missing-font, unresolvable-asset, and unsupported-script conditions are compile-time errors, not silent fallbacks.
- [ ] `::: script` blocks produce no output.
- [ ] `::: @pdf` content blocks emit; `::: @web` and `::: @email` blocks do not.
- [ ] `@pdf`-qualified rule blocks in `::: css` and override blocks in `::: theme` are applied; tokens are inlined as values, not as `var()` references.
- [ ] Page geometry honors `@page`-equivalent settings from `::: theme` / `::: meta`.

## Applicable Rules

- `BE-INPUT-007` — SSRF prevention. The no-network commitment above is the spec-level expression of this rule for the PDF emitter.

## Open Questions

- **Typesetting library** — does `go-text/typesetting` cover enough of shaping and line breaking to adopt, or is a minimal in-tree shaper the safer v1?
- **Byte-layer backend** — does `go-pdf/fpdf` earn its place as the byte-layer backend, or is hand-rolling the object layer cleaner given the Tagged-PDF structure-tree requirement (which fpdf supports poorly)?
- **Font declaration site** — how are author fonts declared in `.papur` source: in the `::: theme` region, a dedicated region, or `::: meta`?
- **Table-split behavior** — minimum viable v1: repeat header row plus `break-inside: avoid` on rows, or also support explicit break hints?
- **Box-tree visibility** — is the box tree a stable, documented public artifact (useful for testing and tooling) or an internal implementation detail?
- **Page size default** — `US Letter`, `A4`, or configurable per project with no global default?
