use std::iter::Iterator;

use smallvec::smallvec;

use crate::{error, point::Point, spacecurve::SpaceCurve, spec::GridSpec};

/// Serpentine row/column scan across an N‑D grid.
#[derive(Debug)]
pub struct Scan {
    /// Number of dimensions in the grid.
    dimension: u32,
    /// Side length per dimension.
    size: u32,
    /// Cached total number of points in the scan.
    length: u32,
}

impl Scan {
    /// Construct a `Scan` curve for the given dimensions and side length.
    pub fn from_dimensions(dimension: u32, size: u32) -> error::Result<Self> {
        let spec = GridSpec::new(dimension, size)?;
        Ok(Self {
            dimension: spec.dimension(),
            size: spec.size(),
            length: spec.length(),
        })
    }
}

impl SpaceCurve for Scan {
    fn name(&self) -> &'static str {
        "Scan"
    }

    fn info(&self) -> &'static str {
        "Serpentine raster scan (boustrophedon) across rows/columns.\n\
        Continuous with minimal turning, but locality drops at row boundaries.\n\
        Useful as a simple, predictable baseline traversal."
    }
    fn length(&self) -> u32 {
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
    fn point(&self, index: u32) -> Point {
        debug_assert!(index < self.length, "index out of bounds");
        // Tracks whether the current dimension should be traversed in reverse.
        let mut should_reverse_direction = false;
        let mut coordinates = smallvec![0; self.dimension as usize];
        let mut remaining_index = index;

        // Iterate dimensions from highest to lowest (e.g., Z -> Y -> X)
        for dim_idx in (0..self.dimension).rev() {
            let stride = self.size.pow(dim_idx);
            let raw_coordinate = remaining_index / stride;

            // If we are in a reversed section, invert the coordinate
            coordinates[dim_idx as usize] = if should_reverse_direction {
                self.size - raw_coordinate - 1
            } else {
                raw_coordinate
            };

            // Determine if the next lower dimension needs to be reversed.
            // If the current coordinate is odd, the next dimension (nested inside)
            // will be scanned backwards.
            if coordinates[dim_idx as usize] % 2 != 0 {
                should_reverse_direction = !should_reverse_direction;
            }

            remaining_index -= raw_coordinate * stride;
        }
        Point::new_with_dimension(self.dimension, coordinates)
    }

    /// Convert N-dimensional coordinates into a 1D index.
    fn index(&self, point: &Point) -> u32 {
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
        let mut index_accumulator = 0;

        // Iterate dimensions from highest to lowest to reconstruct the index
        for (dim_idx, &coordinate) in point.iter().enumerate().rev() {
            let stride = self.size.pow(dim_idx as u32);

            let actual_value = if should_reverse_direction {
                self.size - coordinate - 1
            } else {
                coordinate
            };

            index_accumulator += actual_value * stride;

            // Update direction flip state for the next dimension
            if coordinate % 2 != 0 {
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
        let s = Scan::from_dimensions(2, 3).unwrap();
        assert_eq!(s.point(0), Point::new(vec![0, 0]));
        assert_eq!(s.point(1), Point::new(vec![1, 0]));
        assert_eq!(s.point(2), Point::new(vec![2, 0]));
        assert_eq!(s.point(3), Point::new(vec![2, 1]));
        assert_eq!(s.point(8), Point::new(vec![2, 2]));
    }

    #[test]
    fn test_scan_index_simple() {
        let s = Scan::from_dimensions(2, 3).unwrap();
        assert_eq!(s.index(&Point::new(vec![0, 0])), 0);
        assert_eq!(s.index(&Point::new(vec![1, 0])), 1);
        assert_eq!(s.index(&Point::new(vec![2, 0])), 2);
        assert_eq!(s.index(&Point::new(vec![2, 1])), 3);
        assert_eq!(s.index(&Point::new(vec![2, 2])), 8);
    }

    #[test]
    fn guard_matches_registry() {
        assert!(Scan::from_dimensions(0, 3).is_err());
        assert!(Scan::from_dimensions(2, 0).is_err());
    }

    #[test]
    fn full_sequence_2d_snake() {
        let s = Scan::from_dimensions(2, 3).unwrap();
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
        let s = Scan::from_dimensions(3, 3).unwrap();
        for idx in 0..s.length() {
            let p = s.point(idx);
            assert_eq!(s.index(&p), idx, "roundtrip failed at {idx}");
        }
    }
}
