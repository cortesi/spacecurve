//! Modules implementing individual curve families.

/// Gray-code based traversal over a hyper-rectangular grid.
pub mod gray;
/// Hairy Onion: tiled 2D onion spirals connected in higher dimensions.
pub mod hairyonion;
/// H-curve: a Hilbert-like family using BRGC and orientation transforms.
pub mod hcurve;
/// Classic Hilbert curves and utilities.
pub mod hilbert;
/// Internal 2D Hilbert helpers.
mod hilbert2;
/// Shared helpers for Hilbert variants.
mod hilbert_common;
/// Internal N-D Hilbert helpers.
mod hilbertn;
/// Onion curve family operating on L∞ shells.
///
/// This module is organized into submodules for clarity:
/// - `l2`: L=2 specialization (continuous Gray-code)
/// - `twod`: 2D specialization (continuous spiral)
/// - `threed`: 3D specialization
/// - `rect`: Rectangular onion traversal for half-faces
pub mod onion;
/// Simple serpentine scan (boustrophedon) traversal.
pub mod scan;
/// Z-order (Morton) bit-interleaving.
pub mod zorder;
