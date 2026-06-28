---
status: in-progress
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

## Multiple Blocks

A file may contain more than one block of the same type; the parser never errors on repetition. Blocks merge by their nature:

- **Ordered blocks — `::: css`, `::: script`** — concatenated in document (source) order. Source order is significant (the CSS cascade, script definition/execution order), so a compiler that hoists these into a target file MUST preserve the order in which the blocks appear.
- **Key-value blocks — `::: theme`, `::: meta`** — merged key by key, with later keys winning. This is the same rule the Frontmatter section defines for `---` plus `::: meta`.

An empty block of any type is valid and contributes nothing to the merge.

## Authoring Conventions

These are recommendations for humans, not parser rules — ordering never changes how a file parses.

- Lead with a preamble: `::: meta` (or `---` frontmatter) first, then `::: theme`. Metadata and design tokens are declarations the reader benefits from seeing before the content references them.
- Follow with content (prose). A `.papur` file reads as the document it represents.
- Place `::: css` and `::: script` blocks next to the content they affect, or grouped at the end for page-wide rules — co-location is supported (see Multiple Blocks).

## Acceptance Criteria

- [ ] Parser accepts files only when the extension is exactly `.papur`.
- [ ] Filename middle segments (`.css.papur`, `.js.papur`, `.theme.papur`) do not change parser behavior — every file is parsed identically.
- [ ] Typed content outside of a fence is a parse error in strict mode; in lenient mode it is parsed as content prose.
- [ ] `---` YAML frontmatter at the top of a file is treated as an implicit `::: meta` block.
- [ ] A file containing only prose (no fences, no frontmatter) parses successfully and emits content for every target.
- [ ] Multiple `::: css` or `::: script` blocks are accepted and concatenated in document (source) order, preserving that order in the compiled output.
- [ ] Multiple `::: theme` or `::: meta` blocks are accepted and merged key by key, with later keys winning.
- [ ] An empty block of any type parses successfully and contributes nothing to the output.

## Open Questions

*None — all resolved.*

## Resolved Questions

- **Multiple blocks of same type** — Allowed. Co-location is a core benefit of a mixed-content format, so the parser does not error on repeated blocks; it merges them by block nature. Ordered blocks concatenate in document (source) order — `::: css` (the cascade is source-order sensitive) and `::: script` (definition/execution order matters). Key-value blocks merge with later keys winning — `::: theme` and `::: meta`, consistent with the frontmatter/`::: meta` merge rule already defined in the Frontmatter section. Document order is normative for the ordered blocks: a compiler that hoists CSS/JS into a target file must preserve the source order of the blocks. See the Multiple Blocks section.
- **Block ordering convention** — Recommend prose-first with a metadata/theme preamble: `::: meta` (or `---` frontmatter) first, then `::: theme`, then content, with `::: css` / `::: script` co-located near the content they affect or grouped at the end. This is authoring style guidance only — it is not parser-enforced and ordering never changes parse behavior. Captured in the Authoring Conventions section.
