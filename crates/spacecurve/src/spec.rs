//! Grid specification helpers used by curve constructors and registry
//! validation.

use crate::{
    error,
    error::Error,
    ops,
    types::{Coord, Index},
};

/// Describes the dimensionality and side length of a grid along with derived
/// values.
///
/// The helper centralizes guard logic (non‑zero sizes, power‑of‑two checks,
/// overflow checks) so curve constructors can focus on their own algorithmic
/// invariants.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GridSpec<C: Coord, I: Index> {
    /// Number of dimensions in the grid.
    dimension: u32,
    /// Side length per dimension.
    size: C,
    /// Total number of points (`size^dimension`).
    length: I,
    /// Order (bits per axis) when `size` is a power of two.
    order: Option<u32>,
    /// Bit width per axis when `size` is a power of two.
    bits_per_axis: Option<u32>,
}

impl<C: Coord, I: Index> GridSpec<C, I> {
    /// Construct a spec for any grid (no power‑of‑two requirement).
    ///
    /// - `dimension` must be >= 1
    /// - `size` must be >= 1
    /// - `size.pow(dimension)` must fit inside the index type
    pub fn new(dimension: u32, size: C) -> error::Result<Self> {
        if dimension == 0 {
            return Err(Error::Shape("dimension must be >= 1".to_string()));
        }
        if size.is_zero() {
            return Err(Error::Size("size must be >= 1".to_string()));
        }

        let size_index =
            I::from(size).ok_or_else(|| Error::Size("size exceeds index bounds".to_string()))?;
        let length = ops::checked_pow(size_index, dimension).ok_or_else(|| {
            Error::Size(format!(
                "curve length (size^dimension) exceeds {} bounds",
                I::BITS
            ))
        })?;

        Ok(Self {
            dimension,
            size,
            length,
            order: None,
            bits_per_axis: None,
        })
    }

    /// Construct a spec requiring `size` to be a positive power of two.
    ///
    /// Populates `order` and `bits_per_axis` with `size.trailing_zeros()`.
    pub fn power_of_two(dimension: u32, size: C) -> error::Result<Self> {
        if size.is_zero() || (size & (size - C::one())) != C::zero() {
            return Err(Error::Size(
                "size must be a positive power of two".to_string(),
            ));
        }

        let mut spec = Self::new(dimension, size)?;
        let order = size.trailing_zeros();
        spec.order = Some(order);
        spec.bits_per_axis = Some(order);
        Ok(spec)
    }

    /// Require that the total number of index bits is strictly less than
    /// `limit`.
    ///
    /// Useful for curves that encode indices using `bits_per_axis * dimension`.
    pub fn require_index_bits_lt(&self, limit: u32) -> error::Result<()> {
        if let Some(bits) = self.bits_per_axis {
            let total_bits = (bits as u128) * (self.dimension as u128);
            if total_bits >= limit as u128 {
                return Err(Error::Size(format!(
                    "index requires {total_bits} bits; must be < {limit} for index type"
                )));
            }
        }
        Ok(())
    }

    /// Dimension count.
    pub fn dimension(&self) -> u32 {
        self.dimension
    }

    /// Side length.
    pub fn size(&self) -> C {
        self.size
    }

    /// Total number of points in the grid (`size^dimension`).
    pub fn length(&self) -> I {
        self.length
    }

    /// Order for power‑of‑two grids (when available).
    pub fn order(&self) -> Option<u32> {
        self.order
    }

    /// Bit width per coordinate for power‑of‑two grids (when available).
    pub fn bits_per_axis(&self) -> Option<u32> {
        self.bits_per_axis
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grid_spec_rejects_u32_overflow() {
        let err = GridSpec::<u32, u32>::new(4, 1024).unwrap_err();
        assert!(matches!(err, Error::Size(_)));
    }

    #[test]
    fn grid_spec_accepts_u64_for_large_4d() -> error::Result<()> {
        let spec = GridSpec::<u32, u64>::new(4, 1024)?;
        assert_eq!(spec.length(), 1_099_511_627_776u64);
        Ok(())
    }
}
