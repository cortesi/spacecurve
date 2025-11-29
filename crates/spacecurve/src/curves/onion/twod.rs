//! 2D specialization for the Onion curve (continuous spiral on a square).

/// Compute the onion index for 2D (continuous spiral).
pub fn onion_index_2d(l: u32, p: &[u32]) -> u32 {
    if l <= 1 {
        return 0;
    }
    let x = p[0];
    let y = p[1];
    // 1) Bottom edge
    if y == 0 {
        return x;
    }
    // 2) Right edge
    if x == l - 1 {
        return l - 1 + y;
    }
    // 3) Top edge
    if y == l - 1 {
        return 3 * l - 3 - x;
    }
    // 4) Left edge
    if x == 0 {
        return 4 * l - 4 - y;
    }
    // 5) Inner
    let outer = 4 * l - 4;
    let p_inner = vec![x - 1, y - 1];
    outer + onion_index_2d(l.saturating_sub(2), &p_inner)
}

/// Inverse of `onion_index_2d`.
pub fn onion_point_2d(l: u32, index: u32) -> Vec<u32> {
    if l == 1 {
        return vec![0, 0];
    }
    if l == 0 {
        unreachable!("L==0 is rejected by OnionCurve::new");
    }

    let outer_layer_size = 4 * l - 4;

    if index >= outer_layer_size {
        // Inner square
        let p_inner = onion_point_2d(l.saturating_sub(2), index - outer_layer_size);
        return vec![p_inner[0] + 1, p_inner[1] + 1];
    }

    // Outer layer
    if index < l {
        return vec![index, 0];
    }
    if index < 2 * l - 1 {
        return vec![l - 1, index - l + 1];
    }
    if index < 3 * l - 2 {
        return vec![3 * l - 3 - index, l - 1];
    }
    vec![0, 4 * l - 4 - index]
}
