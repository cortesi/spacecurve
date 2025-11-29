/// The Onion Curve is a space-filling curve named after the core concept of "peeling" an
/// N-dimensional hypercube layer by layer, like an onion.
///
/// See: https://arxiv.org/abs/1801.07399
///
/// **What's changed in this revision**
///
/// The previous N-D generalisation handled the outer shell by splitting it into
/// partitions P_i (first boundary dimension i), then composing the (i-1)-D and
/// (N-i)-D parts with a boustrophedon ("snake") product. In 3D that made two faces of
/// the cube scan in a zig‑zag. This revision removes the snake and **uses an
/// (N-1)-D onion ordering on each half‑face itself** (which in 3D is a 2D onion on
/// the rectangle), eliminating the zig‑zag while keeping strict L∞-layering.
///
/// Notes:
/// * As before, for L=2 we use a continuous Gray‑code generalisation (O(N-1) then
///   reversed O(N-1) with last coordinate as the discriminator).
/// * In N>2 and L>2 the onion curve cannot be fully continuous (see comment below),
///   but this version maximises locality on each shell by using an onion on the
///   half‑faces rather than a stripe snake.
///
/// Continuity impossibility sketch (unchanged):
/// Consider a 3×3×3 cube. Chessboard‑colour cells by parity of the coordinate sum.
/// The outer shell has 26 cells (even). The center cell is White, hence the shell
/// must end on White; any continuous traversal into the next shell would need to
/// enter a Black cell, contradiction.
use crate::{error, point::Point, spacecurve::SpaceCurve, spec::GridSpec};

mod l2;
mod rect;
mod threed;
mod twod;

use l2::{onion_index_l2, onion_point_l2};
use rect::{onion_index_rect, onion_point_rect};
use threed::{onion_index_3d, onion_point_3d};
pub(crate) use twod::{onion_index_2d, onion_point_2d};

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
