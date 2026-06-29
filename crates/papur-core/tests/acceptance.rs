//! Acceptance-criteria coverage for specs 001 (file format) and 002 (attribute
//! syntax).
//!
//! Spec 001: one test per criterion the core library can verify (AC3–AC8); AC1
//! (the exact `.papur` extension) and AC2 (filename middle segments are inert)
//! are extension-layer concerns verified by the `papur` binary's unit tests
//! (`has_papur_extension`). Spec 002: one test per acceptance criterion
//! (`ac1_`–`ac13_`), plus a pinned structure snapshot of the nesting example.

use papur_core::{
    AttrKind, Block, Diagnostic, Namespace, Node, ParseMode, Resolution, RoleRef, RoleRegistry,
    Scope, Span, classify_attr, parse_attributes, parse_document, parse_structure, resolve,
    segment,
};

/// AC3 — typed content outside a fence: a strict-mode error, lenient prose.
#[test]
fn ac3_strict_errors_lenient_degrades() {
    let src = "::: css\n.x\n  color: red\n"; // unterminated reserved fence
    assert!(segment(src, ParseMode::Strict).is_err());
    let lenient = segment(src, ParseMode::Lenient).unwrap();
    assert_eq!(lenient.blocks.len(), 1);
    assert!(matches!(lenient.blocks[0], Block::Content { .. }));
}

/// AC4 — leading `---` frontmatter is treated as an implicit `::: meta` block.
#[test]
fn ac4_frontmatter_is_meta() {
    let stream = segment("---\ntitle: x\n---\n\n# Body\n", ParseMode::Strict).unwrap();
    assert_eq!(
        stream.merged_meta().get("title").and_then(|v| v.as_str()),
        Some("x")
    );
}

/// AC5 — a prose-only file (no fences, no frontmatter) parses as content.
#[test]
fn ac5_prose_only_parses() {
    let stream = segment("# Title\n\nJust prose.\n", ParseMode::Strict).unwrap();
    assert_eq!(stream.blocks.len(), 1);
    assert!(matches!(stream.blocks[0], Block::Content { .. }));
}

/// AC6 — multiple `::: css` / `::: script` blocks keep document order.
#[test]
fn ac6_ordered_layers_keep_source_order() {
    let css = segment("::: css\nA\n:::\n\n::: css\nB\n:::\n", ParseMode::Strict).unwrap();
    assert_eq!(css.css_blocks().collect::<Vec<_>>(), ["A", "B"]);

    let script = segment(
        "::: script\nfirst\n:::\n\n::: script\nsecond\n:::\n",
        ParseMode::Strict,
    )
    .unwrap();
    assert_eq!(
        script.script_blocks().collect::<Vec<_>>(),
        ["first", "second"]
    );
}

/// AC7 — multiple `::: meta` / `::: theme` blocks merge key-by-key, later wins.
#[test]
fn ac7_keyvalue_layers_last_wins() {
    let meta = segment(
        "::: meta\nk: 1\n:::\n\n::: meta\nk: 2\n:::\n",
        ParseMode::Strict,
    )
    .unwrap();
    assert_eq!(
        meta.merged_meta().get("k").and_then(|v| v.as_i64()),
        Some(2)
    );

    let theme = segment(
        "::: theme\nbrand: red\n:::\n\n::: theme\nbrand: blue\n:::\n",
        ParseMode::Strict,
    )
    .unwrap();
    assert_eq!(
        theme.merged_theme().get("brand").and_then(|v| v.as_str()),
        Some("blue")
    );
}

/// AC8 — an empty block of any type parses and contributes nothing.
#[test]
fn ac8_empty_block_contributes_nothing() {
    let css = segment("::: css\n:::\n", ParseMode::Strict).unwrap();
    assert_eq!(css.css_blocks().collect::<Vec<_>>(), [""]);

    let meta = segment("::: meta\n:::\n", ParseMode::Strict).unwrap();
    assert!(meta.merged_meta().is_empty());
}

/// Snapshot of a representative document — the README example — exercising
/// meta, prose with a content fence (`::: grid`), theme, css, and script.
#[test]
fn readme_example_segmentation() {
    let projection: Vec<(String, String)> = segment(README_EXAMPLE, ParseMode::Strict)
        .unwrap()
        .blocks
        .iter()
        .map(|b| match b {
            Block::Content { text, .. } => ("content".to_string(), text.clone()),
            Block::Layer { kind, body, .. } => (kind.keyword().to_string(), body.clone()),
        })
        .collect();
    insta::assert_debug_snapshot!(projection);
}

/// Spec 002 — the attribute-syntax parse entry points are re-exported from the
/// crate root and reachable from a downstream crate.
#[test]
fn attribute_syntax_reexports_reachable() {
    let (tree, diags) = parse_structure("::: hero #top\nhi\n:::", ParseMode::Strict);
    assert!(diags.is_empty());
    assert!(matches!(tree.nodes[0], Node::FencedDiv { .. }));
    assert_eq!(classify_attr("href"), AttrKind::Verbatim);

    let stream = segment("::: a #x\n:::\n::: b #x\n:::", ParseMode::Strict).unwrap();
    let (_doc, doc_diags) = parse_document(&stream);
    assert_eq!(doc_diags.len(), 1);
}

const README_EXAMPLE: &str = r#"::: meta
title: Welcome
lang: en
:::

# Build documents that look designed {.headline}

A markup language for people who like Markdown but love CSS. {.lead}

[Get started]{.btn .primary}(/start) [Read the docs]{.btn .ghost}(/docs)

## Features {#features}

::: grid cols=3
### Fast {.card}
Compiles to clean HTML + CSS. No runtime.

### Familiar {.card}
If you know Markdown, you already know 80% of it.

### Themeable {.card}
Swap a stylesheet, change the entire feel.
:::

::: theme
brand: oklch(60% 0.2 250)
ink: #1a1a1a
paper: #fafafa
:::

::: css
.headline
  font-size: clamp(2rem, 5vw, 4.5rem)

.btn
  padding: $space.sm $space.md
:::

::: script
.btn.primary
  on click
    track('cta-click')
:::
"#;

// ----- Spec 002 (attribute syntax) acceptance criteria -----

fn codes(diags: &[Diagnostic]) -> Vec<&'static str> {
    diags.iter().map(|d| d.code.code()).collect()
}

fn zero_span() -> Span {
    Span {
        start_line: 1,
        start_col: 1,
        start_byte: 0,
        end_byte: 0,
    }
}

fn role(namespace: Namespace, name: &str) -> RoleRef {
    RoleRef {
        namespace,
        name: name.into(),
    }
}

/// A test registry: `loc` is local-only, `glob` global-only, `both` in both.
struct Reg;
impl RoleRegistry for Reg {
    fn has_local(&self, name: &str) -> bool {
        matches!(name, "both" | "loc")
    }
    fn has_global(&self, name: &str) -> bool {
        matches!(name, "both" | "glob")
    }
}

/// AC1 — `{.foo}` attaches a class (role) to its element.
#[test]
fn ac1_class_role() {
    let (attrs, diags) = parse_attributes(".foo", ParseMode::Strict);
    assert!(diags.is_empty());
    assert_eq!(attrs.roles[0].name, "foo");
    assert_eq!(attrs.roles[0].namespace, Namespace::Auto);
}

/// AC2 — `{#id}` attaches an id; a duplicate id in the file is `P020`.
#[test]
fn ac2_id_and_duplicate_lint() {
    let (attrs, _) = parse_attributes("#hero", ParseMode::Strict);
    assert_eq!(attrs.id.as_deref(), Some("hero"));

    let stream = segment("::: a #dup\n:::\n::: b #dup\n:::", ParseMode::Strict).unwrap();
    let (_, diags) = parse_document(&stream);
    assert_eq!(codes(&diags), ["PAPUR-P020"]);
}

/// AC3 — recognized HTML attributes are verbatim; others become `data-`.
#[test]
fn ac3_attribute_classification() {
    assert_eq!(classify_attr("href"), AttrKind::Verbatim);
    assert_eq!(classify_attr("cols"), AttrKind::Data);
    let (attrs, _) = parse_attributes("href=/x cols=3", ParseMode::Strict);
    assert_eq!(attrs.attrs.get("href").map(String::as_str), Some("/x"));
    assert_eq!(attrs.attrs.get("cols").map(String::as_str), Some("3"));
}

/// AC4 — a pre-text role attaches to the heading element (no scope).
#[test]
fn ac4_pre_text_heading_role() {
    let (tree, diags) = parse_structure("### {.hero} Welcome", ParseMode::Strict);
    assert!(diags.is_empty());
    let Node::Heading {
        attrs,
        opens_scope,
        text,
        ..
    } = &tree.nodes[0]
    else {
        panic!("expected heading");
    };
    assert!(!opens_scope);
    assert_eq!(text, "Welcome");
    assert_eq!(attrs.roles[0].name, "hero");
}

/// AC5 — a post-text role opens a section scope holding the following content.
#[test]
fn ac5_post_text_heading_scope() {
    let (tree, _) = parse_structure("### Welcome {.hero}\ninside", ParseMode::Strict);
    let Node::Heading {
        opens_scope,
        attrs,
        children,
        ..
    } = &tree.nodes[0]
    else {
        panic!("expected heading");
    };
    assert!(opens_scope);
    assert_eq!(attrs.roles[0].name, "hero");
    assert!(
        children
            .iter()
            .any(|n| matches!(n, Node::Prose { text, .. } if text.contains("inside")))
    );
}

/// AC6 — nested fenced divs produce nested elements.
#[test]
fn ac6_nested_fenced_divs() {
    let (tree, diags) = parse_structure("::: outer\n::: inner\nx\n:::\n:::", ParseMode::Strict);
    assert!(diags.is_empty());
    let Node::FencedDiv { name, children, .. } = &tree.nodes[0] else {
        panic!("expected fenced div");
    };
    assert_eq!(name, "outer");
    assert!(
        children
            .iter()
            .any(|n| matches!(n, Node::FencedDiv { name, .. } if name == "inner"))
    );
}

/// AC7 — `g.` forces global, `l.` forces local, `.` is local-first then global.
#[test]
fn ac7_namespace_resolution() {
    let reg = Reg;
    let sp = zero_span();
    assert_eq!(
        resolve(
            &role(Namespace::Global, "glob"),
            &reg,
            ParseMode::Strict,
            sp
        )
        .0,
        Resolution::Resolved(Scope::Global)
    );
    assert_eq!(
        resolve(&role(Namespace::Local, "loc"), &reg, ParseMode::Strict, sp).0,
        Resolution::Resolved(Scope::Local)
    );
    assert_eq!(
        resolve(&role(Namespace::Auto, "both"), &reg, ParseMode::Strict, sp).0,
        Resolution::Resolved(Scope::Local)
    );
    assert_eq!(
        resolve(&role(Namespace::Auto, "glob"), &reg, ParseMode::Strict, sp).0,
        Resolution::Resolved(Scope::Global)
    );
}

/// AC8 — an inner fence does not close an outer heading scope.
#[test]
fn ac8_inner_fence_preserves_outer_scope() {
    let (tree, diags) = parse_structure(
        "::: grid\n## Sec {.s}\nA\n::: inner\nB\n:::\nC\n:::",
        ParseMode::Strict,
    );
    assert!(diags.is_empty());
    let Node::FencedDiv { name, children, .. } = &tree.nodes[0] else {
        panic!("expected grid");
    };
    assert_eq!(name, "grid");
    let Node::Heading {
        opens_scope: true,
        children: sec,
        ..
    } = &children[0]
    else {
        panic!("expected section");
    };
    // The section still holds the inner fence and the content after it.
    assert!(
        sec.iter()
            .any(|n| matches!(n, Node::FencedDiv { name, .. } if name == "inner"))
    );
    assert!(
        sec.iter()
            .any(|n| matches!(n, Node::Prose { text, .. } if text.contains("C")))
    );
}

/// AC9 — a dangling `:::` is `P002` in strict mode, literal in lenient mode.
#[test]
fn ac9_dangling_fence() {
    let (_, diags) = parse_structure("text\n:::", ParseMode::Strict);
    assert!(codes(&diags).contains(&"PAPUR-P002"));
    let (tree, ldiags) = parse_structure("text\n:::", ParseMode::Lenient);
    assert!(ldiags.is_empty());
    assert!(
        tree.nodes
            .iter()
            .any(|n| matches!(n, Node::Prose { text, .. } if text == ":::"))
    );
}

/// AC10 — `[text]{.foo}` attaches to the bracketed span and opens no scope.
#[test]
fn ac10_inline_span() {
    let (tree, diags) = parse_structure("see [here]{.link} now", ParseMode::Strict);
    assert!(diags.is_empty());
    assert!(
        tree.nodes
            .iter()
            .all(|n| !matches!(n, Node::Heading { .. }))
    );
    let span = tree
        .nodes
        .iter()
        .find_map(|n| match n {
            Node::InlineSpan { text, attrs, .. } => {
                Some((text.clone(), attrs.roles[0].name.clone()))
            }
            _ => None,
        })
        .expect("inline span");
    assert_eq!(span, ("here".to_string(), "link".to_string()));
}

/// AC11 — a forced-prefix miss is `P023`; an unresolved `.foo` is not an error.
#[test]
fn ac11_forced_miss_and_auto_unresolved() {
    let reg = Reg;
    let sp = zero_span();
    let (res, diag) = resolve(
        &role(Namespace::Global, "absent"),
        &reg,
        ParseMode::Strict,
        sp,
    );
    assert_eq!(res, Resolution::ForcedMiss);
    assert_eq!(diag.unwrap().code.code(), "PAPUR-P023");

    let (res, diag) = resolve(
        &role(Namespace::Auto, "absent"),
        &reg,
        ParseMode::Strict,
        sp,
    );
    assert_eq!(res, Resolution::Unresolved);
    assert!(diag.is_none());
}

/// AC12 — a fenced div's name is its class; trailing attrs apply to it.
#[test]
fn ac12_fenced_div_attributes() {
    let (tree, _) = parse_structure("::: hero .fancy #top cols=2\n:::", ParseMode::Strict);
    let Node::FencedDiv { name, attrs, .. } = &tree.nodes[0] else {
        panic!("expected fenced div");
    };
    assert_eq!(name, "hero");
    assert_eq!(attrs.roles[0].name, "fancy");
    assert_eq!(attrs.id.as_deref(), Some("top"));
    assert_eq!(attrs.attrs.get("cols").map(String::as_str), Some("2"));
}

/// AC13 — degenerate groups: `{}` no-op, `{#a #b}` P021, `{=value}` P022.
#[test]
fn ac13_degenerate_groups() {
    let (empty, d0) = parse_attributes("", ParseMode::Strict);
    assert!(empty.is_empty());
    assert!(d0.is_empty());
    assert_eq!(
        codes(&parse_attributes("#a #b", ParseMode::Strict).1),
        ["PAPUR-P021"]
    );
    assert_eq!(
        codes(&parse_attributes("=value", ParseMode::Strict).1),
        ["PAPUR-P022"]
    );
    assert!(parse_attributes("=value", ParseMode::Lenient).1.is_empty());
}

/// The spec's multi-role nesting example, pinned as a structure snapshot.
#[test]
fn nesting_example_structure_snapshot() {
    let (tree, diags) = parse_structure(NESTING_EXAMPLE, ParseMode::Strict);
    assert!(diags.is_empty());
    insta::assert_debug_snapshot!(tree);
}

const NESTING_EXAMPLE: &str = "::: grid cols=3
### Fast {.carda}
Content.

  ::: grid cols=2
  Still in .carda.

  #### Smaller {.card1}
  In .carda > .card1.
  :::
:::
";
