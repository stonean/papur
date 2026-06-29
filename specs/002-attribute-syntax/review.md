---
spec: 002-attribute-syntax
reviewed-at: 2026-06-29T02:33:10Z
reviewed-against: 7dfc54d28eb1a69c8f0d3a6f23cdb565b8ca5f27
diff-base: 06172ac1ccd2b81c448fb97f6a76f915ef554a61
must-violations: 0
should-violations: 3
low-confidence: 0
captured-issues: 0
skipped-passes: []
---

# Review — 002-attribute-syntax

## Summary

Clean review: **0 MUST violations**, 3 SHOULD (advisory) findings, 0
low-confidence findings. The implementation is a self-contained parser across
the new `attr`, `role`, and `structure` modules in `papur-core`, with no
network, I/O, auth, or persistence surface — so the web-oriented rule files
(`security-backend`, `api-backend`, `security-frontend`, `accessibility-frontend`,
`performance-frontend`) select no applicable patterns. The only cross-cutting
rule file, `configuration-cross`, produces no violations: the numeric literals
in the parser (`3` colons, `6` heading levels) are intrinsic grammar constants,
not operator-tunable values, and the feature introduces no environment
variables. The three advisory findings are minor readability/robustness
improvements, none blocking. **Not blocking** — the spec may advance to `done`.

Rule files loaded: `configuration-cross.md` (cross-cutting). The backend/frontend
rule files were not selected: the reviewed code is a Rust CLI/library parser,
neither a web backend nor a web frontend surface.

## MUST violations (blocking)

_None._

## SHOULD violations (advisory)

### SHOULD: simplicity — grammar literals could be named constants

- **File**: `crates/papur-core/src/structure/mod.rs` (fence-marker length `3`; heading-level bound `6`)
- **Rule**: Simplicity pass (configuration would be a constant / readability). Not a `CFG-CONST-003` violation — these are intrinsic Markdown grammar constants, not operator-tunable values, so the rule explicitly excludes them.
- **Finding**: The `:::`-length offset (`&line.text[leading_ws + 3..]`) and the max heading level (`hashes == 0 || hashes > 6`) appear as bare literals. Naming them (e.g. `FENCE_MARKER_LEN = 3`, `MAX_HEADING_LEVEL = 6`) would make the grammar limits self-documenting.
- **Auto-fixable**: yes
- **Suggested fix**: Introduce module-level `const FENCE_MARKER_LEN: usize = 3;` and `const MAX_HEADING_LEVEL: usize = 6;` and reference them.

### SHOULD: efficiency — `collect_ids` recurses on nesting depth

- **File**: `crates/papur-core/src/structure/mod.rs` (`collect_ids`)
- **Rule**: Efficiency pass (unbounded recursion over input-controlled structure). Not promoted to MUST: no loaded rule covers parser DoS, and papur compiles local source files rather than network-untrusted input, so the risk is low.
- **Finding**: `collect_ids` walks the tree recursively, so recursion depth equals the document's maximum nesting depth. A pathologically deep document (thousands of nested fences/sections) could overflow the stack. `parse_structure` itself is already iterative (explicit frame stack), so only this post-pass is affected.
- **Auto-fixable**: no
- **Suggested fix**: Convert `collect_ids` to an explicit work-stack walk, or cap nesting depth during parsing with a diagnostic. Defer until an input-depth bound is specified.

### SHOULD: efficiency — `fence_depth` rescans the stack per construct

- **File**: `crates/papur-core/src/structure/mod.rs` (`fence_depth`)
- **Rule**: Efficiency pass (repeated work).
- **Finding**: `fence_depth` recomputes the open-fence count by scanning the whole frame stack on every fence opener and every heading. For normal documents this is negligible, but it is O(depth) per construct where a maintained counter would be O(1).
- **Auto-fixable**: no
- **Suggested fix**: Track an `open_fences: u32` counter incremented/decremented as fence frames are pushed/popped. Behavior-preserving but not purely mechanical, so left for a manual pass.

## Low-confidence findings

_None._

## Waived findings

_None._

## Captured issues (pending /papur:groom)

_None — no issues were appended to `specs/inbox.md` during this work window._

## Skipped passes

_None — all five passes ran._
