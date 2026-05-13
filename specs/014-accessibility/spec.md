---
status: draft
dependencies: []
review:
  last-run: null
  reviewed-against: null
  must-violations: 0
  should-violations: 0
  low-confidence: 0
  blocking: false
---

# 014 — Accessibility

Accessibility is non-negotiable in papur. Strict mode is the default. The compiler enforces accessibility standards (WCAG, ARIA, HTML semantics); it does not enforce authoring preferences. This spec covers the lint rules, the bundled accessible patterns the compiler can auto-wire from the behavior layer, and the auto-emitted defaults.

## Strict Mode Is the Default

Lint rules ship enabled. A `--lenient` flag opts out, but the default behavior of every install is strict.

## Build-Time Lint Rules

The compiler enforces the following at build time:

- Missing alt text on images. `![](src "")` is the explicit "decorative" form — empty string is intentional, missing attribute is a lint error.
- Skipped heading levels (e.g., H2 → H4).
- Multiple H1s outside of `<article>` scope.
- Missing `<main>`, `<title>`, or `lang` attribute.
- Color contrast below WCAG AA when `color` and `background-color` are both set on the same role.
- Interactive role on a non-focusable element without proper wrapping.
- Vague link text without `aria-label` (e.g., "click here", "read more").

Color contrast is scoped to **same-role color/background pairs only**. Cascade-based inherited contrast is intentionally out of scope — better tools exist for that, and inheritance is too dynamic to check statically without false positives.

## Bundled Accessible Patterns

The following patterns ship with the compiler and auto-wire full ARIA + keyboard support when used as roles:

| Pattern | Auto-wired |
| --- | --- |
| `.dialog` | Modal with focus trap, ESC to close, focus restore |
| `.disclosure` | Show/hide with `aria-expanded` |
| `.tabs` | Arrow-key navigation, `aria-selected` |
| `.tooltip` | `aria-describedby`, hover + focus |
| `.alert` | `role="alert"`, live region |
| `.live` | `aria-live="polite"` |

A role that matches a bundled pattern gets ARIA, keyboard handlers, and focus management without explicit author wiring. The behavior layer spec covers the auto-wiring mechanism.

## Auto-Emitted Defaults

The web target emits the following without author opt-in:

- Skip link to `<main>` if `<main>` exists.
- `<meta name="viewport" content="width=device-width, initial-scale=1">` for responsive layout.
- `:focus-visible` outline for any role with a behavior handler (the CSS layer spec is the implementation site).
- `lang` attribute on `<html>` from `::: meta` (a warning is emitted if `::: meta` does not set it).

## Compiler Enforces Standards, Not Preferences

This is the load-bearing distinction: accessibility (WCAG, ARIA, HTML semantics) is enforceable because it is a correctness property, not a taste call. Design-system architecture choices (token naming, two-tier organization, atomic vs. semantic class names) are not enforced — see the two-tier note in the theming spec.

## Acceptance Criteria

- [ ] Strict mode is on by default; the compiler exits non-zero on any lint violation.
- [ ] `--lenient` downgrades violations to warnings.
- [ ] `<img>` without `alt` is a lint error; `<img>` with `alt=""` (decorative) is valid.
- [ ] Heading-level skip (H2 → H4 with no H3) is a lint error.
- [ ] Same-role color/background pairs that fall below WCAG AA contrast emit a lint error.
- [ ] A role matching a bundled pattern (e.g., `.dialog`) auto-wires the documented ARIA, keyboard handlers, and focus management.
- [ ] Web output includes the skip link, viewport meta, and `lang` attribute without author opt-in.
- [ ] A document with `::: meta` missing `lang` emits a warning (not a hard error).

## Open Questions

<!-- None recorded. -->
