---
status: draft
dependencies: [009-multi-target]
review:
  last-run: null
  reviewed-against: null
  must-violations: 0
  should-violations: 0
  low-confidence: 0
  blocking: false
---

# 013 — Plain Text Target

Emission rules for the plain text target. The plain emitter produces a text-only rendering suitable for terminal output, plain-text email parts (alongside the HTML email part), or accessibility fallbacks. Dispatch architecture lives in [009-multi-target](../009-multi-target/spec.md).

## Output Shape

- **Text only** — no markup, no styling.
- Headings are rendered with simple underlines or ATX-style prefixes.
- Lists, tables, and blockquotes are rendered in a readable plain-text form.
- Links collapse to `text (url)` form.
- Inline emphasis (`*bold*`, `_italic_`) is preserved in a conventional Markdown-flavored form.

## What Drops

- `::: theme` blocks.
- `::: css` blocks.
- `::: script` blocks.
- `::: html` blocks (unless explicitly `::: @plain` qualified, which is rare).
- All target-qualified blocks for other targets.

## What Stays

- Content prose.
- `::: @plain` content blocks.
- `::: meta` content used for the document title and any author/date fields the renderer chooses to include.

## Acceptance Criteria

- [ ] The plain emitter produces a single text file with no markup characters that suggest a richer format.
- [ ] Headings are visually distinguishable in plain text (e.g., underlined or prefixed with `#`s — the rendering is consistent across the file).
- [ ] Tables render in a readable plain-text grid.
- [ ] `::: theme`, `::: css`, `::: script`, and `::: html` blocks produce no output.
- [ ] `::: @plain` content blocks emit; other target-qualified blocks do not.

## Open Questions

- **Wrap width** — fixed (e.g., 80 columns) or configurable? Wrapping affects readability in terminal vs. plain-text email contexts differently.
- **Heading style** — Setext-style underlines, ATX-style `#` prefixes, or both selectable?
