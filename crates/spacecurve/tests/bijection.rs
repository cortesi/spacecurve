//! Property-based tests verifying the bijection property of space-filling curves.
//!
//! All curves must satisfy: curve.index(curve.point(i)) == i for any valid index i.

#![allow(missing_docs, clippy::tests_outside_test_module)]

use proptest::prelude::*;
use spacecurve::{CURVE_NAMES, pattern_from_name};

/// Generate test configurations: (curve_name, dimension, size, max_index).
/// We use smaller sizes to keep tests fast while still testing the bijection property.
fn curve_configs() -> Vec<(&'static str, u32, u32, u32)> {
    vec![
        // Hilbert (power-of-two, order*dim < 32)
        ("hilbert", 2, 4, 16),   // 4x4 = 16 points
        ("hilbert", 2, 8, 64),   // 8x8 = 64 points
        ("hilbert", 2, 16, 256), // 16x16 = 256 points
        ("hilbert", 3, 4, 64),   // 4^3 = 64 points
        ("hilbert", 4, 2, 16),   // 2^4 = 16 points
        // Scan (any size)
        ("scan", 2, 5, 25),   // 5x5 = 25 points
        ("scan", 2, 10, 100), // 10x10 = 100 points
        ("scan", 3, 4, 64),   // 4^3 = 64 points
        // Z-order (power-of-two)
        ("zorder", 2, 4, 16),
        ("zorder", 2, 8, 64),
        ("zorder", 3, 4, 64),
        // H-curve (power-of-two, dim >= 2)
        ("hcurve", 2, 4, 16),
        ("hcurve", 2, 8, 64),
        ("hcurve", 4, 2, 16),
        // Onion (any size)
        ("onion", 2, 5, 25),
        ("onion", 2, 8, 64),
        ("onion", 3, 4, 64),
        // Hairy Onion (any size)
        ("hairyonion", 2, 5, 25),
        ("hairyonion", 2, 8, 64),
        ("hairyonion", 3, 4, 64),
        // Gray (power-of-two)
        ("gray", 2, 4, 16),
        ("gray", 2, 8, 64),
        ("gray", 3, 4, 64),
    ]
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(500))]

    /// Test bijection property for Hilbert 2D curves.
    #[test]
    fn bijection_hilbert_2d(index in 0u32..256) {
        let curve = pattern_from_name("hilbert", 2, 16).expect("hilbert 2d 16");
        if index < curve.length() {
            let point = curve.point(index);
            let recovered = curve.index(&point);
            prop_assert_eq!(recovered, index, "Hilbert 2D bijection failed");
        }
    }

    /// Test bijection property for Hilbert 3D curves.
    #[test]
    fn bijection_hilbert_3d(index in 0u32..64) {
        let curve = pattern_from_name("hilbert", 3, 4).expect("hilbert 3d 4");
        if index < curve.length() {
            let point = curve.point(index);
            let recovered = curve.index(&point);
            prop_assert_eq!(recovered, index, "Hilbert 3D bijection failed");
        }
    }

    /// Test bijection property for Scan curves.
    #[test]
    fn bijection_scan(index in 0u32..100) {
        let curve = pattern_from_name("scan", 2, 10).expect("scan 2d 10");
        if index < curve.length() {
            let point = curve.point(index);
            let recovered = curve.index(&point);
            prop_assert_eq!(recovered, index, "Scan bijection failed");
        }
    }

    /// Test bijection property for Z-order curves.
    #[test]
    fn bijection_zorder(index in 0u32..256) {
        let curve = pattern_from_name("zorder", 2, 16).expect("zorder 2d 16");
        if index < curve.length() {
            let point = curve.point(index);
            let recovered = curve.index(&point);
            prop_assert_eq!(recovered, index, "Z-order bijection failed");
        }
    }

    /// Test bijection property for H-curve.
    #[test]
    fn bijection_hcurve(index in 0u32..64) {
        let curve = pattern_from_name("hcurve", 2, 8).expect("hcurve 2d 8");
        if index < curve.length() {
            let point = curve.point(index);
            let recovered = curve.index(&point);
            prop_assert_eq!(recovered, index, "H-curve bijection failed");
        }
    }

    /// Test bijection property for Onion curves.
    #[test]
    fn bijection_onion(index in 0u32..64) {
        let curve = pattern_from_name("onion", 2, 8).expect("onion 2d 8");
        if index < curve.length() {
            let point = curve.point(index);
            let recovered = curve.index(&point);
            prop_assert_eq!(recovered, index, "Onion bijection failed");
        }
    }

    /// Test bijection property for Hairy Onion curves.
    #[test]
    fn bijection_hairyonion(index in 0u32..64) {
        let curve = pattern_from_name("hairyonion", 2, 8).expect("hairyonion 2d 8");
        if index < curve.length() {
            let point = curve.point(index);
            let recovered = curve.index(&point);
            prop_assert_eq!(recovered, index, "Hairy Onion bijection failed");
        }
    }

    /// Test bijection property for Gray code curves.
    #[test]
    fn bijection_gray(index in 0u32..256) {
        let curve = pattern_from_name("gray", 2, 16).expect("gray 2d 16");
        if index < curve.length() {
            let point = curve.point(index);
            let recovered = curve.index(&point);
            prop_assert_eq!(recovered, index, "Gray bijection failed");
        }
    }
}

// ============================================================================
// Edge case tests (non-property-based)
// ============================================================================

/// Test bijection at index 0 for all curves.
#[test]
fn bijection_at_zero() {
    for (name, dim, size, _) in curve_configs() {
        let curve = pattern_from_name(name, dim, size).expect("curve");
        let point = curve.point(0);
        let recovered = curve.index(&point);
        assert_eq!(
            recovered, 0,
            "{} (dim={}, size={}) failed at index 0",
            name, dim, size
        );
    }
}

/// Test bijection at the last valid index (length - 1) for all curves.
#[test]
fn bijection_at_last_index() {
    for (name, dim, size, _) in curve_configs() {
        let curve = pattern_from_name(name, dim, size).expect("curve");
        let last = curve.length() - 1;
        let point = curve.point(last);
        let recovered = curve.index(&point);
        assert_eq!(
            recovered, last,
            "{} (dim={}, size={}) failed at last index {}",
            name, dim, size, last
        );
    }
}

/// Test bijection at midpoint for all curves.
#[test]
fn bijection_at_midpoint() {
    for (name, dim, size, _) in curve_configs() {
        let curve = pattern_from_name(name, dim, size).expect("curve");
        let mid = curve.length() / 2;
        let point = curve.point(mid);
        let recovered = curve.index(&point);
        assert_eq!(
            recovered, mid,
            "{} (dim={}, size={}) failed at midpoint {}",
            name, dim, size, mid
        );
    }
}

/// Exhaustive bijection test for small curves (validates every index).
#[test]
fn exhaustive_bijection_small_curves() {
    let small_configs = [
        ("hilbert", 2, 4),
        ("hilbert", 3, 2),
        ("scan", 2, 4),
        ("scan", 3, 3),
        ("zorder", 2, 4),
        ("hcurve", 2, 4),
        ("onion", 2, 4),
        ("hairyonion", 2, 4),
        ("gray", 2, 4),
    ];

    for (name, dim, size) in small_configs {
        let curve = pattern_from_name(name, dim, size).expect("curve");
        for i in 0..curve.length() {
            let point = curve.point(i);
            let recovered = curve.index(&point);
            assert_eq!(
                recovered, i,
                "{} (dim={}, size={}) bijection failed at index {}",
                name, dim, size, i
            );
        }
    }
}

/// Verify all curve types in CURVE_NAMES are testable and satisfy bijection.
#[test]
fn all_registered_curves_satisfy_bijection() {
    // Use the smallest valid configuration for each curve type
    let configs: Vec<(&str, u32, u32)> = CURVE_NAMES
        .iter()
        .map(|&name| {
            // Choose valid (dim, size) for each curve
            match name {
                "hilbert" | "zorder" | "gray" => (name, 2, 4),
                "hcurve" => (name, 2, 4), // hcurve requires dim >= 2
                "scan" | "onion" | "hairyonion" => (name, 2, 4),
                _ => (name, 2, 4), // fallback
            }
        })
        .collect();

    for (name, dim, size) in configs {
        let curve = pattern_from_name(name, dim, size).unwrap_or_else(|e| {
            panic!(
                "Failed to create {} (dim={}, size={}): {}",
                name, dim, size, e
            )
        });

        // Test at least first, middle, and last indices
        let indices = [0, curve.length() / 2, curve.length() - 1];
        for &i in &indices {
            let point = curve.point(i);
            let recovered = curve.index(&point);
            assert_eq!(
                recovered, i,
                "Curve {} (dim={}, size={}) bijection failed at index {}",
                name, dim, size, i
            );
        }
    }
}
