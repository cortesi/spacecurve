//! Lightweight N-dimensional point type used by curve implementations.

use std::{ops::Deref, vec::Vec};

use smallvec::SmallVec;

use crate::types::Coord;

/// Compact N‑dimensional point wrapper used by curves.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Point<C: Coord>(pub SmallVec<[C; 8]>);

impl<C: Coord> Point<C> {
    /// Create a new `Point` from a backing vector.
    pub fn new(vec: impl Into<SmallVec<[C; 8]>>) -> Self {
        Self(vec.into())
    }

    /// Create a new `Point`, asserting the coordinate count matches `dimension`.
    ///
    /// This is a convenience to avoid repeating dimension checks at every callsite.
    pub fn new_with_dimension(dimension: u32, vec: impl Into<SmallVec<[C; 8]>>) -> Self {
        let coords = vec.into();
        debug_assert_eq!(
            coords.len() as u32,
            dimension,
            "Point dimension mismatch: expected {dimension}, got {}",
            coords.len()
        );
        Self(coords)
    }

    /// Calculate the Euclidean distance between two points.
    ///
    /// Preconditions: both points must have the same dimensionality and
    /// originate from the same curve. In debug builds a mismatched
    /// dimensionality triggers a `debug_assert!`. In release builds the
    /// distance is computed over the shared prefix of dimensions.
    pub fn distance(&self, p2: &Self) -> f64 {
        debug_assert!(
            self.len() == p2.len(),
            "Point::distance called with differing dimensions: {} vs {}",
            self.len(),
            p2.len()
        );

        let mut tot: u128 = 0;
        for (a, b) in self.0.iter().zip(p2.0.iter()) {
            let a = a.to_u128().expect("coordinate fits into u128");
            let b = b.to_u128().expect("coordinate fits into u128");
            let d = a.abs_diff(b);
            tot += d * d;
        }
        (tot as f64).sqrt()
    }

    /// Return the point's coordinates as a slice.
    pub fn as_slice(&self) -> &[C] {
        &self.0
    }

    /// Dimensionality of the point.
    pub fn dimension(&self) -> u32 {
        self.0.len() as u32
    }
}

impl<C: Coord> From<Point<C>> for Vec<C> {
    fn from(val: Point<C>) -> Self {
        val.0.to_vec()
    }
}

impl<C: Coord> From<&Point<C>> for Vec<C> {
    fn from(val: &Point<C>) -> Self {
        val.0.to_vec()
    }
}

impl<C: Coord> Deref for Point<C> {
    type Target = [C];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error;

    #[test]
    fn point() -> error::Result<()> {
        let v = Point::new(vec![2_u32, 2_u32]);
        assert_eq!(v.len(), 2);
        Ok(())
    }

    #[test]
    fn distance() -> error::Result<()> {
        let a = Point::new(vec![2_u32, 2_u32]);
        let b = Point::new(vec![2_u32, 1_u32]);
        assert_eq!(a.distance(&b), 1.0);

        let a = Point::new(vec![2_u32, 2_u32]);
        let b = Point::new(vec![0_u32, 2_u32]);
        assert_eq!(a.distance(&b), 2.0);

        let a = Point::new(vec![0_u32, 2_u32]);
        let b = Point::new(vec![0_u32, 0_u32]);
        assert_eq!(a.distance(&b), 2.0);

        let a = Point::new(vec![0_u32, 2_u32]);
        let b = Point::new(vec![0_u32, 2_u32]);
        assert_eq!(a.distance(&b), 0.0);

        Ok(())
    }

    #[test]
    fn distance_handles_large_u64() {
        let a = Point::new(vec![u64::MAX, 0]);
        let b = Point::new(vec![u64::MAX - 2, 0]);
        assert_eq!(a.distance(&b), 2.0);

        let coords: Vec<u64> = (&a).into();
        assert_eq!(coords, vec![u64::MAX, 0]);
    }
}
