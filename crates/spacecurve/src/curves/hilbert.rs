use smallvec::SmallVec;

use crate::{
    curves::{hilbert2, hilbertn},
    error, point,
    spacecurve::SpaceCurve,
    spec::GridSpec,
};

/// Internal dispatcher selecting the 2D or N-D Hilbert core.
#[derive(Debug, Clone, Copy)]
enum HilbertImpl {
    /// Optimised specialised 2D implementation.
    TwoD,
    /// Generic N-dimensional mapping.
    Nd,
}

impl HilbertImpl {
    /// Compute a Hilbert index using the chosen implementation.
    fn index(&self, dimension: u32, order: u32, point: &[u32]) -> u32 {
        match self {
            Self::TwoD => hilbert2::hilbert_index(order, point),
            Self::Nd => hilbertn::hilbert_index(dimension, order, point),
        }
    }

    /// Compute coordinates from an index using the chosen implementation.
    fn point(&self, dimension: u32, order: u32, index: u32) -> SmallVec<[u32; 4]> {
        match self {
            Self::TwoD => hilbert2::hilbert_point(order, index),
            Self::Nd => hilbertn::hilbert_point(dimension, order, index),
        }
    }
}

/// An implementation of the Hilbert curve.
#[derive(Debug)]
pub struct Hilbert {
    /// The order of the curve. The higher this is, the more points we pack into
    /// space.
    pub order: u32,
    /// The number of dimensions of the Hilbert curve.
    pub dimension: u32,
    /// Cached total number of points (`2^(order * dimension)`), computed once
    /// at construction with checked math to avoid overflow in debug/release.
    length: u32,
    /// Chooses between the 2D fast path and the generic N-D logic.
    mapper: HilbertImpl,
}

impl Hilbert {
    /// Construct a Hilbert curve to precisely fit a hypercube with a defined
    /// number of dimensions, and a set size in each dimension. The size must be
    /// a power of two (`size == 2^order`) or the result is an error.
    pub fn from_dimensions(dimension: u32, size: u32) -> error::Result<Self> {
        let spec = GridSpec::power_of_two(dimension, size)?;
        spec.require_index_bits_lt(32)?;

        Ok(Self {
            dimension: spec.dimension(),
            order: spec.order().unwrap(),
            length: spec.length(),
            mapper: if spec.dimension() == 2 {
                HilbertImpl::TwoD
            } else {
                HilbertImpl::Nd
            },
        })
    }
}

impl SpaceCurve for Hilbert {
    fn name(&self) -> &'static str {
        "Hilbert"
    }

    fn info(&self) -> &'static str {
        "Classic continuous space-filling curve with excellent locality.\n\
        Defined recursively via rotations/reflections; widely used in GIS,\n\
        image storage, and indexing; typically clusters better than Z-order."
    }
    fn length(&self) -> u32 {
        self.length
    }
    fn dimensions(&self) -> u32 {
        self.dimension
    }
    fn index(&self, p: &point::Point) -> u32 {
        debug_assert_eq!(p.len(), self.dimension as usize, "point dimension mismatch");
        let side = 1u32 << self.order;
        debug_assert!(
            p.iter().all(|&c| c < side),
            "point coordinate out of bounds"
        );
        self.mapper.index(self.dimension, self.order, p)
    }
    fn point(&self, index: u32) -> point::Point {
        let len = self.length;
        debug_assert!(index < len, "index out of bounds");
        point::Point::new_with_dimension(
            self.dimension,
            self.mapper.point(self.dimension, self.order, index % len),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_dimensions() -> error::Result<()> {
        let h = &Hilbert::from_dimensions(2, 2)?;
        assert_eq!(h.order, 1);
        assert_eq!(h.length(), 4);

        let h = &Hilbert::from_dimensions(3, 2)?;
        assert_eq!(h.order, 1);
        assert_eq!(h.length(), 8);

        if Hilbert::from_dimensions(2, 3).is_ok() {
            panic!("expected error")
        }

        // Guard: 2D order 16 (size 2^16) would produce length 2^32 → reject
        assert!(Hilbert::from_dimensions(2, 1u32 << 16).is_err());
        // 2D order 15 → ok
        assert!(Hilbert::from_dimensions(2, 1u32 << 15).is_ok());

        Ok(())
    }
}
