use crate::{error, ops, point::Point, spacecurve::SpaceCurve, spec::GridSpec};

/// Gray-code based hypercube traversal (BRGC).
#[derive(Debug)]
pub struct Gray {
    /// Number of dimensions in the grid.
    dimension: u32,
    /// Side length per dimension.
    size: u32,
    /// Cached bit width per coordinate (size is always a power of two).
    bits_per_axis: u32,
    /// Cached total number of points in the curve.
    length: u32,
}

impl Gray {
    /// Construct a `Gray` curve for the given dimensions and side length.
    ///
    /// The dimension and size must each be at least 1, and the size must be a
    /// power of two so the Binary Reflected Gray Code remains bijective across
    /// the hypercube.
    pub fn from_dimensions(dimension: u32, size: u32) -> error::Result<Self> {
        let spec = GridSpec::power_of_two(dimension, size)?;
        spec.require_index_bits_lt(32)?;

        Ok(Self {
            dimension: spec.dimension(),
            size: spec.size(),
            bits_per_axis: spec.bits_per_axis().unwrap(),
            length: spec.length(),
        })
    }
}

impl SpaceCurve for Gray {
    fn name(&self) -> &'static str {
        "Gray (BRGC)"
    }

    fn info(&self) -> &'static str {
        "Hypercube traversal using Binary Reflected Gray Code so adjacent\n\
        indices differ by one bit. Requires power-of-two side lengths; fast,\n\
        but spatial locality is weaker than Hilbert/H-curve."
    }
    fn length(&self) -> u32 {
        self.length
    }

    fn dimensions(&self) -> u32 {
        self.dimension
    }

    fn point(&self, index: u32) -> Point {
        debug_assert!(index < self.length, "index out of range");

        // Convert the linear index to Gray code, then deinterleave the bits
        // across coordinates using the same bit layout as Morton order.
        let gray_index = ops::graycode(index);
        Point::new_with_dimension(
            self.dimension,
            ops::deinterleave_lsb(self.dimension, self.bits_per_axis, gray_index),
        )
    }

    fn index(&self, p: &Point) -> u32 {
        debug_assert_eq!(p.len(), self.dimension as usize, "point dimension mismatch");
        debug_assert!(
            p.iter().all(|&coord| coord < self.size),
            "point coordinate out of bounds"
        );

        let gray_index = ops::interleave_lsb(&p[..], self.bits_per_axis);
        let binary_index = ops::igraycode(gray_index);
        debug_assert!(binary_index < self.length, "index conversion overflowed");
        binary_index
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_roundtrip(dimension: u32, size: u32) {
        let gray = Gray::from_dimensions(dimension, size).unwrap();
        for i in 0..gray.length() {
            let point = gray.point(i);
            assert_eq!(
                i,
                gray.index(&point),
                "roundtrip failed at {i} dim {dimension}"
            );
        }
    }

    fn assert_adjacency(gray: &Gray) {
        for i in 1..gray.length() {
            let prev: Vec<u32> = gray.point(i - 1).into();
            let curr: Vec<u32> = gray.point(i).into();
            let mut diff_axes = 0;
            let mut l1 = 0u32;
            for (a, b) in prev.iter().zip(curr.iter()) {
                if a != b {
                    diff_axes += 1;
                    l1 += (*a as i32 - *b as i32).unsigned_abs();
                }
            }
            assert_eq!(
                diff_axes, 1,
                "points {prev:?} and {curr:?} differ in {diff_axes} axes"
            );
            assert_eq!(
                l1, 1,
                "points {prev:?} and {curr:?} differ by {l1} (expected 1-step adjacency)"
            );
        }
    }

    #[test]
    fn test_gray_constructor_rejects_invalid_sizes() {
        assert!(Gray::from_dimensions(0, 2).is_err());
        assert!(Gray::from_dimensions(2, 0).is_err());
        assert!(Gray::from_dimensions(2, 3).is_err());
    }

    #[test]
    fn test_gray_2d_simple() {
        let gray = Gray::from_dimensions(2, 4).unwrap();

        // Test some basic mappings
        let p0 = gray.point(0);
        assert_eq!(gray.index(&p0), 0);

        let p1 = gray.point(1);
        assert_eq!(gray.index(&p1), 1);

        // Test that adjacent indices produce points that differ in exactly one coordinate
        let p0_coords: Vec<u32> = p0.into();
        let p1_coords: Vec<u32> = p1.into();
        let diff_count = p0_coords
            .iter()
            .zip(p1_coords.iter())
            .filter(|(a, b)| a != b)
            .count();
        assert_eq!(diff_count, 1);
    }

    #[test]
    fn test_gray_roundtrip() {
        assert_roundtrip(2, 4);
    }

    #[test]
    fn test_gray_neighbourhood() {
        let gray2 = Gray::from_dimensions(2, 2).unwrap();
        let expected = [vec![0, 0], vec![1, 0], vec![1, 1], vec![0, 1]];
        for (idx, coords) in expected.iter().enumerate() {
            assert_eq!(Vec::<u32>::from(gray2.point(idx as u32)), *coords);
        }

        // For 3D hypercube with size 2 ensure adjacency differs by one coordinate.
        let gray3 = Gray::from_dimensions(3, 2).unwrap();
        for i in 1..gray3.length() {
            let prev: Vec<u32> = gray3.point(i - 1).into();
            let curr: Vec<u32> = gray3.point(i).into();
            let mut diff_count = 0;
            let mut diff_magnitude = 0u32;
            for (a, b) in prev.iter().zip(curr.iter()) {
                if a != b {
                    diff_count += 1;
                    diff_magnitude += (*a as i32 - *b as i32).unsigned_abs();
                }
            }
            assert_eq!(
                diff_count, 1,
                "points {prev:?} and {curr:?} differ in {diff_count} axes"
            );
            assert_eq!(
                diff_magnitude, 1,
                "points {prev:?} and {curr:?} differ by {diff_magnitude}"
            );
        }
    }

    #[test]
    fn test_gray_roundtrip_dims_up_to_four() {
        for dim in 1..=4 {
            assert_roundtrip(dim, 2);
        }
    }

    #[test]
    fn test_gray_adjacency_dims_up_to_four() {
        for dim in 1..=4 {
            let curve = Gray::from_dimensions(dim, 2).unwrap();
            assert_adjacency(&curve);
        }
    }
}
