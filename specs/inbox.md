# Inbox

Temporary inbox for known issues not yet assigned to a feature spec.
Items are migrated to their proper home as specs are written.

- [ ] **Rust CI: dependency scanning + SBOM + provenance** (BE-DEPS-001 / 003 / 004) — wire `cargo audit` (or `cargo deny`) for advisories, SBOM generation (CycloneDX/SPDX), and crate provenance verification into CI for the Rust workspace. Surfaced by 001's review as a project-infra advisory; a chore with no feature-spec home — fold into the first CI / `system.md` spec, then remove this entry.
- [ ] **Markdownlint MD049 in `specs/001-file-format/review.md:69`** — the line uses `*N*` (asterisk emphasis) while that file's consistent emphasis style is underscore, so `npx markdownlint-cli2` fails on the repo-wide glob. Markdown-hygiene chore, no feature home; fix the emphasis to `_N_` (or run a repo-wide lint sweep), then remove this entry. Surfaced incidentally during 002's `/papur:implement`; it is outside 002's feature dir, so it did not block 002's gate.
