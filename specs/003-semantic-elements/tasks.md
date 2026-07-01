# 003 — Semantic Element Registry Tasks

Tasks derived from the [plan](plan.md). Complete in order. Coordinated with 002
(reopened for the `:::`-as-attribute-group grammar change).

## 1. Recognized elements and `element_for`

- [ ] Create `crates/papur-core/src/element/mod.rs` with the `Element` enum (standard variants + `Custom(String)`) and a single `const` standard-tag table.
- [ ] Implement `element_for(name) -> Option<Element>`: exact lowercase standard match; else a valid custom name (lowercase + hyphen) ⇒ `Custom`; else `None`.
- [ ] Register the module in `lib.rs`.
- [ ] Unit tests: every standard tag maps; `recipe-card` ⇒ `Custom`; `Nav`, `hero`, `flibble` ⇒ `None`.
- Done when: `element_for` resolves the standard set, accepts hyphenated customs, and rejects miscased/invalid names.

## 2. `ElementBindings` trait and `resolve_element`

- [ ] Add `ScopeKind { Heading, FencedDiv }`, the `ElementBindings` trait, and `resolve_element(kind, bareword, roles, bindings, mode)`.
- [ ] Implement precedence — bare word > first `as:`-bound role (source order) > scope default (`Section`/`Div`) — emitting `P030` on an invalid bare word and `P032` (warning) on more than one bound role.
- Done when: unit tests over an injected `ElementBindings` cover bare-word override, binding fallback, both defaults, and the `P032` first-wins-warning.

## 3. Diagnostics P030 / P031 / P032

- [ ] Add `InvalidElement` (P030), `MisplacedElement` (P031), `AmbiguousBinding` (P032) to `diagnostic::DiagnosticCode` and its `code()` match.
- [ ] Extend the `codes_map_to_stable_strings` test with the three new codes.
- [ ] Register them in `specs/errors.md`, adding the `P030`–`P039` element-resolution range; `P032` severity is `warning`.
- Done when: `cargo test` passes and `errors.md` lists P030–P032 with severities and introducing spec `003-semantic-elements`.

## 4. Capture the bare word in `attr`

- [ ] Add `element: Option<String>` to `Attributes`; include it in `is_empty`.
- [ ] In `classify_token`, replace the bare-token `P022` branch: capture the first bare token into `element`; a second bare token emits `P031` (strict) / recovers silently (lenient).
- [ ] Replace `bare_token_strict_is_p022` with: one bare token populates `element` and emits no parse-time diagnostic; two bare tokens emit `P031` (strict).
- Done when: `parse_attributes("nav .x")` yields `element = Some("nav")`, one role, no diagnostics; `parse_attributes("nav main")` yields `P031` (strict).

## 5. Resolve and validate in the `structure` pass

- [ ] For post-text headings and fenced divs, resolve via `resolve_element`; push `P030` when `attrs.element` is set but invalid.
- [ ] Pre-text headings and `InlineSpan`: if `attrs.element` is set, push `P031` (no scope opens).
- [ ] Consume 002's `:::`-as-attribute-group change: the fenced-div element comes from `attrs.element`, classes from `attrs.roles`; no implicit primary-class name.
- Done when: structure-level tests show `P030` on an invalid post-text/`:::` bare word, `P031` on a pre-text/inline bare word, and `::: nav` → `Nav` vs `::: .grid` → `Div`+class.

## 6. Acceptance-criteria tests

- [ ] One test per spec acceptance criterion: section/div defaults; bare-word naming both positions; class-never-element; hyphenated custom; `P030` invalid; `P031` misplaced; `{div .card}` downgrade; `as:` binding on heading and `:::`; precedence (bare word > binding > default); `P032` first-wins-warning on two bound roles.
- [ ] Cover strict and lenient mode for the `P030`/`P031` paths.
- Done when: every acceptance criterion has a passing test and `cargo test -p papur-core` is green.
