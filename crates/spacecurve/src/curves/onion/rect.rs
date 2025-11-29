//! Rectangular (per-dimension L) Onion used on half-faces.
//!
//! This is the core fix: instead of composing C1×C2 with a boustrophedon product,
//! we traverse each half‑face using an onion on the (N-1)‑D *rectangular* box.
//!
//! `sizes`: per-dimension side lengths. Coordinates p are 0-based with bounds 0..L_i-1.

/// Compute the index within a rectangular onion traversal.
pub(super) fn onion_index_rect(sizes: &[u32], p: &[u32]) -> u32 {
    let m = sizes.len() as u32;
    if m == 0 {
        return 0;
    }
    if m == 1 {
        return p[0];
    }

    // Compute inner sizes (saturating at 0) and inner check.
    let mut inner_sizes: Vec<u32> = Vec::with_capacity(sizes.len());
    let mut is_inner = true;
    for (&l_i, &q_i) in sizes.iter().zip(p.iter()) {
        let inner = l_i.saturating_sub(2);
        inner_sizes.push(inner);
        if l_i <= 1 || q_i == 0 || q_i == l_i - 1 {
            is_inner = false;
        }
    }

    // Volumes
    let total: u32 = sizes.iter().fold(1u32, |acc, &x| {
        acc.checked_mul(x)
            .expect("Overflow in rectangular total volume")
    });
    let inner_vol: u32 = inner_sizes.iter().fold(1u32, |acc, &x| {
        acc.checked_mul(x)
            .expect("Overflow in rectangular inner volume")
    });
    let outer = total - inner_vol;

    if is_inner {
        // Shift inwards and recurse.
        let mut p_inner = Vec::with_capacity(p.len());
        for (&q, &l_i) in p.iter().zip(sizes.iter()) {
            debug_assert!(l_i >= 2 && q > 0 && q < l_i - 1);
            p_inner.push(q - 1);
        }
        return outer + onion_index_rect(&inner_sizes, &p_inner);
    }

    // 2) Outer layer: find first boundary dimension i*
    let mut i_star: usize = usize::MAX;
    for (i, (&l_i, &q_i)) in sizes.iter().zip(p.iter()).enumerate() {
        if l_i == 0 {
            continue;
        }
        if q_i == 0 || q_i == l_i - 1 {
            i_star = i;
            break;
        }
    }
    assert!(
        i_star != usize::MAX,
        "No boundary coordinate found on outer layer"
    );

    // 3) Offset of partitions P_j for j < i*
    let mut offset_p: u32 = 0;
    for j in 0..i_star {
        let side_factor: u32 = if sizes[j] >= 2 { 2 } else { 1 };
        // pre product: ∏_{k<j} (L_k - 2)
        let pre: u32 = sizes[..j].iter().fold(1u32, |acc, &l_k| {
            acc.checked_mul(l_k.saturating_sub(2))
                .expect("Overflow in pre product")
        });
        // post product: ∏_{k>j} L_k
        let post: u32 = sizes[j + 1..].iter().fold(1u32, |acc, &l_k| {
            acc.checked_mul(l_k).expect("Overflow in post product")
        });
        let size_pj = side_factor
            .checked_mul(pre)
            .and_then(|x| x.checked_mul(post))
            .expect("Overflow in size(P_j)");
        offset_p = offset_p.checked_add(size_pj).expect("Overflow in offset_p");
    }

    // 4) Select sub-part on dimension i* (low vs high). If L_i*==1 there is only one side.
    let pre_i: u32 = sizes[..i_star].iter().fold(1u32, |acc, &l_k| {
        acc.checked_mul(l_k.saturating_sub(2))
            .expect("Overflow in pre_i")
    });
    let post_i: u32 = sizes[i_star + 1..].iter().fold(1u32, |acc, &l_k| {
        acc.checked_mul(l_k).expect("Overflow in post_i")
    });
    let face_block = pre_i.checked_mul(post_i).expect("Overflow in face_block");

    let mut offset_sub = 0u32;
    if sizes[i_star] >= 2 && p[i_star] == sizes[i_star] - 1 {
        offset_sub = face_block;
    }

    // 5) Index within the chosen half‑face using a rectangular onion on remaining dims.
    let mut face_sizes: Vec<u32> = Vec::with_capacity(sizes.len().saturating_sub(1));
    let mut face_coords: Vec<u32> = Vec::with_capacity(p.len().saturating_sub(1));

    // Left block (< i*): sizes - 2, coords - 1
    for &l_k in &sizes[..i_star] {
        face_sizes.push(l_k.saturating_sub(2));
    }
    for &q_k in &p[..i_star] {
        face_coords.push(q_k - 1);
    }
    // Right block (> i*): sizes intact, coords intact
    for &l_k in &sizes[i_star + 1..] {
        face_sizes.push(l_k);
    }
    for &q_k in &p[i_star + 1..] {
        face_coords.push(q_k);
    }

    let i_face = onion_index_rect(&face_sizes, &face_coords);
    offset_p + offset_sub + i_face
}

/// Inverse mapping for `onion_index_rect` on a rectangular face.
pub(super) fn onion_point_rect(sizes: &[u32], mut index: u32) -> Vec<u32> {
    let m = sizes.len();
    if m == 0 {
        return vec![];
    }
    if m == 1 {
        return vec![index];
    }

    // Inner sizes and volumes
    let mut inner_sizes: Vec<u32> = Vec::with_capacity(m);
    for &l_i in sizes.iter() {
        inner_sizes.push(l_i.saturating_sub(2));
    }
    let total: u32 = sizes.iter().fold(1u32, |acc, &x| {
        acc.checked_mul(x)
            .expect("Overflow in rectangular total volume")
    });
    let inner_vol: u32 = inner_sizes.iter().fold(1u32, |acc, &x| {
        acc.checked_mul(x)
            .expect("Overflow in rectangular inner volume")
    });
    let outer = total - inner_vol;

    if index >= outer {
        // Inner
        let idx_inner = index - outer;
        let mut p_inner = onion_point_rect(&inner_sizes, idx_inner);
        for v in &mut p_inner {
            *v += 1;
        }
        return p_inner;
    }

    // Outer: find partition P_i*
    let mut i_star: usize = usize::MAX;
    for j in 0..m {
        let side_factor: u32 = if sizes[j] >= 2 { 2 } else { 1 };
        let pre: u32 = sizes[..j].iter().fold(1u32, |acc, &l_k| {
            acc.checked_mul(l_k.saturating_sub(2))
                .expect("Overflow in pre product")
        });
        let post: u32 = sizes[j + 1..].iter().fold(1u32, |acc, &l_k| {
            acc.checked_mul(l_k).expect("Overflow in post product")
        });
        let size_pj = side_factor
            .checked_mul(pre)
            .and_then(|x| x.checked_mul(post))
            .expect("Overflow in size(P_j)");

        if index < size_pj {
            i_star = j;
            break;
        } else {
            index -= size_pj;
        }
    }
    assert!(
        i_star != usize::MAX,
        "Failed to locate partition in onion_point_rect"
    );

    // Select sub-part (low/high) and compute index within half-face
    let pre_i: u32 = sizes[..i_star].iter().fold(1u32, |acc, &l_k| {
        acc.checked_mul(l_k.saturating_sub(2))
            .expect("Overflow in pre_i")
    });
    let post_i: u32 = sizes[i_star + 1..].iter().fold(1u32, |acc, &l_k| {
        acc.checked_mul(l_k).expect("Overflow in post_i")
    });
    let face_block = pre_i.checked_mul(post_i).expect("Overflow in face_block");

    let high_side: bool;
    if sizes[i_star] >= 2 {
        if index < face_block {
            high_side = false;
        } else {
            index -= face_block;
            high_side = true;
        }
    } else {
        // Only one side when L_i*==1
        high_side = false;
    }

    // Map index to coordinates on the face via rectangular onion
    let mut face_sizes: Vec<u32> = Vec::with_capacity(m - 1);
    // sizes for k< i*: L_k - 2 ; for k> i*: L_k
    for &l_k in &sizes[..i_star] {
        face_sizes.push(l_k.saturating_sub(2));
    }
    for &l_k in &sizes[i_star + 1..] {
        face_sizes.push(l_k);
    }

    let mut face_coords = onion_point_rect(&face_sizes, index);

    // Reconstruct full coordinate
    let mut p = Vec::with_capacity(m);
    // Left block (< i*): shift +1
    let left_len = i_star;
    for _ in 0..left_len {
        let v = face_coords.remove(0);
        p.push(v + 1);
    }
    // Boundary coordinate
    let coord_i = if sizes[i_star] >= 2 {
        if high_side { sizes[i_star] - 1 } else { 0 }
    } else {
        0
    };
    p.push(coord_i);
    // Right block (> i*): direct
    for v in face_coords {
        p.push(v);
    }
    p
}
