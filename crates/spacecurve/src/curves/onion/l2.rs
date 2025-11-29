//! L=2 specialization for the Onion curve (continuous Gray-code).
//!
//! Implements a continuous Gray code that generalises the specific 2D Onion(L=2) curve.
//! Defined recursively: O(N) = (O(N-1), 0) followed by (Reversed(O(N-1)), 1).
//! We use the last dimension as the discriminator to match the 2D definition: (0,0),(1,0),(1,1),(0,1).

/// Compute the onion index for L=2 using Gray-code generalisation.
pub(super) fn onion_index_l2(n: u32, p: &[u32]) -> u32 {
    if n == 0 {
        return 0;
    }
    let dim_prev = n - 1;
    let volume_prev = 1u32 << dim_prev; // 2^(N-1)
    let last = p[n as usize - 1];
    let i_prev = onion_index_l2(dim_prev, &p[..n as usize - 1]);
    if last == 0 {
        i_prev
    } else {
        (volume_prev - 1) - i_prev + volume_prev
    }
}

/// Inverse for the `L=2` specialised onion index.
pub(super) fn onion_point_l2(n: u32, index: u32) -> Vec<u32> {
    if n == 0 {
        return vec![];
    }
    let dim_prev = n - 1;
    let volume_prev = 1u32 << dim_prev;
    let (last, i_prev) = if index < volume_prev {
        (0, index)
    } else {
        let idx = index - volume_prev;
        (1, (volume_prev - 1) - idx)
    };
    let mut p = onion_point_l2(dim_prev, i_prev);
    p.push(last);
    p
}
