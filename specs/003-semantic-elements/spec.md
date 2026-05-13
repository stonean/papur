---
status: draft
dependencies: [002-attribute-syntax]
review:
  last-run: null
  reviewed-against: null
  must-violations: 0
  should-violations: 0
  low-confidence: 0
  blocking: false
---

# 003 — Semantic Element Registry

The canonical mapping from a built-in role keyword to the HTML element the emitter produces. Built-in keywords give authors semantic markup without writing raw HTML; custom roles can opt into a semantic element through a CSS-side `as:` declaration.

This registry depends on the role grammar defined in [002-attribute-syntax](../002-attribute-syntax/spec.md).

## Built-in Role Keywords

| Role keyword | Emits |
| --- | --- |
| `nav` | `<nav>` |
| `header` / `footer` | `<header>` / `<footer>` |
| `main` | `<main>` |
| `article` | `<article>` |
| `aside` | `<aside>` |
| `section` | `<section>` |
| `figure` / `figcaption` | `<figure>` / `<figcaption>` |
| `details` / `summary` | `<details>` / `<summary>` |
| `dialog` | `<dialog>` |
| `blockquote` | `<blockquote>` |
| (any other) | `<section>` if heading-scoped, else `<div>` |

The catch-all rule applies when a role is not in the built-in registry **and** the author has not declared an `as:` override.

## Custom Element Override

Authors can declare an element override on a custom role via the CSS layer:

```text
.product-card
  as: article
  ...
```

The `as:` key is consumed by the parser/emitter, not emitted as a CSS property. It instructs the structure emitter to use the named element for any scope opened by `.product-card`.

`as:` only accepts known semantic HTML tag names. An unknown tag is a lint error.

## Inline vs Block

The position rule from [002-attribute-syntax](../002-attribute-syntax/spec.md) still applies: a pre-text role attaches the class to the heading element (which keeps its own native tag, `<h3>` etc.). A post-text role opens a section scope, and the emitted wrapping element is determined by this registry (or the `as:` override).

## Acceptance Criteria

- [ ] A heading scope opened by `### Welcome {.nav}` emits `<nav>`.
- [ ] A fenced div `::: aside` emits `<aside>`.
- [ ] A heading scope with a custom role (`.product-card`) and no `as:` override emits `<section>`.
- [ ] A custom role with `as: article` in the CSS layer emits `<article>` for that role's scope.
- [ ] `as:` with an unknown tag name is a lint error in strict mode and a warning in lenient mode.
- [ ] Pre-text role on a heading does not change the heading element (`### {.nav} Welcome` emits `<h3 class="nav">Welcome</h3>`, not `<nav>`).

## Open Questions

<!-- None recorded. -->
