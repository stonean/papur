//! Source locations for diagnostics.

/// A span within a source file. Carries 1-based line/column for human-readable
/// messages and 0-based byte offsets for source highlighting (e.g. `miette`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    /// 1-based line of the span's start.
    pub start_line: u32,
    /// 1-based column of the span's start.
    pub start_col: u32,
    /// 0-based byte offset of the span's start.
    pub start_byte: usize,
    /// 0-based byte offset one past the span's end.
    pub end_byte: usize,
}

impl Span {
    /// Length of the span in bytes.
    pub fn len(&self) -> usize {
        self.end_byte.saturating_sub(self.start_byte)
    }

    /// Whether the span covers zero bytes.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
