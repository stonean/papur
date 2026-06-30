//! Attribute brace-group grammar (spec 002).
//!
//! Parses the inner text of a `{.class #id key=value}` group into
//! [`Attributes`]. The caller (the structure scanner) locates the `{…}` group
//! and passes the text *between* the braces; [`parse_attributes`] tokenizes it
//! into class roles, an optional id, and key/value pairs.
//!
//! Diagnostics ([`MultipleIds`](DiagnosticCode::MultipleIds) `P021`,
//! [`MalformedAttribute`](DiagnosticCode::MalformedAttribute) `P022`) are
//! returned in strict mode; in lenient mode recoverable problems degrade
//! silently (the first id wins, malformed tokens are dropped) per
//! `specs/errors.md`. Diagnostic spans use byte offsets *relative to the group
//! text* (groups are single-line); callers offset them into the source.

use indexmap::IndexMap;

use crate::block::ParseMode;
use crate::diagnostic::{Diagnostic, DiagnosticCode};
use crate::span::Span;

/// The lookup directive a role-class prefix encodes (`{.foo}` / `{g.foo}` /
/// `{l.foo}`). The resolution algorithm that consumes it lives in the
/// [`role`](crate::role) module; this is the parsed product the scanner
/// attaches to elements.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Namespace {
    /// `{.foo}` — local-first, then global.
    #[default]
    Auto,
    /// `{g.foo}` — force global.
    Global,
    /// `{l.foo}` — force local.
    Local,
}

/// A namespace-prefixed class reference, parsed from a `.class` token.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoleRef {
    /// The lookup directive from the token's prefix.
    pub namespace: Namespace,
    /// The class name, without the leading `.` or `g.`/`l.` prefix.
    pub name: String,
}

/// Insertion-ordered, last-wins map for `key=value` attributes.
pub type KeyValues = IndexMap<String, String>;

/// One parsed attribute group — the `{.class #id key=value}` brace grammar a
/// heading or inline span uses, and (minus the braces) the `:::` fenced-div
/// header. Every field is optional; an empty group (`{}`) parses to the
/// [`Default`] value (a no-op).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Attributes {
    /// A bare word naming the element (`::: nav` → `element = Some("nav")`).
    /// Meaningful only for the `:::` header, where the absence of a bareword
    /// defaults the block to `<div>`; headings and inline spans carry their own
    /// element and leave this unused. The first bare word wins. What the name
    /// *resolves* to — a standard tag, a custom element, or a lint error — is
    /// owned by spec 003; 002 only captures it.
    pub element: Option<String>,
    /// `.class` tokens in source order, each a role reference.
    pub roles: Vec<RoleRef>,
    /// The `#id` token, if any. At most one is valid (`P021` otherwise).
    pub id: Option<String>,
    /// `key=value` pairs, insertion-ordered, last-wins on a repeated key.
    pub attrs: KeyValues,
}

impl Attributes {
    /// Whether the group contributed nothing (the `{}` no-op case).
    pub fn is_empty(&self) -> bool {
        self.element.is_none()
            && self.roles.is_empty()
            && self.id.is_none()
            && self.attrs.is_empty()
    }
}

/// How a `key=value` pair is emitted by the target layer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttrKind {
    /// `key` is a recognized HTML attribute name — emitted verbatim.
    Verbatim,
    /// Any other key — emitted as `data-{key}`.
    Data,
}

/// The curated allowlist of HTML attribute names emitted verbatim: the WHATWG
/// global attributes plus the common element-standard attributes authors attach
/// inline. This is the single source of truth for the verbatim/`data-` boundary
/// — the emitter trusts [`classify_attr`] rather than re-deriving it. Keys
/// outside the list, including element-specific layout keys like `cols`, are
/// namespaced under `data-`, matching the `::: grid cols=3` → `data-cols`
/// example in the spec. `data-*` and `aria-*` keys are recognized by prefix.
const HTML_ATTRIBUTES: &[&str] = &[
    // WHATWG global attributes.
    "accesskey",
    "autocapitalize",
    "autofocus",
    "class",
    "contenteditable",
    "dir",
    "draggable",
    "enterkeyhint",
    "hidden",
    "id",
    "inert",
    "inputmode",
    "is",
    "itemid",
    "itemprop",
    "itemref",
    "itemscope",
    "itemtype",
    "lang",
    "nonce",
    "part",
    "popover",
    "role",
    "slot",
    "spellcheck",
    "style",
    "tabindex",
    "title",
    "translate",
    // Common element-standard attributes attached inline.
    "accept",
    "action",
    "alt",
    "async",
    "autocomplete",
    "autoplay",
    "charset",
    "checked",
    "cite",
    "content",
    "controls",
    "crossorigin",
    "datetime",
    "decoding",
    "defer",
    "disabled",
    "download",
    "enctype",
    "for",
    "form",
    "height",
    "href",
    "hreflang",
    "integrity",
    "label",
    "list",
    "loading",
    "loop",
    "max",
    "maxlength",
    "media",
    "method",
    "min",
    "minlength",
    "multiple",
    "muted",
    "name",
    "open",
    "pattern",
    "ping",
    "placeholder",
    "poster",
    "preload",
    "readonly",
    "referrerpolicy",
    "rel",
    "required",
    "selected",
    "sizes",
    "src",
    "srcset",
    "step",
    "target",
    "type",
    "value",
    "width",
    "wrap",
];

/// Classify an attribute key as verbatim HTML or a `data-` attribute. Matching
/// is case-insensitive; `data-*` and `aria-*` keys are always verbatim.
pub fn classify_attr(key: &str) -> AttrKind {
    let lower = key.to_ascii_lowercase();
    if lower.starts_with("data-") || lower.starts_with("aria-") {
        return AttrKind::Verbatim;
    }
    if HTML_ATTRIBUTES.contains(&lower.as_str()) {
        AttrKind::Verbatim
    } else {
        AttrKind::Data
    }
}

/// Parse the inner text of a brace group into [`Attributes`].
///
/// In strict mode, problems are reported as diagnostics; in lenient mode they
/// degrade silently (first id wins, malformed tokens dropped). See the module
/// docs for span conventions.
pub fn parse_attributes(group: &str, mode: ParseMode) -> (Attributes, Vec<Diagnostic>) {
    let mut attrs = Attributes::default();
    let mut diags = Vec::new();

    for (start, token) in tokenize(group) {
        classify_token(token, start, mode, &mut attrs, &mut diags);
    }

    (attrs, diags)
}

/// Split a group into whitespace-separated tokens, treating a double-quoted run
/// as part of its token so `key="a b"` stays whole. Returns each token with its
/// byte offset within `group`.
fn tokenize(group: &str) -> Vec<(usize, &str)> {
    let bytes = group.as_bytes();
    let mut tokens = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        while i < bytes.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        if i >= bytes.len() {
            break;
        }
        let start = i;
        let mut in_quote = false;
        while i < bytes.len() {
            let b = bytes[i];
            if b == b'"' {
                in_quote = !in_quote;
            } else if b.is_ascii_whitespace() && !in_quote {
                break;
            }
            i += 1;
        }
        // Breaks only ever land on ASCII whitespace/quote bytes, so the slice
        // stays on UTF-8 char boundaries.
        tokens.push((start, &group[start..i]));
    }
    tokens
}

/// Classify one token and fold it into `attrs`, pushing diagnostics in strict
/// mode for malformed or duplicate-id tokens.
fn classify_token(
    token: &str,
    start: usize,
    mode: ParseMode,
    attrs: &mut Attributes,
    diags: &mut Vec<Diagnostic>,
) {
    let span = token_span(start, token);

    if let Some(name) = token.strip_prefix('#') {
        if name.is_empty() {
            push_malformed(mode, diags, span);
        } else if attrs.id.is_some() {
            // A second id in one group: first wins; strict reports it.
            if mode == ParseMode::Strict {
                diags.push(Diagnostic::new(
                    DiagnosticCode::MultipleIds,
                    format!("more than one `#id` in attribute group; `#{name}` ignored"),
                    span,
                ));
            }
        } else {
            attrs.id = Some(name.to_string());
        }
        return;
    }

    if let Some(role) = parse_class(token) {
        if role.name.is_empty() {
            push_malformed(mode, diags, span);
        } else {
            attrs.roles.push(role);
        }
        return;
    }

    if let Some(eq) = token.find('=') {
        let key = &token[..eq];
        let raw = &token[eq + 1..];
        if key.is_empty() {
            push_malformed(mode, diags, span);
        } else {
            match unquote(raw) {
                Some(value) => {
                    attrs.attrs.insert(key.to_string(), value);
                }
                None => push_malformed(mode, diags, span),
            }
        }
        return;
    }

    // A bare token with no `.`/`#`/`=` names the element (`::: nav`). The first
    // one wins; element resolution and any multiple-element diagnostic are spec
    // 003's concern, so capturing the bareword here is never an error.
    if attrs.element.is_none() {
        attrs.element = Some(token.to_string());
    }
}

/// Parse a `.foo` / `g.foo` / `l.foo` class token into a [`RoleRef`]. Returns
/// `None` when the token is not a class token.
fn parse_class(token: &str) -> Option<RoleRef> {
    let (namespace, name) = if let Some(rest) = token.strip_prefix("g.") {
        (Namespace::Global, rest)
    } else if let Some(rest) = token.strip_prefix("l.") {
        (Namespace::Local, rest)
    } else if let Some(rest) = token.strip_prefix('.') {
        (Namespace::Auto, rest)
    } else {
        return None;
    };
    Some(RoleRef {
        namespace,
        name: name.to_string(),
    })
}

/// Strip surrounding double quotes from a value. An unquoted value is returned
/// as-is; an empty value (`key=`) yields `Some("")`; an unterminated quote
/// yields `None` (a malformed value).
fn unquote(raw: &str) -> Option<String> {
    let Some(inner) = raw.strip_prefix('"') else {
        return Some(raw.to_string());
    };
    match inner.strip_suffix('"') {
        // `inner` non-empty guarantees the opening and closing quotes differ.
        Some(value) if !inner.is_empty() => Some(value.to_string()),
        _ => None,
    }
}

/// A group-relative span for `token` at byte offset `start`. Groups are
/// single-line, so line is always 1 and column tracks the byte offset.
fn token_span(start: usize, token: &str) -> Span {
    Span {
        start_line: 1,
        start_col: start as u32 + 1,
        start_byte: start,
        end_byte: start + token.len(),
    }
}

/// Push a `P022` malformed-attribute diagnostic in strict mode only.
fn push_malformed(mode: ParseMode, diags: &mut Vec<Diagnostic>, span: Span) {
    if mode == ParseMode::Strict {
        diags.push(Diagnostic::new(
            DiagnosticCode::MalformedAttribute,
            "malformed attribute token",
            span,
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn strict(group: &str) -> (Attributes, Vec<Diagnostic>) {
        parse_attributes(group, ParseMode::Strict)
    }

    fn codes(diags: &[Diagnostic]) -> Vec<&'static str> {
        diags.iter().map(|d| d.code.code()).collect()
    }

    #[test]
    fn empty_group_is_a_noop() {
        let (attrs, diags) = strict("");
        assert!(attrs.is_empty());
        assert!(diags.is_empty());
    }

    #[test]
    fn multiple_classes_are_space_separated() {
        let (attrs, diags) = strict(".btn .primary");
        assert!(diags.is_empty());
        assert_eq!(
            attrs.roles,
            vec![
                RoleRef {
                    namespace: Namespace::Auto,
                    name: "btn".into()
                },
                RoleRef {
                    namespace: Namespace::Auto,
                    name: "primary".into()
                },
            ]
        );
    }

    #[test]
    fn namespace_prefixes_are_captured() {
        let (attrs, _) = strict(".a g.b l.c");
        assert_eq!(attrs.roles[0].namespace, Namespace::Auto);
        assert_eq!(attrs.roles[1].namespace, Namespace::Global);
        assert_eq!(attrs.roles[1].name, "b");
        assert_eq!(attrs.roles[2].namespace, Namespace::Local);
    }

    #[test]
    fn id_and_key_values() {
        let (attrs, diags) = strict("#go cols=3 data-x=hello");
        assert!(diags.is_empty());
        assert_eq!(attrs.id.as_deref(), Some("go"));
        assert_eq!(attrs.attrs.get("cols").map(String::as_str), Some("3"));
        assert_eq!(attrs.attrs.get("data-x").map(String::as_str), Some("hello"));
    }

    #[test]
    fn quoted_value_keeps_spaces() {
        let (attrs, diags) = strict(r#"title="a b c""#);
        assert!(diags.is_empty());
        assert_eq!(attrs.attrs.get("title").map(String::as_str), Some("a b c"));
    }

    #[test]
    fn empty_value_is_not_an_error() {
        let (attrs, diags) = strict("key=");
        assert!(diags.is_empty());
        assert_eq!(attrs.attrs.get("key").map(String::as_str), Some(""));
    }

    #[test]
    fn repeated_key_last_wins() {
        let (attrs, _) = strict("x=1 x=2");
        assert_eq!(attrs.attrs.get("x").map(String::as_str), Some("2"));
    }

    #[test]
    fn combined_group() {
        let (attrs, diags) = strict(r#".btn .primary #go data-size="x large""#);
        assert!(diags.is_empty());
        assert_eq!(attrs.roles.len(), 2);
        assert_eq!(attrs.id.as_deref(), Some("go"));
        assert_eq!(
            attrs.attrs.get("data-size").map(String::as_str),
            Some("x large")
        );
    }

    #[test]
    fn multiple_ids_strict_is_p021_first_wins() {
        let (attrs, diags) = strict("#a #b");
        assert_eq!(attrs.id.as_deref(), Some("a"));
        assert_eq!(codes(&diags), vec!["PAPUR-P021"]);
    }

    #[test]
    fn multiple_ids_lenient_recovers_silently() {
        let (attrs, diags) = parse_attributes("#a #b", ParseMode::Lenient);
        assert_eq!(attrs.id.as_deref(), Some("a"));
        assert!(diags.is_empty());
    }

    #[test]
    fn missing_key_strict_is_p022() {
        let (attrs, diags) = strict("=value");
        assert!(attrs.is_empty());
        assert_eq!(codes(&diags), vec!["PAPUR-P022"]);
    }

    #[test]
    fn missing_key_lenient_drops_token() {
        let (attrs, diags) = parse_attributes("=value", ParseMode::Lenient);
        assert!(attrs.is_empty());
        assert!(diags.is_empty());
    }

    #[test]
    fn bare_word_names_the_element() {
        let (attrs, diags) = strict("nav");
        assert!(diags.is_empty());
        assert_eq!(attrs.element.as_deref(), Some("nav"));
    }

    #[test]
    fn first_bare_word_wins() {
        let (attrs, diags) = strict("nav header");
        assert!(diags.is_empty());
        assert_eq!(attrs.element.as_deref(), Some("nav"));
    }

    #[test]
    fn element_with_class_id_and_pair() {
        // The `:::`-header grammar (minus braces): bare word → element.
        let (attrs, diags) = strict("nav .site #top cols=2");
        assert!(diags.is_empty());
        assert_eq!(attrs.element.as_deref(), Some("nav"));
        assert_eq!(attrs.roles[0].name, "site");
        assert_eq!(attrs.id.as_deref(), Some("top"));
        assert_eq!(attrs.attrs.get("cols").map(String::as_str), Some("2"));
    }

    #[test]
    fn unterminated_quote_strict_is_p022() {
        let (attrs, diags) = strict(r#"key="abc"#);
        assert!(attrs.attrs.is_empty());
        assert_eq!(codes(&diags), vec!["PAPUR-P022"]);
    }

    #[test]
    fn recognized_html_attributes_are_verbatim() {
        for key in ["id", "href", "lang", "class", "src", "title", "type"] {
            assert_eq!(classify_attr(key), AttrKind::Verbatim, "{key}");
        }
    }

    #[test]
    fn data_and_aria_prefixes_are_verbatim() {
        assert_eq!(classify_attr("data-x"), AttrKind::Verbatim);
        assert_eq!(classify_attr("aria-label"), AttrKind::Verbatim);
    }

    #[test]
    fn unknown_keys_are_data() {
        for key in ["cols", "foo", "bar"] {
            assert_eq!(classify_attr(key), AttrKind::Data, "{key}");
        }
    }

    #[test]
    fn classification_is_case_insensitive() {
        assert_eq!(classify_attr("HREF"), AttrKind::Verbatim);
    }
}
