# 003 — Semantic Element Registry Plan

Implements [003 — Semantic Element Registry](spec.md).

## Overview

003 decides which HTML element a roled scope's wrapper becomes — a pure,
target-agnostic function over the structure nodes 002 parses. The element is a
**bare word** in the scope's attribute group (a standard tag or a hyphenated
custom element), else a role's `as:` binding, else the scope-type default
(`<section>` for a heading, `<div>` for a fenced div). A class never selects the
element.

The work lands in `papur-core`:

1. A new `element` module — `Element`, `element_for`, the `ElementBindings`
   trait, and the pure `resolve_element` emitters call.
2. A one-field extension to 002's `attr::Attributes` (the bare word), repurposing
   the bare token that is currently `PAPUR-P022`.
3. Validation in the 002 `structure` pass plus three diagnostics in a new
   `P030`–`P039` range.

No emitter is built here; like 002's resolution this ships interface-first with
unit tests over injected bindings (010 is `draft`). Canonical type definitions
are in [data-model.md](data-model.md); this plan covers the decisions.

## Technical Decisions

### Resolution is a pure function in `papur-core`

The semantic element of a scope is target-agnostic — every emitter needs it and
renders it its own way. So `resolve_element` lives in `papur-core::element` and
returns an `Element` emitters map. This mirrors 002, whose data model has 010
walk the structure to produce `<section>`/`<div>`. 010 is `draft`, so 003 is
verified as a pure function.

### Bare word = element, dot = class — and the `:::` grammar moves with it

A bare word in any attribute group names the element; `.class` is only ever a
class. This holds at every position, which required 002's `:::` header to become
a plain attribute group (`::: .grid`, not `::: grid`) — that change is in 002.
003 owns what the bare word *means*: `element_for` resolves it to a standard tag,
a `Custom` element (lowercase + hyphen), or `None` → `P030`. The closed standard
set is typo-safe; custom elements have no registry, so the HTML hyphen rule is
their only check.

### The bare word repurposes 002's malformed-token slot

002's `classify_token` ends with: a bare token (no `.`/`#`/`=`) is `PAPUR-P022`
malformed (a passing `bare_token_strict_is_p022` test). 003 gives it meaning,
populating a new `Attributes.element`. 002's *spec* never specified bare-token
behavior, so this contradicts no 002 acceptance criterion — but it changes 002's
implementation and that test (the `:::` grammar change in 002 carries it).

`attr` stays independent of the registry: `parse_attributes` captures the raw
bare word and flags only arity (a second bare word ⇒ `P031`). Recognition
(`P030`), placement, and binding resolution happen in the structure/resolution
pass, which knows scope kind and has the binding registry.

### `as:` bindings resolve interface-first, like 002's roles

A role's `as:` binding is *data* the role-definition layer (`::: css`, spec 005)
populates; 003 owns the *resolution*. So 003 defines an `ElementBindings` trait
and resolves against it — exactly as 002 ships role resolution against an
injected `RoleRegistry`. 003 depends only on 002; 005 implements the trait later.
Precedence is bare word > first bound role > default; more than one bound role is
`P032`, a **warning** (first wins, nothing breaks).

### Three diagnostics in a new `P030`–`P039` range

Element resolution is its own concern, so it gets its own range rather than
extending 002's `P020`–`P029`: `P030` invalid bare word, `P031` misplaced bare
word, `P032` ambiguous binding (warning). Registered in `errors.md`.

### Class-carry is unchanged

`{nav .x}` → `<nav class="x">`: `.x` is already a `RoleRef` in
`Attributes.roles`, emitted as a class. Resolution chooses only the element.

## Affected Files

| File | Action | Purpose |
| --- | --- | --- |
| `crates/papur-core/src/element/mod.rs` | Create | `Element`, `element_for`, `ElementBindings`, `resolve_element`, tests |
| `crates/papur-core/src/lib.rs` | Modify | Register the `element` module |
| `crates/papur-core/src/attr/mod.rs` | Modify | Add `Attributes.element`; capture bare word + `P031` arity; update `is_empty`; replace `bare_token_strict_is_p022` |
| `crates/papur-core/src/structure/mod.rs` | Modify | Resolve + validate per node (`P030`/`P031`/`P032`); consumes the 002 `:::`-as-attribute-group change |
| `crates/papur-core/src/diagnostic.rs` | Modify | `InvalidElement` (P030), `MisplacedElement` (P031), `AmbiguousBinding` (P032) + `code()` + test |
| `specs/errors.md` | Modify | Register `PAPUR-P030`–`P032` and the `P030`–`P039` range |

(Planning aid; `/papur:implement` derives the real write boundary from git. Coordinated with 002, which is reopened for the `:::` grammar change.)

## Trade-offs

- **`Element` enum with `Custom(String)` vs. a raw string.** The enum makes the
  standard set compile-checked and emitter matches exhaustive, with one `Custom`
  escape hatch for hyphenated names. A raw string would push validation onto
  every consumer; rejected.
- **Bindings via a trait vs. a concrete registry.** The trait keeps 003
  dependency-free and testable before 005 exists, matching 002's `RoleRegistry`.
  Rejected baking in a concrete binding store (would couple 003 to 005).
- **New `P030`–`P039` range vs. extending `P020`–`P029`.** A distinct range keeps
  element diagnostics attributable to 003 and leaves 002's range coherent.
- **Registry check in `structure` vs. `attr`.** Validating `element_for` inside
  `parse_attributes` would couple the low-level parser to the registry. Rejected:
  `attr` captures the bare word syntactically; the resolution pass (which knows
  scope kind and bindings) owns recognition.
- **Keeping `as:` vs. cutting it.** Bare words alone cover every element; `as:`
  adds only central class→element binding. Kept because the maintenance win on
  large collections (re-tag in one line vs. every instance) is real and aligns
  with papur's design-system stance — and it's purely additive over the bare-word
  base.
