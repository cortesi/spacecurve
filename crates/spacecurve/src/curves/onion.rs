/// The Onion Curve is a space-filling curve named after the core concept of "peeling" an
/// N-dimensional hypercube layer by layer, like an onion.
///
/// See: https://arxiv.org/abs/1801.07399
///
/// Notes:
/// * L=2 uses a continuous Gray‑code generalisation (O(N-1) then reversed O(N-1) with
///   the last coordinate as the discriminator).
/// * For N>2 and L>2 the onion curve cannot be fully continuous (see comment below),
///   but this implementation maximises locality on each shell by using an onion on
///   each half‑face instead of a boustrophedon stripe.
///
/// The full implementation (core logic plus 2D/L2/rectangular/3D specialisations) lives
/// in this single module to make the traversal easier to follow.
///
/// Continuity impossibility sketch (unchanged):
/// Consider a 3×3×3 cube. Chessboard‑colour cells by parity of the coordinate sum.
/// The outer shell has 26 cells (even). The center cell is White, hence the shell
/// must end on White; any continuous traversal into the next shell would need to
/// enter a Black cell, contradiction.
use crate::{error, point::Point, spacecurve::SpaceCurve, spec::GridSpec};

/// Onion curve operating on L∞ shells in N‑D.
#[derive(Debug)]
pub struct OnionCurve {
    /// Number of dimensions in the grid.
    dimensions: u32,
    /// Side length per dimension.
    side_length: u32,
    /// Total number of points (L^N).
    length: u32,
}

impl OnionCurve {
    /// Construct a new Onion curve for `dimensions` and `side_length`.
    pub fn new(dimensions: u32, side_length: u32) -> error::Result<Self> {
        let spec = GridSpec::new(dimensions, side_length)?;
        // Special-case overflow guard retained for L=2 where 2^N grows quickly.
        if side_length == 2 && dimensions > 31 {
            return Err(error::Error::Size(
                "For L=2, dimensions must be <= 31 (2^N must fit in u32)".to_string(),
            ));
        }

        Ok(Self {
            dimensions: spec.dimension(),
            side_length: spec.size(),
            length: spec.length(),
        })
    }
}

impl SpaceCurve for OnionCurve {
    fn name(&self) -> &'static str {
        "Onion"
    }

    fn info(&self) -> &'static str {
        "Peels L∞ layers. L=2 uses Gray-code generalisation (continuous); N>2,L>2 is discontinuous."
    }

    fn dimensions(&self) -> u32 {
        self.dimensions
    }

    fn length(&self) -> u32 {
        self.length
    }

    fn index(&self, p: &Point) -> u32 {
        debug_assert_eq!(
            p.len(),
            self.dimensions as usize,
            "point dimension mismatch"
        );
        debug_assert!(
            p.iter().all(|&c| c < self.side_length),
            "point coordinate out of bounds"
        );
        onion_index_nd(self.dimensions, self.side_length, p)
    }

    fn point(&self, index: u32) -> Point {
        debug_assert!(index < self.length, "index out of bounds");
        let coords = onion_point_nd(self.dimensions, self.side_length, index % self.length);
        Point::new_with_dimension(self.dimensions, coords)
    }
}

/// Describes a single L∞ shell within the onion traversal.
#[derive(Clone, Copy, Debug)]
struct Shell {
    /// Layer index from the outside (0 is the outermost shell).
    level: u32,
    /// Side length of the cube for this shell (after trimming `level` layers).
    side: u32,
    /// Cumulative number of points before this shell begins.
    offset: u32,
    /// Index relative to the start of the current shell.
    index_within: u32,
}

/// Checked exponent helper backed by the validated grid specification.
fn pow_u32(base: u32, exp: u32) -> u32 {
    base.checked_pow(exp)
        .expect("Grid specification prevents overflow")
}

/// Number of points on the outer shell of an `side^dimension` cube.
fn shell_size(dimension: u32, side: u32) -> u32 {
    if side == 0 {
        return 0;
    }
    let inner = side.saturating_sub(2);
    pow_u32(side, dimension) - pow_u32(inner, dimension)
}

/// Locate the shell that contains `index`.
fn shell_for_index(dimension: u32, side: u32, mut index: u32) -> Shell {
    let mut side_at_level = side;
    let mut level = 0;
    let mut offset = 0;
    loop {
        let size = shell_size(dimension, side_at_level);
        if index < size {
            return Shell {
                level,
                side: side_at_level,
                offset,
                index_within: index,
            };
        }
        index -= size;
        offset += size;
        level += 1;
        side_at_level = side_at_level.saturating_sub(2);
    }
}

/// Locate the shell and offset for a given point.
fn shell_for_point(dimension: u32, side: u32, point: &[u32]) -> Shell {
    let level = point
        .iter()
        .map(|&c| c.min(side - 1 - c))
        .min()
        .unwrap_or(0);
    let mut side_at_level = side;
    let mut offset = 0;
    for _ in 0..level {
        let size = shell_size(dimension, side_at_level);
        offset += size;
        side_at_level = side_at_level.saturating_sub(2);
    }
    Shell {
        level,
        side: side_at_level,
        offset,
        index_within: 0,
    }
}

/// First boundary coordinate (dimension, high_side) for a shell-local point.
fn first_boundary(local: &[u32], side: u32) -> (usize, bool) {
    for (idx, &coord) in local.iter().enumerate() {
        if coord == 0 {
            return (idx, false);
        }
        if coord + 1 == side {
            return (idx, true);
        }
    }
    debug_assert!(
        false,
        "onion shell requires at least one boundary coordinate"
    );
    (0, false)
}

/// Size of each partition P_j on the shell, ordered by first boundary dimension.
fn partition_sizes(dimension: u32, side: u32) -> Vec<u32> {
    let inner = side.saturating_sub(2);
    (0..dimension)
        .map(|j| {
            let pre = pow_u32(inner, j);
            let post = pow_u32(side, dimension - 1 - j);
            2u32.checked_mul(pre)
                .and_then(|v| v.checked_mul(post))
                .expect("validated shell volume")
        })
        .collect()
}

/// Side lengths of the (N-1)-D face when fixing `boundary_dim`.
fn face_sizes(dimension: u32, side: u32, boundary_dim: usize) -> Vec<u32> {
    let mut sizes = Vec::with_capacity(dimension as usize - 1);
    let inner = side.saturating_sub(2);
    for _ in 0..boundary_dim {
        sizes.push(inner);
    }
    for _ in boundary_dim + 1..dimension as usize {
        sizes.push(side);
    }
    sizes
}

/// Map shell-local coordinates into face-local coordinates.
fn face_coords_from_point(local: &[u32], boundary_dim: usize) -> Vec<u32> {
    let mut coords = Vec::with_capacity(local.len().saturating_sub(1));
    for &coord in &local[..boundary_dim] {
        coords.push(coord.saturating_sub(1));
    }
    for &coord in &local[boundary_dim + 1..] {
        coords.push(coord);
    }
    coords
}

/// Rebuild full shell-local coordinates from face-local ones.
fn rebuild_from_face(
    face_coords: Vec<u32>,
    boundary_dim: usize,
    side: u32,
    high_side: bool,
) -> Vec<u32> {
    let mut coords = Vec::with_capacity(face_coords.len() + 1);
    let mut iter = face_coords.into_iter();
    for _ in 0..boundary_dim {
        coords.push(iter.next().unwrap_or(0) + 1);
    }
    coords.push(if high_side { side - 1 } else { 0 });
    coords.extend(iter);
    coords
}

/// Compute the index within a shell for a shell-local point.
fn onion_shell_index(dimension: u32, side: u32, local: &[u32]) -> u32 {
    if side == 1 {
        return 0;
    }
    if side == 2 {
        return onion_index_l2(dimension, local);
    }
    if dimension == 1 {
        return local[0];
    }
    if dimension == 2 {
        return onion_index_2d(side, local);
    }

    let (boundary_dim, high_side) = first_boundary(local, side);
    let offsets = partition_sizes(dimension, side);
    debug_assert_eq!(
        offsets.iter().sum::<u32>(),
        shell_size(dimension, side),
        "partition sizes should tile the shell"
    );
    let offset_p: u32 = offsets.iter().take(boundary_dim).copied().sum();

    let inner = side.saturating_sub(2);
    let sub_part_size = pow_u32(inner, boundary_dim as u32)
        .checked_mul(pow_u32(side, dimension - 1 - boundary_dim as u32))
        .expect("validated sub part");
    let offset_sub = if high_side { sub_part_size } else { 0 };

    let face_sizes = face_sizes(dimension, side, boundary_dim);
    let face_coords = face_coords_from_point(local, boundary_dim);
    let within = onion_index_rect(&face_sizes, &face_coords);

    offset_p + offset_sub + within
}

/// Compute shell-local coordinates from an index inside the shell.
fn onion_shell_point(dimension: u32, side: u32, mut index: u32) -> Vec<u32> {
    if side == 1 {
        return vec![0; dimension as usize];
    }
    if side == 2 {
        return onion_point_l2(dimension, index);
    }
    if dimension == 1 {
        return vec![index];
    }
    if dimension == 2 {
        return onion_point_2d(side, index);
    }

    let partitions = partition_sizes(dimension, side);
    debug_assert_eq!(
        partitions.iter().sum::<u32>(),
        shell_size(dimension, side),
        "partition sizes should tile the shell"
    );
    let mut boundary_dim = 0usize;
    for (j, size) in partitions.iter().enumerate() {
        if index < *size {
            boundary_dim = j;
            break;
        }
        index -= *size;
    }

    let inner = side.saturating_sub(2);
    let sub_part_size = pow_u32(inner, boundary_dim as u32)
        .checked_mul(pow_u32(side, dimension - 1 - boundary_dim as u32))
        .expect("validated sub part");

    let high_side = if index < sub_part_size {
        false
    } else {
        index -= sub_part_size;
        true
    };

    let face_sizes = face_sizes(dimension, side, boundary_dim);
    let face_coords = onion_point_rect(&face_sizes, index);

    rebuild_from_face(face_coords, boundary_dim, side, high_side)
}

/// Full onion index for a point in an N-D cube.
fn onion_index_nd(dimension: u32, side: u32, point: &[u32]) -> u32 {
    if dimension == 0 || side == 0 {
        return 0;
    }
    if dimension == 3 && side > 2 {
        return onion_index_3d(side, point);
    }
    let shell = shell_for_point(dimension, side, point);
    let local: Vec<u32> = point.iter().map(|&c| c - shell.level).collect();
    let within = onion_shell_index(dimension, shell.side, &local);
    shell.offset + within
}

/// Full onion coordinates for an index in an N-D cube.
fn onion_point_nd(dimension: u32, side: u32, index: u32) -> Vec<u32> {
    if dimension == 0 || side == 0 {
        return vec![];
    }
    if dimension == 3 && side > 2 {
        return onion_point_3d(side, index);
    }
    let shell = shell_for_index(dimension, side, index);
    let local = onion_shell_point(dimension, shell.side, shell.index_within);
    local.into_iter().map(|c| c + shell.level).collect()
}

// === Specialisations ===

/// Compute the onion index for L=2 using Gray-code generalisation.
fn onion_index_l2(n: u32, p: &[u32]) -> u32 {
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
fn onion_point_l2(n: u32, index: u32) -> Vec<u32> {
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

/// Compute the onion index for 2D (continuous spiral).
pub(crate) fn onion_index_2d(l: u32, p: &[u32]) -> u32 {
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
pub(crate) fn onion_point_2d(l: u32, index: u32) -> Vec<u32> {
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

/// Compute the index within a rectangular onion traversal.
fn onion_index_rect(sizes: &[u32], p: &[u32]) -> u32 {
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
fn onion_point_rect(sizes: &[u32], mut index: u32) -> Vec<u32> {
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

/// Cube volume helper dedicated to the specialised 3D ordering.
fn cube_volume(side: u32) -> u32 {
    pow_u32(side, 3)
}

/// Specialised 3D outer-shell ordering that mirrors the published definition.
fn onion_index_3d(side_length: u32, point: &[u32]) -> u32 {
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
fn onion_point_3d(side_length: u32, index: u32) -> Vec<u32> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constructor_guards() {
        // L==0 rejected
        assert!(OnionCurve::new(2, 0).is_err());
        // N==0 rejected
        assert!(OnionCurve::new(0, 4).is_err());
        // Valid small shapes
        let c = OnionCurve::new(2, 3).unwrap();
        assert_eq!(c.length(), 9);
    }

    #[test]
    fn roundtrip_dims_2_to_4_sizes_upto_8() {
        for dim in 2..=4 {
            for size in 2..=8 {
                let curve = OnionCurve::new(dim, size).unwrap();
                for idx in 0..curve.length() {
                    let p = curve.point(idx);
                    assert_eq!(
                        curve.index(&p),
                        idx,
                        "roundtrip failed for dim {dim}, size {size}, idx {idx}"
                    );
                }
            }
        }
    }
}
