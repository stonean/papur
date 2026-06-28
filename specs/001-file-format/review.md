---
spec: 001-file-format
reviewed-at: 2026-06-28T19:10:58Z
reviewed-against: d00bc2682d5f5496288a954388b237aafebc8e8f
diff-base: 35361debe81a1c38f29bc2fe33bdf97690b65b3c
must-violations: 0
should-violations: 2
low-confidence: 1
captured-issues: 0
skipped-passes: []
---

# Review — 001-file-format

## Summary

Clean review. The implementation is a small, well-tested Rust block-segmentation
library plus a thin CLI — no MUST violations across all five passes, so the spec
is not blocked from `done` on review grounds. Stack: Rust (backend/CLI surface);
rule files loaded were `security-backend.md`, `api-backend.md`, and
`configuration-cross.md` (the three `-frontend` files were not selected — papur
has no frontend surface).

Security posture is sound for a local CLI compiler. The one genuinely relevant
security rule, **BE-INPUT-008** (untrusted-input deserialization), is satisfied:
frontmatter/meta YAML is parsed with `yaml-rust2`, a data-only parser that cannot
execute code as a side effect of parsing — the Rust ecosystem's safe loader, with
no unsafe load mode to opt out of. **BE-DEPS-002** (pinned dependencies) is
satisfied by the committed `Cargo.lock` (exact versions + integrity checksums).
The web/auth/API rule families (BE-AUTHN/AUTHZ/API/DATA/LOG, all of
`api-backend.md`) have no surface in a CLI parser. `configuration-cross.md` is
clean: no operator-tunable values or env vars, and the reserved-keyword set and
`PAPUR-P` codes are each centralized in one module.

**Observation (not a rule finding):** `.govern.toml` `[project] languages` still
reads `["Go"]` — stale template state; the project is Rust. Worth correcting for
project hygiene (the review tech-stack check reads `AGENTS.md`, which correctly
says Rust, so this did not block).

## MUST violations (blocking)

None.

## SHOULD violations (advisory)

### SHOULD: EFF — CLI clones the full source once per diagnostic

- **File**: `crates/papur/src/main.rs:88-97`
- **Rule**: Efficiency pass — avoid repeated work over inputs.
- **Finding**: When a parse fails, the error arm builds one `ParseDiagnostic`
  per diagnostic, each calling `NamedSource::new(name.as_str(), source.clone())`
  — cloning the entire source string per diagnostic. Diagnostics are few and
  this is an error path, so impact is negligible, but a single shared source
  (e.g. `Arc<str>` / one `NamedSource`) would avoid the repeated clone.
- **Auto-fixable**: no
- **Suggested fix**: Construct the source code once and share it across the
  related diagnostics, or attach `#[source_code]` to the outer `ParseFailed`.

### SHOULD: BE-DEPS — no dependency vulnerability scanning / SBOM / provenance yet

- **File**: project-level (CI configuration absent)
- **Rule**: BE-DEPS-001 (scan dependencies for known vulnerabilities on every CI
  run), BE-DEPS-003 (SBOM), BE-DEPS-004 (signature/provenance verification).
- **Finding**: The Rust toolchain has no `cargo audit` / Dependabot, SBOM
  generation, or provenance check wired into CI. These rules' verification
  targets CI/deployment specs, which 001 does not introduce, so this is a
  project-infrastructure advisory rather than a defect in 001's code — fold it
  into the first CI/`system.md` spec.
- **Auto-fixable**: no
- **Suggested fix**: Add a `cargo audit` (or `cargo deny`) step and an SBOM
  generator when CI is established for the Rust workspace.

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

None — no issues were appended to `specs/inbox.md` during this work window.

## Skipped passes

None — all five passes ran.
