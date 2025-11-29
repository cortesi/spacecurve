//! CLI utilities for visualizing space-filling curves.
//!
//! This crate exposes helpers used by the `scurve` binary as a tiny library so
//! they can be reused from other binaries (for example, the GUI).

/// Commands for generating images from inputs and patterns.
pub mod cmd;
/// Helpers to render maps and drawing primitives.
pub mod map;

// Re-export command functionality for potential library use.
pub use cmd::*;
