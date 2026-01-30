use smallvec::smallvec;

use crate::{
    error, ops,
    point::Point,
    spacecurve::SpaceCurve,
    spec::GridSpec,
    types::{Coord, Index},
};

/// Serpentine row/column scan across an N‑D grid.
#[derive(Debug)]
pub struct Scan<C: Coord, I: Index> {
    /// Number of dimensions in the grid.
    dimension: u32,
    /// Side length per dimension.
    size: C,
    /// Cached total number of points in the scan.
    length: I,
}

impl<C: Coord, I: Index> Scan<C, I> {
    /// Construct a `Scan` curve for the given dimensions and side length.
    pub fn from_dimensions(dimension: u32, size: C) -> error::Result<Self> {
        let spec = GridSpec::<C, I>::new(dimension, size)?;
        Ok(Self {
            dimension: spec.dimension(),
            size: spec.size(),
            length: spec.length(),
        })
    }

    /// Convert the side length to the index type for arithmetic.
    fn size_index(&self) -> I {
        I::from(self.size).expect("size fits in index type")
    }
}

impl<C: Coord, I: Index> SpaceCurve for Scan<C, I> {
    type Coord = C;
    type Index = I;

    fn name(&self) -> &'static str {
        "Scan"
    }

    fn info(&self) -> &'static str {
        "Serpentine raster scan (boustrophedon) across rows/columns.\n\
        Continuous with minimal turning, but locality drops at row boundaries.\n\
        Useful as a simple, predictable baseline traversal."
    }
    fn length(&self) -> I {
        self.length
    }
    fn dimensions(&self) -> u32 {
        self.dimension
    }

    /// Convert a 1D index into N-dimensional coordinates.
    ///
    /// The scan performs a boustrophedon (ox-turning) traversal. This means
    /// every other row/plane is traversed in reverse order to maintain
    /// continuity between lines.
    fn point(&self, index: I) -> Point<C> {
        debug_assert!(index < self.length, "index out of bounds");
        // Tracks whether the current dimension should be traversed in reverse.
        let mut should_reverse_direction = false;
        let mut coordinates = smallvec![C::zero(); self.dimension as usize];
        let mut remaining_index = index;
        let size_index = self.size_index();

        // Iterate dimensions from highest to lowest (e.g., Z -> Y -> X)
        for dim_idx in (0..self.dimension).rev() {
            let stride = ops::checked_pow(size_index, dim_idx).expect("stride fits index type");
            let raw_coordinate = remaining_index / stride;

            // If we are in a reversed section, invert the coordinate
            let raw_coordinate = C::from(raw_coordinate).expect("coordinate fits coordinate type");
            coordinates[dim_idx as usize] = if should_reverse_direction {
                self.size - raw_coordinate - C::one()
            } else {
                raw_coordinate
            };

            // Determine if the next lower dimension needs to be reversed.
            // If the current coordinate is odd, the next dimension (nested inside)
            // will be scanned backwards.
            if (coordinates[dim_idx as usize] & C::one()) == C::one() {
                should_reverse_direction = !should_reverse_direction;
            }

            remaining_index = remaining_index
                - stride * I::from(raw_coordinate).expect("coordinate fits index type");
        }
        Point::new_with_dimension(self.dimension, coordinates)
    }

    /// Convert N-dimensional coordinates into a 1D index.
    fn index(&self, point: &Point<C>) -> I {
        debug_assert_eq!(
            point.len(),
            self.dimension as usize,
            "point dimension mismatch"
        );
        debug_assert!(
            point.iter().all(|&c| c < self.size),
            "point coordinate out of bounds"
        );
        let mut should_reverse_direction = false;
        let mut index_accumulator = I::zero();
        let size_index = self.size_index();

        // Iterate dimensions from highest to lowest to reconstruct the index
        for (dim_idx, &coordinate) in point.iter().enumerate().rev() {
            let stride =
                ops::checked_pow(size_index, dim_idx as u32).expect("stride fits index type");

            let actual_value = if should_reverse_direction {
                self.size - coordinate - C::one()
            } else {
                coordinate
            };

            index_accumulator = index_accumulator
                + I::from(actual_value).expect("coordinate fits index type") * stride;

            // Update direction flip state for the next dimension
            if (coordinate & C::one()) == C::one() {
                should_reverse_direction = !should_reverse_direction;
            }
        }
        index_accumulator
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scan_point_simple() {
        let s = Scan::<u32, u32>::from_dimensions(2, 3).unwrap();
        assert_eq!(s.point(0), Point::new(vec![0, 0]));
        assert_eq!(s.point(1), Point::new(vec![1, 0]));
        assert_eq!(s.point(2), Point::new(vec![2, 0]));
        assert_eq!(s.point(3), Point::new(vec![2, 1]));
        assert_eq!(s.point(8), Point::new(vec![2, 2]));
    }

    #[test]
    fn test_scan_index_simple() {
        let s = Scan::<u32, u32>::from_dimensions(2, 3).unwrap();
        assert_eq!(s.index(&Point::new(vec![0, 0])), 0);
        assert_eq!(s.index(&Point::new(vec![1, 0])), 1);
        assert_eq!(s.index(&Point::new(vec![2, 0])), 2);
        assert_eq!(s.index(&Point::new(vec![2, 1])), 3);
        assert_eq!(s.index(&Point::new(vec![2, 2])), 8);
    }

    #[test]
    fn guard_matches_registry() {
        assert!(Scan::<u32, u32>::from_dimensions(0, 3).is_err());
        assert!(Scan::<u32, u32>::from_dimensions(2, 0).is_err());
    }

    #[test]
    fn full_sequence_2d_snake() {
        let s = Scan::<u32, u32>::from_dimensions(2, 3).unwrap();
        let expected = vec![
            vec![0, 0],
            vec![1, 0],
            vec![2, 0],
            vec![2, 1],
            vec![1, 1],
            vec![0, 1],
            vec![0, 2],
            vec![1, 2],
            vec![2, 2],
        ];
        for (idx, coords) in expected.iter().enumerate() {
            assert_eq!(Vec::<u32>::from(s.point(idx as u32)), *coords);
            assert_eq!(s.index(&Point::new(coords.clone())), idx as u32);
        }
    }

    #[test]
    fn roundtrip_three_dimensions() {
        let s = Scan::<u32, u32>::from_dimensions(3, 3).unwrap();
        for idx in 0..s.length() {
            let p = s.point(idx);
            assert_eq!(s.index(&p), idx, "roundtrip failed at {idx}");
        }
    }
}
