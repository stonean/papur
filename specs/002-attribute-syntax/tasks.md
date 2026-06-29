# 002 — Attribute Syntax (Roles) Tasks

Tasks derived from the [plan](plan.md). Complete in order.

## 1. Register diagnostics

- [x] Add `DanglingContentFence` (P002), `DuplicateId` (P020), `MultipleIds` (P021), `MalformedAttribute` (P022), `UnresolvedForcedRole` (P023) to `block::DiagnosticCode` and their `code()` arms.
- [x] Add the rows + the `P020`–`P029` Parse range to `specs/errors.md`, attributed to 002.
- [x] **Done when:** `cargo test` compiles with the new variants and each `code()` returns the expected `PAPUR-P…` string; `errors.md` registry lists all five codes.

## 2. Attribute brace-group parser (`attr` module)

- [x] Create `crates/papur-core/src/attr/mod.rs` with `Attributes`, `RoleRef` (constructed from `.class` tokens), `KeyValues`, and `parse_attributes(group, mode)`.
- [x] Handle classes (with `g.`/`l.`/unprefixed namespace capture), `#id`, `key=value`, unquoted single-token values, and double-quoted values with spaces.
- [x] Emit `P021` on a second `#id`; emit `P022` on `{=value}`; treat `{}` as empty and `{key=}` as an empty value (no diagnostic).
- [x] **Done when:** unit tests cover each token form and degenerate form, asserting both the parsed `Attributes` and the emitted diagnostics for strict and lenient mode.

## 3. Attribute classification (verbatim vs `data-`)

- [x] Add the curated `const` HTML-attribute allowlist (WHATWG global + common element-standard) and `classify_attr(key) -> AttrKind`.
- [x] **Done when:** a test asserts representative recognized keys (`id`, `href`, `lang`, …) classify `Verbatim` and arbitrary keys (`cols`, `foo`) classify `Data`.

## 4. Role resolution (`role` module)

- [x] Create `crates/papur-core/src/role/mod.rs` with `Namespace`, the `RoleRegistry` trait, `Resolution`, and `resolve(role, registry, mode)`.
- [x] Implement `Auto` (local-first→global, unresolved emits verbatim with no diagnostic) and forced `Global`/`Local` (emit `P023` when unsatisfiable; lenient warns).
- [x] **Done when:** unit tests against an injected registry verify each branch, including unresolved-`Auto`-is-not-an-error and forced-miss-is-`P023`.

## 5. Fenced-div depth scan + dangling-fence diagnostic (`structure` module)

- [x] Create `crates/papur-core/src/structure/mod.rs` with the stack-based `:::` scanner: `::: name [attrs]` opens, bare `:::` closes, depth recorded per node.
- [x] Build `FencedDiv` nodes (name → primary class, trailing attrs on the same element) and nest inner fences as children.
- [x] Emit `P002` on an unbalanced/dangling marker (strict error; lenient literal content).
- [x] **Done when:** tests cover balanced nesting, trailing-attribute application, and dangling/unbalanced markers in both modes.

## 6. Headings, inline spans, position + heading-scope rules (`structure` module)

- [x] Recognize ATX headings and `[text]{…}` inline spans; attach pre-text groups to the heading element and open a section scope for post-text groups; attach inline-span groups to the bracketed text (never a scope).
- [x] Implement the heading scope rule (close on same-fence-depth sibling at equal-or-higher level, or fence closer; deeper fences are separate outline universes) and assemble the `StructureTree`.
- [x] Verify nested roles produce nested nodes (descendant structure), not stacked classes.
- [x] **Done when:** tests cover pre/post heading attachment, inline-span attachment, scope open/close across fence depths, and the multi-role nesting example from the spec.

## 7. File-scoped duplicate-id lint + public API

- [ ] Add the whole-file duplicate-`id` pass over the assembled tree, emitting `P020`.
- [ ] Wire the modules together behind the parse entry point and re-export the public types from `lib.rs`.
- [ ] **Done when:** a multi-element fixture with a repeated id reports `P020`; `lib.rs` re-exports compile and are reachable from an integration test.

## 8. Acceptance tests + gates

- [ ] Add one test per acceptance criterion in `crates/papur-core/tests/acceptance.rs`, pinning structured output with `insta` snapshots.
- [ ] **Done when:** every acceptance criterion has a passing test, `cargo test` is green, and `npx markdownlint-cli2` passes on the feature directory.
