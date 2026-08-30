/// The Onion Curve is a space-filling curve named after the core concept of
/// "peeling" an N-dimensional hypercube layer by layer, like an onion.
///
/// See: https://arxiv.org/abs/1801.07399
///
/// Notes:
/// * L=2 uses a continuous Gray‑code generalisation (O(N-1) then reversed
///   O(N-1) with the last coordinate as the discriminator).
/// * For N>2 and L>2 the onion curve cannot be fully continuous (see comment
///   below), but this implementation maximises locality on each shell by using
///   an onion on each half‑face instead of a boustrophedon stripe.
///
/// The full implementation (core logic plus 2D/L2/rectangular/3D
/// specialisations) lives in this single module to make the traversal easier to
/// follow.
///
/// Continuity impossibility sketch (unchanged):
/// Consider a 3×3×3 cube. Chessboard‑colour cells by parity of the coordinate
/// sum. The outer shell has 26 cells (even). The center cell is White, hence
/// the shell must end on White; any continuous traversal into the next shell
/// would need to enter a Black cell, contradiction.
use crate::{
    error, ops,
    point::Point,
    spacecurve::SpaceCurve,
    spec::GridSpec,
    types::{Coord, Index},
};

/// Onion curve operating on L∞ shells in N‑D.
#[derive(Debug)]
pub struct OnionCurve<C: Coord, I: Index> {
    /// Number of dimensions in the grid.
    dimensions: u32,
    /// Side length per dimension.
    side_length: C,
    /// Total number of points (L^N).
    length: I,
}

impl<C: Coord, I: Index> OnionCurve<C, I> {
    /// Construct a new Onion curve for `dimensions` and `side_length`.
    pub fn new(dimensions: u32, side_length: C) -> error::Result<Self> {
        let spec = GridSpec::<C, I>::new(dimensions, side_length)?;
        Ok(Self {
            dimensions: spec.dimension(),
            side_length: spec.size(),
            length: spec.length(),
        })
    }
}

impl<C: Coord, I: Index> SpaceCurve for OnionCurve<C, I> {
    type Coord = C;
    type Index = I;
    fn name(&self) -> &'static str {
        "Onion"
    }

    fn info(&self) -> &'static str {
        "Peels L∞ layers. L=2 uses Gray-code generalisation (continuous); N>2,L>2 is discontinuous."
    }

    fn dimensions(&self) -> u32 {
        self.dimensions
    }

    fn length(&self) -> I {
        self.length
    }

    fn index(&self, p: &Point<C>) -> I {
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

    fn point(&self, index: I) -> Point<C> {
        debug_assert!(index < self.length, "index out of bounds");
        let coords = onion_point_nd(self.dimensions, self.side_length, index % self.length);
        Point::new_with_dimension(self.dimensions, coords)
    }
}

/// Describes a single L∞ shell within the onion traversal.
#[derive(Clone, Copy, Debug)]
struct Shell<C: Coord, I: Index> {
    /// Layer index from the outside (0 is the outermost shell).
    level: C,
    /// Side length of the cube for this shell (after trimming `level` layers).
    side: C,
    /// Cumulative number of points before this shell begins.
    offset: I,
    /// Index relative to the start of the current shell.
    index_within: I,
}

/// Checked exponent helper backed by the validated grid specification.
fn pow_size<C: Coord, I: Index>(base: C, exp: u32) -> I {
    let base_index = I::from(base).expect("size fits index type");
    ops::checked_pow(base_index, exp).expect("grid specification prevents overflow")
}

/// Number of points on the outer shell of an `side^dimension` cube.
fn shell_size<C: Coord, I: Index>(dimension: u32, side: C) -> I {
    if side.is_zero() {
        return I::zero();
    }
    let inner = side.saturating_sub(two());
    pow_size::<C, I>(side, dimension) - pow_size::<C, I>(inner, dimension)
}

/// Locate the shell that contains `index`.
fn shell_for_index<C: Coord, I: Index>(dimension: u32, side: C, mut index: I) -> Shell<C, I> {
    let mut side_at_level = side;
    let mut level = C::zero();
    let mut offset = I::zero();
    loop {
        let size = shell_size::<C, I>(dimension, side_at_level);
        if index < size {
            return Shell {
                level,
                side: side_at_level,
                offset,
                index_within: index,
            };
        }
        index = index - size;
        offset = offset + size;
        level = level + C::one();
        side_at_level = side_at_level.saturating_sub(two());
    }
}

/// Locate the shell and offset for a given point.
fn shell_for_point<C: Coord, I: Index>(dimension: u32, side: C, point: &[C]) -> Shell<C, I> {
    let level = point
        .iter()
        .map(|&c| c.min(side - C::one() - c))
        .min()
        .unwrap_or_else(C::zero);
    let mut side_at_level = side;
    let mut offset = I::zero();
    let level_count = to_usize(level);
    for _ in 0..level_count {
        let size = shell_size::<C, I>(dimension, side_at_level);
        offset = offset + size;
        side_at_level = side_at_level.saturating_sub(two());
    }
    Shell {
        level,
        side: side_at_level,
        offset,
        index_within: I::zero(),
    }
}

/// First boundary coordinate (dimension, high_side) for a shell-local point.
fn first_boundary<C: Coord>(local: &[C], side: C) -> (usize, bool) {
    for (idx, &coord) in local.iter().enumerate() {
        if coord.is_zero() {
            return (idx, false);
        }
        if coord + C::one() == side {
            return (idx, true);
        }
    }
    debug_assert!(
        false,
        "onion shell requires at least one boundary coordinate"
    );
    (0, false)
}

/// Size of each partition P_j on the shell, ordered by first boundary
/// dimension.
fn partition_sizes<C: Coord, I: Index>(dimension: u32, side: C) -> Vec<I> {
    let inner = side.saturating_sub(two());
    (0..dimension)
        .map(|j| {
            let pre = pow_size::<C, I>(inner, j);
            let post = pow_size::<C, I>(side, dimension - 1 - j);
            let two = I::one() + I::one();
            two.checked_mul(&pre)
                .and_then(|v| v.checked_mul(&post))
                .expect("validated shell volume")
        })
        .collect()
}

/// Side lengths of the (N-1)-D face when fixing `boundary_dim`.
fn face_sizes<C: Coord>(dimension: u32, side: C, boundary_dim: usize) -> Vec<C> {
    let mut sizes = Vec::with_capacity(dimension as usize - 1);
    let inner = side.saturating_sub(two());
    for _ in 0..boundary_dim {
        sizes.push(inner);
    }
    for _ in boundary_dim + 1..dimension as usize {
        sizes.push(side);
    }
    sizes
}

/// Map shell-local coordinates into face-local coordinates.
fn face_coords_from_point<C: Coord>(local: &[C], boundary_dim: usize) -> Vec<C> {
    let mut coords = Vec::with_capacity(local.len().saturating_sub(1));
    for &coord in &local[..boundary_dim] {
        coords.push(coord.saturating_sub(C::one()));
    }
    for &coord in &local[boundary_dim + 1..] {
        coords.push(coord);
    }
    coords
}

/// Rebuild full shell-local coordinates from face-local ones.
fn rebuild_from_face<C: Coord>(
    face_coords: Vec<C>,
    boundary_dim: usize,
    side: C,
    high_side: bool,
) -> Vec<C> {
    let mut coords = Vec::with_capacity(face_coords.len() + 1);
    let mut iter = face_coords.into_iter();
    for _ in 0..boundary_dim {
        coords.push(iter.next().unwrap_or_else(C::zero) + C::one());
    }
    coords.push(if high_side {
        side - C::one()
    } else {
        C::zero()
    });
    coords.extend(iter);
    coords
}

/// Compute the index within a shell for a shell-local point.
fn onion_shell_index<C: Coord, I: Index>(dimension: u32, side: C, local: &[C]) -> I {
    if side == C::one() {
        return I::zero();
    }
    if side == two() {
        return onion_index_l2::<C, I>(dimension, local);
    }
    if dimension == 1 {
        return I::from(local[0]).expect("coordinate fits index type");
    }
    if dimension == 2 {
        return onion_index_2d::<C, I>(side, local);
    }

    let (boundary_dim, high_side) = first_boundary(local, side);
    let offsets = partition_sizes::<C, I>(dimension, side);
    debug_assert_eq!(
        offsets.iter().copied().fold(I::zero(), |acc, v| acc + v),
        shell_size::<C, I>(dimension, side),
        "partition sizes should tile the shell"
    );
    let offset_p: I = offsets
        .iter()
        .take(boundary_dim)
        .copied()
        .fold(I::zero(), |acc, v| acc + v);

    let inner = side.saturating_sub(two());
    let sub_part_size = pow_size::<C, I>(inner, boundary_dim as u32)
        .checked_mul(&pow_size::<C, I>(side, dimension - 1 - boundary_dim as u32))
        .expect("validated sub part");
    let offset_sub = if high_side { sub_part_size } else { I::zero() };

    let face_sizes = face_sizes(dimension, side, boundary_dim);
    let face_coords = face_coords_from_point(local, boundary_dim);
    let within = onion_index_rect::<C, I>(&face_sizes, &face_coords);

    offset_p + offset_sub + within
}

/// Compute shell-local coordinates from an index inside the shell.
fn onion_shell_point<C: Coord, I: Index>(dimension: u32, side: C, mut index: I) -> Vec<C> {
    if side == C::one() {
        return vec![C::zero(); dimension as usize];
    }
    if side == two() {
        return onion_point_l2::<C, I>(dimension, index);
    }
    if dimension == 1 {
        return vec![to_coord::<C, I>(index)];
    }
    if dimension == 2 {
        return onion_point_2d::<C, I>(side, index);
    }

    let partitions = partition_sizes::<C, I>(dimension, side);
    debug_assert_eq!(
        partitions.iter().copied().fold(I::zero(), |acc, v| acc + v),
        shell_size::<C, I>(dimension, side),
        "partition sizes should tile the shell"
    );
    let mut boundary_dim = 0usize;
    for (j, size) in partitions.iter().enumerate() {
        if index < *size {
            boundary_dim = j;
            break;
        }
        index = index - *size;
    }

    let inner = side.saturating_sub(two());
    let sub_part_size = pow_size::<C, I>(inner, boundary_dim as u32)
        .checked_mul(&pow_size::<C, I>(side, dimension - 1 - boundary_dim as u32))
        .expect("validated sub part");

    let high_side = if index < sub_part_size {
        false
    } else {
        index = index - sub_part_size;
        true
    };

    let face_sizes = face_sizes(dimension, side, boundary_dim);
    let face_coords = onion_point_rect::<C, I>(&face_sizes, index);

    rebuild_from_face(face_coords, boundary_dim, side, high_side)
}

/// Full onion index for a point in an N-D cube.
fn onion_index_nd<C: Coord, I: Index>(dimension: u32, side: C, point: &[C]) -> I {
    if dimension == 0 || side.is_zero() {
        return I::zero();
    }
    if dimension == 3 && side > two() {
        return onion_index_3d::<C, I>(side, point);
    }
    let shell = shell_for_point::<C, I>(dimension, side, point);
    let local: Vec<C> = point.iter().map(|&c| c - shell.level).collect();
    let within = onion_shell_index::<C, I>(dimension, shell.side, &local);
    shell.offset + within
}

/// Full onion coordinates for an index in an N-D cube.
fn onion_point_nd<C: Coord, I: Index>(dimension: u32, side: C, index: I) -> Vec<C> {
    if dimension == 0 || side.is_zero() {
        return vec![];
    }
    if dimension == 3 && side > two() {
        return onion_point_3d::<C, I>(side, index);
    }
    let shell = shell_for_index::<C, I>(dimension, side, index);
    let local = onion_shell_point::<C, I>(dimension, shell.side, shell.index_within);
    local.into_iter().map(|c| c + shell.level).collect()
}

// === Specialisations ===

/// Compute the onion index for L=2 using Gray-code generalisation.
fn onion_index_l2<C: Coord, I: Index>(n: u32, p: &[C]) -> I {
    if n == 0 {
        return I::zero();
    }
    let dim_prev = n - 1;
    let volume_prev = I::one()
        .checked_shl(dim_prev)
        .expect("volume fits index type"); // 2^(N-1)
    let last = p[n as usize - 1];
    let i_prev = onion_index_l2::<C, I>(dim_prev, &p[..n as usize - 1]);
    if last.is_zero() {
        i_prev
    } else {
        (volume_prev - I::one()) - i_prev + volume_prev
    }
}

/// Inverse for the `L=2` specialised onion index.
fn onion_point_l2<C: Coord, I: Index>(n: u32, index: I) -> Vec<C> {
    if n == 0 {
        return vec![];
    }
    let dim_prev = n - 1;
    let volume_prev = I::one()
        .checked_shl(dim_prev)
        .expect("volume fits index type");
    let (last, i_prev) = if index < volume_prev {
        (C::zero(), index)
    } else {
        let idx = index - volume_prev;
        (C::one(), (volume_prev - I::one()) - idx)
    };
    let mut p = onion_point_l2::<C, I>(dim_prev, i_prev);
    p.push(last);
    p
}

/// Compute the onion index for 2D (continuous spiral).
pub(crate) fn onion_index_2d<C: Coord, I: Index>(l: C, p: &[C]) -> I {
    if l <= C::one() {
        return I::zero();
    }
    let x = p[0];
    let y = p[1];
    let l_i = to_index::<C, I>(l);
    let x_i = to_index::<C, I>(x);
    let y_i = to_index::<C, I>(y);
    let one_i = I::one();
    let two_i = one_i + one_i;
    let three_i = two_i + one_i;
    let four_i = two_i + two_i;
    // 1) Bottom edge
    if y.is_zero() {
        return x_i;
    }
    // 2) Right edge
    if x == l - C::one() {
        return (l_i - one_i) + y_i;
    }
    // 3) Top edge
    if y == l - C::one() {
        return three_i * l_i - three_i - x_i;
    }
    // 4) Left edge
    if x.is_zero() {
        return four_i * l_i - four_i - y_i;
    }
    // 5) Inner
    let outer = four_i * l_i - four_i;
    let p_inner = vec![x - C::one(), y - C::one()];
    outer + onion_index_2d::<C, I>(l.saturating_sub(two()), &p_inner)
}

/// Inverse of `onion_index_2d`.
pub(crate) fn onion_point_2d<C: Coord, I: Index>(l: C, index: I) -> Vec<C> {
    if l == C::one() {
        return vec![C::zero(), C::zero()];
    }
    if l.is_zero() {
        unreachable!("L==0 is rejected by OnionCurve::new");
    }

    let l_i = to_index::<C, I>(l);
    let one_i = I::one();
    let two_i = one_i + one_i;
    let three_i = two_i + one_i;
    let four_i = two_i + two_i;
    let outer_layer_size = four_i * l_i - four_i;

    if index >= outer_layer_size {
        // Inner square
        let p_inner = onion_point_2d::<C, I>(l.saturating_sub(two()), index - outer_layer_size);
        return vec![p_inner[0] + C::one(), p_inner[1] + C::one()];
    }

    // Outer layer
    if index < l_i {
        return vec![to_coord::<C, I>(index), C::zero()];
    }
    if index < two_i * l_i - one_i {
        return vec![l - C::one(), to_coord::<C, I>(index - l_i + one_i)];
    }
    if index < three_i * l_i - two_i {
        return vec![
            to_coord::<C, I>(three_i * l_i - three_i - index),
            l - C::one(),
        ];
    }
    vec![C::zero(), to_coord::<C, I>(four_i * l_i - four_i - index)]
}

/// Compute the index within a rectangular onion traversal.
fn onion_index_rect<C: Coord, I: Index>(sizes: &[C], p: &[C]) -> I {
    let m = sizes.len();
    if m == 0 {
        return I::zero();
    }
    if m == 1 {
        return to_index::<C, I>(p[0]);
    }

    // Compute inner sizes (saturating at 0) and inner check.
    let mut inner_sizes: Vec<C> = Vec::with_capacity(sizes.len());
    let mut is_inner = true;
    for (&l_i, &q_i) in sizes.iter().zip(p.iter()) {
        let inner = l_i.saturating_sub(two());
        inner_sizes.push(inner);
        if l_i <= C::one() || q_i.is_zero() || q_i == l_i - C::one() {
            is_inner = false;
        }
    }

    // Volumes
    let total: I = sizes.iter().fold(I::one(), |acc, &x| {
        acc.checked_mul(&to_index::<C, I>(x))
            .expect("overflow in rectangular total volume")
    });
    let inner_vol: I = inner_sizes.iter().fold(I::one(), |acc, &x| {
        acc.checked_mul(&to_index::<C, I>(x))
            .expect("overflow in rectangular inner volume")
    });
    let outer = total - inner_vol;

    if is_inner {
        // Shift inwards and recurse.
        let mut p_inner = Vec::with_capacity(p.len());
        for (&q, &l_i) in p.iter().zip(sizes.iter()) {
            debug_assert!(l_i >= two() && q > C::zero() && q < l_i - C::one());
            p_inner.push(q - C::one());
        }
        return outer + onion_index_rect::<C, I>(&inner_sizes, &p_inner);
    }

    // 2) Outer layer: find first boundary dimension i*
    let mut i_star: usize = usize::MAX;
    for (i, (&l_i, &q_i)) in sizes.iter().zip(p.iter()).enumerate() {
        if l_i.is_zero() {
            continue;
        }
        if q_i.is_zero() || q_i == l_i - C::one() {
            i_star = i;
            break;
        }
    }
    assert!(
        i_star != usize::MAX,
        "No boundary coordinate found on outer layer"
    );

    // 3) Offset of partitions P_j for j < i*
    let mut offset_p: I = I::zero();
    for j in 0..i_star {
        let side_factor: I = if sizes[j] >= two() {
            I::one() + I::one()
        } else {
            I::one()
        };
        // pre product: ∏_{k<j} (L_k - 2)
        let pre: I = sizes[..j].iter().fold(I::one(), |acc, &l_k| {
            acc.checked_mul(&to_index::<C, I>(l_k.saturating_sub(two())))
                .expect("overflow in pre product")
        });
        // post product: ∏_{k>j} L_k
        let post: I = sizes[j + 1..].iter().fold(I::one(), |acc, &l_k| {
            acc.checked_mul(&to_index::<C, I>(l_k))
                .expect("overflow in post product")
        });
        let size_pj = side_factor
            .checked_mul(&pre)
            .and_then(|x| x.checked_mul(&post))
            .expect("overflow in size(P_j)");
        offset_p = offset_p
            .checked_add(&size_pj)
            .expect("overflow in offset_p");
    }

    // 4) Select sub-part on dimension i* (low vs high). If L_i*==1 there is only
    //    one side.
    let pre_i: I = sizes[..i_star].iter().fold(I::one(), |acc, &l_k| {
        acc.checked_mul(&to_index::<C, I>(l_k.saturating_sub(two())))
            .expect("overflow in pre_i")
    });
    let post_i: I = sizes[i_star + 1..].iter().fold(I::one(), |acc, &l_k| {
        acc.checked_mul(&to_index::<C, I>(l_k))
            .expect("overflow in post_i")
    });
    let face_block = pre_i.checked_mul(&post_i).expect("overflow in face_block");

    let mut offset_sub = I::zero();
    if sizes[i_star] >= two() && p[i_star] == sizes[i_star] - C::one() {
        offset_sub = face_block;
    }

    // 5) Index within the chosen half‑face using a rectangular onion on remaining
    //    dims.
    let mut face_sizes: Vec<C> = Vec::with_capacity(sizes.len().saturating_sub(1));
    let mut face_coords: Vec<C> = Vec::with_capacity(p.len().saturating_sub(1));

    // Left block (< i*): sizes - 2, coords - 1
    for &l_k in &sizes[..i_star] {
        face_sizes.push(l_k.saturating_sub(two()));
    }
    for &q_k in &p[..i_star] {
        face_coords.push(q_k - C::one());
    }
    // Right block (> i*): sizes intact, coords intact
    for &l_k in &sizes[i_star + 1..] {
        face_sizes.push(l_k);
    }
    for &q_k in &p[i_star + 1..] {
        face_coords.push(q_k);
    }

    let i_face = onion_index_rect::<C, I>(&face_sizes, &face_coords);
    offset_p + offset_sub + i_face
}

/// Inverse mapping for `onion_index_rect` on a rectangular face.
fn onion_point_rect<C: Coord, I: Index>(sizes: &[C], mut index: I) -> Vec<C> {
    let m = sizes.len();
    if m == 0 {
        return vec![];
    }
    if m == 1 {
        return vec![to_coord::<C, I>(index)];
    }

    // Inner sizes and volumes
    let mut inner_sizes: Vec<C> = Vec::with_capacity(m);
    for &l_i in sizes.iter() {
        inner_sizes.push(l_i.saturating_sub(two()));
    }
    let total: I = sizes.iter().fold(I::one(), |acc, &x| {
        acc.checked_mul(&to_index::<C, I>(x))
            .expect("overflow in rectangular total volume")
    });
    let inner_vol: I = inner_sizes.iter().fold(I::one(), |acc, &x| {
        acc.checked_mul(&to_index::<C, I>(x))
            .expect("overflow in rectangular inner volume")
    });
    let outer = total - inner_vol;

    if index >= outer {
        // Inner
        let idx_inner = index - outer;
        let mut p_inner = onion_point_rect::<C, I>(&inner_sizes, idx_inner);
        for v in &mut p_inner {
            *v = *v + C::one();
        }
        return p_inner;
    }

    // Outer: find partition P_i*
    let mut i_star: usize = usize::MAX;
    for j in 0..m {
        let side_factor: I = if sizes[j] >= two() {
            I::one() + I::one()
        } else {
            I::one()
        };
        let pre: I = sizes[..j].iter().fold(I::one(), |acc, &l_k| {
            acc.checked_mul(&to_index::<C, I>(l_k.saturating_sub(two())))
                .expect("overflow in pre product")
        });
        let post: I = sizes[j + 1..].iter().fold(I::one(), |acc, &l_k| {
            acc.checked_mul(&to_index::<C, I>(l_k))
                .expect("overflow in post product")
        });
        let size_pj = side_factor
            .checked_mul(&pre)
            .and_then(|x| x.checked_mul(&post))
            .expect("overflow in size(P_j)");

        if index < size_pj {
            i_star = j;
            break;
        } else {
            index = index - size_pj;
        }
    }
    assert!(
        i_star != usize::MAX,
        "Failed to locate partition in onion_point_rect"
    );

    // Select sub-part (low/high) and compute index within half-face
    let pre_i: I = sizes[..i_star].iter().fold(I::one(), |acc, &l_k| {
        acc.checked_mul(&to_index::<C, I>(l_k.saturating_sub(two())))
            .expect("overflow in pre_i")
    });
    let post_i: I = sizes[i_star + 1..].iter().fold(I::one(), |acc, &l_k| {
        acc.checked_mul(&to_index::<C, I>(l_k))
            .expect("overflow in post_i")
    });
    let face_block = pre_i.checked_mul(&post_i).expect("overflow in face_block");

    let high_side: bool;
    if sizes[i_star] >= two() {
        if index < face_block {
            high_side = false;
        } else {
            index = index - face_block;
            high_side = true;
        }
    } else {
        // Only one side when L_i*==1
        high_side = false;
    }

    // Map index to coordinates on the face via rectangular onion
    let mut face_sizes: Vec<C> = Vec::with_capacity(m - 1);
    // sizes for k< i*: L_k - 2 ; for k> i*: L_k
    for &l_k in &sizes[..i_star] {
        face_sizes.push(l_k.saturating_sub(two()));
    }
    for &l_k in &sizes[i_star + 1..] {
        face_sizes.push(l_k);
    }

    let mut face_coords = onion_point_rect::<C, I>(&face_sizes, index);

    // Reconstruct full coordinate
    let mut p = Vec::with_capacity(m);
    // Left block (< i*): shift +1
    let left_len = i_star;
    for _ in 0..left_len {
        let v = face_coords.remove(0);
        p.push(v + C::one());
    }
    // Boundary coordinate
    let coord_i = if sizes[i_star] >= two() {
        if high_side {
            sizes[i_star] - C::one()
        } else {
            C::zero()
        }
    } else {
        C::zero()
    };
    p.push(coord_i);
    // Right block (> i*): direct
    for v in face_coords {
        p.push(v);
    }
    p
}

/// Cube volume helper dedicated to the specialised 3D ordering.
fn cube_volume<C: Coord, I: Index>(side: C) -> I {
    pow_size::<C, I>(side, 3)
}

/// Specialised 3D outer-shell ordering that mirrors the published definition.
fn onion_index_3d<C: Coord, I: Index>(side_length: C, point: &[C]) -> I {
    debug_assert_eq!(point.len(), 3);

    let layer = point
        .iter()
        .map(|&coord| coord.min(side_length - C::one() - coord))
        .min()
        .unwrap_or_else(C::zero);
    let inner = side_length - layer * two();

    if inner <= C::one() {
        return cube_volume::<C, I>(side_length) - I::one();
    }

    let local = [point[0] - layer, point[1] - layer, point[2] - layer];
    let mut offset = cube_volume::<C, I>(side_length) - cube_volume::<C, I>(inner);
    let face_area = pow_size::<C, I>(inner, 2);

    if local[0].is_zero() {
        let idx = onion_index_nd::<C, I>(2, inner, &[local[1], local[2]]);
        return offset + idx;
    }
    offset = offset + face_area;

    if local[0] == inner - C::one() {
        let idx = onion_index_nd::<C, I>(2, inner, &[local[1], local[2]]);
        return offset + idx;
    }
    offset = offset + face_area;

    let inner_minus_two = inner.saturating_sub(two());
    if inner_minus_two.is_zero() {
        return offset;
    }

    if local[1].is_zero() && local[2].is_zero() {
        return offset + to_index::<C, I>(local[0] - C::one());
    }
    offset = offset + to_index::<C, I>(inner_minus_two);

    if local[1].is_zero() && !local[2].is_zero() && local[2] < inner - C::one() {
        let idx = onion_index_nd::<C, I>(
            2,
            inner_minus_two,
            &[local[0] - C::one(), local[2] - C::one()],
        );
        return offset + idx;
    }
    offset = offset + pow_size::<C, I>(inner_minus_two, 2);

    if local[1].is_zero() && local[2] == inner - C::one() {
        return offset + to_index::<C, I>(local[0] - C::one());
    }
    offset = offset + to_index::<C, I>(inner_minus_two);

    if local[1] == inner - C::one() && local[2].is_zero() {
        return offset + to_index::<C, I>(local[0] - C::one());
    }
    offset = offset + to_index::<C, I>(inner_minus_two);

    if local[1] == inner - C::one() && !local[2].is_zero() && local[2] < inner - C::one() {
        let idx = onion_index_nd::<C, I>(
            2,
            inner_minus_two,
            &[local[0] - C::one(), local[2] - C::one()],
        );
        return offset + idx;
    }
    offset = offset + pow_size::<C, I>(inner_minus_two, 2);

    if local[1] == inner - C::one() && local[2] == inner - C::one() {
        return offset + to_index::<C, I>(local[0] - C::one());
    }
    offset = offset + to_index::<C, I>(inner_minus_two);

    if local[2].is_zero() {
        let idx = onion_index_nd::<C, I>(
            2,
            inner_minus_two,
            &[local[0] - C::one(), local[1] - C::one()],
        );
        return offset + idx;
    }
    offset = offset + pow_size::<C, I>(inner_minus_two, 2);

    let idx = onion_index_nd::<C, I>(
        2,
        inner_minus_two,
        &[local[0] - C::one(), local[1] - C::one()],
    );
    offset + idx
}

/// Inverse of the specialised 3D outer-shell ordering.
fn onion_point_3d<C: Coord, I: Index>(side_length: C, index: I) -> Vec<C> {
    let mut remaining = index;
    let mut layer = C::zero();
    let mut current_len = side_length;

    loop {
        let next_len = current_len.saturating_sub(two());
        let shell_size = cube_volume::<C, I>(current_len) - cube_volume::<C, I>(next_len);
        if remaining < shell_size {
            break;
        }
        remaining = remaining - shell_size;
        layer = layer + C::one();
        current_len = next_len;
    }

    if current_len <= C::one() {
        return vec![layer, layer, layer];
    }

    let inner = current_len;
    let inner_minus_two = inner.saturating_sub(two());
    let face_area = pow_size::<C, I>(inner, 2);

    if remaining < face_area {
        let yz = onion_point_nd::<C, I>(2, inner, remaining);
        return vec![layer, yz[0] + layer, yz[1] + layer];
    }
    remaining = remaining - face_area;

    if remaining < face_area {
        let yz = onion_point_nd::<C, I>(2, inner, remaining);
        return vec![layer + inner - C::one(), yz[0] + layer, yz[1] + layer];
    }
    remaining = remaining - face_area;

    if inner_minus_two.is_zero() {
        return vec![layer, layer, layer + inner - C::one()];
    }

    if remaining < to_index::<C, I>(inner_minus_two) {
        return vec![layer + C::one() + to_coord::<C, I>(remaining), layer, layer];
    }
    remaining = remaining - to_index::<C, I>(inner_minus_two);

    let rect_area = pow_size::<C, I>(inner_minus_two, 2);

    if remaining < rect_area {
        let coords = onion_point_nd::<C, I>(2, inner_minus_two, remaining);
        return vec![
            layer + C::one() + coords[0],
            layer,
            layer + C::one() + coords[1],
        ];
    }
    remaining = remaining - rect_area;

    if remaining < to_index::<C, I>(inner_minus_two) {
        return vec![
            layer + C::one() + to_coord::<C, I>(remaining),
            layer,
            layer + inner - C::one(),
        ];
    }
    remaining = remaining - to_index::<C, I>(inner_minus_two);

    if remaining < to_index::<C, I>(inner_minus_two) {
        return vec![
            layer + C::one() + to_coord::<C, I>(remaining),
            layer + inner - C::one(),
            layer,
        ];
    }
    remaining = remaining - to_index::<C, I>(inner_minus_two);

    if remaining < rect_area {
        let coords = onion_point_nd::<C, I>(2, inner_minus_two, remaining);
        return vec![
            layer + C::one() + coords[0],
            layer + inner - C::one(),
            layer + C::one() + coords[1],
        ];
    }
    remaining = remaining - rect_area;

    if remaining < to_index::<C, I>(inner_minus_two) {
        let offset = to_coord::<C, I>(remaining);
        return vec![
            layer + C::one() + offset,
            layer + inner - C::one(),
            layer + inner - C::one(),
        ];
    }
    remaining = remaining - to_index::<C, I>(inner_minus_two);

    if remaining < rect_area {
        let coords = onion_point_nd::<C, I>(2, inner_minus_two, remaining);
        return vec![
            layer + C::one() + coords[0],
            layer + C::one() + coords[1],
            layer,
        ];
    }
    remaining = remaining - rect_area;

    let coords = onion_point_nd::<C, I>(2, inner_minus_two, remaining);
    vec![
        layer + C::one() + coords[0],
        layer + C::one() + coords[1],
        layer + inner - C::one(),
    ]
}

/// Return the constant `2` in the coordinate type.
fn two<T: Coord>() -> T {
    T::one() + T::one()
}

/// Convert a coordinate value into an index type for arithmetic.
fn to_index<C: Coord, I: Index>(value: C) -> I {
    I::from(value).expect("value fits index type")
}

/// Convert an index back into a coordinate value.
fn to_coord<C: Coord, I: Index>(value: I) -> C {
    C::from(value).expect("value fits coordinate type")
}

/// Convert a coordinate value into `usize` for indexing.
fn to_usize<T: Coord>(value: T) -> usize {
    value.to_usize().expect("value fits usize")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constructor_guards() {
        // L==0 rejected
        assert!(OnionCurve::<u32, u32>::new(2, 0).is_err());
        // N==0 rejected
        assert!(OnionCurve::<u32, u32>::new(0, 4).is_err());
        // Valid small shapes
        let c = OnionCurve::<u32, u32>::new(2, 3).unwrap();
        assert_eq!(c.length(), 9);
    }

    #[test]
    fn roundtrip_dims_2_to_4_sizes_upto_8() {
        for dim in 2..=4 {
            for size in 2..=8 {
                let curve = OnionCurve::<u32, u32>::new(dim, size).unwrap();
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
