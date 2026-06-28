//! Acceptance-criteria coverage for spec 001 (file format).
//!
//! One test per spec acceptance criterion the core library can verify (AC3–AC8).
//! AC1 (the exact `.papur` extension) and AC2 (filename middle segments are
//! inert) are extension-layer concerns, verified by the `papur` binary's unit
//! tests (`has_papur_extension`).

use papur_core::{segment, Block, ParseMode};

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

    let script =
        segment("::: script\nfirst\n:::\n\n::: script\nsecond\n:::\n", ParseMode::Strict).unwrap();
    assert_eq!(script.script_blocks().collect::<Vec<_>>(), ["first", "second"]);
}

/// AC7 — multiple `::: meta` / `::: theme` blocks merge key-by-key, later wins.
#[test]
fn ac7_keyvalue_layers_last_wins() {
    let meta = segment("::: meta\nk: 1\n:::\n\n::: meta\nk: 2\n:::\n", ParseMode::Strict).unwrap();
    assert_eq!(meta.merged_meta().get("k").and_then(|v| v.as_i64()), Some(2));

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
