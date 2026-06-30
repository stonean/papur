---
spec: 002-attribute-syntax
reviewed-at: 2026-06-30T15:47:26Z
reviewed-against: 2af8175b2820cd018839dffc426c87b670655c05
diff-base: 6bf77a5057fd45f26ab4d5325c4cfa65dee5b986
must-violations: 0
should-violations: 0
low-confidence: 0
captured-issues: 0
skipped-passes: []
---

# Review — 002-attribute-syntax

## Summary

Clean across all five passes: **0 MUST, 0 SHOULD, 0 low-confidence**, non-blocking.
The reviewed scope is the role/scope parser in `papur-core` (`attr`, `structure`,
`role`, `diagnostic`, the re-exports in `lib.rs`, and the acceptance tests) — a
pure, in-memory parsing library with no network, database, filesystem,
concurrency, logging, environment, or HTTP-API surface. The quadratic
inline-span finding the prior pass surfaced is now committed (`2af8175`), so this
run reproduces the clean result; no code changed in the review window
(`6bf77a5..HEAD` is that single fix), consistent with the idempotency contract.
The spec stays eligible for `done` from the review gate's side.

Rule files loaded: `api-backend.md`, `concurrency-backend.md`,
`configuration-cross.md`, `observability-backend.md`, `performance-backend.md`,
`quality-cross.md`, `reliability-backend.md`, `security-backend.md` — the
`backend` + `cross` selection per `.govern.toml` `[rules] surfaces = ["backend"]`,
with no `[[review.disabled-rule-files]]` entries. The server-oriented families
select no patterns because the module introduces none of their gated surfaces:

- **security-backend** — no AUTHN/AUTHZ/API/DATA/LOG/ERR/DEPS surface; no
  SQL/shell/template construction; no user-controlled filesystem paths; no
  untrusted-binary deserialization (the only YAML is the frontmatter handled by
  001, data-only). No reachable panic from input: the production `.expect()`
  calls in `structure/mod.rs` are each guarded by a stack / `fence_count`
  invariant ("stack always has the root frame", "a fence is open" only after
  `fence_count > 0`, etc.); every other `unwrap`/`panic!` is test-only.
  `BE-INPUT-006` (resource exhaustion / ReDoS) does not fire — the scanners use
  no regex, and `collect_ids` (`structure/mod.rs:179`) walks the tree with an
  explicit work stack precisely to avoid call-stack overflow on deep input.
- **configuration-cross** — clean. The `HTML_ATTRIBUTES` allowlist and the
  `FENCE_MARKER` / `FENCE_MARKER_LEN` / `MAX_HEADING_LEVEL` grammar constants are
  named and module-local (`CFG-CONST-002`); `HTML_ATTRIBUTES` is the centralized
  single source of truth for the verbatim/`data-` boundary the plan commits to.
  The `:::` marker also appearing in 001's block scanner is **not** a
  `CFG-CONST-001` violation: that rule targets operator-tunable cross-module
  *defaults* that drift, whereas `:::` is a fixed grammar sigil, and the
  block-segmentation (001) and content-fence (002) parsers are deliberately
  decoupled per the spec boundary — each correctly owns its module-local marker.
- **concurrency / observability / performance / reliability-backend** — no shared
  state, metrics, queries, pools, timeouts, retries, or service lifecycle;
  nothing selected.
- **quality-cross** `QUAL-STUB-001` — no silent stubs. `RoleRegistry` is an
  interface whose population is documented downstream work (theming / CSS layer);
  `resolve()` is fully implemented and fails loudly (`PAPUR-P023`) on a
  forced-prefix miss in strict mode rather than passing through.

## MUST violations (blocking)

*None.*

## SHOULD violations (advisory)

*None.*

## Low-confidence findings

*None.*

## Waived findings

*None.*

## Captured issues (pending /papur:groom)

*None — `specs/inbox.md` had no additions in the review window (`6bf77a5..HEAD`).*

## Skipped passes

*None — all five passes ran.*

## Note for the gate (informational, not a finding)

Acceptance criterion 13/14 — *"A `:::` header parses as an attribute group: a
bare word names the element, a `.class` adds a class…"* — is unchecked, and
`parse_fence_header` (`structure/mod.rs:502`) still takes the first `:::` token
as the literal `name` rather than dot-prefix → class / bare-word → element. This
is the open work that reopened the spec from `done` to `in-progress`, not a rule
violation: element resolution is owned by spec 003, and `QUAL-STUB-001` does not
apply because the path does real parsing work rather than silently passing
through. Completing that AC is a `/papur:implement` concern; the review gate is
clean for the code as it stands.
