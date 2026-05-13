# SPEC.md

> **Status:** v0 draft. Captures the design conversation to date. Open questions are listed at the end.
> **Codename:** _TBD_ — file extension is `.mh` as a placeholder.

## What this is

A markdown-flavored markup language that transpiles to semantic, accessible HTML + CSS — and, via the same source, to other targets like print, PDF, email-safe HTML, and plain text. Designed for authors who want Markdown's simplicity with CSS's expressive power, without sacrificing semantic HTML or accessibility.

## Guiding principles

1. **Prose-first.** Plain paragraphs need no syntax. Markdown's reading rule still applies.
2. **One file, fenced regions.** A single `.mh` file can hold content, styles, behavior, theme, and metadata as fenced blocks. Files can be specialized by naming convention.
3. **Roles, not utilities.** Style intent is expressed as named roles (`.hero`, `.card`), never atomic utility soup.
4. **Three parallel layers, keyed by role.** Structure (content), style (`::: css`), behavior (`::: script`) all attach to the same role names.
5. **Semantic and accessible by default.** Strict mode is the default. Lint rules ship enabled.
6. **The compiler enforces standards, not preferences.** Accessibility (WCAG, ARIA, HTML semantics) is non-negotiable. Authoring style (design system architecture, naming conventions, two-tier tokens, etc.) is the author's call.
7. **Multi-target via AST.** Same source compiles to web, print, email, plain text. Target-specific tweaks live alongside defaults in the same source.
8. **No runtime.** Behaviors compile to vanilla JS at build time. No framework dependency for output.

## Hello, page

```
::: meta
title: Welcome
lang: en
:::

# Build documents that look designed {.headline}

A markup language for people who like Markdown but love CSS. {.lead}

[Get started]{.btn .primary}(/start) [Read the docs]{.btn .ghost}(/docs)

## Features {#features}

::: grid cols=3
### Fast {.card}
Compiles to clean HTML + CSS. No runtime.

### Familiar {.card}
If you know Markdown, you already know 80% of it.

### Themeable {.card}
Swap a stylesheet, change the entire feel.
:::

::: theme
brand: oklch(60% 0.2 250)
ink: #1a1a1a
paper: #fafafa
space.sm: 0.5rem
space.md: 1rem
space.lg: 2rem
radius.md: 0.5rem

@dark
  ink: #fafafa
  paper: #1a1a1a
:::

::: css
.headline
  font-size: clamp(2rem, 5vw, 4.5rem)
  line-height: 1.1

.lead
  font-size: 1.25rem
  color: $ink
  max-width: 60ch

.btn
  padding: $space.sm $space.md
  border-radius: $radius.md
  &.primary
    background: $brand
    color: $paper
  &.ghost
    border: 2px solid currentColor

.card
  padding: $space.lg
  background: $paper
:::

::: script
.btn.primary
  on click
    track('cta-click')
:::
```

## File format

### Extension and naming

The actual extension is always `.mh`. Filenames are signage for humans and tooling — the parser only knows about fences.

| Filename                | Convention                                  |
| ----------------------- | ------------------------------------------- |
| `page.mh`               | Mixed-content file (typical page)           |
| `nav.mh`                | A partial (content fragment)                |
| `styles.css.mh`         | Primarily a `::: css` block                 |
| `actions.js.mh`         | Primarily a `::: script` block              |
| `mytheme.theme.mh`      | Primarily a `::: theme` block               |

The middle segment is a type hint for humans. The compiler treats all `.mh` files identically.

### Fences are always required

All typed content lives in fenced blocks. There are no implicit wrappers from filenames. What you see is what gets parsed.

```
::: meta
title: My Page
lang: en
:::

::: theme
brand: oklch(60% 0.2 250)
:::

::: css
.hero
  font-size: 3rem
:::

::: script
.btn
  on click
    ...
:::

::: html
<custom-element></custom-element>
:::
```

YAML frontmatter (`---`) is supported as compatibility shorthand for `::: meta`.

## Attribute syntax (roles)

Roles use Pandoc-style attribute syntax: `{.class #id key=value}`.

### Position-determined scope

```
### {.hero} Welcome        # role applies to heading element only (inline)
### Welcome {.hero}        # role applies to heading's section scope (block)
```

### Namespace prefixes

| Prefix      | Meaning                                          |
| ----------- | ------------------------------------------------ |
| `{.foo}`    | Local-first lookup, falls through to global      |
| `{g.foo}`   | Force global                                     |
| `{l.foo}`   | Force local                                      |

### Heading scope rule

A roled heading opens a scope. The scope closes at the first of:

1. A sibling heading at the **same fence depth** with equal or higher level, or
2. The closer of the fence the heading was opened inside.

Headings at deeper fence depths do **not** close outer scopes — each fence is its own outline universe. This matches HTML5 sectioning semantics.

### Multi-role nesting

Multiple roles in nested contexts produce nested elements (descendant selectors), not stacked classes on one element:

```
::: grid cols=3
### Fast {.carda}
Content.

  ::: grid cols=2
  Still in .carda.

  #### Smaller {.card1}
  In .carda > .card1.
  :::
:::
```

→

```html
<div class="grid" data-cols="3">
  <section class="carda">
    <h3>Fast</h3>
    <p>Content.</p>
    <div class="grid" data-cols="2">
      <p>Still in .carda.</p>
      <section class="card1">
        <h4>Smaller</h4>
        <p>In .carda > .card1.</p>
      </section>
    </div>
  </section>
</div>
```

## Fenced divs

`::: name` creates a named block. Attributes after the name pass through:

```
::: hero
# Welcome
:::

::: grid cols=3
...
:::
```

## Semantic element registry

Built-in role keywords resolve to HTML elements:

| Role keyword            | Emits                            |
| ----------------------- | -------------------------------- |
| `nav`                   | `<nav>`                          |
| `header` / `footer`     | `<header>` / `<footer>`          |
| `main`                  | `<main>`                         |
| `article`               | `<article>`                      |
| `aside`                 | `<aside>`                        |
| `section`               | `<section>`                      |
| `figure` / `figcaption` | `<figure>` / `<figcaption>`      |
| `details` / `summary`   | `<details>` / `<summary>`        |
| `dialog`                | `<dialog>`                       |
| `blockquote`            | `<blockquote>`                   |
| (any other)             | `<section>` if heading-scoped, else `<div>` |

Custom roles can declare an element override in CSS:

```
.product-card
  as: article
  ...
```

## Theming

### The `::: theme` block

```
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

@print
  brand: black

@email
  brand: #0066cc
:::
```

### Token emission

Dot-notation flattens with hyphens:

| Source                | Emitted CSS variable      |
| --------------------- | ------------------------- |
| `space.md: 1rem`      | `--space-md: 1rem`        |
| `brand.hover: ...`    | `--brand-hover: ...`      |

### Reference syntax

In `::: css`, theme tokens are referenced with the `$` sigil (Sass-flavored, zero ambiguity):

```
::: css
.btn
  background: $brand
  padding: $space.sm $space.md
  border-radius: $radius.md

  &:hover
    background: $brand.hover
:::
```

The compiler resolves `$brand`:
- **Web target:** `var(--brand)` — runtime-themeable
- **Print / email / static targets:** inlined value from the relevant variant block

### Variant qualifiers

| Qualifier         | Activates when                                                   |
| ----------------- | ---------------------------------------------------------------- |
| `@dark`           | `prefers-color-scheme: dark` or `[data-theme="dark"]`            |
| `@light`          | Explicit light mode                                              |
| `@high-contrast`  | `prefers-contrast: more`                                         |
| `@reduced-motion` | `prefers-reduced-motion: reduce`                                 |
| `@web`            | Web target (default)                                             |
| `@print`          | Print target                                                     |
| `@email`          | Email target                                                     |

### Scoped theming

A `::: theme` block inside a fenced div scopes overrides to that container:

```
::: footer
  ::: theme
  paper: $ink
  ink: $paper
  :::

  Content here uses inverted tokens.
:::
```

### Component-scoped values

Components use raw CSS custom properties for local-only knobs — *not* theme syntax. The visual distinction is intentional:

```
.card
  --pad: $space.md
  --radius: $radius.lg

  padding: var(--pad)
  border-radius: var(--radius)

  &.compact
    --pad: $space.sm
```

`$theme-token` = global theme. `var(--local)` = component-internal. Eyes can tell which is which.

### Breakpoints

Breakpoints are themed values:

```
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

### Computed tokens

```
::: theme
base: 1rem
text.sm: base * 0.875
text.md: base
text.lg: base * 1.25
:::
```

For runtime themes, compiles to `calc(var(--base) * 1.25)`. For static targets, inlined.

### Two-tier pattern (recommended, not enforced)

Authors are encouraged to separate primitives (raw scale) from semantic tokens (the API):

```
::: theme
# Primitives
blue.500: #2266dd
gray.900: #1a1a1a

# Semantic
brand: $blue.500
ink: $gray.900
:::
```

The compiler does not enforce the separation — author discretion.

## CSS layer

CSS lives in `::: css` blocks or files (`*.css.mh` by convention). Sass-flavored indented syntax: no curly braces, no required semicolons.

```
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

### Target qualifiers

```
.card
  padding: $space.lg

  @web
    box-shadow: 0 2px 8px rgba(0,0,0,0.1)

  @print
    border: 1pt solid black
    break-inside: avoid

  @email
    background: $surface
```

### Accessibility qualifiers

```
.fade-in
  transition: opacity 300ms

  @reduced-motion
    transition: none

.button
  &:focus-visible
    outline: 2px solid $focus
    outline-offset: 2px
```

`:focus-visible` is auto-emitted for any role with a behavior handler, unless explicitly suppressed.

## Behavior layer

> _Design pending — see Open Questions._

Behavior attaches handlers to roles. Compile target is vanilla JS at build time (no runtime framework). Recognized interaction patterns auto-emit correct ARIA.

Placeholder syntax:

```
::: script
.btn.primary
  on click
    track('cta-click')

.accordion .header
  on click
    toggle .open on closest('.accordion')
:::
```

When the compiler recognizes a known pattern (accordion, dialog, tabs, etc.), it auto-wires:

- ARIA states (`aria-expanded`, `aria-controls`, `aria-hidden`, etc.)
- Keyboard handlers (Space, Enter, ESC, arrow keys)
- Focus management (trap, restore, initial focus)
- Roles (`role="button"`, `role="dialog"`, etc.)

The behavior layer is silently dropped from non-web targets.

## Raw HTML

```
::: html
<video controls>
  <source src="demo.webm" type="video/webm">
</video>
:::
```

`::: html` is web-only by default; dropped from print/email/plain targets unless a target qualifier explicitly includes it.

Inline raw HTML mid-paragraph (Markdown-style) is permitted for things like `<kbd>`, `<sub>`, `<sup>`.

## Includes

```
@include nav.mh
@include theme.css.mh
@include reset.css.mh
```

### Context-sensitive resolution

`@include` inside a typed block pulls only the matching block content from the included file:

```
::: css
@include theme.css.mh         # pulls only ::: css content from theme.css.mh
:::
```

A mixed `.mh` file included from a CSS block contributes only its `::: css` content; from a script block, only its `::: script`; from content, only the content.

### No props

Includes are dumb text inclusion. No parameterization, no scope. If you need variation, write `nav-dark.mh` and `nav-light.mh`.

## Multi-target output

The `.mh` source parses to a target-agnostic AST. Per-target transformers emit:

- **web** — HTML + CSS
- **print** — HTML + print-CSS, or via PDF
- **email** — table-based HTML with inlined CSS
- **plain** — plain text
- _(extensible)_

### Target-qualified content blocks

```
::: @web
@include interactive-demo.mh
:::

::: @print
*See the live version at example.com/demo*
:::
```

Default behavior: a block with no `@target` is rendered for all targets.

### Behaviors dropped silently

The `::: script` layer is omitted entirely from non-web targets.

## Accessibility

### Strict mode is the default

Lint rules ship enabled. `--lenient` flag opts out.

### Build-time lint rules

- Missing alt text on images (`![](src "")` is the explicit "decorative" form)
- Skipped heading levels (H2 → H4)
- Multiple H1s outside `<article>`
- Missing `<main>`, `<title>`, `lang` attribute
- Color contrast below WCAG AA when `color` and `background-color` are both set on the same role
- Interactive role on a non-focusable element without proper wrapping
- Vague link text without `aria-label`

Color contrast is scoped to same-role pairs only. Cascade-based inherited contrast is intentionally out of scope — better tools exist for that.

### Bundled accessible patterns

Patterns ship with the compiler with full ARIA + keyboard support:

- `.dialog` — modal with focus trap, ESC to close, focus restore
- `.disclosure` — show/hide with `aria-expanded`
- `.tabs` — arrow-key nav, `aria-selected`
- `.tooltip` — `aria-describedby`, hover + focus
- `.alert` — `role="alert"`, live region
- `.live` — `aria-live="polite"`

### Auto-emitted defaults

- Skip link to `<main>` if `<main>` exists
- `<meta viewport>` for responsive
- `:focus-visible` for any interactive role
- `lang` attribute on `<html>` (warning if `::: meta` doesn't set it)

## Forms

> _Deferred — needs its own design pass._

Rough sketch (not locked):

```
::: form action="/signup" method="post"
> label Email
> @ email required

> label Password
> @ password required min=8

> button Sign up {.btn .primary}
:::
```

Labels mandatory. Association (`for`/`id`) auto-generated. Validation errors auto-linked via `aria-describedby`. Required fields get `aria-required`.

## Open questions

- **Language name** (currently `.mh` placeholder)
- **Behavior DSL** — full syntax and grammar
- **Behavior compile output** — likely vanilla JS at build time; needs confirmation
- **Forms** — full design
- **AST shape** — mdast-derived (with extensions) or custom
- **Block ordering convention** — recommend define-first or prose-first?
- **Multiple blocks of same type** — allow or disallow multiple `::: css` blocks in one file?

## Locked decisions (quick reference)

| Topic                            | Decision                                                                   |
| -------------------------------- | -------------------------------------------------------------------------- |
| File extension                   | `.mh` (placeholder)                                                        |
| Filename type hints              | Convention only, no parser behavior                                        |
| Fences                           | Always required                                                            |
| Attribute syntax                 | Pandoc-style `{.class #id key=value}`                                      |
| Role position semantics          | Pre-text = inline, post-text = block scope                                 |
| Namespace prefixes               | `g.` global, `l.` local, default = local-first                             |
| Heading scope rule               | Sibling at same fence depth (≥ level), or enclosing fence closer           |
| Nested fence isolation           | Outer heading scopes survive inner fences                                  |
| Multi-role nesting               | Produces descendant elements, not stacked classes                          |
| Frontmatter                      | `---` YAML supported alongside `::: meta`                                  |
| Raw HTML                         | Fenced via `::: html`, web-only by default                                 |
| CSS syntax                       | Sass-flavored indented                                                     |
| Theme token sigil                | `$`                                                                        |
| Dot-notation flattening          | Hyphen (`space.md` → `--space-md`)                                         |
| Two-tier enforcement             | Not enforced (recommendation only)                                         |
| Component-scoped tokens          | Raw CSS custom properties, not theme syntax                                |
| Strict mode                      | Default; lenient is opt-out                                                |
| Accessible patterns              | Bundled with compiler                                                      |
| Color contrast check             | Same-role color/bg pairs only                                              |
| Behavior in non-web targets      | Silently dropped                                                           |
| Includes                         | Context-sensitive, no props, no scope                                      |
| Meta-principle                   | Compiler enforces standards (a11y), not preferences (style)                |
