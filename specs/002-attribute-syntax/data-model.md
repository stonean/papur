# 002 — Attribute Syntax Data Model

The data structures produced by **attribute and structure parsing** — the pass
that consumes the raw [`Block::Content`](../001-file-format/data-model.md) spans
001 leaves unparsed and produces the role/scope skeleton the downstream layers
key against. This pass parses three things: attribute brace groups, the
content-fence / heading scope tree, and namespace-prefixed role references plus
their resolution.

It does **not** emit HTML (the web emitter, [010-target-web](../010-target-web/spec.md),
walks this model to produce the `<section>` / `<div>` output shown in the spec)
and does **not** freeze the canonical AST (deferred to
[016-language-specification](../016-language-specification/spec.md); the
mdast-vs-custom question stays open). Full Markdown prose/inline parsing is also
out of scope — this pass scans content spans for the three role constructs
(ATX headings, `[text]{…}` inline spans, `::: name` fenced divs) and holds the
remaining prose as text ranges.

All types live in the `papur-core` crate, in new `attr`, `structure`, and `role`
modules alongside the existing `block` module.

## Attribute group — `attr` module

```rust
/// One parsed `{.class #id key=value}` brace group. Every field is optional;
/// `{}` parses to an empty `Attributes` (a no-op, not an error).
pub struct Attributes {
    /// `.class` tokens, in source order. Each is a role reference because a
    /// class is the join key the style/behavior layers resolve against.
    pub roles: Vec<RoleRef>,
    /// `#id` token. At most one is valid; a second `#id` in the same group is a
    /// lint error (`PAPUR-P021`). In lenient mode the first id wins.
    pub id: Option<String>,
    /// `key=value` pairs, insertion-ordered, last-wins on a repeated key.
    pub attrs: KeyValues,
}

/// Insertion-ordered, last-wins map for `key=value` attributes. Backed by
/// `indexmap::IndexMap`, mirroring the `block` module's `KeyMap`.
pub type KeyValues = IndexMap<String, String>;

/// How a `key=value` pair is emitted by the target layer. Computed at parse
/// time from the curated HTML-attribute allowlist; the emitter (010) trusts it.
pub enum AttrKind {
    /// `key` is a recognized HTML attribute name (global, or standard for the
    /// target element) per the WHATWG HTML standard — emitted verbatim.
    Verbatim,
    /// Any other key — emitted as `data-{key}`.
    Data,
}

/// Classify an attribute key. Backed by a curated, centralized allowlist of
/// WHATWG global attributes plus common element-standard attributes (a single
/// `const` set per the constitution's shared-constants rule). The allowlist is
/// the documented source of truth for the verbatim/`data-` boundary.
pub fn classify_attr(key: &str) -> AttrKind;

/// Parse a single brace group. Returns the parsed attributes plus any
/// diagnostics (duplicate id, malformed token). Whitespace separates tokens; an
/// unquoted value is one whitespace-delimited token, and a value with spaces
/// must be double-quoted (`key="a b"`).
pub fn parse_attributes(group: &str, mode: ParseMode) -> (Attributes, Vec<Diagnostic>);
```

## Role reference and resolution — `role` module

```rust
/// A namespace-prefixed class reference, parsed from a `.class` token.
pub struct RoleRef {
    pub namespace: Namespace,
    pub name: String,
}

/// The lookup directive a role-class prefix encodes.
pub enum Namespace {
    /// `{.foo}` — local-first, then global. An unresolved `Auto` is NOT an
    /// error: the class is emitted verbatim so plain-CSS classes work.
    Auto,
    /// `{g.foo}` — force global. Unresolvable in global scope ⇒ `PAPUR-P023`.
    Global,
    /// `{l.foo}` — force local. Unresolvable in local scope ⇒ `PAPUR-P023`.
    Local,
}

/// Abstraction over the set of defined roles. 002 owns the resolution
/// *algorithm*; the registry's *population* is downstream work — local
/// definitions come from same-document layers and the global set arrives with
/// theming ([004-theming](../004-theming/spec.md)) and the CSS layer
/// ([005-css-layer](../005-css-layer/spec.md)). Tests inject a registry
/// directly. Until those land the global set is empty, so a forced `g.foo`
/// resolves only against an injected registry.
pub trait RoleRegistry {
    fn has_local(&self, name: &str) -> bool;
    fn has_global(&self, name: &str) -> bool;
}

/// The outcome of resolving a `RoleRef` against a registry.
pub enum Resolution {
    /// Matched in the named scope.
    Resolved { scope: Scope },
    /// `Auto` matched nowhere — emit `class="name"` verbatim, no diagnostic.
    Unresolved,
    /// A forced prefix could not be satisfied — `PAPUR-P023` (strict error;
    /// lenient emits unresolved and records a warning).
    ForcedMiss,
}

pub enum Scope { Local, Global }

/// Resolve one role reference. Pure over the registry; no I/O.
pub fn resolve(role: &RoleRef, registry: &dyn RoleRegistry, mode: ParseMode)
    -> (Resolution, Option<Diagnostic>);
```

## Structure / scope tree — `structure` module

```rust
/// The role/scope skeleton parsed from one content span. A provisional model
/// (016 will formalize the canonical AST); it carries exactly what roles and
/// scopes need, holding prose as text ranges rather than a full Markdown tree.
pub struct StructureTree {
    pub nodes: Vec<Node>,
    pub mode: ParseMode,
}

pub enum Node {
    /// An ATX heading. A pre-text attribute group attaches to the heading
    /// element only (`attrs`, `opens_scope = false`); a post-text group opens a
    /// section scope (`opens_scope = true`) closed per the heading scope rule.
    Heading {
        level: u8,
        attrs: Attributes,
        opens_scope: bool,
        fence_depth: u32,
        children: Vec<Node>,
        span: Span,
    },
    /// A `::: name [attrs]` fenced div. `name` is the primary class; trailing
    /// `.class`/`#id`/`key=value` go on the same element. Nested fences produce
    /// nested nodes (descendant structure, not stacked classes).
    FencedDiv {
        name: String,
        attrs: Attributes,
        fence_depth: u32,
        children: Vec<Node>,
        span: Span,
    },
    /// An inline `[text]{attrs}` span. Attaches to the bracketed text and never
    /// opens a scope.
    InlineSpan { text: String, attrs: Attributes, span: Span },
    /// Unparsed prose held verbatim for the downstream Markdown/AST pass.
    Prose { text: String, span: Span },
}
```

Scope nesting is established by two rules implemented here:

- **Heading scope rule** — a post-text roled heading's scope closes at the first
  sibling heading at the **same fence depth** with equal-or-higher level, or at
  the closer of the fence it was opened inside. Headings at deeper fence depths
  do not close outer scopes — each fence is its own outline universe.
- **Fenced-div depth** — `::: name` opens and a bare `:::` closes; a stack
  tracks depth. An unbalanced/dangling marker is `PAPUR-P002`.

## Diagnostics

```rust
/// New variants added to the existing `block::DiagnosticCode` enum. Each maps to
/// a permanent `PAPUR-P` code registered in `specs/errors.md`.
pub enum DiagnosticCode {
    // … existing 001 variants (UnterminatedFence P001, MalformedFrontmatter P010) …

    /// An unbalanced or dangling `:::` content fence (strict error; lenient
    /// treats the marker as literal content). Fence range. → PAPUR-P002
    DanglingContentFence,
    /// The same `id` appears on more than one element in the file (lint error).
    /// → PAPUR-P020
    DuplicateId,
    /// More than one `#id` in a single attribute group (lint error; lenient
    /// keeps the first). → PAPUR-P021
    MultipleIds,
    /// A malformed attribute token, e.g. `{=value}` (strict error; lenient
    /// literal). → PAPUR-P022
    MalformedAttribute,
    /// A forced namespace prefix (`g.`/`l.`) resolves to no definition (strict
    /// error; lenient emits unresolved and warns). → PAPUR-P023
    UnresolvedForcedRole,
}
```

## Notes

- **Provisional vs canonical.** `StructureTree`/`Node` is 002's working model.
  016 may rename or subsume it when it formalizes the AST; 002 deliberately
  keeps it minimal (roles + scopes + prose ranges) to avoid pre-empting that
  decision.
- **Resolution is interface-first.** The algorithm (local-first→global, forced
  prefixes, the `P023` boundary) ships in 002; the *data* it resolves against is
  populated by 004/005. This split lets 002 satisfy its resolution acceptance
  criteria via injected test registries today.
- **`classify_attr` is the verbatim/`data-` source of truth.** A single curated
  allowlist; emitters never re-derive the boundary.
- **Empty/degenerate groups are not errors except where noted.** `{}` → empty
  `Attributes`; `{key=}` → `key` with an empty value; `{#a #b}` → `P021`;
  `{=value}` → `P022`.
