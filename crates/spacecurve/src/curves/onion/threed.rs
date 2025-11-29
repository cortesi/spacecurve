//! 3D specialization for the Onion curve.

use super::{onion_index_nd, onion_point_nd, pow_u32};

/// Cube volume helper dedicated to the specialised 3D ordering.
fn cube_volume(side: u32) -> u32 {
    pow_u32(side, 3)
}

/// Specialised 3D outer-shell ordering that mirrors the published definition.
pub(super) fn onion_index_3d(side_length: u32, point: &[u32]) -> u32 {
    debug_assert_eq!(point.len(), 3);

    let layer = point
        .iter()
        .map(|&coord| coord.min(side_length - 1 - coord))
        .min()
        .unwrap_or(0);
    let inner = side_length - layer * 2;

    if inner <= 1 {
        return cube_volume(side_length) - 1;
    }

    let local = [point[0] - layer, point[1] - layer, point[2] - layer];
    let mut offset = cube_volume(side_length) - cube_volume(inner);
    let face_area = pow_u32(inner, 2);

    if local[0] == 0 {
        let idx = onion_index_nd(2, inner, &[local[1], local[2]]);
        return offset + idx;
    }
    offset += face_area;

    if local[0] == inner - 1 {
        let idx = onion_index_nd(2, inner, &[local[1], local[2]]);
        return offset + idx;
    }
    offset += face_area;

    let inner_minus_two = inner.saturating_sub(2);
    if inner_minus_two == 0 {
        return offset;
    }

    if local[1] == 0 && local[2] == 0 {
        return offset + (local[0] - 1);
    }
    offset += inner_minus_two;

    if local[1] == 0 && local[2] > 0 && local[2] < inner - 1 {
        let idx = onion_index_nd(2, inner_minus_two, &[local[0] - 1, local[2] - 1]);
        return offset + idx;
    }
    offset += pow_u32(inner_minus_two, 2);

    if local[1] == 0 && local[2] == inner - 1 {
        return offset + (local[0] - 1);
    }
    offset += inner_minus_two;

    if local[1] == inner - 1 && local[2] == 0 {
        return offset + (local[0] - 1);
    }
    offset += inner_minus_two;

    if local[1] == inner - 1 && local[2] > 0 && local[2] < inner - 1 {
        let idx = onion_index_nd(2, inner_minus_two, &[local[0] - 1, local[2] - 1]);
        return offset + idx;
    }
    offset += pow_u32(inner_minus_two, 2);

    if local[1] == inner - 1 && local[2] == inner - 1 {
        return offset + (local[0] - 1);
    }
    offset += inner_minus_two;

    if local[2] == 0 {
        let idx = onion_index_nd(2, inner_minus_two, &[local[0] - 1, local[1] - 1]);
        return offset + idx;
    }
    offset += pow_u32(inner_minus_two, 2);

    let idx = onion_index_nd(2, inner_minus_two, &[local[0] - 1, local[1] - 1]);
    offset + idx
}

/// Inverse of the specialised 3D outer-shell ordering.
pub(super) fn onion_point_3d(side_length: u32, index: u32) -> Vec<u32> {
    let mut remaining = index;
    let mut layer = 0u32;
    let mut current_len = side_length;

    loop {
        let next_len = current_len.saturating_sub(2);
        let shell_size = cube_volume(current_len) - cube_volume(next_len);
        if remaining < shell_size {
            break;
        }
        remaining -= shell_size;
        layer += 1;
        current_len = next_len;
    }

    if current_len <= 1 {
        return vec![layer, layer, layer];
    }

    let inner = current_len;
    let inner_minus_two = inner.saturating_sub(2);
    let face_area = pow_u32(inner, 2);

    if remaining < face_area {
        let yz = onion_point_nd(2, inner, remaining);
        return vec![layer, yz[0] + layer, yz[1] + layer];
    }
    remaining -= face_area;

    if remaining < face_area {
        let yz = onion_point_nd(2, inner, remaining);
        return vec![layer + inner - 1, yz[0] + layer, yz[1] + layer];
    }
    remaining -= face_area;

    if inner_minus_two == 0 {
        return vec![layer, layer, layer + inner - 1];
    }

    if remaining < inner_minus_two {
        return vec![layer + 1 + remaining, layer, layer];
    }
    remaining -= inner_minus_two;

    let rect_area = pow_u32(inner_minus_two, 2);

    if remaining < rect_area {
        let coords = onion_point_nd(2, inner_minus_two, remaining);
        return vec![layer + 1 + coords[0], layer, layer + 1 + coords[1]];
    }
    remaining -= rect_area;

    if remaining < inner_minus_two {
        return vec![layer + 1 + remaining, layer, layer + inner - 1];
    }
    remaining -= inner_minus_two;

    if remaining < inner_minus_two {
        return vec![layer + 1 + remaining, layer + inner - 1, layer];
    }
    remaining -= inner_minus_two;

    if remaining < rect_area {
        let coords = onion_point_nd(2, inner_minus_two, remaining);
        return vec![
            layer + 1 + coords[0],
            layer + inner - 1,
            layer + 1 + coords[1],
        ];
    }
    remaining -= rect_area;

    if remaining < inner_minus_two {
        return vec![layer + 1 + remaining, layer + inner - 1, layer + inner - 1];
    }
    remaining -= inner_minus_two;

    if remaining < rect_area {
        let coords = onion_point_nd(2, inner_minus_two, remaining);
        return vec![layer + 1 + coords[0], layer + 1 + coords[1], layer];
    }
    remaining -= rect_area;

    let coords = onion_point_nd(2, inner_minus_two, remaining);
    vec![
        layer + 1 + coords[0],
        layer + 1 + coords[1],
        layer + inner - 1,
    ]
}
