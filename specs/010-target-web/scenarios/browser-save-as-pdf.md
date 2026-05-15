---
section: "Output Shape"
---

# Browser Save-as-PDF

## Context

A user views a papur-compiled web document in a browser and chooses the browser's print dialog, which they use to save the page as a PDF. The compiler is not involved at this point — the browser is consuming the web target's HTML and CSS.

This is distinct from [011-target-pdf](../../011-target-pdf/spec.md), where the compiler emits PDF bytes directly from the AST. That path produces a higher-fidelity, tagged, PDF/UA-conformant artifact and is the canonical way to produce a PDF from papur. The browser-print path is a no-cost side effect of shipping a working web target.

## Behavior

The web target emits CSS that produces a reasonable result when the user saves to PDF via the browser print dialog:

- `@pdf`-qualified rule blocks in `::: css` are emitted as `@media print { ... }` declarations in the web target's stylesheet. The browser applies them when the user invokes print or save-as-PDF.
- Page-oriented rules (`@page` size, margins, page breaks) authored via `@pdf` blocks are honored by the browser's print pipeline through the standard CSS print model.
- The web target makes no special accommodation beyond emitting the `@pdf` blocks as `@media print` — page breaking, font substitution, and structure are whatever the browser does.

The output is not Tagged PDF. The browser produces a glyphs-and-rectangles PDF without a structure tree. Authors who need an accessible PDF artifact use the pdf target ([011-target-pdf](../../011-target-pdf/spec.md)), not the browser-print path.

## Edge Cases

- **`::: script` interaction with print.** The web target ships JS. The browser's print pipeline runs against the post-script DOM, so author scripts that mutate the page can affect what saves to PDF. This is browser behavior, not papur behavior — papur does not attempt to freeze the DOM before print.
- **`::: @pdf` content blocks.** Target-qualified content blocks (`::: @pdf` ... `:::`) do **not** appear in the web target output and therefore do not appear when a user saves the web view as PDF. `::: @pdf` is a *target* qualifier, not a *medium* qualifier — it gates emission on the compile target, not on the browser's print medium. Authors who want a phrase to appear in the browser-printed PDF but not the on-screen web view should use a `@pdf` *rule block* in `::: css` (e.g., `display: none` outside print, visible in print), not a `::: @pdf` content block.

## Open Questions

- **Are `@pdf` rule blocks emitted as `@media print` on the web target, or only on the pdf target?** Treating `@pdf` rule blocks as `@media print` on web is what makes this scenario work; the alternative is that authors must hand-write `@media print` to control browser-print output. The first option is more ergonomic and is what this scenario assumes; needs explicit confirmation at clarify time.

## Resolved Questions

<!-- None yet. -->
