use std::marker::PhantomData;

use smallvec::SmallVec;

use crate::{
    curves::{hilbert2, hilbertn},
    error, point,
    spacecurve::SpaceCurve,
    spec::GridSpec,
    types::{Coord, Index},
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
    fn index<C: Coord, I: Index>(&self, dimension: u32, order: u32, point: &[C]) -> I {
        match self {
            Self::TwoD => hilbert2::hilbert_index::<C, I>(order, point),
            Self::Nd => hilbertn::hilbert_index::<C, I>(dimension, order, point),
        }
    }

    /// Compute coordinates from an index using the chosen implementation.
    fn point<C: Coord, I: Index>(&self, dimension: u32, order: u32, index: I) -> SmallVec<[C; 8]> {
        match self {
            Self::TwoD => hilbert2::hilbert_point::<C, I>(order, index),
            Self::Nd => hilbertn::hilbert_point::<C, I>(dimension, order, index),
        }
    }
}

/// An implementation of the Hilbert curve.
#[derive(Debug)]
pub struct Hilbert<C: Coord, I: Index> {
    /// The order of the curve. The higher this is, the more points we pack into
    /// space.
    pub order: u32,
    /// The number of dimensions of the Hilbert curve.
    pub dimension: u32,
    /// Cached total number of points (`2^(order * dimension)`), computed once
    /// at construction with checked math to avoid overflow in debug/release.
    length: I,
    /// Chooses between the 2D fast path and the generic N-D logic.
    mapper: HilbertImpl,
    /// Marks the coordinate type so it remains part of the public API.
    _coord: PhantomData<C>,
}

impl<C: Coord, I: Index> Hilbert<C, I> {
    /// Construct a Hilbert curve to precisely fit a hypercube with a defined
    /// number of dimensions, and a set size in each dimension. The size must be
    /// a power of two (`size == 2^order`) or the result is an error.
    pub fn from_dimensions(dimension: u32, size: C) -> error::Result<Self> {
        let spec = GridSpec::<C, I>::power_of_two(dimension, size)?;
        spec.require_index_bits_lt(I::BITS)?;

        Ok(Self {
            dimension: spec.dimension(),
            order: spec.order().unwrap(),
            length: spec.length(),
            mapper: if spec.dimension() == 2 {
                HilbertImpl::TwoD
            } else {
                HilbertImpl::Nd
            },
            _coord: PhantomData,
        })
    }
}

impl<C: Coord, I: Index> SpaceCurve for Hilbert<C, I> {
    type Coord = C;
    type Index = I;

    fn name(&self) -> &'static str {
        "Hilbert"
    }

    fn info(&self) -> &'static str {
        "Classic continuous space-filling curve with excellent locality.\n\
        Defined recursively via rotations/reflections; widely used in GIS,\n\
        image storage, and indexing; typically clusters better than Z-order."
    }
    fn length(&self) -> I {
        self.length
    }
    fn dimensions(&self) -> u32 {
        self.dimension
    }
    fn index(&self, p: &point::Point<C>) -> I {
        debug_assert_eq!(p.len(), self.dimension as usize, "point dimension mismatch");
        let side = C::one() << self.order;
        debug_assert!(
            p.iter().all(|&c| c < side),
            "point coordinate out of bounds"
        );
        self.mapper.index(self.dimension, self.order, p)
    }
    fn point(&self, index: I) -> point::Point<C> {
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
        let h = &Hilbert::<u32, u32>::from_dimensions(2, 2)?;
        assert_eq!(h.order, 1);
        assert_eq!(h.length(), 4);

        let h = &Hilbert::<u32, u32>::from_dimensions(3, 2)?;
        assert_eq!(h.order, 1);
        assert_eq!(h.length(), 8);

        if Hilbert::<u32, u32>::from_dimensions(2, 3).is_ok() {
            panic!("expected error")
        }

        // Guard: 2D order 16 (size 2^16) would produce length 2^32 → reject
        assert!(Hilbert::<u32, u32>::from_dimensions(2, 1u32 << 16).is_err());
        // 2D order 15 → ok
        assert!(Hilbert::<u32, u32>::from_dimensions(2, 1u32 << 15).is_ok());

        Ok(())
    }
}
