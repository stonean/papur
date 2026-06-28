//! Core library for the papur markup compiler.
//!
//! The compile pipeline is: source (`.papur`) → parser → AST → target emitter →
//! output (see `specs/system.md`). Spec 001 implements *block segmentation* —
//! the pre-AST pass that splits a source into typed layer blocks and content
//! spans. Those types arrive in the `block` module in a later task.
