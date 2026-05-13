---
status: draft
dependencies: [002-attribute-syntax, 003-semantic-elements, 009-multi-target, 014-accessibility]
review:
  last-run: null
  reviewed-against: null
  must-violations: 0
  should-violations: 0
  low-confidence: 0
  blocking: false
---

# 006 — Behavior Layer

The behavior layer attaches handlers to roles. Behaviors compile to vanilla JS at build time — no framework, no runtime dependency. Recognized interaction patterns auto-emit correct ARIA, keyboard handlers, and focus management.

This spec depends on the role grammar in [002-attribute-syntax](../002-attribute-syntax/spec.md), the semantic registry in [003-semantic-elements](../003-semantic-elements/spec.md), and the accessibility commitments in [014-accessibility](../014-accessibility/spec.md). It interacts with the auto-emitted `:focus-visible` rule defined by the CSS layer.

## Block Form

Handlers live in `::: script` blocks, keyed on role selectors:

```text
::: script
.btn.primary
  on click
    track('cta-click')

.accordion .header
  on click
    toggle .open on closest('.accordion')
:::
```

The exact DSL grammar is an open question (see below). The intent is a small, declarative surface: events, conditions, and side effects on roles — not arbitrary imperative JavaScript.

## Auto-Wired Accessibility

When the compiler recognizes a known interaction pattern (accordion, dialog, tabs, disclosure, tooltip, alert, live region — the accessibility spec lists the bundled set), it auto-wires:

- ARIA states (`aria-expanded`, `aria-controls`, `aria-hidden`, etc.)
- Keyboard handlers (Space, Enter, ESC, arrow keys)
- Focus management (trap, restore, initial focus)
- Roles (`role="button"`, `role="dialog"`, etc.)

A role that does not match a bundled pattern still compiles, but the author is responsible for ARIA and keyboard support.

## Non-Web Targets

The behavior layer is silently dropped from non-web targets (print, email, plain) — see [009-multi-target](../009-multi-target/spec.md). No warning is emitted; the dropped behavior is a contract, not a mistake.

## Acceptance Criteria

- [ ] A handler attached to a role selector compiles to a vanilla-JS event listener on every element matching that role.
- [ ] The emitted JS has no framework dependency and runs against the DOM produced by the structure layer.
- [ ] A role recognized as a bundled accessible pattern (e.g., `.dialog`) auto-emits the correct ARIA attributes, keyboard handlers, and focus management without explicit author wiring.
- [ ] An unrecognized role with a handler compiles, but the author is responsible for ARIA — strict mode warns if interactive markup lacks an accessible name.
- [ ] On print, email, and plain targets, the `::: script` block produces no output.

## Open Questions

- **Behavior DSL grammar** — the full syntax (event names, condition expressions, side-effect verbs, target expressions like `closest('.x')`) needs a complete design pass. The examples above are placeholders.
- **Compile output shape** — vanilla JS at build time is the intent, but the exact emission strategy (inline `<script>`, separate file, module vs IIFE) needs confirmation.
- **Custom behavior patterns** — can authors register their own auto-wired patterns, or is the bundled set fixed?
