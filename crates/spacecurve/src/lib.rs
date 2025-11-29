//! Core library for working with space‑filling curves.
//!
//! Exposes curve implementations and the [`SpaceCurve`] trait used by the CLI
//! and GUI crates in this workspace.
//!
//! # Supported Curves
//!
//! - Hilbert
//! - Z-order (Morton)
//! - Gray Code
//! - H-curve
//! - Scan (Boustrophedon)
//! - Onion / Hairy Onion (experimental)

/// Implementations of specific space‑filling curves.
pub mod curves;
/// Error types used across the crate.
pub mod error;
/// Internal bit operations shared by curve implementations.
#[doc(hidden)]
pub mod ops;
/// N‑dimensional points and helpers.
pub mod point;
/// The `SpaceCurve` trait and related utilities.
pub mod spacecurve;
/// Grid specification helpers shared across curves.
pub mod spec;

// Back-compat re-exports for top-level curve modules
pub use curves::{gray, hairyonion, hcurve, hilbert, onion, scan, zorder};

use crate::spacecurve::SpaceCurve;

/// Central registry of curve metadata and constructors.
pub mod registry;

/// List of curve names accepted by the public APIs and CLI.
pub use registry::{CURVE_NAMES, curve_names};

/// Construct a curve by name with the requested dimensionality and size.
///
/// Returns an error if the combination is invalid or the name is unknown.
pub fn pattern_from_name(
    name: &str,
    dimension: u32,
    size: u32,
) -> error::Result<Box<dyn SpaceCurve + 'static>> {
    registry::construct(name, dimension, size)
}
