//! Benchmarks for low-level bit operations.

use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};
use spacecurve::ops::{deinterleave_lsb, interleave_lsb};

/// Benchmark the `interleave_lsb` operation.
fn bench_interleave(c: &mut Criterion) {
    let mut group = c.benchmark_group("interleave_lsb");

    // 2D case
    let coords_2d = [0xAAAAu32, 0x5555u32]; // 16-bit values
    group.bench_function("2D", |b| {
        b.iter(|| interleave_lsb::<u32, u32>(black_box(&coords_2d), black_box(16)))
    });

    // 3D case
    let coords_3d = [0x111u32, 0x222u32, 0x333u32]; // 10-bit values
    group.bench_function("3D", |b| {
        b.iter(|| interleave_lsb::<u32, u32>(black_box(&coords_3d), black_box(10)))
    });

    group.finish();
}

/// Benchmark the `deinterleave_lsb` operation.
fn bench_deinterleave(c: &mut Criterion) {
    let mut group = c.benchmark_group("deinterleave_lsb");

    // 2D case (Morton code)
    let morton_2d = 0xAAAAAAAAu32; // Arbitrary pattern
    group.bench_function("2D", |b| {
        b.iter(|| deinterleave_lsb::<u32, u32>(black_box(2), black_box(16), black_box(morton_2d)))
    });

    // 3D case
    let morton_3d = 0x24924924u32; // Arbitrary pattern
    group.bench_function("3D", |b| {
        b.iter(|| deinterleave_lsb::<u32, u32>(black_box(3), black_box(10), black_box(morton_3d)))
    });

    group.finish();
}

#[allow(missing_docs, clippy::missing_docs_in_private_items)]
mod bench_defs {
    use super::*;
    criterion_group!(benches, bench_interleave, bench_deinterleave);
}
pub use bench_defs::benches;
criterion_main!(benches);
