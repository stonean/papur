# papur

A markdown-flavored markup language that compiles to semantic, accessible HTML, CSS, and beyond.

papur takes a single `.papur` source and transpiles it to web (HTML + CSS), PDF, email-safe HTML, and plain text. It is designed for authors who want Markdown's simplicity with CSS's expressive power, without sacrificing semantic HTML or accessibility.

> **Status:** pre-implementation. The language is being designed spec-first. See `specs/` for the feature specs.

## Guiding principles

1. **Prose-first.** Plain paragraphs need no syntax. Markdown's reading rule still applies.
2. **One file, fenced regions.** A single `.papur` file can hold content, styles, behavior, theme, and metadata as fenced blocks.
3. **Roles, not utilities.** Style intent is expressed as named roles (`.hero`, `.card`), never atomic utility soup.
4. **Three parallel layers, keyed by role.** Structure (content), style (`::: css`), and behavior (`::: script`) all attach to the same role names.
5. **Semantic and accessible by default.** Strict mode is the default. Lint rules ship enabled.
6. **The compiler enforces standards, not preferences.** Accessibility (WCAG, ARIA, HTML semantics) is non-negotiable. Authoring style (design system architecture, naming conventions, two-tier tokens, etc.) is the author's call.
7. **Multi-target via AST.** Same source compiles to web, PDF, email, plain text. Target-specific tweaks live alongside defaults in the same source.
8. **No runtime.** Behaviors compile to vanilla JS at build time. No framework dependency for output.

## Example

```text
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
space.md: 1rem
radius.md: 0.5rem

@dark
  ink: #fafafa
  paper: #1a1a1a
:::

::: css
.headline
  font-size: clamp(2rem, 5vw, 4.5rem)
  line-height: 1.1

.btn
  padding: $space.sm $space.md
  border-radius: $radius.md
  &.primary
    background: $brand
    color: $paper
:::

::: script
.btn.primary
  on click
    track('cta-click')
:::
```

## Project layout

- [`.ductus/constitution.md`](.ductus/constitution.md) — Principles, pipeline, and quality standards from the `ductus` framework.
- [`AGENTS.md`](AGENTS.md) — Agent rules: tech stack, conventions, workflow, gotchas, boundaries.
- [`specs/`](specs/) — Numbered feature specs that compose the language design.
  - [`specs/system.md`](specs/system.md) — Compiler architecture (source → AST → per-target emitters).
  - `specs/NNN-feature/spec.md` — Individual feature specs.

## Development workflow

This project uses the [govern](https://github.com/stonean/govern) framework for spec-driven development. Features move through the pipeline: **spec → plan → tasks → implement**. See [`.ductus/constitution.md`](.ductus/constitution.md) for the full lifecycle.

## License

[MIT](LICENSE).
