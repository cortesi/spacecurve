//! Grid specification helpers used by curve constructors and registry validation.

use crate::{error, error::Error};

/// Describes the dimensionality and side length of a grid along with derived values.
///
/// The helper centralizes guard logic (non‑zero sizes, power‑of‑two checks, overflow checks)
/// so curve constructors can focus on their own algorithmic invariants.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GridSpec {
    /// Number of dimensions in the grid.
    dimension: u32,
    /// Side length per dimension.
    size: u32,
    /// Total number of points (`size^dimension`).
    length: u32,
    /// Order (bits per axis) when `size` is a power of two.
    order: Option<u32>,
    /// Bit width per axis when `size` is a power of two.
    bits_per_axis: Option<u32>,
}

impl GridSpec {
    /// Construct a spec for any grid (no power‑of‑two requirement).
    ///
    /// - `dimension` must be >= 1
    /// - `size` must be >= 1
    /// - `size.pow(dimension)` must fit inside `u32`
    pub fn new(dimension: u32, size: u32) -> error::Result<Self> {
        if dimension == 0 {
            return Err(Error::Shape("dimension must be >= 1".to_string()));
        }
        if size == 0 {
            return Err(Error::Size("size must be >= 1".to_string()));
        }

        let length = size.checked_pow(dimension).ok_or_else(|| {
            Error::Size("curve length (size^dimension) exceeds u32 bounds".to_string())
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
    pub fn power_of_two(dimension: u32, size: u32) -> error::Result<Self> {
        if size == 0 || !size.is_power_of_two() {
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

    /// Require that the total number of index bits is strictly less than `limit`.
    ///
    /// Useful for curves that encode indices into `u32` using `bits_per_axis * dimension`.
    pub fn require_index_bits_lt(&self, limit: u32) -> error::Result<()> {
        if let Some(bits) = self.bits_per_axis {
            let total_bits = (bits as u64) * (self.dimension as u64);
            if total_bits >= limit as u64 {
                return Err(Error::Size(format!(
                    "index requires {total_bits} bits; must be < {limit} for u32 indices"
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
    pub fn size(&self) -> u32 {
        self.size
    }

    /// Total number of points in the grid (`size^dimension`).
    pub fn length(&self) -> u32 {
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
