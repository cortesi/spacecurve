//! The `SpaceCurve` trait describing a family of curves.

use std::fmt;

use crate::{
    point,
    types::{Coord, Index},
};

/// SpaceCurve is the core trait for space‑filling curves.
///
/// Invariants and preconditions (apply to all implementations):
/// - `dimensions()` is fixed at construction and defines the required point arity.
/// - `index` expects a [`point::Point`] whose length matches `dimensions()` and whose
///   coordinates lie in `[0, size-1]` for the curve.
/// - `point` expects `index < length()`.
/// - Constructors are responsible for validating dimensionality and bounds (via
///   the shared [`spec::GridSpec`] helpers); callers should treat out‑of‑range
///   inputs as undefined behaviour. Implementations retain lightweight
///   `debug_assert!` guards for development builds.
pub trait SpaceCurve: fmt::Debug {
    /// Coordinate type used by the curve.
    type Coord: Coord;
    /// Index type used by the curve.
    type Index: Index;
    /// A short human-friendly name for this curve.
    ///
    /// This is intended for UI display and logs.
    fn name(&self) -> &'static str;

    /// A concise, multi-line description of the curve.
    ///
    /// The returned string may contain embedded newlines. The caller is
    /// responsible for wrapping the text for display.
    fn info(&self) -> &'static str;
    /// Calculate the linear index of an N-dimensional point. The dimension of
    /// the point must match that of the curve.
    fn index(&self, p: &point::Point<Self::Coord>) -> Self::Index;
    /// Calculate the coordinates of a point from a linear index. The returned
    /// point will have a dimension matching that of the curve.
    fn point(&self, index: Self::Index) -> point::Point<Self::Coord>;
    /// What is the maximum linear offset supported by this curve?
    fn length(&self) -> Self::Index;
    /// How many dimensions does the curve have?
    fn dimensions(&self) -> u32;
}
