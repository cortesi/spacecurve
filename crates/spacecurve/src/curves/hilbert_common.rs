//! Shared helpers for Hilbert variants (2D and N-D).

use crate::{
    ops,
    types::{Coord, Index},
};

/// Bitmask with `width` least‑significant bits set. Returns `0` when `width` is
/// zero or overflows `u32` shifts, avoiding panics.
#[inline]
pub fn bitmask<T: Coord>(width: u32) -> T {
    match width {
        0 => T::zero(),
        w => T::one()
            .checked_shl(w)
            .map(|value| value - T::one())
            .unwrap_or_else(T::max_value),
    }
}

/// Left rotation over `width` bits with masking. Undefined inputs are masked
/// rather than panicking so helpers remain total.
#[inline]
pub fn lrot<T: Coord>(word: T, shift: u32, width: u32) -> T {
    let width = width % T::BITS;
    if width == 0 {
        return T::zero();
    }
    let mask = bitmask::<T>(width);
    let shift = shift % width;
    let w = word & mask;
    let shift = shift as usize;
    let back_shift = (width - (shift as u32)) as usize;
    ((w << shift) | (w >> back_shift)) & mask
}

/// Right rotation over `width` bits with masking.
#[inline]
pub fn rrot<T: Coord>(word: T, shift: u32, width: u32) -> T {
    let width = width % T::BITS;
    if width == 0 {
        return T::zero();
    }
    let mask = bitmask::<T>(width);
    let shift = shift % width;
    let w = word & mask;
    let shift = shift as usize;
    let back_shift = (width - (shift as u32)) as usize;
    ((w >> shift) | (w << back_shift)) & mask
}

/// Extract a bit range `[start, end)` from `word` limited to `width` bits.
#[inline]
pub fn bitrange<T: Coord>(word: T, width: u32, start: u32, end: u32) -> T {
    if start >= end || width == 0 {
        return T::zero();
    }
    let clamped_end = end.min(width);
    let clamped_start = start.min(clamped_end);
    let len = clamped_end - clamped_start;
    if len == 0 {
        return T::zero();
    }
    let shift = width.saturating_sub(clamped_end) as usize;
    (word >> shift) & bitmask::<T>(len)
}

/// Set bit `pos` (0‑indexed from LSB in `[0, width)`) to `bit` (0/1) with
/// masking instead of panicking.
#[inline]
pub fn setbit<T: Coord>(word: T, width: u32, pos: u32, bit: T) -> T {
    if width == 0 || pos >= width {
        return word;
    }
    let mask = T::one()
        .checked_shl(width - pos - 1)
        .unwrap_or_else(T::zero);
    if (bit & T::one()) == T::one() {
        word | mask
    } else {
        word & !mask
    }
}

/// Count trailing set bits in `word` within `width` bits.
#[inline]
pub fn tsb<T: Coord>(word: T, width: u32) -> u32 {
    let width = width.min(T::BITS);
    if width == 0 {
        return 0;
    }
    let masked = word & bitmask::<T>(width);
    let inverted = !masked;
    inverted.trailing_zeros().min(width)
}

/// Rotate the 2‑bit label used by the 2D Hilbert state machine.
#[inline]
pub fn rot2<I: Index>(label: I) -> I {
    let zero = I::zero();
    let one = I::one();
    let two = one << 1usize;
    let three = two | one;
    match label & three {
        v if v == zero => zero,
        v if v == one => two,
        v if v == two => one,
        _ => three,
    }
}

/// Gray code limited to the low two bits.
#[inline]
pub fn gray2<I: Index>(word: I) -> I {
    let mask = (I::one() << 2usize) - I::one();
    ops::graycode(word) & mask
}
