---
status: planned
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

How a roled scope chooses the HTML element its wrapper becomes. The element is named by a **bare word** in the scope's attribute group; when none is given it defaults by scope type. A class never selects the element — `.name` is only ever a class. One rule holds at every position: **bare word = element, dot = class**.

This builds on the role grammar in [002-attribute-syntax](../002-attribute-syntax/spec.md), whose attribute group (`{…}` on headings, the bare `:::` header on fenced divs) is where the bare word and classes are written.

## Recognized Elements

A bare word names the wrapper element. It is valid when it is **either**:

- one of the standard registry tags — `nav`, `header`, `footer`, `main`, `article`, `aside`, `section`, `figure`, `figcaption`, `details`, `summary`, `dialog`, `blockquote`, `div`, `span` — matched exactly, lowercase; **or**
- a valid **custom element** name (lowercase, contains a hyphen, per the HTML standard) — e.g. `recipe-card`, `my-widget`. These are accepted as-is and emitted verbatim.

Anything else (a non-hyphenated word that is not a standard tag, e.g. `hero`, `grid`, `flibble`) is a lint error — it must be a class (`.hero`). The standard set is closed and typo-safe (it has canonical spellings to check against); custom names carry no registry, so the hyphen is their only validation. `div`/`span` are in the set so a heading scope can be downgraded from its `<section>` default.

## Default Element

When a scope is opened with no element bare word and no binding, the wrapper defaults by scope type:

| Scope | Opened by | Default |
| --- | --- | --- |
| Heading scope | a post-text role on a heading | `<section>` |
| Fenced div | `:::` with no element bare word | `<div>` |

A roled heading delimits a document section, so `<section>` is its natural default (matching Pandoc `--section-divs`); a `:::` block is literally a div. A plain heading with no role opens no scope and stays a flat `<h3>` etc.

## Naming the Element

In any attribute group a bare word names the element and a `.word` adds a class — independent of each other:

| Source | Emits |
| --- | --- |
| `## Latest {.post}` | `<section class="post">` |
| `## Latest {article .post}` | `<article class="post">` |
| `## Latest {div .post}` | `<div class="post">` |
| `::: .grid cols=3` | `<div class="grid" data-cols="3">` |
| `::: nav .site` | `<nav class="site">` |
| `::: recipe-card` | `<recipe-card>` |

Rules:

- The role's class(es) are always carried onto the wrapper, so the style and behavior layers can target it.
- At most one element bare word per group; a second is a lint error.
- An element bare word only has meaning where a scope opens (a post-text role or a fenced div). In a pre-text or inline position it is a lint error — a heading cannot retag itself.

## Element Bindings (`as:`)

A role can bind an element once, so every use of that role gets it without repeating the bare word. The binding is declared in the role-definition layer (the `::: css` block — spec 005 owns the authoring syntax; this spec owns what it resolves to):

```text
.post
  as: article
```

Now the binding applies wherever the role opens a scope — heading or fenced div, identically:

| Source (with `.post` bound to `article`) | Emits |
| --- | --- |
| `## Latest {.post}` | `<article class="post">` |
| `::: .post` | `<article class="post">` |

This is the maintenance path: re-tagging a whole collection is a one-line edit instead of touching every instance. The bound value is itself a Recognized Element (standard tag or custom element).

**Precedence** — most specific wins:

> use-site **bare word** > role **`as:` binding** > scope-type **default**

So a bare word overrides a binding (`{section .post}` → `<section class="post">` even when `.post` binds `article`), and a binding overrides the default. When a scope carries **more than one** `as:`-bound role, the **first in source order wins** and a lint **warning** is emitted (stacking is meaningless — an element is one tag) — it never errors.

## Inline vs Block

The position rule from [002-attribute-syntax](../002-attribute-syntax/spec.md) determines whether a role opens a scope:

- **Pre-text** (`### {.post} Latest`) attaches the class to the heading element; it keeps its native tag (`<h3 class="post">`). No scope opens, and an element bare word is not valid here.
- **Post-text** (`### Latest {.post}`) opens a heading scope; its element is the bare word, else the binding, else `<section>`.

## Diagnostics

Three codes in a new `P030`–`P039` element-resolution range (registered in [`errors.md`](../errors.md)):

| Code | Condition | Strict | Lenient |
| --- | --- | --- | --- |
| `PAPUR-P030` | A bare word that is neither a standard tag nor a valid custom element (`{flibble}`, `::: hero`) | error | warn; use the scope default |
| `PAPUR-P031` | A misplaced element bare word: in a pre-text/inline position, or more than one in a group | error | warn; ignore the bare word |
| `PAPUR-P032` | A scope with more than one `as:`-bound role | **warning** | warning |

`P030`/`P031` follow the strict-error / lenient-warning pattern; `P032` is always a warning — the first binding still applies, so nothing breaks.

## Acceptance Criteria

- [ ] A heading scope with no bare word emits `<section>`: `## Latest {.post}` → `<section class="post">`.
- [ ] A fenced div with no bare word emits `<div>`: `::: .grid cols=3` → `<div class="grid" data-cols="3">`.
- [ ] A bare word names the element at either position: `### Fast {nav .x}` → `<nav class="x">`; `::: nav .site` → `<nav class="site">`.
- [ ] A class never selects the element: `### Fast {.nav}` → `<section class="nav">`, not `<nav>`.
- [ ] A hyphenated bare word is a custom element: `::: recipe-card` → `<recipe-card>`; `### Pancakes {recipe-card .x}` → `<recipe-card class="x">`.
- [ ] An invalid bare word (not a standard tag, not a valid custom name) is `PAPUR-P030` (strict error; lenient warning + scope default): `::: hero` → error (use `.hero`).
- [ ] An element bare word in a pre-text/inline position, or a second bare word in one group, is `PAPUR-P031`; the heading keeps its native tag.
- [ ] `div`/`span` downgrade a heading scope: `### Fast {div .card}` → `<div class="card">`.
- [ ] An `as:` binding sets a role's element wherever it opens a scope: with `.post { as: article }`, both `## Latest {.post}` and `::: .post` emit `<article class="post">`.
- [ ] Precedence holds: a use-site bare word overrides a binding (`{section .post}` → `<section class="post">`), and a binding overrides the scope default.
- [ ] Two `as:`-bound roles on one scope: the first in source order wins and `PAPUR-P032` (warning) is emitted; never an error.

## Resolved Questions

- **Bare word = element, dot = class — uniformly.** The same rule holds in a heading group (`{nav .x}`) and a fenced-div header (`::: nav`); a class never resolves to an element by name. This removes the hidden registry-lookup whose structural meaning the author couldn't see — the same magic removed from `.class` resolution. The `:::` header consequently adopts dotted classes (`::: .grid`, not `::: grid`); the grammar change is recorded in 002.
- **A bare word is a standard tag or a valid custom element.** The closed standard set is typo-checked; custom elements are recognized by the HTML hyphen rule (no registry to check against). Everything else is `P030`.
- **Defaults by scope type.** Heading scope → `<section>` (a heading delimits a section; Pandoc `--section-divs`); fenced div → `<div>`. Override with a bare word.
- **`as:` is kept and owned here.** It selects an *element*, which is structural — this spec's concern, not the CSS layer's. It earns its keep as the maintenance path for collections (one binding governs every instance). Authored in the role-definition layer (`::: css`); spec 005 owns the syntax, this spec owns the resolution and precedence (bare word > binding > default).
- **Multiple bound roles → first wins, warn.** Stacking elements is meaningless, but it should not break a build, so the first binding applies and `P032` is a warning, not an error.
- **The wrapper carries the role's class.** Every scope wrapper emits `class="{role}"` so the style/behavior layers can target it, regardless of how the element was chosen.

## Open Questions

<!-- None recorded. -->
