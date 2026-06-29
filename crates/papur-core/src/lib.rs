//! Core library for the papur markup compiler.
//!
//! The compile pipeline is: source (`.papur`) → parser → AST → target emitter →
//! output (see `specs/system.md`). Spec 001 implements *block segmentation* —
//! the pre-AST pass that splits a source into typed layer blocks and content
//! spans. Those types live in the [`block`] module; diagnostics in
//! [`diagnostic`]; source locations in [`span`].

pub mod attr;
pub mod block;
pub mod diagnostic;
pub mod role;
pub mod span;
pub mod structure;

pub use block::{Block, BlockStream, KeyMap, LayerKind, ParseMode, YamlValue, segment};
pub use diagnostic::{Diagnostic, DiagnosticCode};
pub use span::Span;
