---
spec: 001-file-format
reviewed-at: 2026-06-28T22:10:41Z
reviewed-against: da70a6b17249f0c67c424bfa64259b47b6d10cd4
diff-base: 35361debe81a1c38f29bc2fe33bdf97690b65b3c
must-violations: 0
should-violations: 0
low-confidence: 1
captured-issues: 1
skipped-passes: []
---

# Review — 001-file-format

## Summary

Clean review, SHOULD findings addressed. The implementation is a small,
well-tested Rust block-segmentation library plus a thin CLI — no MUST violations
across all five passes, so the spec is not blocked from `done`. Stack: Rust
(backend/CLI surface); rule files loaded were `security-backend.md`,
`api-backend.md`, and `configuration-cross.md` (the three `-frontend` files were
not selected — papur has no frontend surface).

Security posture is sound for a local CLI compiler. **BE-INPUT-008**
(untrusted-input deserialization) is satisfied: frontmatter/meta YAML is parsed
with `yaml-rust2`, a data-only parser that cannot execute code as a side effect
of parsing. **BE-DEPS-002** (pinned dependencies) is satisfied by the committed
`Cargo.lock`. The web/auth/API rule families (all of `api-backend.md`,
BE-AUTHN/AUTHZ/API/DATA/LOG) have no surface in a CLI parser.
`configuration-cross.md` is clean: no operator-tunable values or env vars, and
the reserved-keyword set and `PAPUR-P` codes are each centralized in one module.

The two SHOULD advisories from the initial pass have been resolved: the CLI now
shares one `Arc<str>` of the source across all diagnostics (was: full clone per
finding), and the CI dependency-scanning/SBOM/provenance gap (BE-DEPS-001/003/004)
— project-infrastructure with no spec home — has been logged to `specs/inbox.md`
as a chore for the first CI/`system.md` spec. One low-confidence CRLF note
remains open (tracked below).

**Observation (not a rule finding):** `.govern.toml` `[project] languages` still
reads `["Go"]` — stale template state; the project is Rust. Worth correcting for
project hygiene (the review tech-stack check reads `AGENTS.md`, which correctly
says Rust, so this did not block).

## MUST violations (blocking)

None.

## SHOULD violations (advisory)

None outstanding.

- **Resolved — EFF (CLI source clone):** `crates/papur/src/main.rs` now builds a
  single `Arc<str>` of the source and shares it across all parse diagnostics
  instead of cloning the full string per finding.
- **Relocated — BE-DEPS-001/003/004 (CI dependency scanning / SBOM / provenance):**
  project-infrastructure with no spec home; logged to `specs/inbox.md` as a chore
  (see Captured issues) for the first CI/`system.md` spec.

## Low-confidence findings

### Low-confidence: QUAL — CRLF body capture retains a trailing carriage return

- **File**: `crates/papur-core/src/block/scanner.rs` (`scan`, body capture)
- **Confidence**: 70
- **Finding**: Layer-body and frontmatter capture strips a trailing `\n`
  (`strip_suffix('\n')`) but not a preceding `\r`, so under CRLF line endings the
  last body line keeps a trailing `\r`. The plan flagged CRLF as a refine-later
  edge case; fence-line matching already tolerates `\r`. Captured here so it is
  not lost.
- **Suggested fix**: Normalize trailing `\r` when trimming the body terminator,
  or add a CRLF fixture and decide the normalization contract explicitly.

## Waived findings

None.

## Captured issues (pending /papur:groom)

- **Rust CI: dependency scanning + SBOM + provenance** (BE-DEPS-001/003/004) —
  logged to `specs/inbox.md`; wire `cargo audit`/`cargo deny`, SBOM generation,
  and crate provenance into CI when the first CI/`system.md` spec is written.

## Skipped passes

None — all five passes ran.
