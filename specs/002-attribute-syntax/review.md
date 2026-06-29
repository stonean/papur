---
spec: 002-attribute-syntax
reviewed-at: 2026-06-29T02:42:39Z
reviewed-against: 5fecc8b528bd6c0d334817776e2c75a94ef55af7
diff-base: 06172ac1ccd2b81c448fb97f6a76f915ef554a61
must-violations: 0
should-violations: 0
low-confidence: 0
captured-issues: 0
skipped-passes: []
---

# Review — 002-attribute-syntax

## Summary

Clean review: **0 MUST violations, 0 SHOULD violations**, 0 low-confidence
findings. The implementation is a self-contained parser across the new `attr`,
`role`, and `structure` modules in `papur-core`, with no network, I/O, auth, or
persistence surface — so the web-oriented rule files (`security-backend`,
`api-backend`, `security-frontend`, `accessibility-frontend`,
`performance-frontend`) select no applicable patterns. The only cross-cutting
rule file, `configuration-cross`, produces no violations: the feature introduces
no operator-tunable values or environment variables. The three advisory findings
from the prior run have been applied (see [Resolved advisories](#resolved-advisories)).
**Not blocking** — the spec may advance to `done`.

Rule files loaded: `configuration-cross.md` (cross-cutting). The backend/frontend
rule files were not selected: the reviewed code is a Rust CLI/library parser,
neither a web backend nor a web frontend surface.

## MUST violations (blocking)

_None._

## SHOULD violations (advisory)

_None._

## Resolved advisories

The three SHOULD findings from the initial review pass were applied in commit
`5fecc8b` (behavior-preserving — the nesting-example snapshot and all tests are
unchanged):

1. **Simplicity** — grammar literals named: `FENCE_MARKER` / `FENCE_MARKER_LEN`
   (the `:::` marker) and `MAX_HEADING_LEVEL` (`6`) in
   `crates/papur-core/src/structure/mod.rs`.
2. **Efficiency** — `collect_ids` converted from recursion to an explicit
   work-stack walk, so a pathologically deep document cannot overflow the call
   stack.
3. **Efficiency** — fence depth tracked with an O(1) counter threaded through the
   builder functions, replacing the per-construct frame-stack rescan (which also
   removed a second stack scan in `close_fence`).

## Low-confidence findings

_None._

## Waived findings

_None._

## Captured issues (pending /papur:groom)

_None — no issues were appended to `specs/inbox.md` during this work window._

## Skipped passes

_None — all five passes ran._
