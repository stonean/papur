# Inbox

Temporary inbox for known issues not yet assigned to a feature spec.
Items are migrated to their proper home as specs are written.

<!-- Rules:
     - Do not frontfill bugs that are not being actively worked on.
     - Write specs for areas being actively touched — let adoption spread naturally.
     - As specs are written, items migrate from here into spec updates or new scenarios.
     - The goal is for this file to eventually be empty and deleted.

     Format each item as a checkbox list entry with a brief description and any relevant
     context. Audit findings written by /govern follow a stricter form (see
     specs/008-security-rules/spec.md): `- [ ] {Rule ID}: {artifact} does not address — {summary}`.
     Manual entries can use the simpler form below. When an item is migrated, remove it from
     this list. -->

- [ ] **Rust CI: dependency scanning + SBOM + provenance** (BE-DEPS-001 / 003 / 004) — wire `cargo audit` (or `cargo deny`) for advisories, SBOM generation (CycloneDX/SPDX), and crate provenance verification into CI for the Rust workspace. Surfaced by 001's review as a project-infra advisory; a chore with no feature-spec home — fold into the first CI / `system.md` spec, then remove this entry.
- [ ] **Markdownlint MD049 in `specs/001-file-format/review.md:69`** — the line uses `*N*` (asterisk emphasis) while that file's consistent emphasis style is underscore, so `npx markdownlint-cli2` fails on the repo-wide glob. Markdown-hygiene chore, no feature home; fix the emphasis to `_N_` (or run a repo-wide lint sweep), then remove this entry. Surfaced incidentally during 002's `/papur:implement`; it is outside 002's feature dir, so it did not block 002's gate.
