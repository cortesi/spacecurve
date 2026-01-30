//! Numeric traits and default type aliases for curve coordinates and indices.

use std::fmt::Debug;

use num_traits::{CheckedShl, FromPrimitive, PrimInt, ToPrimitive, Unsigned};

/// Coordinate type used to represent point components and side lengths.
///
/// Implemented for `u32`, `u64`, and `u128`.
pub trait Coord:
    Copy + Debug + PrimInt + Unsigned + FromPrimitive + ToPrimitive + CheckedShl + Send + Sync + 'static
{
    /// Bit width of the coordinate type.
    const BITS: u32;
}

impl Coord for u32 {
    const BITS: Self = Self::BITS;
}

impl Coord for u64 {
    const BITS: u32 = Self::BITS;
}

impl Coord for u128 {
    const BITS: u32 = Self::BITS;
}

/// Index type used to represent linear offsets and curve lengths.
///
/// Implemented for `u32`, `u64`, and `u128`.
pub trait Index: Coord {}

impl Index for u32 {}

impl Index for u64 {}

impl Index for u128 {}

/// Default coordinate type used by the CLI/GUI and convenience helpers.
pub type DefaultCoord = u32;
/// Default index type used by the CLI/GUI and convenience helpers.
pub type DefaultIndex = u64;
