//! Shared helpers for Hilbert variants (2D and N‑D).
use crate::ops;

/// Bitmask with `width` least‑significant bits set. Returns `0` when `width` is
/// zero or overflows `u32` shifts, avoiding panics.
#[inline]
pub fn bitmask(width: u32) -> u32 {
    match width {
        0 => 0,
        w => 1u32.checked_shl(w).unwrap_or(0).wrapping_sub(1),
    }
}

/// Left rotation over `width` bits with masking. Undefined inputs are masked
/// rather than panicking so helpers remain total.
#[inline]
pub fn lrot(word: u32, shift: u32, width: u32) -> u32 {
    let width = width % 32;
    if width == 0 {
        return 0;
    }
    let mask = bitmask(width);
    let shift = shift % width;
    let w = word & mask;
    ((w << shift) | (w >> (width - shift))) & mask
}

/// Right rotation over `width` bits with masking.
#[inline]
pub fn rrot(word: u32, shift: u32, width: u32) -> u32 {
    let width = width % 32;
    if width == 0 {
        return 0;
    }
    let mask = bitmask(width);
    let shift = shift % width;
    let w = word & mask;
    ((w >> shift) | (w << (width - shift))) & mask
}

/// Extract a bit range `[start, end)` from `word` limited to `width` bits.
#[inline]
pub fn bitrange(word: u32, width: u32, start: u32, end: u32) -> u32 {
    if start >= end || width == 0 {
        return 0;
    }
    let clamped_end = end.min(width);
    let clamped_start = start.min(clamped_end);
    let len = clamped_end - clamped_start;
    if len == 0 {
        return 0;
    }
    let shift = width.saturating_sub(clamped_end);
    (word >> shift) & bitmask(len)
}

/// Set bit `pos` (0‑indexed from LSB in `[0, width)`) to `bit` (0/1) with
/// masking instead of panicking.
#[inline]
pub fn setbit(word: u32, width: u32, pos: u32, bit: u32) -> u32 {
    if width == 0 || pos >= width {
        return word;
    }
    let mask = 1u32.checked_shl(width - pos - 1).unwrap_or(0);
    if bit & 1 == 1 {
        word | mask
    } else {
        word & !mask
    }
}

/// Count trailing set bits in `word` within `width` bits.
#[inline]
pub fn tsb(word: u32, width: u32) -> u32 {
    let mut count = 0;
    let mut v = word & bitmask(width);
    while count < width && (v & 1) == 1 {
        v >>= 1;
        count += 1;
    }
    count
}

/// Rotate the 2‑bit label used by the 2D Hilbert state machine.
#[inline]
pub fn rot2(label: u32) -> u32 {
    match label & 3 {
        0 => 0,
        1 => 2,
        2 => 1,
        _ => 3,
    }
}

/// Gray code limited to the low two bits.
#[inline]
pub fn gray2(word: u32) -> u32 {
    ops::graycode(word) & 3
}
