//! Merge views over a [`BlockStream`] (spec 001, task 6).
//!
//! The block list is canonical; these accessors derive merged views on demand.
//! Key-value layers (`meta`, `theme`) fold left-to-right with later keys
//! winning; ordered layers (`css`, `script`) are yielded in document order.

use yaml_rust2::{Yaml, YamlLoader};

use super::{Block, BlockStream, KeyMap, LayerKind};

/// Convert a YAML mapping key to a string. Non-scalar keys are skipped.
fn key_to_string(key: &Yaml) -> Option<String> {
    match key {
        Yaml::String(s) | Yaml::Real(s) => Some(s.clone()),
        Yaml::Integer(i) => Some(i.to_string()),
        Yaml::Boolean(b) => Some(b.to_string()),
        _ => None,
    }
}

impl BlockStream {
    /// Fold every `::: meta` block (including normalized frontmatter)
    /// left-to-right; later keys win. Empty blocks contribute nothing.
    pub fn merged_meta(&self) -> KeyMap {
        self.merge_kv(LayerKind::Meta)
    }

    /// Fold every `::: theme` block left-to-right; later keys win.
    pub fn merged_theme(&self) -> KeyMap {
        self.merge_kv(LayerKind::Theme)
    }

    fn merge_kv(&self, want: LayerKind) -> KeyMap {
        let mut map = KeyMap::new();
        for block in &self.blocks {
            let Block::Layer { kind, body, .. } = block else {
                continue;
            };
            if *kind != want || body.trim().is_empty() {
                continue;
            }
            let Ok(docs) = YamlLoader::load_from_str(body) else {
                continue;
            };
            let Some(Yaml::Hash(hash)) = docs.into_iter().next() else {
                continue;
            };
            for (k, v) in hash {
                if let Some(key) = key_to_string(&k) {
                    map.insert(key, v);
                }
            }
        }
        map
    }

    /// `::: css` block bodies, in document (source) order.
    pub fn css_blocks(&self) -> impl Iterator<Item = &str> {
        self.layer_bodies(LayerKind::Css)
    }

    /// `::: script` block bodies, in document (source) order.
    pub fn script_blocks(&self) -> impl Iterator<Item = &str> {
        self.layer_bodies(LayerKind::Script)
    }

    fn layer_bodies(&self, want: LayerKind) -> impl Iterator<Item = &str> {
        self.blocks.iter().filter_map(move |b| match b {
            Block::Layer { kind, body, .. } if *kind == want => Some(body.as_str()),
            _ => None,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::block::{ParseMode, segment};

    #[test]
    fn merged_meta_later_keys_win() {
        let src = "::: meta\ntitle: First\nlang: en\n:::\n\n::: meta\ntitle: Second\n:::\n";
        let stream = segment(src, ParseMode::Strict).unwrap();
        let meta = stream.merged_meta();
        assert_eq!(meta.get("title").and_then(|v| v.as_str()), Some("Second"));
        assert_eq!(meta.get("lang").and_then(|v| v.as_str()), Some("en"));
    }

    #[test]
    fn frontmatter_and_meta_merge_identically() {
        let a = segment(
            "---\ntitle: x\n---\n\n::: meta\nlang: en\n:::\n",
            ParseMode::Strict,
        )
        .unwrap();
        let b = segment("::: meta\ntitle: x\nlang: en\n:::\n", ParseMode::Strict).unwrap();
        assert_eq!(a.merged_meta(), b.merged_meta());
    }

    #[test]
    fn css_blocks_preserve_source_order() {
        let src = "::: css\n.a { }\n:::\n\nProse.\n\n::: css\n.b { }\n:::\n";
        let stream = segment(src, ParseMode::Strict).unwrap();
        let css: Vec<&str> = stream.css_blocks().collect();
        assert_eq!(css, vec![".a { }", ".b { }"]);
    }

    #[test]
    fn empty_meta_block_contributes_nothing() {
        let src = "::: meta\n:::\n\n::: meta\ntitle: x\n:::\n";
        let stream = segment(src, ParseMode::Strict).unwrap();
        let meta = stream.merged_meta();
        assert_eq!(meta.len(), 1);
        assert_eq!(meta.get("title").and_then(|v| v.as_str()), Some("x"));
    }
}
