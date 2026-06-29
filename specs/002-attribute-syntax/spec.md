---
status: done
dependencies: [001-file-format]
review:
  last-run: 2026-06-29T02:42:39Z
  reviewed-against: 5fecc8b528bd6c0d334817776e2c75a94ef55af7
  must-violations: 0
  should-violations: 0
  low-confidence: 0
  blocking: false
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

## Fenced Divs

`::: name` creates a named block. Attributes after the name pass through as element attributes (with `data-` for non-standard keys, per the emitter):

```text
::: hero
# Welcome
:::

::: grid cols=3
...
:::
```

The block name is the div's primary class. Additional `.class`, `#id`, and `key=value` attributes may follow the name on the same line and apply to the same `<div>` — `::: hero .fancy #top cols=2` emits `<div class="hero fancy" id="top" data-cols="2">`.

Fenced divs and roled headings both open scopes; the heading scope rule above governs both.

## Acceptance Criteria

- [x] `{.foo}` attaches `class="foo"` to the immediately preceding (or following) element per the position rule.
- [x] `{#id}` attaches `id="id"`; duplicate ids in the same file are a lint error.
- [x] `{key=value}` attaches `data-key="value"` for non-standard keys; recognized HTML attribute names (global or element-standard, per the WHATWG HTML standard) pass through verbatim.
- [x] Pre-text role on a heading attaches to the heading element only.
- [x] Post-text role on a heading opens a section scope; the scope closes per the heading scope rule.
- [x] Nested fenced divs produce nested elements; descendant CSS selectors written against parent.child match the emitted structure.
- [x] `g.foo` always resolves to a global role definition; `l.foo` always resolves to a local one; an unprefixed `.foo` resolves local-first then global.
- [x] An inner fence does not close an outer heading scope opened in the parent fence.
- [x] An unbalanced or dangling `:::` content-fence marker is detected: in strict mode it is a parse error; in lenient mode it is treated as literal content. Because the fenced-div parser tracks fence depth, this is provable here — relocated from [001-file-format](../001-file-format/spec.md), whose block segmentation leaves content fences opaque and so cannot detect it.
- [x] An inline attribute group `[text]{.foo}` attaches to the bracketed span it follows and never opens a scope; only headings carry the pre-text/post-text section-scope distinction.
- [x] A forced namespace prefix that cannot be satisfied is a resolution error (strict: lint error; lenient: emit unresolved and warn); an unresolved unprefixed `.foo` is emitted verbatim and is not an error.
- [x] A fenced div applies its name as the primary class and any trailing `.class`/`#id`/`key=value` attributes to the same `<div>`.
- [x] Degenerate attribute groups behave per **Edge Cases**: `{}` is a no-op, `{#a #b}` is a lint error, `{=value}` is a strict-mode parse error and lenient-mode literal content.

## Edge Cases

- **Empty group `{}`** — attaches nothing; a no-op, not an error.
- **Multiple ids in one group `{#a #b}`** — a lint error; an element may carry at most one id. (Distinct from the duplicate-id criterion above, which catches the *same* id reused across different elements.)
- **Empty value `{key=}`** — emits an empty value (`data-key=""`); not an error.
- **Missing key `{=value}`** — a malformed token: a parse error in strict mode, literal content in lenient mode.

## Open Questions

<!-- None recorded — resolved decisions are captured in the body above. -->
