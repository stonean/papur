---
status: draft
dependencies: [001-file-format]
review:
  last-run: null
  reviewed-against: null
  must-violations: 0
  should-violations: 0
  low-confidence: 0
  blocking: false
next-criterion: 9
---

# 004 — Theming

Theme tokens are global design-system values consumed by the CSS layer and inlined by static target emitters. The theme layer is the project's single source of truth for design tokens. This spec depends on the fence rules defined in [001-file-format](../001-file-format/spec.md).

## The `::: theme` Block

```text
::: theme
brand: oklch(60% 0.2 250)
brand.hover: oklch(50% 0.2 250)

ink: #1a1a1a
paper: #fafafa

space.sm: 0.5rem
space.md: 1rem
space.lg: 2rem

radius.md: 0.5rem

@dark
  ink: #fafafa
  paper: #1a1a1a

@pdf
  brand: black

@email
  brand: #0066cc
:::
```

Each key/value pair is a token. Indented variant blocks (`@dark`, `@pdf`, `@email`, etc.) define overrides activated under specific conditions.

## Token Emission

Dot-notation keys flatten to CSS custom properties with hyphens:

| Source | Emitted CSS variable |
| --- | --- |
| `space.md: 1rem` | `--space-md: 1rem` |
| `brand.hover: ...` | `--brand-hover: ...` |

The hyphen flattening rule is total: every nested segment becomes a hyphenated suffix.

## Reference Syntax

In the CSS layer, theme tokens are referenced with the `$` sigil — Sass-flavored, zero ambiguity:

```text
::: css
.btn
  background: $brand
  padding: $space.sm $space.md
  border-radius: $radius.md

  &:hover
    background: $brand.hover
:::
```

The compiler resolves `$brand` based on target:

- **Web target** — emitted as `var(--brand)`, runtime-themeable.
- **PDF / email / static targets** — inlined value from the relevant variant block.

## Variant Qualifiers

| Qualifier | Activates when |
| --- | --- |
| `@dark` | `prefers-color-scheme: dark` or `[data-theme="dark"]` |
| `@light` | Explicit light mode |
| `@high-contrast` | `prefers-contrast: more` |
| `@reduced-motion` | `prefers-reduced-motion: reduce` |
| `@web` | Web target (default) |
| `@pdf` | PDF target |
| `@email` | Email target |

Variant qualifiers are shared between `::: theme` and `::: css`.

## Scoped Theming

A `::: theme` block inside a fenced div scopes its overrides to that container:

```text
::: footer
  ::: theme
  paper: $ink
  ink: $paper
  :::

  Content here uses inverted tokens.
:::
```

A scoped theme block emits a CSS custom-property override on the enclosing container's selector for runtime targets, and inlines the override into the contained content for static targets.

## Component-Scoped Values

Components use raw CSS custom properties for local-only knobs — **not** theme syntax. The visual distinction is intentional:

```text
.card
  --pad: $space.md
  --radius: $radius.lg

  padding: var(--pad)
  border-radius: var(--radius)

  &.compact
    --pad: $space.sm
```

`$theme-token` = global theme. `var(--local)` = component-internal. Readers can tell which is which at a glance.

## Breakpoints

Breakpoints are themed values:

```text
::: theme
breakpoint.sm: 640px
breakpoint.md: 768px
breakpoint.lg: 1024px
:::

::: css
.hero
  font-size: $text.lg

  @breakpoint.md
    font-size: $text.xl
:::
```

`@breakpoint.NAME` is consumed by the CSS layer to emit a media query keyed to the themed value.

## Computed Tokens

Tokens can reference other tokens with arithmetic:

```text
::: theme
base: 1rem
text.sm: base * 0.875
text.md: base
text.lg: base * 1.25
:::
```

For runtime themes, the compiler emits `calc(var(--base) * 1.25)`. For static targets, the value is inlined.

## Two-Tier Pattern (Recommended, Not Enforced)

Authors are encouraged to separate primitives (raw scale) from semantic tokens (the API):

```text
::: theme
# Primitives
blue.500: #2266dd
gray.900: #1a1a1a

# Semantic
brand: $blue.500
ink: $gray.900
:::
```

The compiler does not enforce the separation — author discretion. This is a "compiler enforces standards, not preferences" decision: the two-tier pattern is a design-system practice, not a correctness property.

## Acceptance Criteria

- [ ] AC1: `space.md: 1rem` emits `--space-md: 1rem` on the root scope for web targets.
- [ ] AC2: `$brand` in `::: css` resolves to `var(--brand)` on web and to the literal value (e.g. `oklch(60% 0.2 250)`) on pdf / email / plain.
- [ ] AC3: `@dark` overrides emit under `@media (prefers-color-scheme: dark)` and under `[data-theme="dark"]` on web; on static targets, only the active variant for that target is inlined.
- [ ] AC4: A `::: theme` block inside a fenced div emits its overrides scoped to that container's selector.
- [ ] AC5: `--pad: $space.md` resolves the `$space.md` reference at compile time, then emits the resulting value on the component selector.
- [ ] AC6: `@breakpoint.md` produces a media query at the themed pixel value.
- [ ] AC7: `text.lg: base * 1.25` emits `calc(var(--base) * 1.25)` on web and the inlined value on static targets.
- [ ] AC8: The two-tier pattern is not enforced — a project that defines only semantic tokens compiles without warnings.

## Open Questions

<!-- None recorded. -->
