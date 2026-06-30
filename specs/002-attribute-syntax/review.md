---
spec: 002-attribute-syntax
reviewed-at: 2026-06-30T16:45:59Z
reviewed-against: 30ec393a590ba3210faa5dbd73f315732b4e0afa
diff-base: 2af8175b2820cd018839dffc426c87b670655c05
must-violations: 0
should-violations: 0
low-confidence: 0
captured-issues: 1
skipped-passes: []
---

# Review — 002-attribute-syntax

## Summary

Clean across all five passes: **0 MUST, 0 SHOULD, 0 low-confidence**, non-blocking.
This run re-reviews the `:::`-header grammar change committed in `30ec393`
(diff window `2af8175..HEAD`): `parse_fence_header` now parses the entire header
through `parse_attributes` as a unified attribute group — a bare word names the
element (`Attributes.element`), `.class` adds a class, `#id`/`key=value` apply —
and `FencedDiv` drops its `name` field for `attrs`. Element *resolution* stays
deferred to spec 003; 002 captures the bareword and stops. The change is a net
simplification (less bespoke parsing, one fewer node field) and introduces no
new security, efficiency, or correctness risk. 22 tests pass, `cargo clippy
--all-targets -- -D warnings` is clean, and the feature dir is
markdownlint-clean. The spec is unblocked from the review gate's side.

Rule files loaded: `api-backend.md`, `concurrency-backend.md`,
`configuration-cross.md`, `observability-backend.md`, `performance-backend.md`,
`quality-cross.md`, `reliability-backend.md`, `security-backend.md` — the
`backend` + `cross` selection per `.govern.toml` `[rules] surfaces = ["backend"]`,
no `[[review.disabled-rule-files]]`. The reviewed code remains a pure in-memory
parser with no network, database, filesystem, concurrency, logging, environment,
or HTTP-API surface, so the backend-service rule families select no patterns.

## MUST violations (blocking)

*None.*

## SHOULD violations (advisory)

*None.*

## Low-confidence findings

*None.*

## Waived findings

*None.*

## Captured issues (pending /papur:groom)

- **Markdownlint MD049 in `specs/001-file-format/review.md:69`** — the line uses
  `*N*` (asterisk emphasis) where that file's consistent style is underscore, so
  `npx markdownlint-cli2` fails on the repo-wide glob. A markdown-hygiene chore
  outside 002's feature dir; it did not block 002's gate. Appended to
  `specs/inbox.md` during the implement run that produced `30ec393`. Run
  `/papur:groom` to route it.

## Skipped passes

*None — all five passes ran.*

## Pass notes

- **Security** — The change adds no security surface: `element` capture is a
  `String` clone, and `parse_fence_header` does no I/O, regex, or recursion.
  Nothing in `security-backend.md` selects.
- **Reuse** — The change *removes* duplication: `parse_fence_header` previously
  hand-split the first token as a "name" and then parsed the remainder; it now
  routes the whole header through `parse_attributes`, the single attribute-group
  parser shared with headings and inline spans.
- **Quality** — Diagnostic offsets in `parse_fence_header` are correct (byte and
  column bases account for `:::` plus leading whitespace, matching the prior
  convention); the bare-word path is first-wins with no diagnostic; every
  `FencedDiv` consumer (`finish_frame`, `collect_ids`, tests) was updated; the
  insta snapshot regenerated to the dotted `::: .grid` form and was inspected
  (`element: None`, `roles: [grid]`, `cols` data attr → `<div class="grid"
  data-cols="3">`, nesting preserved as descendant structure). One **documented
  consequence, not a finding**: a bare word inside a *heading* `{…}` group (e.g.
  a forgotten dot, `## {hero}`) is now captured into the unused `element` field
  rather than emitting `PAPUR-P022`. This is intended under the unified grammar
  (`Attributes.element` is "meaningful only for the `:::` header"), and bare-word
  validity is owned by spec 003's element resolution — so the diagnostic moves to
  the 003 layer rather than being lost. No loaded rule covers it; recorded here
  for visibility.
- **Efficiency** — `parse_fence_header` is linear in header length; no new
  quadratic or unbounded path.
- **Simplicity** — Net simpler: one fewer `Node` field and the removal of the
  bespoke name/attr split. No premature abstraction or dead branch introduced.

## Note for the gate (informational)

The acceptance criterion *"A `:::` header parses as an attribute group…"* is now
implemented and covered by `ac12_fence_header_attribute_group`. With this review
clean against `30ec393`, the `/papur:implement` completion gate can verify the
acceptance criteria and propose the `in-progress → done` transition. Task 9
subtask 3 (which required this review re-run against the changed code) is now
satisfiable.
