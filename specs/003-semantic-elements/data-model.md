# 003 — Semantic Element Registry Data Model

The types that resolve a roled scope to its wrapper element. They extend the 002
attribute/structure pass ([002 data model](../002-attribute-syntax/data-model.md))
and live in `papur-core`: a new `element` module, one field added to
`attr::Attributes`, and three `DiagnosticCode` variants. Resolution is pure — no
HTML is emitted here; each target emitter maps the resolved `Element` to its own
output, as 002 defers emission to 010.

## Recognized elements — `element` module

```rust
/// The wrapper element a roled scope resolves to. The standard variants are a
/// closed set; `Custom` carries a validated custom-element name (lowercase,
/// contains a hyphen). `Div`/`Span` are members so a heading scope can be
/// downgraded from its `<section>` default.
pub enum Element {
    Nav, Header, Footer, Main, Article, Aside, Section,
    Figure, Figcaption, Details, Summary, Dialog, Blockquote,
    Div, Span,
    /// A valid custom element, e.g. `recipe-card`. Emitted verbatim.
    Custom(String),
}

/// Resolve a bare word to an `Element`:
/// - a standard registry tag (exact, lowercase) ⇒ that variant;
/// - otherwise a valid custom-element name (lowercase, contains a hyphen,
///   per the HTML standard) ⇒ `Custom`;
/// - otherwise `None` ⇒ not an element (→ `P030`). The standard table is one
///   `const` (the constitution's shared-constants rule) — the source of truth
///   spec 005's `as:` values are also validated against.
pub fn element_for(name: &str) -> Option<Element>;

/// The scope kind being resolved, from the 002 structure node.
pub enum ScopeKind { Heading, FencedDiv }

/// Look up a role's `as:` element binding. 003 owns the resolution *algorithm*;
/// the binding *data* is populated by the role-definition layer (`::: css`,
/// spec 005) and injected here — mirroring 002's `RoleRegistry`. Empty until 005
/// lands, so tests inject bindings directly.
pub trait ElementBindings {
    /// The element a role is bound to via `as:`, if any (raw, unresolved).
    fn bound_element(&self, role: &str) -> Option<&str>;
}

/// Resolve a scope's wrapper element. Precedence: use-site `bareword` >
/// first `as:`-bound `role` (source order) > scope-type default
/// (`Section` for `Heading`, `Div` for `FencedDiv`). Returns the element plus
/// any diagnostics (`P030` invalid bare word, `P032` multiple bound roles).
pub fn resolve_element(
    kind: ScopeKind,
    bareword: Option<&str>,
    roles: &[RoleRef],
    bindings: &dyn ElementBindings,
    mode: ParseMode,
) -> (Element, Vec<Diagnostic>);
```

Class emission is unchanged from 002 — the role classes ride on `Attributes.roles`;
this chooses only the element.

## Extension to `attr::Attributes`

```rust
/// 002's `Attributes` gains one field. A bare token in an attribute group — no
/// `.`/`#`/`=` prefix — names the wrapper element of the scope the group opens.
/// Used by both heading groups and the `:::` header (002's grammar change).
pub struct Attributes {
    pub roles: Vec<RoleRef>,   // unchanged (002)
    pub id: Option<String>,    // unchanged (002)
    pub attrs: KeyValues,      // unchanged (002)
    /// `nav` in `{nav .x}` / `::: nav`. Captured raw (not yet checked against the
    /// registry). First bare word wins; a second is `PAPUR-P031`.
    pub element: Option<String>,
}
```

`parse_attributes` changes at the bare-token branch of `classify_token`: today a
bare token is `PAPUR-P022` malformed (`bare_token_strict_is_p022`); it now
populates `element` (first wins; a second emits `P031`). `Attributes::is_empty`
also tests `element.is_none()`. The registry check (`P030`) is *not* done here —
that keeps `attr` independent of the `element` module; recognition, placement,
and binding resolution happen in the structure/resolution pass.

## Diagnostics

```rust
/// New variants on `diagnostic::DiagnosticCode`, in a new P030–P039
/// element-resolution range (spec 003).
pub enum DiagnosticCode {
    // … existing 001 + 002 variants (P001, P002, P010, P020–P023) …
    /// A bare word that is neither a standard tag nor a valid custom element
    /// (`{flibble}`, `::: hero`). Strict: error. Lenient: warn + scope default.
    /// → PAPUR-P030
    InvalidElement,
    /// A misplaced element bare word: in a pre-text/inline position, or more than
    /// one in a group. Strict: error. Lenient: warn + ignore. → PAPUR-P031
    MisplacedElement,
    /// A scope with more than one `as:`-bound role; the first in source order
    /// wins. Always a warning (never an error). → PAPUR-P032
    AmbiguousBinding,
}
```

## Resolution and validation in the `structure` pass

The scanner produces `Node::Heading { attrs, opens_scope, … }` and
`Node::FencedDiv { attrs, … }` (002, post grammar change). 003 resolves and
validates as those nodes are built:

| Node | Element source | Validation |
| --- | --- | --- |
| `Heading`, `opens_scope = true` (post-text) | `attrs.element`, else binding, else `Section` | `attrs.element` set but invalid ⇒ `P030`; >1 bound role ⇒ `P032` |
| `Heading`, `opens_scope = false` (pre-text) | — | `attrs.element.is_some()` ⇒ `P031` (a heading cannot retag itself) |
| `InlineSpan` | — | `attrs.element.is_some()` ⇒ `P031` |
| `FencedDiv` | `attrs.element`, else binding, else `Div` | same as a post-text heading |

The structure pass owns the diagnostics so the CLI reports them at parse time;
each emitter recomputes the `Element` via `resolve_element` (the target-agnostic
decision).

## Notes

- **Resolution is pure and emitter-agnostic** — the algorithm ships here; the
  inputs (scope kind, bare word, roles) come from the 002 structure pass and the
  binding data from 005. 010 is `draft`, so 003 is tested as a pure function over
  injected bindings, not through HTML.
- **The closed standard set lives in one `const`** — `element_for` is the single
  source of truth; emitters and spec 005's `as:` validation never re-derive it.
- **Custom elements need no binding** — a hyphenated bare word (`::: recipe-card`)
  resolves to `Custom` directly; the `as:` binding is only for mapping a class to
  a *different* element name across many instances.
- **Bare-token repurposing** — 002's spec never specified bare-token behavior
  (its Edge Cases cover `{}`, `{#a #b}`, `{key=}`, `{=value}`); the `P022`
  outcome was an implementation default. 003 gives the token meaning, so the
  `bare_token_strict_is_p022` test is replaced by element-bare-word tests.
