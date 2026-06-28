---
spec: 001-file-format
reviewed-at: 2026-06-28T23:18:34Z
reviewed-against: f8fa684e4a6fb73f697e6b1a2c7798ffafaa2693
diff-base: 35361debe81a1c38f29bc2fe33bdf97690b65b3c
must-violations: 0
should-violations: 0
low-confidence: 0
captured-issues: 1
skipped-passes: []
---

# Review — 001-file-format

## Summary

Clean review, all findings addressed. The implementation is a small, well-tested
Rust block-segmentation library plus a thin CLI — no MUST violations across all
five passes. Stack: Rust (backend/CLI surface); rule files loaded were
`security-backend.md`, `api-backend.md`, and `configuration-cross.md` (the three
`-frontend` files were not selected — papur has no frontend surface).

Security posture is sound for a local CLI compiler. **BE-INPUT-008**
(untrusted-input deserialization) is satisfied: frontmatter/meta YAML is parsed
with `yaml-rust2`, a data-only parser that cannot execute code as a side effect
of parsing. **BE-DEPS-002** (pinned dependencies) is satisfied by the committed
`Cargo.lock`. The web/auth/API rule families have no surface in a CLI parser.
`configuration-cross.md` is clean: no operator-tunable values or env vars, and
the reserved-keyword set and `PAPUR-P` codes are each centralized in one module.

All findings from the initial pass are now resolved:

- **EFF (CLI source clone)** — fixed: the CLI shares one `Arc<str>` of the source
  across all diagnostics.
- **CRLF body normalization** (was low-confidence) — fixed via the
  `crlf-line-endings` scenario: `normalize_body()` converts `\r\n` to `\n` and
  drops the trailing terminator, so CRLF and LF sources yield identical
  layer/frontmatter bodies (two regression tests).
- **BE-DEPS-001/003/004 (CI dependency scanning / SBOM / provenance)** —
  project-infrastructure with no spec home; logged to `specs/inbox.md` as a chore
  (see Captured issues).

## MUST violations (blocking)

None.

## SHOULD violations (advisory)

None outstanding (EFF source-clone resolved; BE-DEPS relocated to the inbox).

## Low-confidence findings

None outstanding — the CRLF body-normalization note was resolved via the
`crlf-line-endings` scenario.

## Waived findings

None.

## Captured issues (pending /papur:groom)

- **Rust CI: dependency scanning + SBOM + provenance** (BE-DEPS-001/003/004) —
  logged to `specs/inbox.md`; wire `cargo audit`/`cargo deny`, SBOM generation,
  and crate provenance into CI when the first CI/`system.md` spec is written.

## Skipped passes

None — all five passes ran.
