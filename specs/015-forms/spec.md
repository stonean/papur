---
status: draft
dependencies: [002-attribute-syntax, 014-accessibility]
review:
  last-run: null
  reviewed-against: null
  must-violations: 0
  should-violations: 0
  low-confidence: 0
  blocking: false
---

# 015 — Forms

Deferred. Forms need a dedicated design pass that aligns labels, fields, validation, and error wiring with papur's role grammar ([002-attribute-syntax](../002-attribute-syntax/spec.md)) and accessibility commitments ([014-accessibility](../014-accessibility/spec.md)).

## Rough Sketch (Not Locked)

```text
::: form action="/signup" method="post"
> label Email
> @ email required

> label Password
> @ password required min=8

> button Sign up {.btn .primary}
:::
```

## Tentative Commitments

These are the properties any form design should preserve. They are not yet acceptance criteria — they are inputs to the design pass.

- Labels are mandatory. A field without a label is a lint error.
- Label/field association (`for`/`id`) is auto-generated from the label text and field name.
- Validation errors are auto-linked via `aria-describedby`.
- Required fields receive `aria-required`.
- Native HTML5 validation attributes (`required`, `min`, `max`, `pattern`, etc.) are supported.

## Acceptance Criteria

<!-- Acceptance criteria will be filled in during the design pass. The spec currently
     has no testable contract because the syntax and emission rules are not yet decided. -->

## Open Questions

- **Field syntax** — is `> @ type` the right marker? Alternatives: `> input type=email`, `> field type=email`, dedicated keyword per type. The chosen syntax must round-trip cleanly through the role grammar.
- **Validation strategy** — native HTML5 validation only, or papur-layer validation that emits both HTML5 attributes and richer error messaging?
- **Error rendering** — automatic insertion of error containers, or explicit author-placed slots?
- **Multi-step forms** — supported as a first-class pattern, or expressed by composing simpler forms?
- **CSRF and security** — does papur emit CSRF token slots, or is the form purely structural and security is left to the form action's server-side handler?
- **Non-web targets** — does a form render anywhere besides web? Likely web-only, but the pdf/email/plain emitters need to know whether to drop it silently or emit a static representation.
