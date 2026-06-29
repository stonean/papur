# Errors

papur is a command-line compiler, so an "error" is a **diagnostic**: a stable
code, a human-readable message, and a source location. Diagnostics are rendered
with `miette` (source-highlighted). This document defines the diagnostic format
and the registry of codes in use; each spec that introduces a code appends it
here.

## Diagnostic format

Every diagnostic carries:

- **code** — a stable `PAPUR-` identifier (see naming below).
- **message** — one human-readable sentence describing the problem.
- **span** — a source location: a byte offset plus a 1-based line and column,
  rendered as `file:line:col` with the offending text underlined.

A strict-mode parse failure looks like this:

```text
Error:
  × PAPUR-P001: unterminated `::: css` fence
   ╭─[page.papur:3:1]
 3 │ ::: css
   · ───┬───
   ·    ╰── here
   ╰────
```

## Code naming

Codes are `PAPUR-<Letter><NNN>`: the letter groups by compiler phase and `NNN`
is a zero-padded number. Codes are **permanent** — once assigned, a code is
never renumbered or reused, even if the diagnostic moves or is reworded (the
same stability rule that rule IDs follow). Numbers are grouped into ranges by
concern, so gaps in the sequence are intentional.

| Letter | Phase |
| --- | --- |
| `P` | Parse (block segmentation, frontmatter, …) |

Parse (`P`) ranges:

| Range | Concern |
| --- | --- |
| `P001`–`P009` | Fence / block segmentation |
| `P010`–`P019` | Frontmatter |
| `P020`–`P029` | Attribute groups / roles |

## Registry

| Code | Severity | Meaning | Introduced by |
| --- | --- | --- | --- |
| `PAPUR-P001` | error | A reserved layer fence was opened but never closed. | 001-file-format |
| `PAPUR-P002` | error | An unbalanced or dangling `:::` content fence. | 002-attribute-syntax |
| `PAPUR-P010` | error | Leading YAML frontmatter could not be parsed. | 001-file-format |
| `PAPUR-P020` | error | The same `id` is used on more than one element in a file. | 002-attribute-syntax |
| `PAPUR-P021` | error | More than one `#id` in a single attribute group. | 002-attribute-syntax |
| `PAPUR-P022` | error | A malformed attribute token (e.g. `{=value}`). | 002-attribute-syntax |
| `PAPUR-P023` | error | A forced namespace prefix (`g.`/`l.`) resolved to no definition. | 002-attribute-syntax |

## Strict vs lenient mode

papur defaults to **strict** mode: diagnostics are hard errors and the compiler
exits non-zero. In **lenient** mode (`--lenient`), recoverable parse problems —
typed content that escaped its fence, malformed frontmatter — degrade to literal
content instead of erroring. The mode is a parse-time input; see
[001-file-format](001-file-format/spec.md).

Detecting an unbalanced or dangling `:::` *content* fence is not a 001 concern
(block segmentation leaves content fences opaque); that diagnostic
(`PAPUR-P002`) is owned by [002-attribute-syntax](002-attribute-syntax/spec.md),
whose fenced-div parser tracks fence depth.

## Exit codes

| Code | Meaning |
| --- | --- |
| `0` | Success. |
| non-zero | One or more diagnostics (strict mode), or an I/O or usage error. |
