# 001 — File Format Tasks

Tasks derived from the [plan](plan.md). Complete in order.

## 1. Scaffold the Rust workspace

- [x] Create the workspace `Cargo.toml` with members `papur-core` and `papur`
- [x] Create `crates/papur-core` (lib) and `crates/papur` (bin) with minimal `lib.rs` / `main.rs` that build
- [x] Add `/target` to `.gitignore`
- [x] Record the stack (Rust, single binary) plus build/test/run commands in `AGENTS.md` (Tech Stack, Commands)
- [x] Done when `cargo build` succeeds on the empty workspace

## 2. Define the block-segmentation data model

- [x] Add `Span`, `ParseMode`, `LayerKind`, `Block`, `BlockStream` in `papur-core` per [data-model.md](data-model.md)
- [x] Add `Diagnostic` and `DiagnosticCode` per data-model.md
- [x] Add dependency crates (`clap`, YAML crate, `miette`, `thiserror`, `indexmap`; `insta` dev); verify each license is permissive (MIT/BSD/ISC/Apache-2.0) per Boundaries and record any Apache `NOTICE` in `THIRD_PARTY_NOTICES`
- [x] Done when the types compile and are constructible in a unit test

## 3. Implement the line scanner

- [x] Recognize reserved layer openers (`meta`/`theme`/`css`/`script`/`html`) at column 0; capture the raw body until the closing `:::`
- [x] Accumulate everything else as `Content` spans, leaving content fences (`::: grid`, `::: @web`) and their markers untouched
- [x] Compute a correct `Span` (line/col/byte) for every block
- [x] Done when `segment()` returns the correct ordered `Block` list for mixed-content fixtures

## 4. Implement strict / lenient mode

- [ ] Strict: unterminated reserved fence → `PAPUR-P001`
- [ ] Lenient: the same inputs degrade to `Content` prose with no error (AC3)
- [ ] `ParseMode` defaults to Strict
- [ ] Done when both modes are covered by tests

## 5. Implement frontmatter → meta normalization

- [ ] Detect a leading `---…---` block; parse YAML into an ordered `KeyMap`; emit it as an implicit top `::: meta` block (AC4)
- [ ] Malformed frontmatter YAML → `PAPUR-P010`
- [ ] A non-leading `---` stays `Content` (Markdown thematic break)
- [ ] Done when frontmatter and an equivalent `::: meta` block produce identical merged meta

## 6. Implement the merge accessors

- [ ] `merged_meta()` / `merged_theme()` fold all matching blocks left to right, later keys win; empty blocks contribute nothing
- [ ] `css_blocks()` / `script_blocks()` preserve document (source) order
- [ ] Done when merge and ordering match the multiple-block acceptance criteria

## 7. Wire the CLI

- [ ] `clap` binary accepting a file path and `--lenient`
- [ ] Reject any path whose extension is not exactly `.papur` before segmentation (AC1)
- [ ] Confirm filename middle segments do not change behavior (AC2)
- [ ] Render diagnostics with `miette` (source-highlighted)
- [ ] Done when the CLI segments a file and prints labeled diagnostics on error

## 8. Establish compiler-diagnostic conventions in errors.md

- [ ] Replace the `specs/errors.md` template with the compiler-diagnostic format (code, message, line/col span) and the `PAPUR-P` code registry
- [ ] Done when errors.md documents `PAPUR-P001` and `PAPUR-P010` and the code-naming convention

## 9. Acceptance-criteria test suite

- [ ] One test per acceptance criterion (all 8), including the prose-only file (AC5), empty blocks, css/script source order, and theme/meta last-wins
- [ ] Snapshot tests (`insta`) for representative fixtures, including the README example file
- [ ] Done when all 8 acceptance criteria are covered and `cargo test` is green

## 10. Validation gate

- [ ] `cargo test`, `cargo clippy`, and `cargo fmt --check` are clean
- [ ] `npx markdownlint-cli2` is clean on `specs/001-file-format`
- [ ] Done when every check above passes
