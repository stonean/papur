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
next-criterion: 6
---

# 007 — Raw HTML

An escape hatch for HTML that papur's structure layer cannot or should not express through roles — custom elements, inline media markup, anything web-only. Raw HTML is fenced; see [001-file-format](../001-file-format/spec.md) for the fence rule.

## Block Form

```text
::: html
<video controls>
  <source src="demo.webm" type="video/webm">
</video>
:::
```

`::: html` content is passed through to the web target verbatim. The parser tokenizes it as a single opaque block; it is **not** re-parsed as papur content.

## Inline Raw HTML

Inline raw HTML mid-paragraph (Markdown-style) is permitted for things like `<kbd>`, `<sub>`, `<sup>`. The standard Markdown rule applies: HTML written inline is passed through to the web target.

## Web-Only by Default

`::: html` is web-only by default. The block is dropped from pdf, email, and plain targets unless a target qualifier explicitly includes it.

To opt into a non-web target, qualify the block:

```text
::: @pdf html
<!-- raw HTML that should appear in the pdf target -->
:::
```

The qualified form is rare; most raw HTML belongs to the web target only. Even on the pdf target the qualified form is advisory — the pdf emitter consumes the AST directly and may decline raw HTML it cannot represent as structure-tree-tagged output.

## Acceptance Criteria

- [ ] AC1: A `::: html` block emits its contents verbatim on the web target.
- [ ] AC2: A `::: html` block is omitted from pdf, email, and plain targets by default.
- [ ] AC3: Inline raw HTML in a paragraph (e.g. `Press <kbd>Esc</kbd>`) is preserved on the web target.
- [ ] AC4: A target-qualified `::: @pdf html` block is acknowledged by the pdf target (emission rules for raw HTML in the pdf target are owned by the pdf target spec — see below).
- [ ] AC5: Raw HTML inside a `::: html` block is not re-parsed as papur content — papur-style attribute syntax inside the block is not interpreted.

## See also

- [011-target-pdf](../011-target-pdf/spec.md) — owns the emission rules for target-qualified `::: @pdf html` blocks.

## Open Questions

<!-- None recorded. -->
