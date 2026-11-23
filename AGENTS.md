# Project: papur

A markdown-flavored markup language that compiles to semantic, accessible HTML, CSS, and beyond.

> **Agents:** whether a rule belongs in this file is decided by the test in the constitution's [§drift-prevention](.ductus/constitution.md#drift-prevention); the block below says where any other learning goes.

<!-- /ductus maintains the region between the BEGIN and END markers below:
     where a learning goes, from the constitution's §drift-prevention. Edits
     inside it are overwritten; the rest of this file is preserved
     byte-for-byte. -->
<!-- BEGIN ductus:memory-routing -->
> **Where a learning goes** ([§drift-prevention](.ductus/constitution.md#drift-prevention)). Commit whatever would help a teammate, since an agent's per-user memory holds only personal facts and external pointers. Then file it by who it is true for: every project running the pipeline, the framework's constitution, which is upstream, so raise it with ductus rather than edit `.ductus/constitution.md`; your organization's projects, the shared constitution it registers; this project alone, this file only for what every session needs before acting, and otherwise where its work is read: architecture in `system.md`, `events.md` or `errors.md`, read before changing architecture, configuration or deployment; a feature's decision in its spec, read when the spec is worked; a procedure in `docs/` or a skill, read when the task comes up; a one-file trap in a comment or test there, read when the file is opened; how a lesson was learned in its commit.
<!-- END ductus:memory-routing -->

## Constitution

See [.ductus/constitution.md](.ductus/constitution.md) — guiding principles, development pipeline, spec lifecycle, and quality standards that govern this project.

<!-- /ductus maintains the region between the BEGIN and END markers below.
     Registered shared constitutions (.ductus/config.toml [constitutions.*])
     are listed there, one line each. Empty when none are registered. Edits
     inside it are overwritten; the rest of this file is preserved
     byte-for-byte. -->
<!-- BEGIN ductus:constitutions -->

<!-- END ductus:constitutions -->

## Tech Stack

papur is a CLI compiler shipped as a **single self-contained binary** — end
users install nothing else (no Node, no interpreter, no runtime).

| Layer | Technology | Role |
| --- | --- | --- |
| **Language** | Rust 1.95+ | Compiler implementation |
| **CLI** | `clap` | Argument parsing |
| **Diagnostics** | `miette` + `thiserror` | Source-highlighted compiler errors |
| **Markdown** | `markdown-rs` | CommonMark → mdast (downstream specs) |
| **CSS** | `lightningcss` | `::: css` layer parse/transform (downstream specs) |
| **Tests** | `cargo test` + `insta` | Unit + snapshot tests |

Workspace layout: `crates/papur-core` (library — parser, blocks, AST, emitters)
and `crates/papur` (the `papur` binary). `markdown-rs` and `lightningcss`
are recorded project-level choices, wired in by the specs that use them.

## Commands

- Build: `cargo build`
- Build (release): `cargo build --release`
- Run: `cargo run -p papur -- <file.papur>`
- Test: `cargo test`
- Lint: `cargo clippy --all-targets -- -D warnings`
- Format: `cargo fmt`
- Markdown lint: `npx markdownlint-cli2`

## Project Structure

- `.ductus/constitution.md` -- Principles, pipeline, quality standards
- `AGENTS.md` -- Agent rules: tech stack, conventions, workflow, gotchas, boundaries
- `CLAUDE.md` -- `@AGENTS.md` + Claude Code-specific configuration
- `specs/`
  - `system.md` -- Architecture, shared conventions
  - `events.md` -- Global event catalog
  - `errors.md` -- Error handling conventions and codes
  - `inbox.md` -- Temporary inbox for known issues not yet assigned to a spec
  - `templates/` -- Starter files for spec, plan, tasks, data-model, research, scenario, and spec-and-plan
  - `{NNN-feature-name}/`
    - `spec.md` -- Requirements, contracts, acceptance criteria (standard track)
    - `spec-and-plan.md` -- *(lightweight track)* Combined spec and plan for small, single-module features
    - `research.md` -- *(optional)* Background research, prior art, context
    - `plan.md` -- Implementation approach, technical decisions
    - `data-model.md` -- *(optional)* Generated during plan phase
    - `tasks.md` -- Discrete work items derived from the plan
    - `scenarios/` -- Bug fixes, edge cases, detailed behavior elaborations
      - `{slug}.md` -- Individual scenario created via the elaborate command

<!-- Add project-specific directories below (e.g., src/, cmd/, modules/) -->

## Skills

<!-- Optional. List skill files (Anthropic/Claude Code "skills" — context-loaded
     instruction packs) that augment AGENTS.md for specific task types. This is
     distinct from this repo's `framework/workflows/` — workflows are `govern`-
     scaffolded slash commands for routine ops (lint, test, format), while skills
     are an agent-platform feature for richer in-context instruction packs. Leave
     empty if you don't decompose into skills.

     Example (file paths are illustrative — adopters place skill files wherever
     their agent platform expects them):
     | Skill | Activates on |
     | --- | --- |
     | `code-review` skill | Code review on auth or session paths |
     | `db-migration` skill | Editing migration files |

     Per-platform mapping (Claude Code skills, Cursor rules, etc.) is the
     adopter's call — `govern` defines the index pattern, not the location.
-->

## Code Style

<!-- Define code patterns and conventions specific to your tech stack. -->

## Testing

<!-- Define testing conventions, test types, and file placement. -->

## Workflow

- This project uses **trunk-based development**: commit directly to `main`. Do not create feature branches or open PRs for pipeline work — integrate small, frequent commits straight to the trunk. (Overrides the agent default of branching off the default branch.)

<!-- Process rules — pre-action checks, required ordering, lookup steps. "Always do X
     before Y" style. Distinct from Boundaries (hard "never" limits) and Gotchas
     (technical/framework quirks).
     Example:

- Before working on any spec beyond its spec.md, verify all dependency specs have status `done`. If any dependency is not done, work on the earliest incomplete dependency instead.
- Read `framework/commands/{name}.md` before recommending or describing a slash command — don't guess from the name.
- Run `make lint` after edits in `pkg/` before marking a task complete.

-->

## Gotchas

<!-- Document things agents consistently get wrong — framework quirks, version-specific behavior,
     and non-obvious conventions that waste cycles. Focus on what's surprising, not what's standard.
     Example:

- pgx v5 uses `pgx.RowToStructByName`, not the older `Scan` pattern
- templ components must not import `net/http` — pass values through parameters
- `air` does not watch `.sql` files by default — add the pattern to `.air.toml`
- NATS JetStream consumer names must be globally unique, not just per-stream

-->

- The pre-commit hook runs `ductus derive-dependencies`, which regenerates each spec's frontmatter `dependencies:` from its **inline body links** to other specs (`[NNN-slug](../NNN-slug/spec.md)`) — the frontmatter is a derived index, so a hand edit to it is overwritten — and **rejects cycles** in the derived graph. Keep cross-references that point at a *downstream* spec (e.g. a foundational layer spec referencing a target spec that consumes it) under a `## See also` heading — links there are not treated as dependencies. An inline body link in the spec's prose or acceptance criteria becomes a hard edge and can induce a cycle that blocks every commit until moved.

## Boundaries

- Follow tasks.md literally — do not skip ahead to later pipeline phases. When tasks say to set status to `planned`, stop there. The user advances to the next phase explicitly.
- Third-party work must be under an MIT-equivalent permissive license: **MIT, BSD-2-Clause, BSD-3-Clause, ISC, or Apache-2.0**. Anything else — MPL, OFL, LGPL, GPL/AGPL, CC variants, proprietary, "no-license-stated" — is rejected. Apache-2.0 brings a `NOTICE` propagation obligation; honor it via `THIRD_PARTY_NOTICES`. Verify the LICENSE file before drawing on a project.
- Third-party work must be attributed when drawn on — whether by porting code, transcribing an algorithm, or adopting a substantial documented design idea. The MIT-style legal floor only requires this for copied code; this project's floor is broader (any use, not just copyright-triggering use). Attribute in `research.md` for design-level borrowing, and in source comments plus a `THIRD_PARTY_NOTICES` file when code is ported.

<!-- Define additional project-specific boundaries. Common patterns:

- Never implement without a spec — follow the pipeline: spec → plan → tasks → implement
- Never commit secrets or .env files
- Never modify CI/CD config without asking
- Ask before adding new dependencies
- Ask before changing database schema

-->
