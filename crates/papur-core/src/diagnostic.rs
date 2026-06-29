//! Compiler diagnostics.
//!
//! papur is a CLI compiler, so a diagnostic is a stable code, a message, and a
//! source [`Span`] — not the web/JSON error envelope in `specs/errors.md`'s
//! template. See `specs/errors.md` for the `PAPUR-P` code registry. Rendering
//! (source-highlighted output via `miette`) is the CLI's concern; the core
//! library only produces the structured data.

use crate::span::Span;

/// A single compiler diagnostic produced during parsing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    /// The stable diagnostic code.
    pub code: DiagnosticCode,
    /// A human-readable message describing the problem.
    pub message: String,
    /// Where in the source the problem occurs.
    pub span: Span,
}

impl Diagnostic {
    /// Construct a diagnostic.
    pub fn new(code: DiagnosticCode, message: impl Into<String>, span: Span) -> Self {
        Self {
            code,
            message: message.into(),
            span,
        }
    }
}

/// The stable set of parse diagnostics the compiler can emit. Each maps to a
/// permanent `PAPUR-P` code documented in `specs/errors.md`. Codes group by
/// concern: `P001`–`P009` fence/block segmentation, `P010`–`P019` frontmatter,
/// `P020`–`P029` attribute groups / roles (spec 002).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticCode {
    /// A reserved layer fence was opened but never closed (strict mode).
    UnterminatedFence,
    /// An unbalanced or dangling `:::` content fence (strict error; lenient
    /// treats the marker as literal content). Owned by spec 002 — block
    /// segmentation leaves content fences opaque, so 001 cannot detect it.
    DanglingContentFence,
    /// Leading YAML frontmatter could not be parsed.
    MalformedFrontmatter,
    /// The same `id` appears on more than one element in the file (lint error).
    DuplicateId,
    /// More than one `#id` in a single attribute group (lint error; lenient
    /// keeps the first id).
    MultipleIds,
    /// A malformed attribute token such as `{=value}` (strict error; lenient
    /// treats it as literal content).
    MalformedAttribute,
    /// A forced namespace prefix (`g.`/`l.`) resolved to no definition (strict
    /// error; lenient emits the class unresolved and records a warning).
    UnresolvedForcedRole,
}

impl DiagnosticCode {
    /// The permanent `PAPUR-P` code string for this diagnostic.
    pub fn code(self) -> &'static str {
        match self {
            Self::UnterminatedFence => "PAPUR-P001",
            Self::DanglingContentFence => "PAPUR-P002",
            Self::MalformedFrontmatter => "PAPUR-P010",
            Self::DuplicateId => "PAPUR-P020",
            Self::MultipleIds => "PAPUR-P021",
            Self::MalformedAttribute => "PAPUR-P022",
            Self::UnresolvedForcedRole => "PAPUR-P023",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codes_map_to_stable_strings() {
        assert_eq!(DiagnosticCode::UnterminatedFence.code(), "PAPUR-P001");
        assert_eq!(DiagnosticCode::DanglingContentFence.code(), "PAPUR-P002");
        assert_eq!(DiagnosticCode::MalformedFrontmatter.code(), "PAPUR-P010");
        assert_eq!(DiagnosticCode::DuplicateId.code(), "PAPUR-P020");
        assert_eq!(DiagnosticCode::MultipleIds.code(), "PAPUR-P021");
        assert_eq!(DiagnosticCode::MalformedAttribute.code(), "PAPUR-P022");
        assert_eq!(DiagnosticCode::UnresolvedForcedRole.code(), "PAPUR-P023");
    }
}
