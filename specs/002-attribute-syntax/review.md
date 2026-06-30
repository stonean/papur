---
spec: 002-attribute-syntax
reviewed-at: 2026-06-30T00:33:17Z
reviewed-against: 6bf77a5057fd45f26ab4d5325c4cfa65dee5b986
diff-base: 06172ac1ccd2b81c448fb97f6a76f915ef554a61
must-violations: 0
should-violations: 0
low-confidence: 0
captured-issues: 0
skipped-passes: []
---

# Review — 002-attribute-syntax

## Summary

Re-review against the current rule set, which expanded since the prior pass:
`6bf77a5` set `[rules] surfaces = ["backend"]` and added `concurrency-backend`,
`observability-backend`, `performance-backend`, `reliability-backend`, and
`quality-cross`. **0 MUST violations, 0 SHOULD.** The pass surfaced one advisory
efficiency finding (a quadratic inline-span scan over a prose run); it has since
been fixed in the working tree (see [Resolved advisories](#resolved-advisories)).
All 88 workspace tests pass and `cargo clippy --all-targets -D warnings` is
clean. **Not blocking** — the spec stays eligible for `done`.

Rule files loaded: `api-backend.md`, `concurrency-backend.md`,
`configuration-cross.md`, `observability-backend.md`, `performance-backend.md`,
`quality-cross.md`, `reliability-backend.md`, `security-backend.md`. The reviewed
code is a self-contained parser (brace-group grammar, role resolution, and the
content-fence/heading scope tree) with no network, I/O, auth, persistence, or
concurrency surface, so the server-oriented families select no patterns:

- **security-backend** — no AUTHN/AUTHZ/API/DATA/LOG/ERR surface. No
  deserialization beyond the data-only YAML already covered by 001. No reachable
  panic from input: the five production `.expect()` calls in `structure/mod.rs`
  are each guarded by a stack/`fence_count` invariant; every other `unwrap`/
  `panic!` is test-only.
- **configuration-cross** — clean. The `HTML_ATTRIBUTES` allowlist and the
  `FENCE_MARKER` / `FENCE_MARKER_LEN` / `MAX_HEADING_LEVEL` grammar constants are
  module-local (`CFG-CONST-002`). The `:::` marker also appearing in 001's block
  scanner was considered under `CFG-CONST-001` and is **not** a violation: that
  rule targets operator-tunable cross-module *defaults* that drift, whereas
  `:::` is a fixed grammar sigil, and the block-segmentation (001) and
  content-fence (002) parsers are deliberately decoupled per the spec boundary —
  each correctly owns its own module-local marker.
- **concurrency / observability / performance / reliability-backend** — no
  shared state, metrics, queries, pools, timeouts, or service lifecycle; nothing
  selected.
- **quality-cross** `QUAL-STUB-001` — no silent stubs. The `RoleRegistry` trait
  is an interface whose population is downstream (theming / CSS layer), explicitly
  documented as such; `resolve()` is fully implemented and fails loudly
  (`PAPUR-P023`) on a forced-prefix miss in strict mode.

## MUST violations (blocking)

_None._

## SHOULD violations (advisory)

_None outstanding — the efficiency finding below was fixed in the working tree._

## Resolved advisories

### EFF — quadratic inline-span scan over a prose run (fixed)

- **File**: `crates/papur-core/src/structure/mod.rs` (`split_inline` /
  `try_inline`)
- **Was**: For each `[` in a prose run, `try_inline` called `after.find(']')`,
  scanning to the end of the run when no `]` followed; a single prose run of *M*
  unmatched `[` characters was therefore **O(M²)**. Reachable through the library
  API (`parse_structure` / `parse_document`), which the planned playground will
  drive with untrusted input.
- **Fix**: `try_inline` now returns a three-way `InlineMatch` (`Span` /
  `NoClose` / `NoMatch{resume}`). On a non-match the splitter resumes just past
  the examined `]` — every `[` before it shares that `]` and would fail
  identically — and a `[` with no following `]` ends the search. The `]`-search
  never revisits a byte, so the splitter is linear. Segmentation is unchanged
  (locked by the new `bracket_runs_stay_prose_and_later_span_still_matches`
  test). 88 tests pass; clippy clean.

## Low-confidence findings

_None._

## Waived findings

_None._

## Captured issues (pending /papur:groom)

_None — no issues were appended to `specs/inbox.md` during this work window._

## Skipped passes

_None — all five passes ran._
