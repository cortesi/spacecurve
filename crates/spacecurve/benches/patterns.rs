//! Benchmarks for space-filling curve point and index operations across all curve types.

use std::hint::black_box;

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use spacecurve::pattern_from_name;

/// Benchmark configurations: (curve_name, dimension, size).
/// For power-of-two curves (hilbert, zorder, hcurve, gray): size must be power of 2.
/// For flexible curves (scan, onion, hairyonion): any size works.
fn bench_configs() -> Vec<(&'static str, u32, u32)> {
    vec![
        // Hilbert curve - 2D optimized and N-D general
        ("hilbert", 2, 4),
        ("hilbert", 2, 8),
        ("hilbert", 2, 16),
        ("hilbert", 3, 4),
        ("hilbert", 3, 8),
        ("hilbert", 4, 4),
        // Z-order (Morton) curve
        ("zorder", 2, 4),
        ("zorder", 2, 8),
        ("zorder", 2, 16),
        ("zorder", 3, 4),
        ("zorder", 3, 8),
        // H-curve
        ("hcurve", 2, 4),
        ("hcurve", 2, 8),
        ("hcurve", 2, 16),
        ("hcurve", 3, 4),
        // Scan (boustrophedon)
        ("scan", 2, 4),
        ("scan", 2, 8),
        ("scan", 2, 16),
        ("scan", 3, 4),
        ("scan", 3, 8),
        // Onion
        ("onion", 2, 4),
        ("onion", 2, 8),
        ("onion", 2, 16),
        ("onion", 3, 4),
        // Hairy Onion
        ("hairyonion", 2, 4),
        ("hairyonion", 2, 8),
        ("hairyonion", 2, 16),
        ("hairyonion", 3, 4),
        // Gray code
        ("gray", 2, 4),
        ("gray", 2, 8),
        ("gray", 2, 16),
        ("gray", 3, 4),
        ("gray", 3, 8),
    ]
}

/// Benchmark the `point` operation (index -> coordinates) for all curve types.
fn bench_point(c: &mut Criterion) {
    let mut group = c.benchmark_group("point");

    for (name, dim, size) in bench_configs() {
        let curve = pattern_from_name(name, dim, size).expect("valid curve");
        let midpoint = curve.length() / 2;

        group.bench_function(BenchmarkId::new(name, format!("{dim}d-{size}")), |b| {
            b.iter(|| curve.point(black_box(midpoint)))
        });
    }

    group.finish();
}

/// Benchmark the `index` operation (coordinates -> index) for all curve types.
fn bench_index(c: &mut Criterion) {
    let mut group = c.benchmark_group("index");

    for (name, dim, size) in bench_configs() {
        let curve = pattern_from_name(name, dim, size).expect("valid curve");
        let pt = curve.point(curve.length() / 2);

        group.bench_function(BenchmarkId::new(name, format!("{dim}d-{size}")), |b| {
            b.iter(|| curve.index(black_box(&pt)))
        });
    }

    group.finish();
}

/// Compare 2D optimized Hilbert vs N-D Hilbert at various sizes.
fn bench_hilbert_2d_vs_nd(c: &mut Criterion) {
    let mut group = c.benchmark_group("hilbert_2d_vs_nd");

    // Test at various sizes where 2D optimization should show benefit
    for size in [4, 8, 16, 32, 64] {
        let curve_2d = pattern_from_name("hilbert", 2, size).expect("hilbert 2d");
        let midpoint = curve_2d.length() / 2;
        let pt = curve_2d.point(midpoint);

        // Benchmark point operation
        group.bench_function(BenchmarkId::new("point", format!("2d-{size}")), |b| {
            b.iter(|| curve_2d.point(black_box(midpoint)))
        });

        // Benchmark index operation
        group.bench_function(BenchmarkId::new("index", format!("2d-{size}")), |b| {
            b.iter(|| curve_2d.index(black_box(&pt)))
        });
    }

    group.finish();
}

/// Benchmark scaling behavior: how performance changes with curve size.
fn bench_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("scaling");

    // Test how each curve type scales with size (2D only for consistency)
    let curves = ["hilbert", "scan", "zorder", "onion"];
    let sizes = [4, 8, 16, 32];

    for name in curves {
        for &size in &sizes {
            let curve = match pattern_from_name(name, 2, size) {
                Ok(c) => c,
                Err(_) => continue, // Skip invalid configurations
            };
            let midpoint = curve.length() / 2;

            group.bench_function(BenchmarkId::new(format!("{}_point", name), size), |b| {
                b.iter(|| curve.point(black_box(midpoint)))
            });
        }
    }

    group.finish();
}

#[allow(missing_docs, clippy::missing_docs_in_private_items)]
mod bench_defs {
    use super::*;
    criterion_group!(
        benches,
        bench_point,
        bench_index,
        bench_hilbert_2d_vs_nd,
        bench_scaling
    );
}

pub use bench_defs::benches;
criterion_main!(benches);
