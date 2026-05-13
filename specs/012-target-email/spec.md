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

# 012 — Email Target

Emission rules for the email target. The email emitter produces HTML that survives the constraints of real-world email clients: table-based layout, inlined CSS, no JS, conservative property set. Dispatch architecture lives in [009-multi-target](../009-multi-target/spec.md).

## Output Shape

- **HTML** — table-based layout. Semantic elements that email clients ignore or render inconsistently are downgraded to tables and `<div>`s. Roles still drive class names so the inlined CSS can target them.
- **CSS** — inlined into element `style=""` attributes. The `@email` variant from [004-theming](../004-theming/spec.md) and [005-css-layer](../005-css-layer/spec.md) is applied.
- **No JS** — the behavior layer ([006-behavior-layer](../006-behavior-layer/spec.md)) is silently dropped.
- **No raw HTML** — `::: html` blocks ([007-raw-html](../007-raw-html/spec.md)) are dropped unless explicitly qualified `::: @email html`. Raw HTML the author writes for the web is unlikely to survive email clients.

## Theme Tokens on Email

Theme tokens are inlined as their resolved values. `$brand` emits the value from the `@email` variant block if present, otherwise the default. CSS custom properties (`var()`) are not used — email client support is too inconsistent.

## Layout Downgrade

The email emitter rewrites the structural layer to the table-based idiom email clients understand:

- A `::: grid` fenced div emits as nested `<table>` rows/cells, not CSS grid.
- Block scopes that would emit a semantic element on web (`<section>`, `<article>`) emit as `<div>` with the class preserved.
- Inline-block layout patterns are rewritten as table-cell patterns.

## What Drops

- `::: script` blocks.
- `::: html` blocks (unless `@email`-qualified).
- `::: @web` and `::: @print` qualified blocks.
- Modern CSS features unsupported in major email clients (custom properties, grid, modern color functions in some cases).

## What Stays

- Content prose and roles.
- `@email`-qualified blocks across `::: theme`, `::: css`, and content.
- `::: @email` content blocks.

## Acceptance Criteria

- [ ] The email emitter produces a single HTML document with all CSS inlined into `style` attributes.
- [ ] `::: grid` fenced divs emit as nested tables.
- [ ] No `<style>` block or external stylesheet is emitted; every style is inlined.
- [ ] `var()` references do not appear in the output; all theme tokens are inlined as values.
- [ ] `::: script` blocks produce no output.
- [ ] `::: @email` content blocks emit; other target-qualified blocks do not.

## Open Questions

- **Target client matrix** — which email clients does the emitter formally support (Gmail, Outlook desktop, iOS Mail, Apple Mail, etc.)? The choice determines which CSS features the emitter avoids.
- **Dark mode support** — `@media (prefers-color-scheme: dark)` works in some clients and not others. Does the emitter ship `@email` dark variants as `@media` blocks, or duplicate the structure with class-based overrides?
- **Image handling** — inline base64, external URLs, or both? Affects deliverability and rendering reliability.
