---
status: in-progress
dependencies: [001-file-format]
review:
  last-run: 2026-06-30T16:45:59Z
  reviewed-against: 30ec393a590ba3210faa5dbd73f315732b4e0afa
  must-violations: 0
  should-violations: 0
  low-confidence: 0
  blocking: false
next-criterion: 14
---

# 002 — Attribute Syntax (Roles)

How authors attach roles (classes, ids, key/value attributes) to elements. Roles are the join key across the three layers — structure, style, and behavior — so the syntax must be unambiguous and position-aware. This spec depends on the fence rules defined in [001-file-format](../001-file-format/spec.md).

## Pandoc-Style Attributes

Roles use Pandoc-style attribute syntax: `{.class #id key=value}`.

```text
[Get started]{.btn .primary}(/start)

### Welcome {.hero}
```

Multiple classes are space-separated inside one brace group. Keys are `key=value` pairs; an unquoted value is a single whitespace-delimited token (no quotes required), and a value containing spaces must be quoted (`key="a b"`).

A `key` is emitted as a verbatim HTML attribute when it is a recognized HTML attribute name — the global attributes plus the standard attributes for the target element, per the WHATWG HTML standard (the concrete allowlist is fixed in the plan). Every other `key` is emitted as `data-{key}`.

## Position-Determined Scope

Whether a role applies to a single element (inline) or to a scope (block) is determined by **where** the attribute sits relative to the heading text:

```text
### {.hero} Welcome        # role applies to the heading element only (inline)
### Welcome {.hero}        # role applies to the heading's section scope (block)
```

Pre-text attributes attach to the heading element; post-text attributes open a section scope keyed to the role.

This pre-text/post-text distinction is unique to **headings**, because only headings open outline scopes. For non-heading targets there is no section scope:

- **Inline spans** use the bracketed form `[text]{.foo}`; the attribute group attaches to the bracketed span it immediately follows — attachment is unambiguous and never opens a scope.
- **Block-level non-heading elements** (paragraphs, list items) take an attribute group on the immediately preceding element; a group that opens a line before an element attaches to the following element. Neither opens a scope.

## Namespace Prefixes

Role lookups resolve through a local-first → global fallback chain. Authors can override the lookup with namespace prefixes:

| Prefix | Meaning |
| --- | --- |
| `{.foo}` | Local-first lookup, falls through to global |
| `{g.foo}` | Force global |
| `{l.foo}` | Force local |

Resolution outcomes:

- An unprefixed `{.foo}` that resolves in neither scope is **not** an error — `class="foo"` is emitted verbatim, so authors may target plain CSS classes that have no registered role.
- A forced prefix that cannot be satisfied in its scope — `{g.foo}` with no global definition, or `{l.foo}` with no local one — is a resolution error: a lint error in strict mode; in lenient mode the class is emitted unresolved and a warning is recorded.

## Heading Scope Rule

A roled heading opens a scope. The scope closes at the first of:

1. A sibling heading at the **same fence depth** with equal or higher level, or
2. The closer of the fence the heading was opened inside.

Headings at deeper fence depths do **not** close outer scopes — each fence is its own outline universe. This matches HTML5 sectioning semantics.

## Multi-Role Nesting

Multiple roles in nested contexts produce nested elements (descendant selectors), not stacked classes on one element:

```text
::: .grid cols=3
### Fast {.carda}
Content.

  ::: .grid cols=2
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

## Fenced Divs

`:::` opens a fenced block. Everything after the `:::` on the opening line is an **attribute group** — the same grammar a heading uses inside `{…}`, minus the braces (a `:::` line is not prose, so it needs no delimiter):

```text
::: .hero
# Welcome
:::

::: .grid cols=3
...
:::
```

Inside that group a **bare word names the element**, a `.class` adds a class, and `#id` / `key=value` apply as on any element. So `::: .grid cols=3` emits `<div class="grid" data-cols="3">` (no element bareword → the fenced-div default `<div>`), `::: nav .site` emits `<nav class="site">`, and `::: .hero .fancy #top cols=2` emits `<div class="hero fancy" id="top" data-cols="2">`.

A class therefore carries an explicit dot — `::: .grid`, not `::: grid` — so the `:::` header matches the heading attribute grammar exactly, rather than treating the first token as an implicit primary class. What a bare word resolves to — a standard tag, a custom element, or a lint error — is owned by spec 003 (semantic elements); 002 owns the grammar that parses it.

Fenced divs and roled headings both open scopes; the heading scope rule above governs both.

## Acceptance Criteria

- [x] AC1: `{.foo}` attaches `class="foo"` to the immediately preceding (or following) element per the position rule.
- [x] AC2: `{#id}` attaches `id="id"`; duplicate ids in the same file are a lint error.
- [x] AC3: `{key=value}` attaches `data-key="value"` for non-standard keys; recognized HTML attribute names (global or element-standard, per the WHATWG HTML standard) pass through verbatim.
- [x] AC4: Pre-text role on a heading attaches to the heading element only.
- [x] AC5: Post-text role on a heading opens a section scope; the scope closes per the heading scope rule.
- [x] AC6: Nested fenced divs produce nested elements; descendant CSS selectors written against parent.child match the emitted structure.
- [x] AC7: `g.foo` always resolves to a global role definition; `l.foo` always resolves to a local one; an unprefixed `.foo` resolves local-first then global.
- [x] AC8: An inner fence does not close an outer heading scope opened in the parent fence.
- [x] AC9: An unbalanced or dangling `:::` content-fence marker is detected: in strict mode it is a parse error; in lenient mode it is treated as literal content. Because the fenced-div parser tracks fence depth, this is provable here — relocated from [001-file-format](../001-file-format/spec.md), whose block segmentation leaves content fences opaque and so cannot detect it.
- [x] AC10: An inline attribute group `[text]{.foo}` attaches to the bracketed span it follows and never opens a scope; only headings carry the pre-text/post-text section-scope distinction.
- [x] AC11: A forced namespace prefix that cannot be satisfied is a resolution error (strict: lint error; lenient: emit unresolved and warn); an unresolved unprefixed `.foo` is emitted verbatim and is not an error.
- [ ] AC12: A `:::` header parses as an attribute group: a bare word names the element, a `.class` adds a class, and `#id`/`key=value` apply to the element. `::: .grid cols=3` → `<div class="grid" data-cols="3">`; `::: nav .site` → `<nav class="site">`. A class carries an explicit dot; there is no implicit primary-class name. Element resolution is owned by spec 003.
- [x] AC13: Degenerate attribute groups behave per **Edge Cases**: `{}` is a no-op, `{#a #b}` is a lint error, `{=value}` is a strict-mode parse error and lenient-mode literal content.

## Edge Cases

- **Empty group `{}`** — attaches nothing; a no-op, not an error.
- **Multiple ids in one group `{#a #b}`** — a lint error; an element may carry at most one id. (Distinct from the duplicate-id criterion above, which catches the *same* id reused across different elements.)
- **Empty value `{key=}`** — emits an empty value (`data-key=""`); not an error.
- **Missing key `{=value}`** — a malformed token: a parse error in strict mode, literal content in lenient mode.

## Open Questions

<!-- None recorded — resolved decisions are captured in the body above. -->
