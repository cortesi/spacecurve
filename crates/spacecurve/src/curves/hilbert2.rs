use smallvec::{SmallVec, smallvec};

use super::hilbert_common::{gray2, rot2};

/// 2D Hilbert index for a point `p` at a given `order`.
pub fn hilbert_index(order: u32, point: &[u32]) -> u32 {
    let mut index_acc = 0;
    let mut entry_state = 0;
    let mut direction_state = 0;
    for step in 0..order {
        let bit_offset = order - step - 1;
        let a_bit = (point[1] >> bit_offset) & 1;
        let b_bit = (point[0] >> bit_offset) & 1;
        let label: u32 = (a_bit | b_bit << 1) ^ entry_state;
        let word = match direction_state {
            0 => gray2(rot2(label)),
            _ => gray2(label),
        };
        if word == 3 {
            entry_state = 3 - entry_state;
        }
        index_acc = (index_acc << 2) | word;
        if word == 0 || word == 3 {
            direction_state ^= 1;
        }
    }
    index_acc
}

/// 2D Hilbert point for a given `order` and `index`.
pub fn hilbert_point(order: u32, index: u32) -> SmallVec<[u32; 4]> {
    let hwidth = order * 2;
    let mut entry_state = 0;
    let mut direction_state = 0;
    // Use 32-bit coordinate masks to avoid artificial 16-bit limits.
    let mut x_coord: u32 = 0;
    let mut y_coord: u32 = 0;
    for step in 0..order {
        // Extract 2 bits from the index
        let word = (index >> (hwidth - (step * 2) - 2)) & 3;

        let label = match direction_state {
            0 => rot2(gray2(word)) ^ entry_state,
            _ => gray2(word) ^ entry_state,
        };

        let bit_mask: u32 = 1 << (order - step - 1);

        if (label & 2) != 0 {
            x_coord |= bit_mask;
        }
        if (label & 1) != 0 {
            y_coord |= bit_mask;
        }

        if word == 3 {
            entry_state = 3 - entry_state;
        }
        if word == 0 || word == 3 {
            direction_state ^= 1;
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
        assert_eq!(rot2(1), 2);
        assert_eq!(rot2(2), 1);
    }

    #[test]
    fn test_graycode() {
        assert_eq!(gray2(1), 1);
        assert_eq!(gray2(3), 2);
    }

    #[test]
    fn test_index() {
        assert!(hilbert_index(3, &[5, 6]) == 45);
        assert!(hilbert_point(3, 45).as_slice() == [5, 6]);
    }

    #[test]
    fn test_symmetry() {
        for m in 2u32..5u32 {
            for i in 0u32..2u32.pow(2 * m) {
                let p = hilbert_point(m, i);
                let r = hilbert_index(m, &p);
                assert!(i == r);
            }
        }
    }
}
