//! Lightweight N‑dimensional point type used by curve implementations.

use std::{ops::Deref, vec::Vec};

use smallvec::SmallVec;

/// Compact N‑dimensional point wrapper used by curves.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Point(pub SmallVec<[u32; 4]>);

impl Point {
    /// Create a new `Point` from a backing vector.
    pub fn new(vec: impl Into<SmallVec<[u32; 4]>>) -> Self {
        Self(vec.into())
    }

    /// Create a new `Point`, asserting the coordinate count matches `dimension`.
    ///
    /// This is a convenience to avoid repeating dimension checks at every callsite.
    pub fn new_with_dimension(dimension: u32, vec: impl Into<SmallVec<[u32; 4]>>) -> Self {
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
            let d = (*a as i128 - *b as i128).abs();
            tot += (d * d) as u128;
        }
        (tot as f64).sqrt()
    }

    /// Return the point's coordinates as a slice.
    pub fn as_slice(&self) -> &[u32] {
        &self.0
    }

    /// Dimensionality of the point.
    pub fn dimension(&self) -> u32 {
        self.0.len() as u32
    }
}

impl From<Point> for Vec<u32> {
    fn from(val: Point) -> Self {
        val.0.to_vec()
    }
}

impl From<&Point> for Vec<u32> {
    fn from(val: &Point) -> Self {
        val.0.to_vec()
    }
}

impl Deref for Point {
    type Target = [u32];
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
        let v = Point::new(vec![2, 2]);
        assert_eq!(v.len(), 2);
        Ok(())
    }

    #[test]
    fn distance() -> error::Result<()> {
        let a = Point::new(vec![2, 2]);
        let b = Point::new(vec![2, 1]);
        assert_eq!(a.distance(&b), 1.0);

        let a = Point::new(vec![2, 2]);
        let b = Point::new(vec![0, 2]);
        assert_eq!(a.distance(&b), 2.0);

        let a = Point::new(vec![0, 2]);
        let b = Point::new(vec![0, 0]);
        assert_eq!(a.distance(&b), 2.0);

        let a = Point::new(vec![0, 2]);
        let b = Point::new(vec![0, 2]);
        assert_eq!(a.distance(&b), 0.0);

        Ok(())
    }
}
