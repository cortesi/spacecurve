use crate::{
    curves::onion::{onion_index_2d, onion_point_2d},
    error,
    point::Point,
    spacecurve::SpaceCurve,
    spec::GridSpec,
    types::{Coord, Index},
};

/// A continuous N-dimensional generalization of the Onion Curve.
/// It relaxes strict layering constraints (impossible for N>=3) by tiling the
/// space with continuous 2D Onion spirals connected via snake ordering.
#[derive(Debug)]
pub struct HairyOnionCurve<C: Coord, I: Index> {
    /// Number of dimensions in the grid.
    dimensions: u32,
    /// Side length per dimension.
    side_length: C,
    /// Total number of points (L^N).
    length: I,
}

impl<C: Coord, I: Index> HairyOnionCurve<C, I> {
    /// Construct a new Hairy Onion curve for `dimensions` and `side_length`.
    pub fn new(dimensions: u32, side_length: C) -> error::Result<Self> {
        let spec = GridSpec::<C, I>::new(dimensions, side_length)?;
        Ok(Self {
            dimensions: spec.dimension(),
            side_length: spec.size(),
            length: spec.length(),
        })
    }
}

impl<C: Coord, I: Index> SpaceCurve for HairyOnionCurve<C, I> {
    type Coord = C;
    type Index = I;
    fn name(&self) -> &'static str {
        "Hairy Onion"
    }

    fn info(&self) -> &'static str {
        "A stacked variant of the Onion curve."
    }
    fn dimensions(&self) -> u32 {
        self.dimensions
    }

    fn length(&self) -> I {
        self.length
    }

    fn index(&self, p: &Point<C>) -> I {
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

    fn point(&self, index: I) -> Point<C> {
        debug_assert!(index < self.length, "index out of bounds");
        let coords = hairy_onion_point_recursive::<C, I>(
            self.dimensions,
            self.side_length,
            index % self.length,
        );
        Point::new_with_dimension(self.dimensions, coords)
    }
}

// --- Generalized N-D Hairy Onion Implementation (Tiled 2D Onion) ---

// Helper function to calculate the index recursively.
/// Recursive index for the N‑D Hairy Onion using tiled 2D onions.
fn hairy_onion_index_recursive<C: Coord, I: Index>(n: u32, l: C, p: &[C]) -> I {
    // Base cases
    if l <= C::one() || n == 0 {
        return I::zero();
    }

    // Base Case N=1: Linear Scan
    if n == 1 {
        return to_index::<C, I>(p[0]);
    }

    // Base Case N=2: Standard 2D Onion
    if n == 2 {
        return onion_index_2d::<C, I>(l, p);
    }

    // Recursive Step N>2: Tiled 2D Onion with Snake Ordering

    // 1. Divide the point: First 2 dimensions and the remaining N-2 dimensions.
    let p_2d = &p[0..2];
    let p_rest = &p[2..];

    // 2. Calculate recursive index for the remaining dimensions (The Tile Index)
    let index_rest = hairy_onion_index_recursive::<C, I>(n - 2, l, p_rest);

    // 3. Calculate the 2D index (Index within the tile)
    let index_2d = onion_index_2d::<C, I>(l, p_2d);
    let l_i = to_index::<C, I>(l);
    let volume_2d = l_i.checked_mul(&l_i).expect("validated 2D volume");

    // 4. Apply Snake ordering (reversal) for continuity based on the Tile Index
    //    parity
    let index_2d_effective = if (index_rest & I::one()) == I::one() {
        (volume_2d - I::one()) - index_2d
    } else {
        index_2d
    };

    // 5. Combine indices
    index_rest * volume_2d + index_2d_effective
}

// Helper function to calculate the point from the index recursively (Inverse
// mapping).
/// Inverse of `hairy_onion_index_recursive`: recover coordinates from index.
fn hairy_onion_point_recursive<C: Coord, I: Index>(n: u32, l: C, index: I) -> Vec<C> {
    if n == 0 {
        return vec![];
    }
    if l == C::one() {
        return vec![C::zero(); n as usize];
    }
    if l.is_zero() {
        unreachable!("L==0 is rejected by HairyOnionCurve::new");
    }

    // Base Case N=1
    if n == 1 {
        return vec![to_coord::<C, I>(index)];
    }

    // Base Case N=2
    if n == 2 {
        return onion_point_2d::<C, I>(l, index);
    }

    // Recursive Step N>2

    let l_i = to_index::<C, I>(l);
    let volume_2d = l_i.checked_mul(&l_i).expect("validated 2D volume");

    // 1. Decompose the index
    let index_rest = index / volume_2d; // Tile index
    let index_2d_effective = index % volume_2d; // Index within tile (potentially reversed)

    // 2. Calculate P_rest recursively (Inverse Tile Index)
    let p_rest = hairy_onion_point_recursive::<C, I>(n - 2, l, index_rest);

    // 3. Determine the actual Index_2D by inverting the Snake reversal
    let index_2d = if (index_rest & I::one()) == I::one() {
        (volume_2d - I::one()) - index_2d_effective
    } else {
        index_2d_effective
    };

    // 4. Calculate P_2D (Point within the tile)
    let p_2d = onion_point_2d::<C, I>(l, index_2d);

    // 5. Combine the points
    let mut p = p_2d;
    p.extend(p_rest);
    p
}

/// Convert a coordinate value into an index type for arithmetic.
fn to_index<C: Coord, I: Index>(value: C) -> I {
    I::from(value).expect("value fits index type")
}

/// Convert an index back into a coordinate value.
fn to_coord<C: Coord, I: Index>(value: I) -> C {
    C::from(value).expect("value fits coordinate type")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constructor_guards() {
        // L==0 rejected
        assert!(HairyOnionCurve::<u32, u32>::new(2, 0).is_err());
        // N==0 rejected
        assert!(HairyOnionCurve::<u32, u32>::new(0, 4).is_err());
        // Valid shapes
        let c = HairyOnionCurve::<u32, u32>::new(2, 3).unwrap();
        assert_eq!(c.length(), 9);
    }

    #[test]
    fn roundtrip_dims_2_to_4_sizes_upto_8() {
        for dim in 2..=4 {
            for size in 2..=8 {
                let curve = HairyOnionCurve::<u32, u32>::new(dim, size).unwrap();
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
