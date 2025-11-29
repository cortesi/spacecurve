use crate::{error, ops, point, spacecurve::SpaceCurve, spec::GridSpec};

/// An implementation of the Z Order curve.
#[derive(Debug)]
pub struct ZOrder {
    /// The bit width of each co-ordinate
    pub bitwidth: u32,
    /// The number of dimensions
    pub dimension: u32,
    /// Cached total number of points (`2^(bitwidth * dimension)`), computed
    /// once at construction with checked math to avoid overflow.
    length: u32,
}

impl ZOrder {
    /// Construct a Z Order curve to precisely fit a hypercube with a defined
    /// number of dimensions, and a set size in each dimension. The size must be
    /// a number 2**n, where n is an integer, or the result is an error.
    pub fn from_dimensions(dimension: u32, size: u32) -> error::Result<Self> {
        let spec = GridSpec::power_of_two(dimension, size)?;
        spec.require_index_bits_lt(32)?;
        let bitwidth = spec.bits_per_axis().unwrap();
        Ok(Self {
            dimension: spec.dimension(),
            bitwidth,
            length: spec.length(),
        })
    }
}

impl SpaceCurve for ZOrder {
    fn name(&self) -> &'static str {
        "Z-order (Morton)"
    }

    fn info(&self) -> &'static str {
        "Interleaves coordinate bits to form keys (Morton code).\n\
        Extremely fast and pairs well with quad/oct-trees, but preserves\n\
        neighborhood worse than Hilbert/H-curve and may exhibit long jumps."
    }
    fn length(&self) -> u32 {
        self.length
    }
    fn dimensions(&self) -> u32 {
        self.dimension
    }
    fn point(&self, index: u32) -> point::Point {
        debug_assert!(index < self.length, "index out of range");
        point::Point::new_with_dimension(
            self.dimension,
            ops::deinterleave_lsb(self.dimension, self.bitwidth, index),
        )
    }
    fn index(&self, p: &point::Point) -> u32 {
        debug_assert_eq!(p.len(), self.dimension as usize, "point dimension mismatch");
        let side = if self.bitwidth == 0 {
            1
        } else {
            1u32 << self.bitwidth
        };
        debug_assert!(
            p.iter().all(|&coord| coord < side),
            "point coordinate out of bounds"
        );
        ops::interleave_lsb(&p[..], self.bitwidth)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_dimensions_guard() {
        // 2 * 16 = 32 bits total → reject
        assert!(ZOrder::from_dimensions(2, 1u32 << 16).is_err());
        // 2 * 15 = 30 bits total → ok
        assert!(ZOrder::from_dimensions(2, 1u32 << 15).is_ok());
        // 4 * 8 = 32 bits total → reject
        assert!(ZOrder::from_dimensions(4, 1u32 << 8).is_err());
        // 4 * 7 = 28 bits total → ok
        assert!(ZOrder::from_dimensions(4, 1u32 << 7).is_ok());
    }

    #[test]
    fn sequence_matches_reference_2d() {
        let curve = ZOrder::from_dimensions(2, 4).unwrap();
        let expected = vec![
            vec![0, 0],
            vec![1, 0],
            vec![0, 1],
            vec![1, 1],
            vec![2, 0],
            vec![3, 0],
            vec![2, 1],
            vec![3, 1],
            vec![0, 2],
            vec![1, 2],
            vec![0, 3],
            vec![1, 3],
            vec![2, 2],
            vec![3, 2],
            vec![2, 3],
            vec![3, 3],
        ];
        for (idx, coords) in expected.iter().enumerate() {
            assert_eq!(Vec::<u32>::from(curve.point(idx as u32)), *coords);
        }
    }

    #[test]
    fn roundtrip_holds_for_small_cases() {
        let curve = ZOrder::from_dimensions(3, 4).unwrap();
        for i in 0..curve.length() {
            let point = curve.point(i);
            assert_eq!(curve.index(&point), i);
        }
    }

    #[test]
    fn roundtrip_dims_up_to_four() {
        for dim in 1..=4 {
            let curve = ZOrder::from_dimensions(dim, 2).unwrap();
            for i in 0..curve.length() {
                let point = curve.point(i);
                assert_eq!(
                    curve.index(&point),
                    i,
                    "roundtrip failed for dim {dim} at {i}"
                );
            }
        }
    }
}
