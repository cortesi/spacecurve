//! Support operations for curve calculation.

use smallvec::{SmallVec, smallvec};

/// Convert a binary index to its Binary Reflected Gray Code (BRGC) form.
pub fn graycode(x: u32) -> u32 {
    x ^ (x >> 1)
}

/// Inverse Gray code: recover binary from a BRGC value `x`.
pub fn igraycode(x: u32) -> u32 {
    let mut g = x;
    let mut b = x;
    loop {
        if g == 0 {
            return b;
        }
        g >>= 1;
        b ^= g;
    }
}

#[inline]
const fn bitmask(bits: u32) -> u32 {
    if bits >= 32 {
        u32::MAX
    } else {
        match bits {
            0 => 0,
            b => 1u32.wrapping_shl(b) - 1,
        }
    }
}

/// Transpose a vector of n d-bit numbers into a vector of d n-bit numbers.
pub fn bit_transpose(d: u32, v: &[u32]) -> Vec<u32> {
    let mut ret = vec![0; d as usize];
    for (off, x) in v.iter().enumerate() {
        for bit in 0..d {
            if x & (1 << bit) != 0 {
                ret[(d - bit - 1) as usize] |= 1 << (v.len() - off - 1);
            }
        }
    }
    ret
}

/// Spreads bits of a 16-bit number so that there is 1 zero between each bit.
/// (e.g., 1011 -> 1000101)
/// Used for 2D Morton codes.
fn part1by1(mut n: u32) -> u32 {
    n &= 0x0000ffff;
    n = (n ^ (n << 8)) & 0x00ff00ff;
    n = (n ^ (n << 4)) & 0x0f0f0f0f;
    n = (n ^ (n << 2)) & 0x33333333;
    n = (n ^ (n << 1)) & 0x55555555;
    n
}

/// Compresses bits of a 32-bit number, selecting every other bit.
/// Inverse of part1by1.
fn compact1by1(mut n: u32) -> u32 {
    n &= 0x55555555;
    n = (n ^ (n >> 1)) & 0x33333333;
    n = (n ^ (n >> 2)) & 0x0f0f0f0f;
    n = (n ^ (n >> 4)) & 0x00ff00ff;
    n = (n ^ (n >> 8)) & 0x0000ffff;
    n
}

/// Spreads bits of a 10-bit number so that there are 2 zeroes between each bit.
/// (e.g., 1011 -> 1001001)
/// Used for 3D Morton codes.
fn part1by2(mut n: u32) -> u32 {
    n &= 0x000003ff;
    n = (n ^ (n << 16)) & 0xff0000ff;
    n = (n ^ (n << 8)) & 0x0300f00f;
    n = (n ^ (n << 4)) & 0x030c30c3;
    n = (n ^ (n << 2)) & 0x09249249;
    n
}

/// Compresses bits of a 32-bit number, selecting every third bit.
/// Inverse of part1by2.
fn compact1by2(mut n: u32) -> u32 {
    n &= 0x09249249;
    n = (n ^ (n >> 2)) & 0x030c30c3;
    n = (n ^ (n >> 4)) & 0x0300f00f;
    n = (n ^ (n >> 8)) & 0xff0000ff;
    n = (n ^ (n >> 16)) & 0x000003ff;
    n
}

#[inline]
fn interleave_lsb_const<const D: usize>(coords: &[u32; D], bits_per_axis: u32) -> u32 {
    if D == 0 || bits_per_axis == 0 {
        return 0;
    }

    match D {
        2 if bits_per_axis <= 16 => {
            let mask = bitmask(bits_per_axis);
            return part1by1(coords[0] & mask) | (part1by1(coords[1] & mask) << 1);
        }
        3 if bits_per_axis <= 10 => {
            let mask = bitmask(bits_per_axis);
            return part1by2(coords[0] & mask)
                | (part1by2(coords[1] & mask) << 1)
                | (part1by2(coords[2] & mask) << 2);
        }
        _ => {}
    }

    let mut value = 0u32;
    for bit in 0..bits_per_axis {
        for (dim, coord) in coords.iter().enumerate() {
            let bit_val = (coord >> bit) & 1;
            value |= bit_val << (bit * (D as u32) + dim as u32);
        }
    }
    value
}

#[inline]
fn deinterleave_lsb_const<const D: usize>(bits_per_axis: u32, value: u32) -> [u32; D] {
    let mut coords = [0u32; D];
    if D == 0 || bits_per_axis == 0 {
        return coords;
    }

    match D {
        2 if bits_per_axis <= 16 => {
            let mask = bitmask(bits_per_axis);
            coords[0] = compact1by1(value) & mask;
            coords[1] = compact1by1(value >> 1) & mask;
            return coords;
        }
        3 if bits_per_axis <= 10 => {
            let mask = bitmask(bits_per_axis);
            coords[0] = compact1by2(value) & mask;
            coords[1] = compact1by2(value >> 1) & mask;
            coords[2] = compact1by2(value >> 2) & mask;
            return coords;
        }
        _ => {}
    }

    for bit in 0..bits_per_axis {
        for (dim, coord) in coords.iter_mut().enumerate() {
            let bit_index = bit * (D as u32) + dim as u32;
            let bit_val = (value >> bit_index) & 1;
            *coord |= bit_val << bit;
        }
    }
    coords
}

/// Interleave the least-significant bits of each coordinate into a single value.
///
/// `bits_per_axis` defines how many bits should be read from every coordinate.
/// Bits are interleaved from least-significant to most-significant order to
/// match the conventional Morton/Z-order encoding.
pub fn interleave_lsb(coords: &[u32], bits_per_axis: u32) -> u32 {
    if coords.is_empty() || bits_per_axis == 0 {
        return 0;
    }

    match coords.len() {
        1 => interleave_lsb_const::<1>(&[coords[0]], bits_per_axis),
        2 => interleave_lsb_const::<2>(&[coords[0], coords[1]], bits_per_axis),
        3 => interleave_lsb_const::<3>(&[coords[0], coords[1], coords[2]], bits_per_axis),
        4 => {
            interleave_lsb_const::<4>(&[coords[0], coords[1], coords[2], coords[3]], bits_per_axis)
        }
        _ => interleave_generic(coords, bits_per_axis),
    }
}

fn interleave_generic(coords: &[u32], bits_per_axis: u32) -> u32 {
    let dimension = coords.len();
    let mut value = 0u32;
    for bit in 0..bits_per_axis {
        for (dim, coord) in coords.iter().enumerate() {
            let bit_val = (coord >> bit) & 1;
            value |= bit_val << (bit * (dimension as u32) + dim as u32);
        }
    }
    value
}

/// Deinterleave a Morton/Z-order code into coordinate components.
pub fn deinterleave_lsb(dimension: u32, bits_per_axis: u32, value: u32) -> SmallVec<[u32; 4]> {
    if dimension == 0 {
        return smallvec![];
    }
    if bits_per_axis == 0 {
        return smallvec![0; dimension as usize];
    }

    match dimension {
        1 => {
            let [a] = deinterleave_lsb_const::<1>(bits_per_axis, value);
            return smallvec![a];
        }
        2 => {
            let [a, b] = deinterleave_lsb_const::<2>(bits_per_axis, value);
            return smallvec![a, b];
        }
        3 => {
            let [a, b, c] = deinterleave_lsb_const::<3>(bits_per_axis, value);
            return smallvec![a, b, c];
        }
        4 => {
            let [a, b, c, d] = deinterleave_lsb_const::<4>(bits_per_axis, value);
            return smallvec![a, b, c, d];
        }
        _ => {}
    }

    deinterleave_generic(dimension, bits_per_axis, value)
}

fn deinterleave_generic(dimension: u32, bits_per_axis: u32, value: u32) -> SmallVec<[u32; 4]> {
    let mut coords = smallvec![0u32; dimension as usize];
    for bit in 0..bits_per_axis {
        for dim in 0..dimension {
            let bit_index = bit * dimension + dim;
            let bit_val = (value >> bit_index) & 1;
            coords[dim as usize] |= bit_val << bit;
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
                    let morton = interleave_lsb(&coords, bits);
                    let roundtrip = deinterleave_lsb(dim, bits, morton);
                    assert_eq!(roundtrip.as_slice(), coords);
                }
            }
        }
    }

    #[test]
    fn test_transpose() {
        let v: Vec<u32> = vec![0b00, 0b01, 0b10, 0b11];
        assert_eq!(v, bit_transpose(4, &bit_transpose(2, &v)));
        let expected: Vec<u32> = vec![0b0011, 0b0101];
        assert_eq!(bit_transpose(2, &v), expected);
    }

    #[test]
    fn test_graycode() {
        assert_eq!(graycode(3), 2);
        assert_eq!(graycode(4), 6);
        for i in 0..10 {
            assert_eq!(igraycode(graycode(i)), i);
            assert_eq!(graycode(igraycode(i)), i);
        }
    }
}
