use crate::{
    curves::onion::{onion_index_2d, onion_point_2d},
    error,
    point::Point,
    spacecurve::SpaceCurve,
    spec::GridSpec,
};

/// A continuous N-dimensional generalization of the Onion Curve.
/// It relaxes strict layering constraints (impossible for N>=3) by tiling the space
/// with continuous 2D Onion spirals connected via snake ordering.
#[derive(Debug)]
pub struct HairyOnionCurve {
    /// Number of dimensions in the grid.
    dimensions: u32,
    /// Side length per dimension.
    side_length: u32,
    /// Total number of points (L^N).
    length: u32,
}

impl HairyOnionCurve {
    /// Construct a new Hairy Onion curve for `dimensions` and `side_length`.
    pub fn new(dimensions: u32, side_length: u32) -> error::Result<Self> {
        let spec = GridSpec::new(dimensions, side_length)?;
        Ok(Self {
            dimensions: spec.dimension(),
            side_length: spec.size(),
            length: spec.length(),
        })
    }
}

impl SpaceCurve for HairyOnionCurve {
    fn name(&self) -> &'static str {
        "Hairy Onion"
    }

    fn info(&self) -> &'static str {
        "A stacked variant of the Onion curve."
    }
    fn dimensions(&self) -> u32 {
        self.dimensions
    }

    fn length(&self) -> u32 {
        self.length
    }

    fn index(&self, p: &Point) -> u32 {
        debug_assert_eq!(
            p.len(),
            self.dimensions as usize,
            "point dimension mismatch"
        );
        debug_assert!(
            p.iter().all(|&c| c < self.side_length),
            "point coordinate out of bounds"
        );
        hairy_onion_index_recursive(self.dimensions, self.side_length, p)
    }

    fn point(&self, index: u32) -> Point {
        debug_assert!(index < self.length, "index out of bounds");
        let coords =
            hairy_onion_point_recursive(self.dimensions, self.side_length, index % self.length);
        Point::new_with_dimension(self.dimensions, coords)
    }
}

// --- Generalized N-D Hairy Onion Implementation (Tiled 2D Onion) ---

// Helper function to calculate the index recursively.
/// Recursive index for the N‑D Hairy Onion using tiled 2D onions.
fn hairy_onion_index_recursive(n: u32, l: u32, p: &[u32]) -> u32 {
    // Base cases
    if l <= 1 || n == 0 {
        return 0;
    }

    // Base Case N=1: Linear Scan
    if n == 1 {
        return p[0];
    }

    // Base Case N=2: Standard 2D Onion
    if n == 2 {
        return onion_index_2d(l, p);
    }

    // Recursive Step N>2: Tiled 2D Onion with Snake Ordering

    // 1. Divide the point: First 2 dimensions and the remaining N-2 dimensions.
    let p_2d = &p[0..2];
    let p_rest = &p[2..];

    // 2. Calculate recursive index for the remaining dimensions (The Tile Index)
    let index_rest = hairy_onion_index_recursive(n - 2, l, p_rest);

    // 3. Calculate the 2D index (Index within the tile)
    let index_2d = onion_index_2d(l, p_2d);
    let volume_2d = l * l;

    // 4. Apply Snake ordering (reversal) for continuity based on the Tile Index parity
    let index_2d_effective = if index_rest % 2 == 1 {
        (volume_2d - 1) - index_2d
    } else {
        index_2d
    };

    // 5. Combine indices
    index_rest * volume_2d + index_2d_effective
}

// Helper function to calculate the point from the index recursively (Inverse mapping).
/// Inverse of `hairy_onion_index_recursive`: recover coordinates from index.
fn hairy_onion_point_recursive(n: u32, l: u32, index: u32) -> Vec<u32> {
    if n == 0 {
        return vec![];
    }
    if l == 1 {
        return vec![0; n as usize];
    }
    if l == 0 {
        unreachable!("L==0 is rejected by HairyOnionCurve::new");
    }

    // Base Case N=1
    if n == 1 {
        return vec![index];
    }

    // Base Case N=2
    if n == 2 {
        return onion_point_2d(l, index);
    }

    // Recursive Step N>2

    let volume_2d = l * l;

    // 1. Decompose the index
    let index_rest = index / volume_2d; // Tile index
    let index_2d_effective = index % volume_2d; // Index within tile (potentially reversed)

    // 2. Calculate P_rest recursively (Inverse Tile Index)
    let p_rest = hairy_onion_point_recursive(n - 2, l, index_rest);

    // 3. Determine the actual Index_2D by inverting the Snake reversal
    let index_2d = if index_rest % 2 == 1 {
        (volume_2d - 1) - index_2d_effective
    } else {
        index_2d_effective
    };

    // 4. Calculate P_2D (Point within the tile)
    let p_2d = onion_point_2d(l, index_2d);

    // 5. Combine the points
    let mut p = p_2d;
    p.extend(p_rest);
    p
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constructor_guards() {
        // L==0 rejected
        assert!(HairyOnionCurve::new(2, 0).is_err());
        // N==0 rejected
        assert!(HairyOnionCurve::new(0, 4).is_err());
        // Valid shapes
        let c = HairyOnionCurve::new(2, 3).unwrap();
        assert_eq!(c.length(), 9);
    }

    #[test]
    fn roundtrip_dims_2_to_4_sizes_upto_8() {
        for dim in 2..=4 {
            for size in 2..=8 {
                let curve = HairyOnionCurve::new(dim, size).unwrap();
                for idx in 0..curve.length() {
                    let p = curve.point(idx);
                    assert_eq!(
                        curve.index(&p),
                        idx,
                        "roundtrip failed for dim {dim}, size {size}, idx {idx}"
                    );
                }
            }
        }
    }
}
