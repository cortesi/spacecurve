use smallvec::{SmallVec, smallvec};

use super::hilbert_common::{gray2, rot2};
use crate::types::{Coord, Index};

/// 2D Hilbert index for a point `p` at a given `order`.
pub fn hilbert_index<C: Coord, I: Index>(order: u32, point: &[C]) -> I {
    let mut index_acc = I::zero();
    let mut entry_state = I::zero();
    let mut direction_state = I::zero();
    let zero = I::zero();
    let one = I::one();
    let two = one << 1usize;
    let three = two | one;
    for step in 0..order {
        let bit_offset = (order - step - 1) as usize;
        let a_bit = (point[1] >> bit_offset) & C::one();
        let b_bit = (point[0] >> bit_offset) & C::one();
        let label = (if a_bit == C::zero() { zero } else { one })
            | (if b_bit == C::zero() { zero } else { two });
        let label = label ^ entry_state;
        let word = if direction_state.is_zero() {
            gray2(rot2(label))
        } else {
            gray2(label)
        };
        if word == three {
            entry_state = three - entry_state;
        }
        index_acc = (index_acc << 2usize) | word;
        if word == zero || word == three {
            direction_state = direction_state ^ one;
        }
    }
    index_acc
}

/// 2D Hilbert point for a given `order` and `index`.
pub fn hilbert_point<C: Coord, I: Index>(order: u32, index: I) -> SmallVec<[C; 8]> {
    let hwidth = order * 2;
    let mut entry_state = I::zero();
    let mut direction_state = I::zero();
    let zero = I::zero();
    let one = I::one();
    let two = one << 1usize;
    let three = two | one;
    // Use 32-bit coordinate masks to avoid artificial 16-bit limits.
    let mut x_coord = C::zero();
    let mut y_coord = C::zero();
    for step in 0..order {
        // Extract 2 bits from the index
        let shift = (hwidth - (step * 2) - 2) as usize;
        let word = (index >> shift) & three;

        let label = if direction_state.is_zero() {
            rot2(gray2(word)) ^ entry_state
        } else {
            gray2(word) ^ entry_state
        };

        let bit_mask = C::one() << ((order - step - 1) as usize);

        if (label & two) != zero {
            x_coord = x_coord | bit_mask;
        }
        if (label & one) != zero {
            y_coord = y_coord | bit_mask;
        }

        if word == three {
            entry_state = three - entry_state;
        }
        if word == zero || word == three {
            direction_state = direction_state ^ one;
        }
    }
    smallvec![x_coord, y_coord]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::curves::hilbert_common::{gray2, rot2};

    #[test]
    fn test_rot() {
        assert_eq!(rot2(1u32), 2u32);
        assert_eq!(rot2(2u32), 1u32);
    }

    #[test]
    fn test_graycode() {
        assert_eq!(gray2(1u32), 1u32);
        assert_eq!(gray2(3u32), 2u32);
    }

    #[test]
    fn test_index() {
        assert!(hilbert_index::<u32, u32>(3, &[5, 6]) == 45);
        assert!(hilbert_point::<u32, u32>(3, 45).as_slice() == [5, 6]);
    }

    #[test]
    fn test_symmetry() {
        for m in 2u32..5u32 {
            for i in 0u32..2u32.pow(2 * m) {
                let p = hilbert_point::<u32, u32>(m, i);
                let r = hilbert_index::<u32, u32>(m, &p);
                assert!(i == r);
            }
        }
    }
}
