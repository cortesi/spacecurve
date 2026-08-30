//! Support operations for curve calculation.

use smallvec::{SmallVec, smallvec};

use crate::types::{Coord, Index};

/// Convert a binary index to its Binary Reflected Gray Code (BRGC) form.
pub fn graycode<I: Index>(x: I) -> I {
    x ^ (x >> 1)
}

/// Inverse Gray code: recover binary from a BRGC value `x`.
pub fn igraycode<I: Index>(x: I) -> I {
    let mut g = x;
    let mut b = x;
    loop {
        if g.is_zero() {
            return b;
        }
        g = g >> 1;
        b = b ^ g;
    }
}

/// Checked exponentiation helper for index types.
pub fn checked_pow<I: Index>(base: I, exp: u32) -> Option<I> {
    let mut acc = I::one();
    for _ in 0..exp {
        acc = acc.checked_mul(&base)?;
    }
    Some(acc)
}

/// Transpose a vector of n d-bit numbers into a vector of d n-bit numbers.
pub fn bit_transpose<C: Coord>(d: u32, v: &[C]) -> SmallVec<[C; 8]> {
    let mut ret = smallvec![C::zero(); d as usize];
    for (off, x) in v.iter().enumerate() {
        for bit in 0..d {
            if (*x & (C::one() << bit)) != C::zero() {
                let idx = (d - bit - 1) as usize;
                let shift = v.len().saturating_sub(off + 1) as u32;
                ret[idx] = ret[idx] | (C::one() << shift);
            }
        }
    }
    ret
}

/// Interleave the least-significant bits of each coordinate into a single
/// value.
///
/// `bits_per_axis` defines how many bits should be read from every coordinate.
/// Bits are interleaved from least-significant to most-significant order to
/// match the conventional Morton/Z-order encoding.
pub fn interleave_lsb<C: Coord, I: Index>(coords: &[C], bits_per_axis: u32) -> I {
    if coords.is_empty() || bits_per_axis == 0 {
        return I::zero();
    }

    let dimension = coords.len() as u32;
    let mut value = I::zero();
    for bit in 0..bits_per_axis {
        for (dim, coord) in coords.iter().enumerate() {
            let bit_val = (*coord >> (bit as usize)) & C::one();
            if bit_val != C::zero() {
                let shift = (bit * dimension + dim as u32) as usize;
                value = value | (I::one() << shift);
            }
        }
    }
    value
}

/// Deinterleave a Morton/Z-order code into coordinate components.
pub fn deinterleave_lsb<C: Coord, I: Index>(
    dimension: u32,
    bits_per_axis: u32,
    value: I,
) -> SmallVec<[C; 8]> {
    if dimension == 0 {
        return smallvec![];
    }
    if bits_per_axis == 0 {
        return smallvec![C::zero(); dimension as usize];
    }

    let mut coords = smallvec![C::zero(); dimension as usize];
    for bit in 0..bits_per_axis {
        for dim in 0..dimension {
            let bit_index = bit * dimension + dim;
            let bit_val = (value >> (bit_index as usize)) & I::one();
            if bit_val != I::zero() {
                coords[dim as usize] = coords[dim as usize] | (C::one() << (bit as usize));
            }
        }
    }
    coords
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interleave_roundtrip() {
        for dim in 1u32..=4 {
            for bits in 0..=3 {
                let max = 1u32 << bits;
                let combos = max.pow(dim);
                for idx in 0..combos {
                    let mut coords = vec![0u32; dim as usize];
                    let mut v = idx;
                    for slot in (0..dim as usize).rev() {
                        coords[slot] = v % max;
                        v /= max;
                    }
                    let morton = interleave_lsb::<u32, u32>(&coords, bits);
                    let roundtrip = deinterleave_lsb::<u32, u32>(dim, bits, morton);
                    assert_eq!(roundtrip.as_slice(), coords);
                }
            }
        }
    }

    #[test]
    fn test_transpose() {
        let v: Vec<u32> = vec![0b00, 0b01, 0b10, 0b11];
        assert_eq!(
            v.as_slice(),
            bit_transpose(4, &bit_transpose(2, &v)).as_slice()
        );
        let expected: Vec<u32> = vec![0b0011, 0b0101];
        assert_eq!(bit_transpose(2, &v).as_slice(), expected.as_slice());
    }

    #[test]
    fn test_graycode() {
        assert_eq!(graycode(3u32), 2u32);
        assert_eq!(graycode(4u32), 6u32);
        for i in 0..10u32 {
            assert_eq!(igraycode(graycode(i)), i);
            assert_eq!(graycode(igraycode(i)), i);
        }
    }
}
