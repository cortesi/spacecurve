// --- Adjacency helpers ---

/// Check if two 2D points are adjacent (Manhattan distance <= 1).
#[inline]
pub fn is_adjacent_2d(a: &[u32; 2], b: &[u32; 2]) -> bool {
    let dx = (a[0] as i32 - b[0] as i32).abs();
    let dy = (a[1] as i32 - b[1] as i32).abs();
    dx + dy <= 1
}

/// Check if two 3D points are adjacent (Manhattan distance <= 1).
#[inline]
pub fn is_adjacent_3d(a: &[u32; 3], b: &[u32; 3]) -> bool {
    let dx = (a[0] as i32 - b[0] as i32).abs();
    let dy = (a[1] as i32 - b[1] as i32).abs();
    let dz = (a[2] as i32 - b[2] as i32).abs();
    dx + dy + dz <= 1
}

/// Advance the snake offset by `increment`, wrapping at `curve_length`.
///
/// Returns the new offset value. If `curve_length` is zero or None, returns 0.0.
pub fn advance_snake_offset(offset: f32, increment: f32, curve_length: Option<u32>) -> f32 {
    let Some(len) = curve_length else {
        return offset + increment;
    };
    let len_f = len as f32;
    if len_f <= 0.0 {
        return 0.0;
    }
    let new_offset = offset + increment;
    if new_offset >= len_f {
        new_offset.rem_euclid(len_f)
    } else {
        new_offset
    }
}

/// Calculate which segments the snake should occupy given an offset and length percentage.
pub fn calculate_snake_segments(
    snake_offset: f32,
    snake_length_percent: f32,
    curve_length: u32,
) -> Vec<usize> {
    let mut segments = Vec::new();
    fill_snake_segments(
        &mut segments,
        snake_offset,
        snake_length_percent,
        curve_length,
    );
    segments
}

/// Fill a preallocated buffer with the indices occupied by the snake overlay.
pub fn fill_snake_segments(
    out: &mut Vec<usize>,
    snake_offset: f32,
    snake_length_percent: f32,
    curve_length: u32,
) {
    out.clear();

    if curve_length == 0 {
        return;
    }

    let start_offset = snake_offset as u32;
    let snake_length = ((snake_length_percent / 100.0) * curve_length as f32).round() as u32;
    let snake_length = snake_length.max(1);

    if out.capacity() < snake_length as usize {
        out.reserve(snake_length as usize - out.capacity());
    }

    for i in 0..snake_length {
        let segment_index = (start_offset + i) % curve_length;
        out.push(segment_index as usize);
    }
}

/// Build an O(1) membership mask for fast neighbour lookups without allocation.
pub fn snake_membership_mask<'a>(
    segments: &[usize],
    total_points: usize,
    scratch: &'a mut Vec<bool>,
) -> &'a [bool] {
    if scratch.len() < total_points {
        scratch.resize(total_points, false);
    } else {
        scratch[..total_points].fill(false);
    }

    for &segment_index in segments {
        if segment_index < total_points {
            scratch[segment_index] = true;
        }
    }

    &scratch[..total_points]
}

/// Check membership in a boolean mask safely.
#[inline]
pub fn snake_mask_contains(mask: &[bool], idx: usize) -> bool {
    mask.get(idx).copied().unwrap_or(false)
}
