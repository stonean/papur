---
status: draft
dependencies: [004-theming, 005-css-layer, 006-behavior-layer, 007-raw-html, 009-multi-target]
review:
  last-run: null
  reviewed-against: null
  must-violations: 0
  should-violations: 0
  low-confidence: 0
  blocking: false
---

# 011 — Print Target

Emission rules for the print target. The print emitter produces HTML + print-CSS suitable for browser printing, or a PDF via a downstream renderer. Dispatch architecture lives in [009-multi-target](../009-multi-target/spec.md).

## Output Shape

- **HTML** — same structural shape as the web target, trimmed of web-only blocks.
- **CSS** — emitted with `@page` rules and print-oriented defaults; the `@print` variant from [004-theming](../004-theming/spec.md) and [005-css-layer](../005-css-layer/spec.md) is applied.
- **No JS** — the behavior layer ([006-behavior-layer](../006-behavior-layer/spec.md)) is silently dropped.

## Theme Tokens on Print

Theme tokens are inlined as their resolved values. `$brand` does not emit `var(--brand)`; it emits the value from the `@print` variant block if present, otherwise the default value. This produces self-contained CSS that does not depend on runtime variables.

## What Drops

- `::: script` blocks — silently dropped.
- `::: html` blocks — dropped unless explicitly qualified `::: @print html` ([007-raw-html](../007-raw-html/spec.md)).
- `::: @web` qualified blocks — dropped.
- Auto-emitted web defaults (skip link, viewport meta tag) — dropped (not meaningful in print).

## What Stays

- Content prose and roles.
- `@print`-qualified blocks across `::: theme`, `::: css`, and content.
- `::: @print` content blocks.
- `prefers-color-scheme` and other media-condition variants — collapsed at compile time (print is not interactive).

## Acceptance Criteria

- [ ] The print emitter produces HTML + CSS with inlined theme tokens.
- [ ] `::: script` blocks produce no output.
- [ ] `::: @print` content blocks emit; `::: @web` and `::: @email` blocks do not.
- [ ] `@print` qualified rule blocks in `::: css` and override blocks in `::: theme` are applied.
- [ ] The emitted CSS includes appropriate `@page` rules for page size and margins.
- [ ] Tokens are inlined as values, not as `var()` references.

## Open Questions

- **PDF rendering** — should papur ship a built-in PDF renderer, or does the compiler stop at HTML + print-CSS and leave PDF generation to a downstream tool (browser print, weasyprint, etc.)?
- **Page size defaults** — what `@page` size is the default (US Letter, A4, configurable per project)?
