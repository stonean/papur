---
section: "Fences Are Always Required"
---

# Crlf-line-endings

## Context

A `.papur` source authored or saved with Windows-style CRLF (`\r\n`) line endings — common from Windows editors or tools that rewrite end-of-line markers. The same document may exist with LF (`\n`) endings elsewhere; both must compile identically.

## Behavior

Segmentation of a CRLF source produces a `BlockStream` equivalent to its LF twin. Layer block bodies (`::: css` / `::: script` / `::: theme` / `::: meta`) and normalized frontmatter are captured with LF line endings: the scanner strips the trailing `\r` from each captured body line so a CRLF file and its LF equivalent yield byte-identical block bodies and identical `merged_meta` / `merged_theme` results. Fence recognition (reserved openers, the bare `:::` close, and `---` frontmatter delimiters) already tolerates a trailing `\r`, so block boundaries are detected identically under either line ending.

## Edge Cases

- Content blocks are captured verbatim (CRLF preserved); only typed layer and frontmatter bodies are normalized, since downstream Markdown parsing of content normalizes its own whitespace.
- Diagnostic spans (`start_byte` / `end_byte`) remain offsets into the original (CRLF) source so `miette` highlights the correct bytes; normalization affects only the owned body `String`, not span arithmetic.
- A lone `\r` (classic Mac CR-only line ending) is out of scope — only `\r\n` pairs are normalized.

## Open Questions

*None — captured during scenario authoring.*

## Resolved Questions

*None yet.*
