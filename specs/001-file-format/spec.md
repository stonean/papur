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

# 001 — File Format

The on-disk representation of a papur source. Defines the file extension, filename conventions, the universal fence rule, and frontmatter support.

## Extension

The papur extension is `.papur`. The parser only treats `.papur` files as papur source.

## Filename Conventions

Filenames are signage for humans and tooling. The parser does not branch on the filename — it only inspects fences inside the file.

| Filename | Convention |
| --- | --- |
| `page.papur` | Mixed-content file (typical page) |
| `nav.papur` | A partial (content fragment) |
| `styles.css.papur` | Primarily a `::: css` block |
| `actions.js.papur` | Primarily a `::: script` block |
| `mytheme.theme.papur` | Primarily a `::: theme` block |

The middle segment (`.css`, `.js`, `.theme`) is a type hint for humans. The compiler treats all `.papur` files identically.

## Fences Are Always Required

All typed content lives in fenced blocks. There are no implicit wrappers derived from filenames or extensions — what you see is what gets parsed.

```text
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

A `.papur` file that consists only of prose (no fences) is treated as content; the prose becomes part of the AST exactly as Markdown would interpret it.

## Frontmatter

YAML frontmatter (`---`) is supported as compatibility shorthand for `::: meta`. The following two prefixes are equivalent:

```text
---
title: My Page
lang: en
---
```

```text
::: meta
title: My Page
lang: en
:::
```

When both forms appear in the same file, the parser merges them; later keys win.

## Acceptance Criteria

- [ ] Parser accepts files only when the extension is exactly `.papur`.
- [ ] Filename middle segments (`.css.papur`, `.js.papur`, `.theme.papur`) do not change parser behavior — every file is parsed identically.
- [ ] Typed content outside of a fence is a parse error in strict mode; in lenient mode it is parsed as content prose.
- [ ] `---` YAML frontmatter at the top of a file is treated as an implicit `::: meta` block.
- [ ] A file containing only prose (no fences, no frontmatter) parses successfully and emits content for every target.

## Open Questions

- **Multiple blocks of same type** — allow or disallow multiple `::: css` (or `::: theme`, `::: script`) blocks in one file? If allowed, define merge semantics; if disallowed, the parser must error.
- **Block ordering convention** — recommend prose-first (content before `::: theme` / `::: css` / `::: script`) or define-first? Not parser-enforced either way; the question is style guidance for authors.
