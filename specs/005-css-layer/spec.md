---
status: draft
dependencies: [001-file-format, 002-attribute-syntax, 004-theming, 006-behavior-layer, 014-accessibility]
review:
  last-run: null
  reviewed-against: null
  must-violations: 0
  should-violations: 0
  low-confidence: 0
  blocking: false
---

# 005 — CSS Layer

The styling layer. CSS lives in `::: css` blocks (or files conventionally named `*.css.papur`, per [001-file-format](../001-file-format/spec.md)) and is written in Sass-flavored indented syntax. Theme tokens come from [004-theming](../004-theming/spec.md); roles come from [002-attribute-syntax](../002-attribute-syntax/spec.md).

## Sass-Flavored Indented Syntax

No curly braces. No required semicolons. Nesting is expressed through indentation. The `&` parent selector and SCSS-style nested rules are supported.

```text
::: css
.btn
  display: inline-block
  padding: $space.sm $space.md
  border-radius: $radius.md

  &.primary
    background: $brand
    color: $paper

  &.ghost
    border: 2px solid currentColor
:::
```

## Theme Token References

Theme tokens are referenced with `$` (see [004-theming](../004-theming/spec.md)). `$theme-token` resolves to `var(--theme-token)` on web and to the inlined value on static targets.

## Target Qualifiers

Target qualifiers scope rule blocks to a specific output target:

```text
.card
  padding: $space.lg

  @web
    box-shadow: 0 2px 8px rgba(0,0,0,0.1)

  @pdf
    border: 1pt solid black
    break-inside: avoid

  @email
    background: $surface
```

A `@target` block emits only when compiling for that target. Unqualified rules emit for every target.

## Accessibility Qualifiers

Accessibility qualifiers expose user-preference media queries with first-class syntax:

```text
.fade-in
  transition: opacity 300ms

  @reduced-motion
    transition: none

.button
  &:focus-visible
    outline: 2px solid $focus
    outline-offset: 2px
```

`:focus-visible` is **auto-emitted** for any role that has a behavior handler ([006-behavior-layer](../006-behavior-layer/spec.md)), unless explicitly suppressed. This is a "compiler enforces standards" decision and is also addressed by [014-accessibility](../014-accessibility/spec.md).

## Breakpoint Qualifiers

`@breakpoint.NAME` references a themed breakpoint and emits a media query at the resolved value. See [004-theming](../004-theming/spec.md) for breakpoint definitions.

## Acceptance Criteria

- [ ] A `.btn` rule with `padding: $space.sm $space.md` emits the resolved theme values per target (var() on web, inlined elsewhere).
- [ ] Nested `&.primary` produces `.btn.primary` in the compiled output.
- [ ] `@web` blocks emit only on the web target; `@pdf` blocks emit only on the pdf target; `@email` blocks emit only on the email target.
- [ ] `@reduced-motion` emits under `@media (prefers-reduced-motion: reduce)` on web.
- [ ] Any role with a handler in `::: script` automatically gets a `:focus-visible` rule unless the author has written one explicitly.
- [ ] Sass `&` parent selector chains correctly across multiple nesting levels.
- [ ] Missing semicolons and missing braces are not parse errors (indentation is the syntactic structure).

## Open Questions

<!-- None recorded. -->
