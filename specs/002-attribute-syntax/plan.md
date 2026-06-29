# 002 — Attribute Syntax (Roles) Plan

Implements [002 — Attribute Syntax (Roles)](spec.md).

## Overview

002 parses the role grammar all three papur layers share. It consumes the raw
[`Block::Content`](../001-file-format/data-model.md) spans that 001 segmentation
leaves opaque and produces a **role/scope skeleton** plus diagnostics. The work
splits into three new `papur-core` modules — `attr` (brace-group grammar),
`structure` (content-fence + heading scope tree), and `role` (namespace
resolution) — and extends the shared `diagnostic` enum.

The pass stops at the skeleton: it does **not** emit HTML (that is the web
emitter, [010-target-web](../010-target-web/spec.md)) and does **not** freeze
the canonical AST (that is [016-language-specification](../016-language-specification/spec.md),
whose mdast-vs-custom question stays open). Full Markdown parsing is out of
scope — 002 scans for the three role constructs (ATX headings, `[text]{…}`
spans, `::: name` fences) and leaves prose as text ranges. See
[data-model.md](data-model.md) for the type definitions.

## Technical Decisions

### Scope boundary: role/scope skeleton, not AST or HTML

002 owns the grammar, the scope/nesting rules, role resolution, and the
diagnostics — everything that is *uniquely* about how roles attach. It produces
a provisional `StructureTree` (see data-model) that 010 walks to emit HTML and
that 016 will subsume into the canonical AST. This keeps 002 self-contained:
nothing it builds depends on 010/016 existing, and it avoids re-implementing
Markdown block/inline parsing, which belongs to the later AST integration.

### Three new modules in `papur-core`

| Module | Responsibility |
| --- | --- |
| `attr` | Parse `{.class #id key=value}`; classify keys verbatim-vs-`data-`; degenerate-form diagnostics. |
| `structure` | Scan a content span for headings / inline spans / `::: name` fences; track fence depth; build the scope tree; emit the dangling-fence diagnostic. |
| `role` | `RoleRef` + `Namespace`; the `RoleRegistry` abstraction; the local-first→global resolution algorithm; the forced-prefix diagnostic. |

The existing `diagnostic` module gains new `DiagnosticCode` variants; `lib.rs`
re-exports the new public types.

### Attribute grammar (`attr`)

`parse_attributes(group, mode) -> (Attributes, Vec<Diagnostic>)`. Tokens inside
one brace group are whitespace-separated: `.x` → role, `#x` → id, `x=y` → pair.
An unquoted value is a single whitespace-delimited token; a value containing
spaces must be double-quoted (`key="a b"`). Single-quote values are deferred
(see Trade-offs). Degenerate forms: `{}` → empty `Attributes` (no-op); `{key=}`
→ empty value (not an error); `{#a #b}` → `PAPUR-P021`; `{=value}` → `PAPUR-P022`.

### Verbatim-vs-`data-` classification

`classify_attr(key)` checks a single curated `const` allowlist — the WHATWG
global attributes plus common element-standard attributes — and returns
`Verbatim` or `Data`. This is the documented source of truth for the boundary;
the emitter (010) trusts it rather than re-deriving. Centralizing the list as
one constant satisfies the constitution's shared-constants rule.

### Fenced-div depth tracking + dangling-fence diagnostic

A stack-based scan over `:::` markers: `::: name [attrs]` opens, a bare `:::`
closes. Depth is recorded on each node (the heading scope rule needs it). An
unbalanced or dangling marker is `PAPUR-P002` — a strict-mode parse error,
lenient-mode literal content. This is the capability `specs/errors.md` assigns
to 002 (001's block segmentation leaves content fences opaque, so it cannot
detect this).

### Heading scope rule and position rules

A post-text roled heading opens a section scope; a pre-text group attaches to
the heading element only. Inline `[text]{…}` spans attach to the bracketed text
and never open a scope. A scope closes at the first sibling heading at the
**same fence depth** with equal-or-higher level, or at the fence closer.
Deeper-fence headings never close an outer scope — each fence is its own outline
universe (HTML5 sectioning). Nesting is structural: nested roles produce nested
nodes (descendant relationships), never stacked classes on one element.

### Role resolution (`role`)

`resolve(role, registry, mode)` implements the lookup: `Auto` = local-first then
global (an unresolved `Auto` emits the class verbatim, no diagnostic); `Global` /
`Local` force a scope and emit `PAPUR-P023` when unsatisfiable. The set of
defined roles is behind the `RoleRegistry` trait; its population is downstream
(local definitions from same-document layers; the global set from
[004-theming](../004-theming/spec.md) / [005-css-layer](../005-css-layer/spec.md)).
Until those land the global set is empty, so 002's resolution ACs are verified
with injected test registries.

### Diagnostics and code allocation

Extend `diagnostic::DiagnosticCode` and register the codes in `specs/errors.md`,
adding a new Parse range for attribute/role concerns:

| Code | Meaning | Mode behavior |
| --- | --- | --- |
| `PAPUR-P002` | Unbalanced/dangling `:::` content fence | strict error / lenient literal |
| `PAPUR-P020` | Duplicate `id` in the same file | lint error / lenient keeps both |
| `PAPUR-P021` | Multiple `#id` in one attribute group | lint error / lenient first wins |
| `PAPUR-P022` | Malformed attribute token (`{=value}`) | strict error / lenient literal |
| `PAPUR-P023` | Forced namespace prefix unresolved | strict error / lenient warn |

`P002` extends the existing fence range (`P001`–`P009`); `P020`–`P029` is a new
range for attribute/role diagnostics.

### Testing

Acceptance tests in `crates/papur-core/tests/acceptance.rs`, one per spec
acceptance criterion, plus per-module unit tests. Structured outputs are pinned
with `insta` snapshots, matching the 001 pattern. The duplicate-id lint is a
whole-file pass, so it is exercised through the top-level parse entry point.

## Affected Files

| File | Action | Purpose |
| --- | --- | --- |
| `crates/papur-core/src/attr/mod.rs` | Create | Brace-group grammar: `Attributes`, `RoleRef`, `classify_attr`, `parse_attributes` |
| `crates/papur-core/src/structure/mod.rs` | Create | Content-span scanner, scope tree, fenced-div depth, dangling-fence diagnostic |
| `crates/papur-core/src/role/mod.rs` | Create | `Namespace`, `RoleRegistry`, `resolve` |
| `crates/papur-core/src/diagnostic.rs` | Modify | Add `P002`/`P020`–`P023` variants and `code()` arms |
| `crates/papur-core/src/lib.rs` | Modify | `pub mod attr/structure/role` + re-exports |
| `crates/papur-core/tests/acceptance.rs` | Modify | One test per acceptance criterion |
| `crates/papur-core/tests/snapshots/` | Create | New `insta` snapshots |
| `specs/errors.md` | Modify | Register `P002`/`P020`–`P023`; add the `P020`–`P029` range row |

The table is a planning aid; `/papur:implement` derives the authoritative write
boundary from git history.

## Data Model

Defined in [data-model.md](data-model.md): `Attributes` / `KeyValues` /
`AttrKind` (`attr`), `RoleRef` / `Namespace` / `RoleRegistry` / `Resolution`
(`role`), `StructureTree` / `Node` (`structure`), and the new `DiagnosticCode`
variants.

## Trade-offs

- **Provisional structure model.** `StructureTree`/`Node` may be renamed or
  subsumed when 016 formalizes the AST. Accepted: 002 needs *a* tree to express
  scope/nesting, and keeping it minimal (roles + scopes + prose ranges) limits
  the eventual churn. Alternative — block on 016 first — rejected: it would
  stall every layer that keys on roles.
- **Resolution algorithm ships before its data.** Forced `g.foo` resolution
  can't fire against real global definitions until theming/CSS land, so runtime
  coverage of `P023` is partial until then; tests use injected registries.
  Alternative — defer resolution entirely to 004/005 — rejected: the resolution
  semantics are part of *this* spec's contract (AC-7 and the forced-prefix AC).
- **Construct-scanning, not a Markdown parser.** 002 recognizes only headings,
  `[text]{…}` spans, and `::: name` fences; it does not parse general Markdown.
  Risk: `{…}`-like text inside code spans/blocks could be misread. Mitigation:
  treat fenced/inline code as opaque; capture any surviving edge as a scenario.
- **Quoting is double-quote only initially.** Single-quoted values are deferred
  to a follow-up scenario rather than specified now.
- **HTML allowlist maintenance.** The curated verbatim-attribute list can drift
  from the evolving WHATWG set. Mitigation: one centralized constant with a
  focused test; updates are a localized edit.
