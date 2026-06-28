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
---

# 002 — Attribute Syntax (Roles)

How authors attach roles (classes, ids, key/value attributes) to elements. Roles are the join key across the three layers — structure, style, and behavior — so the syntax must be unambiguous and position-aware. This spec depends on the fence rules defined in [001-file-format](../001-file-format/spec.md).

## Pandoc-Style Attributes

Roles use Pandoc-style attribute syntax: `{.class #id key=value}`.

```text
[Get started]{.btn .primary}(/start)

### Welcome {.hero}
```

Multiple classes are space-separated inside one brace group. Keys are `key=value` pairs (no quotes required for unbroken tokens).

## Position-Determined Scope

Whether a role applies to a single element (inline) or to a scope (block) is determined by **where** the attribute sits relative to the heading text:

```text
### {.hero} Welcome        # role applies to the heading element only (inline)
### Welcome {.hero}        # role applies to the heading's section scope (block)
```

Pre-text attributes attach to the heading element; post-text attributes open a section scope keyed to the role.

## Namespace Prefixes

Role lookups resolve through a local-first → global fallback chain. Authors can override the lookup with namespace prefixes:

| Prefix | Meaning |
| --- | --- |
| `{.foo}` | Local-first lookup, falls through to global |
| `{g.foo}` | Force global |
| `{l.foo}` | Force local |

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

Fenced divs and roled headings both open scopes; the heading scope rule above governs both.

## Acceptance Criteria

- [ ] `{.foo}` attaches `class="foo"` to the immediately preceding (or following) element per the position rule.
- [ ] `{#id}` attaches `id="id"`; duplicate ids in the same file are a lint error.
- [ ] `{key=value}` attaches `data-key="value"` for non-standard keys; standard HTML attributes pass through verbatim.
- [ ] Pre-text role on a heading attaches to the heading element only.
- [ ] Post-text role on a heading opens a section scope; the scope closes per the heading scope rule.
- [ ] Nested fenced divs produce nested elements; descendant CSS selectors written against parent.child match the emitted structure.
- [ ] `g.foo` always resolves to a global role definition; `l.foo` always resolves to a local one; an unprefixed `.foo` resolves local-first then global.
- [ ] An inner fence does not close an outer heading scope opened in the parent fence.
- [ ] An unbalanced or dangling `:::` content-fence marker is detected: in strict mode it is a parse error; in lenient mode it is treated as literal content. Because the fenced-div parser tracks fence depth, this is provable here — relocated from [001-file-format](../001-file-format/spec.md), whose block segmentation leaves content fences opaque and so cannot detect it.

## Open Questions

<!-- None recorded — resolved decisions are captured in the body above. -->
