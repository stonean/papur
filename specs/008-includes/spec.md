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
next-criterion: 8
---

# 008 — Includes

The `@include` directive pulls content from another `.papur` file into the current source. Includes have no props and no scope — they are dumb text inclusion with one twist: when used inside a typed fence, they pull only the matching block from the included file. This spec depends on the fence rules defined in [001-file-format](../001-file-format/spec.md).

## Basic Form

```text
@include nav.papur
@include theme.css.papur
@include reset.css.papur
```

A bare `@include path.papur` at the top level of a file inlines the entire included file into the current parse stream as if its content had been typed there.

## Context-Sensitive Resolution

`@include` inside a typed fence pulls only the matching block content from the included file:

```text
::: css
@include theme.css.papur         # pulls only ::: css content from theme.css.papur
:::
```

The rule is symmetric across fence types:

- A mixed `.papur` file included from a `::: css` block contributes only its `::: css` content.
- A mixed file included from a `::: script` block contributes only its `::: script` content.
- A mixed file included from content (top-level prose, not inside a fence) contributes only its content prose.

This makes it natural to keep one file per role with a `::: css` and `::: script` block for that role, then `@include` it from a layered consumer.

## No Props, No Scope

Includes are dumb text inclusion. No parameterization. No scope isolation. If you need variation, write two files (`nav-dark.papur` and `nav-light.papur`).

The decision is deliberate: parameterized includes are templating, and templating belongs in a different tool. papur's "three parallel layers, keyed by role" model already accommodates variation through theme overrides and target qualifiers.

## Acceptance Criteria

- [ ] AC1: A top-level `@include path.papur` inlines the entire included file's content at the include site.
- [ ] AC2: An `@include` inside `::: css` pulls only `::: css` content from the included file.
- [ ] AC3: An `@include` inside `::: script` pulls only `::: script` content from the included file.
- [ ] AC4: An `@include` inside top-level prose (no fence) pulls only the content prose from the included file.
- [ ] AC5: Cyclic includes are a parse error.
- [ ] AC6: An include path that does not end in `.papur` is a parse error.
- [ ] AC7: Includes do not introduce a new scope — roles and ids from the included file are merged into the including file's namespace.

## Open Questions

<!-- None recorded. -->
