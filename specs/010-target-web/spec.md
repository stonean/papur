---
status: draft
dependencies: [003-semantic-elements, 004-theming, 005-css-layer, 006-behavior-layer, 007-raw-html, 009-multi-target, 014-accessibility]
review:
  last-run: null
  reviewed-against: null
  must-violations: 0
  should-violations: 0
  low-confidence: 0
  blocking: false
---

# 010 — Web Target

Emission rules for the web target. The web emitter produces HTML and CSS from a papur AST. It is the default target and the one with the richest feature surface — all other targets degrade from web.

Dispatch architecture lives in [009-multi-target](../009-multi-target/spec.md). Cross-cutting behavior comes from the layer specs.

## Output Shape

- **HTML** — generated from the content layer (prose + roles + fenced divs), with semantic element selection from [003-semantic-elements](../003-semantic-elements/spec.md).
- **CSS** — generated from the `::: css` blocks ([005-css-layer](../005-css-layer/spec.md)) and the theme tokens ([004-theming](../004-theming/spec.md)).
- **JS** — generated from the `::: script` blocks ([006-behavior-layer](../006-behavior-layer/spec.md)), compiled to vanilla JS at build time.
- **Raw HTML** — `::: html` blocks ([007-raw-html](../007-raw-html/spec.md)) emit verbatim.

## Theme Tokens on Web

Theme tokens emit as CSS custom properties on the root scope. `$brand` in `::: css` resolves to `var(--brand)`, making the theme runtime-themeable (e.g., a `[data-theme="dark"]` attribute toggle).

## Auto-Emitted Defaults

The web emitter auto-emits a small set of defaults the author can override but does not need to write:

- A skip link to `<main>` if `<main>` exists in the document.
- `<meta name="viewport" content="width=device-width, initial-scale=1">` in `<head>`.
- `:focus-visible` outline for any role with a behavior handler (see [005-css-layer](../005-css-layer/spec.md) and [014-accessibility](../014-accessibility/spec.md)).
- `lang` attribute on `<html>` from `::: meta` (warning if `::: meta` does not set it).

## Acceptance Criteria

- [ ] A papur file with content, `::: theme`, `::: css`, and `::: script` blocks compiles to HTML, CSS, and JS files (or an inlined HTML document, per build config).
- [ ] Theme tokens emit as CSS custom properties on the root scope; references emit as `var(--token)`.
- [ ] Roles emit as semantic elements per [003-semantic-elements](../003-semantic-elements/spec.md).
- [ ] The skip link, viewport meta tag, and `lang` attribute are auto-emitted on every compiled document.
- [ ] `::: @web` qualified blocks emit; `::: @print` / `::: @email` qualified blocks do not.
- [ ] Behaviors compile to vanilla JS and ship without a framework runtime.

## Open Questions

- **JS emission shape** — inline `<script>` block, separate file, or both? (Shared with [006-behavior-layer](../006-behavior-layer/spec.md).)
- **CSS emission shape** — inline `<style>`, separate stylesheet, or both?
