#![warn(missing_docs)]

//! Core library for working with space-filling curves.
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
//!
//! # Numeric widths
//!
//! Curve implementations are generic over coordinate and index widths. The
//! default helpers use `DefaultCoord` (`u32`) and `DefaultIndex` (`u64`) with
//! compile-time selection only. To use other widths, call
//! `curve_from_name_typed` or construct curve types directly. There are no
//! runtime width switches.

/// Implementations of specific space‑filling curves.
pub mod curves;
/// Error types used across the crate.
pub mod error;
/// Evaluation metrics for curves.
pub mod evals;
/// Internal bit operations shared by curve implementations.
#[doc(hidden)]
pub mod ops;
/// N‑dimensional points and helpers.
pub mod point;
/// The `SpaceCurve` trait and related utilities.
mod spacecurve;
/// Grid specification helpers shared across curves.
pub mod spec;
/// Numeric trait bounds and default type aliases.
pub mod types;

pub use crate::{
    spacecurve::SpaceCurve,
    types::{Coord, DefaultCoord, DefaultIndex, Index},
};

/// Central registry of curve metadata and constructors.
pub mod registry;

/// Default point type used by the CLI and GUI.
pub type DefaultPoint = point::Point<DefaultCoord>;

/// Default curve trait object used by the CLI and GUI.
pub type DefaultCurve = dyn SpaceCurve<Coord = DefaultCoord, Index = DefaultIndex>;

/// Construct a curve by name with default numeric widths.
///
/// Returns an error if the combination is invalid or the name is unknown.
pub fn curve_from_name(
    name: &str,
    dimension: u32,
    size: DefaultCoord,
) -> error::Result<Box<DefaultCurve>> {
    registry::construct::<DefaultCoord, DefaultIndex>(name, dimension, size)
}

/// Construct a curve by name with explicit numeric widths.
///
/// Returns an error if the combination is invalid or the name is unknown.
pub fn curve_from_name_typed<C: Coord, I: Index>(
    name: &str,
    dimension: u32,
    size: C,
) -> error::Result<Box<dyn SpaceCurve<Coord = C, Index = I> + 'static>> {
    registry::construct::<C, I>(name, dimension, size)
}
