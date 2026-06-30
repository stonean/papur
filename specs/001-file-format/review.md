---
spec: 001-file-format
reviewed-at: 2026-06-30T00:33:17Z
reviewed-against: 6bf77a5057fd45f26ab4d5325c4cfa65dee5b986
diff-base: 35361debe81a1c38f29bc2fe33bdf97690b65b3c
must-violations: 0
should-violations: 0
low-confidence: 0
captured-issues: 1
skipped-passes: []
---

# Review — 001-file-format

## Summary

Re-review against the current rule set, which expanded since the prior pass:
`6bf77a5` set `[rules] surfaces = ["backend"]` and added `concurrency-backend`,
`observability-backend`, `performance-backend`, `reliability-backend`, and
`quality-cross`. **0 MUST violations, 0 SHOULD.** The pass surfaced one advisory
efficiency finding (a quadratic scan on repeated unterminated reserved fences);
it has since been fixed in the working tree (see [Resolved advisories](#resolved-advisories)).
All 88 workspace tests pass and `cargo clippy --all-targets -D warnings` is
clean. **Not blocking** — the spec stays eligible for `done`.

Rule files loaded: `api-backend.md`, `concurrency-backend.md`,
`configuration-cross.md`, `observability-backend.md`, `performance-backend.md`,
`quality-cross.md`, `reliability-backend.md`, `security-backend.md`. papur is a
single-binary CLI compiler — no network, HTTP, database, auth, persistence,
secrets, env vars, concurrency, or service surface — so the server-oriented
families select no applicable patterns:

- **security-backend** — AUTHN/AUTHZ/API/DATA/LOG/ERR have no surface in a local
  compiler. `BE-INPUT-008` (untrusted-input deserialization) is satisfied:
  frontmatter/meta YAML is parsed with `yaml-rust2`, a data-only loader that
  cannot execute code as a side effect of parsing. `BE-INPUT-004` (path
  traversal) does not apply: the file path is the operator's own CLI argument,
  not input crossing a privilege boundary, and there is no base-directory
  confinement model for a compiler. `BE-DEPS-002` (pinned dependencies) is
  satisfied by the committed `Cargo.lock`. `BE-DEPS-001/003/004` (CI scanning /
  SBOM / provenance) are project-infrastructure with no spec home — already
  parked in `specs/inbox.md` (see Captured issues), not counted here.
- **configuration-cross** — clean. No env vars and no operator-tunable values
  (the `--lenient` flag is a parse mode, not a tunable). The reserved
  layer-keyword set and the `PAPUR-P` codes are each centralized in one module
  (`CFG-CONST-002`, module-local — correct placement).
- **concurrency / observability / performance / reliability-backend** — these
  verify design-time commitments for services (shared state, metrics, queries,
  pools, timeouts, shutdown). A one-shot CLI that reads a file and exits has no
  such surface; nothing selected.
- **quality-cross** `QUAL-STUB-001` — no silent stubs; every code path that
  implies work performs it or fails loudly.

## MUST violations (blocking)

_None._

## SHOULD violations (advisory)

_None outstanding — the efficiency finding below was fixed in the working tree._

## Resolved advisories

### EFF — quadratic rescan on repeated unterminated reserved fences (fixed)

- **File**: `crates/papur-core/src/block/scanner.rs`
- **Was**: When a reserved opener (`::: css`, `::: meta`, …) had no closing
  `:::`, the forward scan ran to EOF and `i` advanced by **one**, so a file of
  *N* consecutive unterminated reserved openers rescanned `[i, n)` for each `i` —
  **O(N²)**. Empirically: `--lenient` on `N` copies of `::: css` timed
  0.66s / 2.05s / 8.18s at N = 20k / 40k / 80k.
- **Fix**: Added a `closes_exhausted` latch — once a forward scan reaches EOF
  without a close, no later line can hold one (each subsequent opener searches a
  suffix of the same range), so later openers skip the rescan. Behavior is
  preserved exactly: strict mode still emits one `PAPUR-P001` per unterminated
  opener (locked by the new `many_unterminated_fences_each_report_once` test).
  The scan is now linear — N = 160k is instant where N = 80k previously took
  8.18s. 88 tests pass; clippy clean.

## Low-confidence findings

_None._

## Waived findings

_None._

## Captured issues (pending /papur:groom)

- **Rust CI: dependency scanning + SBOM + provenance** (BE-DEPS-001 / 003 / 004)
  — appended to `specs/inbox.md` during this work window: wire `cargo audit` /
  `cargo deny`, SBOM generation (CycloneDX/SPDX), and crate provenance into CI
  when the first CI / `system.md` spec is written. Informational; does not affect
  blocking or exit code.

## Skipped passes

_None — all five passes ran._
