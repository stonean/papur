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

/// The stable set of parse diagnostics 001 can emit. Each maps to a permanent
/// `PAPUR-P` code documented in `specs/errors.md`.
///
/// Note: detecting an unbalanced or *dangling* content-fence marker is **not**
/// in this set — that requires content-fence depth tracking, which block
/// segmentation deliberately does not do. It is owned by spec 002
/// (attribute-syntax), whose fenced-div parser tracks depth.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticCode {
    /// A reserved layer fence was opened but never closed (strict mode).
    UnterminatedFence,
    /// Leading YAML frontmatter could not be parsed.
    MalformedFrontmatter,
}

impl DiagnosticCode {
    /// The permanent `PAPUR-P` code string for this diagnostic.
    pub fn code(self) -> &'static str {
        match self {
            Self::UnterminatedFence => "PAPUR-P001",
            Self::MalformedFrontmatter => "PAPUR-P010",
        }
    }
}
