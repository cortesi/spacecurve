/*
The H-curve, described in "Towards Optimal Locality in Mesh-Indexings" by R.
Niedermeier , K. Reinhardt  and P. Sanders.

This implementation is a corrected version based on the algorithm described in:
Cyclic space-filling curves and their clustering property, Igor V. Netay.

The original C implementation by Netay contained an error in Grey/InvGrey usage
for D>=3, leading to discontinuities, which is fixed here.
*/
use crate::{error, ops, point, spacecurve::SpaceCurve, spec::GridSpec};

// Convention used in low-level functions:
// d: Dimension
// n: Order (Precision)

// Standard Binary Reflected Grey Code (BRGC).
/// Binary Reflected Gray Code (BRGC) of `val` with bit-width `d`.
fn grey(d: u32, val: u32) -> u32 {
    // Masking ensures we only consider 'd' bits.
    // Assumes d < 32, enforced by HCurve constructor.
    let mask = (1 << d) - 1;
    let val2 = val & mask;
    val2 ^ (val2 >> 1)
}

// Corrected parity function (sum of bits mod 2).
/// Parity (sum of bits modulo 2).
fn parity(val: u32) -> u32 {
    val.count_ones() % 2
}

/*
Note on Grey code functions:
The C implementation used confusing names (grey_fast implemented Inverse Grey,
and grey_inverse_fast implemented Grey). We rename them here for clarity.
*/

// Retrieves Grey(val) from cache (lower half of corners[0]).
// Corresponds to C: grey_inverse_fast.
/// Cached Gray code from the precomputed corner tables.
fn cached_grey(_d: u32, val: u32, corners: &[Vec<u32>]) -> u32 {
    corners[0][val as usize]
}

// Retrieves InverseGrey(val) from cache (upper half of corners[0]).
// Corresponds to C: grey_fast.
/// Cached inverse Gray code from the precomputed corner tables.
fn cached_inv_grey(d: u32, val: u32, corners: &[Vec<u32>]) -> u32 {
    // Assumes d < 32.
    corners[0][(val + (1 << d)) as usize]
}

// d=Dimension, n=Order.
/// Precompute corner-index tables for dimension `d` and order `n`.
fn corner_indexes(d: u32, n: u32) -> Vec<Vec<u32>> {
    // Assumes d < 32.
    let size = 1u32 << d;
    let mut v = vec![vec![0u32; (size * 2) as usize]; (n + 1) as usize];

    // Initialize Grey codes cache (n=0 case).
    for i in 0..size {
        let g = grey(d, i);
        // Store Inverse Grey in the upper half
        v[0][(g + size) as usize] = i;
        // Store Grey in the lower half
        v[0][i as usize] = g;
    }

    // Build the rest of the tables recursively.
    for n1 in 1..=n {
        for r in 0..size {
            // Entry corner: Alphas are all 'r'.
            let mut alphas = vec![r; n1 as usize];
            v[n1 as usize][r as usize] = h_index_alphas(d, n1, &alphas[..], &v);

            // Exit corner: Last alpha is flipped (r^1).
            alphas[(n1 - 1) as usize] ^= 1;
            v[n1 as usize][(r + size) as usize] = h_index_alphas(d, n1, &alphas[..], &v);
        }
    }
    v
}

// encode_h in C. (Point to Index)
// Uses u64 for internal calculations (r) to prevent overflow during intermediate steps.
/// Compute H-curve index from alpha vectors.
fn h_index_alphas(d: u32, n: u32, alphas: &[u32], corners: &[Vec<u32>]) -> u32 {
    debug_assert_eq!(alphas.len(), n as usize);
    if alphas.len() != n as usize {
        return 0;
    }
    let mut r: u64 = 0;
    let two_power_d = 1u32 << d;
    let two_power_d_64 = 1u64 << d;

    // Iterate from least significant alpha (i=n-1) to most significant (i=0).
    for i in (0..n).rev() {
        let alpha = alphas[i as usize] % two_power_d;

        // 1. Calculate the transformation (r_shift) based on orientation.
        let alpha_inv = alpha ^ (two_power_d - 1);
        // This logic relies on the corrected parity function.
        let need_to_change_last = 1 ^ parity(alpha_inv);

        let index = (alpha_inv + two_power_d * need_to_change_last) as usize;
        // k = n - 1 - i (depth)
        let k = (n - 1 - i) as usize;
        let r_shift = corners[k][index] as u64;

        let mut current_r_shift = r_shift;

        // Condition of reversal (specific rule from the algorithm).
        if (d % 2 == 1) && (n == 1) {
            current_r_shift = (current_r_shift ^ (two_power_d_64 - 1)).wrapping_add(1);
        }

        // Calculate sub_cell_size S = 2^(d * k).
        let shift = (d as u64) * (k as u64);
        let sub_cell_size = 1u64 << shift;

        // 2. Transform the current index r (from lower levels).
        // r = (r - r_shift) % S. Use wrapping arithmetic.
        r = r.wrapping_sub(current_r_shift);
        r %= sub_cell_size;

        // 3. Calculate the index chunk r0 for this level.
        // CRITICAL FIX: Use Inverse Grey for encoding (Point->Index).
        // The C implementation incorrectly used Grey (grey_inverse_fast).
        let r0 = cached_inv_grey(d, alpha, corners) as u64;

        // 4. Combine. r = r0*S + r_transformed.
        r += r0 * sub_cell_size;
    }
    r as u32
}

// d=Dimension, n=Order.
/// Point to index mapping for the H-curve.
fn h_index(d: u32, n: u32, p: &[u32], corners: &[Vec<u32>]) -> u32 {
    debug_assert_eq!(p.len(), d as usize);
    if p.len() != d as usize {
        return 0;
    }
    // Transpose coordinates P (D elements, N bits) to Alphas (N elements, D bits).
    // We pass N (Order) as the width (bits per coordinate) to bit_transpose.
    h_index_alphas(d, n, &ops::bit_transpose(n, p), corners)
}

// decode_h in C. (Index to Point)
/// Index to point mapping for the H-curve.
fn h_point(d: u32, n: u32, idx: u32, corners: &[Vec<u32>]) -> Vec<u32> {
    let mut alphas = vec![0; n as usize];
    let two_power_d = 1u32 << d;
    let two_power_d_64 = 1u64 << d;

    // r must be u64 as intermediate values during decoding can exceed 2^(D*N).
    let mut r: u64 = idx as u64;

    // Iterate from most significant alpha (i=0) to least significant (i=n-1).
    for i in 0..n {
        let k = n - 1 - i;
        let shift = k as u64 * d as u64;

        // 1. Extract the relevant d bits chunk (r0).
        let r0 = (r >> shift) % two_power_d_64;
        let r0_u32 = r0 as u32;

        // 2. Calculate Alpha.
        // CRITICAL FIX: Use Grey code for decoding (Index->Point) to ensure continuity.
        // The C implementation incorrectly used Inverse Grey (grey_fast).
        let alpha = cached_grey(d, r0_u32, corners);
        alphas[i as usize] = alpha;

        // 3. Calculate the transformation (r_shift). This must match the encoding logic.
        let alpha_inv = alpha ^ (two_power_d - 1);
        let need_to_change_last = 1 ^ parity(alpha_inv);
        let index = (alpha_inv + two_power_d * need_to_change_last) as usize;

        let mut r_shift = corners[k as usize][index] as u64;

        // Condition of reversal.
        if d % 2 == 1 && n == 1 {
            r_shift ^= two_power_d_64 - 1;
            r_shift = r_shift.wrapping_add(1);
        }

        // 4. Apply the inverse transformation.
        // This prepares the lower bits of r for the next iteration.
        r = r.wrapping_add(r_shift);
    }
    // Transpose Alphas (N elements, D bits) back to coordinates (D elements, N bits).
    // We pass D (Dimension) as the width (bits per alpha) to bit_transpose.
    ops::bit_transpose(d, &alphas)
}

/// An implementation of the H curve generalization.
#[derive(Debug)]
pub struct HCurve {
    /// The order of the curve (N).
    pub order: u32,
    /// The dimension of the H curve (D).
    pub dimension: u32,
    /// Precomputed corner index tables used by point/index mapping.
    corners: Vec<Vec<u32>>,
}

impl HCurve {
    /// Construct an H curve to precisely fit a hypercube.
    pub fn from_dimensions(dimension: u32, size: u32) -> error::Result<Self> {
        if dimension < 2 {
            return Err(error::Error::Shape("Dimension must be >= 2".to_string()));
        }

        let spec = GridSpec::power_of_two(dimension, size)?;
        let order = spec.order().unwrap();

        // Enforce constraints required by the implementation (u32 limits and bit shifts).
        if dimension >= 32 {
            return Err(error::Error::Shape("Dimension must be < 32".to_string()));
        }
        if (order as u64) * (dimension as u64) >= 32 {
            return Err(error::Error::Size(
                "Curve size exceeds u32 limits (D*O must be < 32)".to_string(),
            ));
        }

        // Precompute corner index tables once per instance.
        let corners = corner_indexes(dimension, order);

        Ok(Self {
            dimension,
            order,
            corners,
        })
    }
}

impl SpaceCurve for HCurve {
    fn name(&self) -> &'static str {
        "H-curve"
    }

    fn info(&self) -> &'static str {
        "Hilbert-like family based on Binary Reflected Gray Code with\n\
        orientation transforms (Niedermeier–Reinhardt–Sanders; Netay).\n\
        Continuous on 2^n grids and often offering strong locality with\n\
        relatively simple bit operations."
    }
    fn length(&self) -> u32 {
        // Calculate 2^(D*O). Safe due to constructor checks.
        1u32 << (self.order * self.dimension)
    }
    fn dimensions(&self) -> u32 {
        self.dimension
    }
    fn point(&self, index: u32) -> point::Point {
        let d = self.dimension;
        let n = self.order;
        let hpoint = h_point(d, n, index, &self.corners);
        point::Point::new_with_dimension(self.dimension, hpoint)
    }

    fn index(&self, p: &point::Point) -> u32 {
        let d = self.dimension;
        let n = self.order;
        h_index(d, n, &p[..], &self.corners)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_2d_order3() -> error::Result<()> {
        let curve = HCurve::from_dimensions(2, 8)?;
        for idx in 0..curve.length() {
            let point = curve.point(idx);
            assert_eq!(curve.index(&point), idx, "roundtrip failed at {idx}");
        }
        Ok(())
    }

    #[test]
    fn roundtrip_3d_order1() -> error::Result<()> {
        let curve = HCurve::from_dimensions(3, 2)?;
        for idx in 0..curve.length() {
            let point = curve.point(idx);
            assert_eq!(curve.index(&point), idx, "3D order1 mismatch at {idx}");
        }
        Ok(())
    }

    #[test]
    fn roundtrip_3d_order2() -> error::Result<()> {
        let curve = HCurve::from_dimensions(3, 4)?;
        for idx in 0..curve.length() {
            let point = curve.point(idx);
            assert_eq!(curve.index(&point), idx, "3D order2 mismatch at {idx}");
        }
        Ok(())
    }
}
