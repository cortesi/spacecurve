/*
The H-curve, described in "Towards Optimal Locality in Mesh-Indexings" by R.
Niedermeier , K. Reinhardt  and P. Sanders.

This implementation is a corrected version based on the algorithm described in:
Cyclic space-filling curves and their clustering property, Igor V. Netay.

The original C implementation by Netay contained an error in Grey/InvGrey usage
for D>=3, leading to discontinuities, which is fixed here.
*/
use std::marker::PhantomData;

use smallvec::SmallVec;

use crate::{
    curves::hilbert_common::bitmask,
    error, ops, point,
    spacecurve::SpaceCurve,
    spec::GridSpec,
    types::{Coord, Index},
};

// Convention used in low-level functions:
// d: Dimension
// n: Order (Precision)

// Standard Binary Reflected Grey Code (BRGC).
/// Binary Reflected Gray Code (BRGC) of `val` with bit-width `d`.
fn grey<I: Index>(d: u32, val: I) -> I {
    // Masking ensures we only consider 'd' bits.
    // Assumes d < Index::BITS, enforced by HCurve constructor.
    let mask = bitmask::<I>(d);
    let val2 = val & mask;
    val2 ^ (val2 >> 1)
}

// Corrected parity function (sum of bits mod 2).
/// Parity (sum of bits modulo 2).
fn parity<I: Index>(val: I) -> I {
    let ones = val.to_u128().expect("parity input fits u128").count_ones();
    if ones.is_multiple_of(2) {
        I::zero()
    } else {
        I::one()
    }
}

/*
Note on Grey code functions:
The C implementation used confusing names (grey_fast implemented Inverse Grey,
and grey_inverse_fast implemented Grey). We rename them here for clarity.
*/

// Retrieves Grey(val) from cache (lower half of corners[0]).
// Corresponds to C: grey_inverse_fast.
/// Cached Gray code from the precomputed corner tables.
fn cached_grey<I: Index>(val: I, corners: &[Vec<I>]) -> I {
    let idx = to_usize(val);
    corners[0][idx]
}

// Retrieves InverseGrey(val) from cache (upper half of corners[0]).
// Corresponds to C: grey_fast.
/// Cached inverse Gray code from the precomputed corner tables.
fn cached_inv_grey<I: Index>(d: u32, val: I, corners: &[Vec<I>]) -> I {
    // Assumes d < Index::BITS.
    let offset = I::one().checked_shl(d).unwrap_or_else(I::zero);
    let idx = to_usize(val + offset);
    corners[0][idx]
}

// d=Dimension, n=Order.
/// Precompute corner-index tables for dimension `d` and order `n`.
fn corner_indexes<I: Index>(d: u32, n: u32) -> Vec<Vec<I>> {
    let size = 1usize
        .checked_shl(d)
        .expect("dimension fits usize for corner table");
    let table_len = size.checked_mul(2).expect("corner table fits usize");
    let mut v = vec![vec![I::zero(); table_len]; (n + 1) as usize];
    let size_i = I::from(size).expect("corner table size fits index");

    // Initialize Grey codes cache (n=0 case).
    for i in 0..size {
        let i_i = I::from(i).expect("corner index fits index type");
        let g = grey(d, i_i);
        // Store Inverse Grey in the upper half
        v[0][to_usize(g + size_i)] = i_i;
        // Store Grey in the lower half
        v[0][i] = g;
    }

    // Build the rest of the tables recursively.
    for n1 in 1..=n {
        for r in 0..size {
            // Entry corner: Alphas are all 'r'.
            let r_i = I::from(r).expect("corner index fits index type");
            let mut alphas = vec![r_i; n1 as usize];
            v[n1 as usize][r] = h_index_alphas(d, n1, &alphas[..], &v);

            // Exit corner: Last alpha is flipped (r^1).
            alphas[(n1 - 1) as usize] = alphas[(n1 - 1) as usize] ^ I::one();
            v[n1 as usize][r + size] = h_index_alphas(d, n1, &alphas[..], &v);
        }
    }
    v
}

// encode_h in C. (Point to Index)
// Uses u64 for internal calculations (r) to prevent overflow during
// intermediate steps.
/// Compute H-curve index from alpha vectors.
fn h_index_alphas<I: Index>(d: u32, n: u32, alphas: &[I], corners: &[Vec<I>]) -> I {
    debug_assert_eq!(alphas.len(), n as usize);
    if alphas.len() != n as usize {
        return I::zero();
    }
    let mut r: u128 = 0;
    let two_power_d = I::one().checked_shl(d).unwrap_or_else(I::zero);
    let two_power_d_128 = 1u128 << d;

    // Iterate from least significant alpha (i=n-1) to most significant (i=0).
    for i in (0..n).rev() {
        let alpha = alphas[i as usize] % two_power_d;

        // 1. Calculate the transformation (r_shift) based on orientation.
        let alpha_inv = alpha ^ (two_power_d - I::one());
        // This logic relies on the corrected parity function.
        let need_to_change_last = I::one() ^ parity(alpha_inv);

        let index = to_usize(alpha_inv + two_power_d * need_to_change_last);
        // k = n - 1 - i (depth)
        let k = (n - 1 - i) as usize;
        let r_shift = to_u128(corners[k][index]);

        let mut current_r_shift = r_shift;

        // Condition of reversal (specific rule from the algorithm).
        if (d % 2 == 1) && (n == 1) {
            current_r_shift = (current_r_shift ^ (two_power_d_128 - 1)).wrapping_add(1);
        }

        // Calculate sub_cell_size S = 2^(d * k).
        let shift = (d as u128) * (k as u128);
        let sub_cell_size = 1u128 << shift;

        // 2. Transform the current index r (from lower levels).
        // r = (r - r_shift) % S. Use wrapping arithmetic.
        r = r.wrapping_sub(current_r_shift);
        r %= sub_cell_size;

        // 3. Calculate the index chunk r0 for this level.
        // CRITICAL FIX: Use Inverse Grey for encoding (Point->Index).
        // The C implementation incorrectly used Grey (grey_inverse_fast).
        let r0 = to_u128(cached_inv_grey(d, alpha, corners));

        // 4. Combine. r = r0*S + r_transformed.
        r += r0 * sub_cell_size;
    }
    I::from(r).expect("index fits index type")
}

// d=Dimension, n=Order.
/// Point to index mapping for the H-curve.
fn h_index<C: Coord, I: Index>(d: u32, n: u32, p: &[C], corners: &[Vec<I>]) -> I {
    debug_assert_eq!(p.len(), d as usize);
    if p.len() != d as usize {
        return I::zero();
    }
    // Transpose coordinates P (D elements, N bits) to Alphas (N elements, D bits).
    // We pass N (Order) as the width (bits per coordinate) to bit_transpose.
    let alphas = ops::bit_transpose(n, p);
    let converted: Vec<I> = alphas
        .iter()
        .map(|&value| I::from(value).expect("alpha fits index type"))
        .collect();
    h_index_alphas(d, n, &converted, corners)
}

// decode_h in C. (Index to Point)
/// Index to point mapping for the H-curve.
fn h_point<C: Coord, I: Index>(d: u32, n: u32, idx: I, corners: &[Vec<I>]) -> SmallVec<[C; 8]> {
    let mut alphas = vec![I::zero(); n as usize];
    let two_power_d = I::one().checked_shl(d).unwrap_or_else(I::zero);
    let two_power_d_128 = 1u128 << d;

    // r must be u64 as intermediate values during decoding can exceed 2^(D*N).
    let mut r: u128 = to_u128(idx);

    // Iterate from most significant alpha (i=0) to least significant (i=n-1).
    for i in 0..n {
        let k = n - 1 - i;
        let shift = k as u128 * d as u128;

        // 1. Extract the relevant d bits chunk (r0).
        let r0 = (r >> shift) % two_power_d_128;
        let r0_u32 = r0 as u32;

        // 2. Calculate Alpha.
        // CRITICAL FIX: Use Grey code for decoding (Index->Point) to ensure continuity.
        // The C implementation incorrectly used Inverse Grey (grey_fast).
        let alpha = cached_grey(I::from(r0_u32).expect("alpha fits index type"), corners);
        alphas[i as usize] = alpha;

        // 3. Calculate the transformation (r_shift). This must match the encoding
        //    logic.
        let alpha_inv = alpha ^ (two_power_d - I::one());
        let need_to_change_last = I::one() ^ parity(alpha_inv);
        let index = to_usize(alpha_inv + two_power_d * need_to_change_last);

        let mut r_shift = to_u128(corners[k as usize][index]);

        // Condition of reversal.
        if d % 2 == 1 && n == 1 {
            r_shift ^= two_power_d_128 - 1;
            r_shift = r_shift.wrapping_add(1);
        }

        // 4. Apply the inverse transformation.
        // This prepares the lower bits of r for the next iteration.
        r = r.wrapping_add(r_shift);
    }
    // Transpose Alphas (N elements, D bits) back to coordinates (D elements, N
    // bits). We pass D (Dimension) as the width (bits per alpha) to
    // bit_transpose.
    let coords = ops::bit_transpose(d, &alphas);
    coords
        .iter()
        .map(|&value| C::from(value).expect("coordinate fits target type"))
        .collect()
}

/// An implementation of the H curve generalization.
#[derive(Debug)]
pub struct HCurve<C: Coord, I: Index> {
    /// The order of the curve (N).
    pub order: u32,
    /// The dimension of the H curve (D).
    pub dimension: u32,
    /// Cached total number of points (`2^(order * dimension)`).
    length: I,
    /// Precomputed corner index tables used by point/index mapping.
    corners: Vec<Vec<I>>,
    /// Marks the coordinate type so it remains part of the public API.
    _coord: PhantomData<C>,
}

impl<C: Coord, I: Index> HCurve<C, I> {
    /// Construct an H curve to precisely fit a hypercube.
    pub fn from_dimensions(dimension: u32, size: C) -> error::Result<Self> {
        if dimension < 2 {
            return Err(error::Error::Shape("Dimension must be >= 2".to_string()));
        }

        let spec = GridSpec::<C, I>::power_of_two(dimension, size)?;
        let order = spec.order().unwrap();

        // Enforce constraints required by the implementation (u32 limits and bit
        // shifts).
        if dimension >= 32 {
            return Err(error::Error::Shape("Dimension must be < 32".to_string()));
        }
        let total_bits = (order as u128) * (dimension as u128);
        if total_bits >= I::BITS as u128 {
            return Err(error::Error::Size(format!(
                "Curve size exceeds index limits (D*O must be < {})",
                I::BITS
            )));
        }

        // Precompute corner index tables once per instance.
        let corners = corner_indexes::<I>(dimension, order);
        let length = I::one()
            .checked_shl(order * dimension)
            .ok_or_else(|| error::Error::Size("Curve size exceeds index limits".to_string()))?;

        Ok(Self {
            dimension,
            order,
            length,
            corners,
            _coord: PhantomData,
        })
    }
}

impl<C: Coord, I: Index> SpaceCurve for HCurve<C, I> {
    type Coord = C;
    type Index = I;

    fn name(&self) -> &'static str {
        "H-curve"
    }

    fn info(&self) -> &'static str {
        "Hilbert-like family based on Binary Reflected Gray Code with\n\
        orientation transforms (Niedermeier–Reinhardt–Sanders; Netay).\n\
        Continuous on 2^n grids and often offering strong locality with\n\
        relatively simple bit operations."
    }
    fn length(&self) -> I {
        self.length
    }
    fn dimensions(&self) -> u32 {
        self.dimension
    }
    fn point(&self, index: I) -> point::Point<C> {
        let d = self.dimension;
        let n = self.order;
        let hpoint = h_point::<C, I>(d, n, index, &self.corners);
        point::Point::new_with_dimension(self.dimension, hpoint)
    }

    fn index(&self, p: &point::Point<C>) -> I {
        let d = self.dimension;
        let n = self.order;
        h_index::<C, I>(d, n, &p[..], &self.corners)
    }
}

/// Convert an index to `usize` for table indexing.
fn to_usize<I: Index>(value: I) -> usize {
    value.to_usize().expect("value fits usize")
}

/// Convert an index to `u128` for intermediate arithmetic.
fn to_u128<I: Index>(value: I) -> u128 {
    value.to_u128().expect("value fits u128")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_2d_order3() -> error::Result<()> {
        let curve = HCurve::<u32, u32>::from_dimensions(2, 8)?;
        for idx in 0..curve.length() {
            let point = curve.point(idx);
            assert_eq!(curve.index(&point), idx, "roundtrip failed at {idx}");
        }
        Ok(())
    }

    #[test]
    fn roundtrip_3d_order1() -> error::Result<()> {
        let curve = HCurve::<u32, u32>::from_dimensions(3, 2)?;
        for idx in 0..curve.length() {
            let point = curve.point(idx);
            assert_eq!(curve.index(&point), idx, "3D order1 mismatch at {idx}");
        }
        Ok(())
    }

    #[test]
    fn roundtrip_3d_order2() -> error::Result<()> {
        let curve = HCurve::<u32, u32>::from_dimensions(3, 4)?;
        for idx in 0..curve.length() {
            let point = curve.point(idx);
            assert_eq!(curve.index(&point), idx, "3D order2 mismatch at {idx}");
        }
        Ok(())
    }
}
