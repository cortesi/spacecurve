use smallvec::{SmallVec, smallvec};

use super::hilbert_common::{bitmask, bitrange, lrot, rrot, setbit, tsb};
use crate::ops;

// A generalized N-dimensional implementation of the Hilbert curve. This is
// about 10x slower than the specialised 2D version, so should only be used if
// we really need N dimensions.

/// Forward transform used by the N‑D Hilbert mapping.
fn transform(entry: u32, direction: u32, width: u32, x: u32) -> u32 {
    let mask = bitmask(width);
    rrot((x ^ entry) & mask, direction + 1, width)
}

/// Inverse of `transform`.
fn itransform(entry: u32, direction: u32, width: u32, x: u32) -> u32 {
    let mask = bitmask(width);
    lrot(x & mask, direction + 1, width) ^ entry
}

/// Direction function for the N‑D Hilbert mapping.
fn direction(x: u32, n: u32) -> u32 {
    let masked = x & bitmask(n);
    if masked == 0 {
        0
    } else if masked.is_multiple_of(2) {
        tsb(masked.wrapping_sub(1), n) % n
    } else {
        tsb(masked, n) % n
    }
}

/// Entry function for the N‑D Hilbert mapping.
fn entry(x: u32) -> u32 {
    match x {
        0 => 0,
        _ => ops::graycode(2 * ((x - 1) / 2)),
    }
}

/// N‑D Hilbert: compute point coordinates for `index`.
pub fn hilbert_point(dimension: u32, order: u32, index: u32) -> SmallVec<[u32; 4]> {
    let hwidth = order * dimension;
    let mut entry_state = 0;
    let mut direction_state = 0;
    let mut point = smallvec![0; dimension as usize];
    for order_idx in 0..order {
        let word = bitrange(
            index,
            hwidth,
            order_idx * dimension,
            order_idx * dimension + dimension,
        );
        let mut label = ops::graycode(word);
        label = itransform(entry_state, direction_state, dimension, label);
        for coord in 0..dimension {
            let bit_val = bitrange(label, dimension, coord, coord + 1);
            point[coord as usize] = setbit(point[coord as usize], order, order_idx, bit_val);
        }
        entry_state ^= lrot(entry(word), direction_state + 1, dimension);
        direction_state = (direction_state + direction(word, dimension) + 1) % dimension;
    }
    point
}

/// N‑D Hilbert: compute linear index for `point`.
pub fn hilbert_index(dimension: u32, order: u32, point: &[u32]) -> u32 {
    let mut index_acc = 0;
    let mut entry_state = 0;
    let mut direction_state = 0;
    for order_idx in 0..order {
        let mut label = 0;
        for coord in 0..dimension {
            let bit_val = bitrange(
                point[(dimension - coord - 1) as usize],
                order,
                order_idx,
                order_idx + 1,
            );
            label |= bit_val << coord;
        }
        label = transform(entry_state, direction_state, dimension, label);

        let word = ops::igraycode(label);
        entry_state ^= lrot(entry(word), direction_state + 1, dimension);
        direction_state = (direction_state + direction(word, dimension) + 1) % dimension;
        index_acc = (index_acc << dimension) | word;
    }
    index_acc
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hilbert() {
        let m = 3;
        for n in 2..5 {
            for i in 0..2u32.pow(n * m) {
                let v = hilbert_point(n, m, i);
                assert_eq!(i, hilbert_index(n, m, &v));
            }
        }
    }

    #[test]
    fn test_transform() {
        // These values are from the example on p 18 of Hamilton
        assert_eq!(transform(0, 1, 2, 3), 3);
        assert_eq!(transform(3, 0, 2, 2), 2);
        assert_eq!(transform(3, 0, 2, 1), 1);
    }

    #[test]
    fn test_rot() {
        fn rotpair(left: u32, right: u32, i: u32, width: u32) {
            assert_eq!(rrot(left, i, width), right);
            assert_eq!(lrot(right, i, width), left);
            assert_eq!(lrot(left, i, width), rrot(left, width - i, width));
        }
        rotpair(2, 1, 1, 2);
        rotpair(1, 2, 1, 2);
        rotpair(0, 0, 1, 2);
        rotpair(3, 3, 1, 2);
        rotpair(4, 2, 1, 3);
        rotpair(4, 1, 2, 3);
        rotpair(1, 2, 2, 3);
        rotpair(1, 1, 3, 3);
    }

    #[test]
    fn test_tsb() {
        assert_eq!(tsb(1, 5), 1);
        assert_eq!(tsb(2, 5), 0);
        assert_eq!(tsb(3, 5), 2);
        assert_eq!(tsb(2u32.pow(5) - 1, 5), 5);
        assert_eq!(tsb(0, 5), 0);
    }

    #[test]
    fn test_setbit() {
        assert_eq!(setbit(0, 3, 0, 1), 4);
        assert_eq!(setbit(4, 3, 2, 1), 5);
        assert_eq!(setbit(4, 3, 0, 0), 0);
    }

    #[test]
    fn test_bitrange() {
        fn checkbit(i: u32, width: u32, start: u32, end: u32, expected: u32) {
            let e = bitrange(i, width, start, end);
            assert_eq!(e, expected);
        }
        checkbit(1, 5, 4, 5, 1);
        checkbit(2, 5, 4, 5, 0);
        checkbit(2, 5, 3, 5, 2);
        checkbit(2, 5, 3, 4, 1);
        checkbit(3, 5, 3, 5, 3);
        checkbit(3, 5, 0, 5, 3);
        checkbit(4, 5, 2, 3, 1);
        checkbit(4, 5, 2, 4, 2);
        checkbit(4, 5, 2, 2, 0);
    }
}
