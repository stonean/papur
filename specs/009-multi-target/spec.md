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

# 009 — Multi-Target Output

The dispatch architecture that lets one `.papur` source compile to multiple output targets. This spec governs the AST → emitter contract, the target-qualified content syntax, and cross-target rules that apply uniformly (e.g., behaviors drop on non-web targets). It depends on the file format defined in [001-file-format](../001-file-format/spec.md).

> Per-target emission rules live in their own specs:
>
> - `010-target-web` — HTML + CSS
> - `011-target-pdf` — tagged PDF, emitted directly from the AST
> - `012-target-email` — table-based HTML with inlined CSS
> - `013-target-plain` — plain text

## AST-Driven Dispatch

The parser produces one AST per source file. Each target emitter walks the same AST and produces output for that target. The AST is not target-aware; emitters are.

The set of targets is extensible. New targets are added as new spec files numbered above the existing target specs.

## Target-Qualified Content Blocks

A fenced block can be scoped to one or more targets:

```text
::: @web
@include interactive-demo.papur
:::

::: @pdf
*See the live version at example.com/demo*
:::
```

Default behavior: a block with no `@target` qualifier is rendered for all targets.

The `@target` qualifier also applies to `::: theme` overrides and rule blocks in `::: css`.

## Behaviors Drop Silently on Non-Web Targets

The `::: script` layer is omitted entirely from non-web targets. No warning is emitted; the behavior is a contract.

## Target Detection at Compile Time

The compile target is supplied at compile time (CLI flag, build config, API parameter). The author does not select a target inside a `.papur` source — only the content within it.

## Acceptance Criteria

- [ ] One parser pass produces one AST that can be walked by any registered emitter.
- [ ] A block with no `@target` qualifier is emitted for every target.
- [ ] A block qualified `::: @web` is emitted only when the compile target is web.
- [ ] A block qualified `::: @pdf` is emitted only when the compile target is pdf.
- [ ] A block qualified `::: @email` is emitted only when the compile target is email.
- [ ] The `::: script` layer is silently omitted from pdf, email, and plain target output.
- [ ] Adding a new target emitter does not require changes to the parser or the AST.

## Open Questions

<!-- None recorded. -->
