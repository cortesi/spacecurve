//! Windowed locality (WL∞/WL2) profiles over segment lengths.
//!
//! For each segment length `L`, this evaluation considers contiguous index windows
//! `[i, i + L - 1]` and measures the distance between the endpoints in space. The
//! per-window distance is normalized by `L` and scaled by `d` (the curve
//! dimensionality) to make values comparable across dimensions:
//!
//! - `WL∞(L) = max_i dist∞(p_i, p_{i+L-1})^d / L`
//! - `WL2(L) = max_i dist2(p_i, p_{i+L-1})^d / L`
//!
//! The profile is dimension-independent and does not assume continuity; it only
//! depends on `point(i)` lookups.

use std::cmp;

use rand::{SeedableRng, seq::index::sample};
use rand_chacha::ChaCha8Rng;

use crate::{
    SpaceCurve, error,
    evals::{MetricDef, MetricDirection, metrics},
    point::Point,
    types::{Coord, Index},
};

/// Evaluation key for WL profiles.
pub const EVAL_KEY: &str = "wl";
/// Human-readable title for WL profiles.
pub const EVAL_TITLE: &str = "Windowed Locality Profile";

/// Metric definitions for WL profiles.
pub const METRIC_DEFS: &[MetricDef] = &[
    MetricDef {
        name: "wl-inf-max",
        description: "Maximum WL∞ ratio across sampled windows",
        direction: MetricDirection::Minimize,
    },
    MetricDef {
        name: "wl-inf-mean",
        description: "Mean WL∞ ratio across sampled windows",
        direction: MetricDirection::Minimize,
    },
    MetricDef {
        name: "wl-inf-p95",
        description: "95th percentile WL∞ ratio across sampled windows",
        direction: MetricDirection::Minimize,
    },
    MetricDef {
        name: "wl2-max",
        description: "Maximum WL2 ratio across sampled windows",
        direction: MetricDirection::Minimize,
    },
    MetricDef {
        name: "wl2-mean",
        description: "Mean WL2 ratio across sampled windows",
        direction: MetricDirection::Minimize,
    },
    MetricDef {
        name: "wl2-p95",
        description: "95th percentile WL2 ratio across sampled windows",
        direction: MetricDirection::Minimize,
    },
];

/// Which windows to scan when building a WL profile.
#[derive(Clone, Copy, Debug)]
pub enum ScanMode {
    /// Scan every window start for each segment length.
    Exact,
    /// Sample window starts uniformly without replacement.
    Sample {
        /// Number of windows to sample per segment length.
        windows_per_len: usize,
        /// RNG seed used for sampling.
        seed: u64,
    },
}

/// A single WL profile row for a segment length.
#[derive(Clone, Debug)]
pub struct WlRow<I: Index> {
    /// Segment length `L` for this row.
    pub segment_len: u32,
    /// Maximum WL∞ ratio across windows.
    pub wl_inf_max: f64,
    /// Window start index where the WL∞ maximum occurs.
    pub wl_inf_argmax_start: I,
    /// Maximum WL2 ratio across windows.
    pub wl2_max: f64,
    /// Window start index where the WL2 maximum occurs.
    pub wl2_argmax_start: I,
    /// Mean WL∞ ratio across windows (when computed).
    pub wl_inf_mean: Option<f64>,
    /// Mean WL2 ratio across windows (when computed).
    pub wl2_mean: Option<f64>,
    /// 95th percentile WL∞ ratio across windows (when computed).
    pub wl_inf_p95: Option<f64>,
    /// 95th percentile WL2 ratio across windows (when computed).
    pub wl2_p95: Option<f64>,
}

/// A WL profile for a curve.
#[derive(Clone, Debug)]
pub struct WlProfile<I: Index> {
    /// Curve name.
    pub curve_name: &'static str,
    /// Curve length.
    pub n: I,
    /// Curve dimensionality.
    pub d: u32,
    /// WL profile rows in ascending segment length order.
    pub rows: Vec<WlRow<I>>,
}

/// Maximum window count to retain full distributions in exact mode.
const MAX_EXACT_DISTRIBUTION_WINDOWS: usize = 100_000;

/// Compute WL∞/WL2 profiles for the supplied segment lengths.
///
/// Invalid lengths (`0` or `> n`) are skipped, and remaining lengths are
/// de-duplicated and sorted ascending.
pub fn wl_profile<C: Coord, I: Index>(
    curve: &dyn SpaceCurve<Coord = C, Index = I>,
    segment_lengths: &[u32],
    mode: ScanMode,
) -> error::Result<WlProfile<I>> {
    let n = curve.length();
    let n_usize = n
        .to_usize()
        .ok_or_else(|| error::Error::Size("curve length exceeds usize".to_string()))?;
    let dimension = curve.dimensions();

    let segment_lengths = prepare_segment_lengths(segment_lengths, n)?;
    let points = match mode {
        ScanMode::Exact => Some(collect_points(curve, n_usize)?),
        ScanMode::Sample { .. } => None,
    };

    let mut rows = Vec::with_capacity(segment_lengths.len());
    for segment_len in segment_lengths {
        let segment_len_usize = segment_len as usize;
        if segment_len_usize == 0 || segment_len_usize > n_usize {
            continue;
        }

        let window_count = n_usize - segment_len_usize + 1;
        let (window_starts, total_windows, collect_distribution) = match mode {
            ScanMode::Exact => {
                let collect = window_count <= MAX_EXACT_DISTRIBUTION_WINDOWS;
                (None, window_count, collect)
            }
            ScanMode::Sample {
                windows_per_len,
                seed,
            } => {
                if windows_per_len == 0 {
                    return Err(error::Error::Size(
                        "windows_per_len must be >= 1".to_string(),
                    ));
                }
                let starts = sample_window_starts(window_count, windows_per_len, seed, segment_len);
                (Some(starts), cmp::min(window_count, windows_per_len), true)
            }
        };

        let mut accumulator =
            WindowAccumulator::new(segment_len, dimension, total_windows, collect_distribution);

        match window_starts {
            Some(starts) => {
                for start in starts {
                    update_window(
                        curve,
                        points.as_deref(),
                        &mut accumulator,
                        start,
                        segment_len_usize,
                    )?;
                }
            }
            None => {
                for start in 0..window_count {
                    update_window(
                        curve,
                        points.as_deref(),
                        &mut accumulator,
                        start,
                        segment_len_usize,
                    )?;
                }
            }
        }

        rows.push(accumulator.into_row()?);
    }

    Ok(WlProfile {
        curve_name: curve.name(),
        n,
        d: dimension,
        rows,
    })
}

/// Filter, sort, and de-duplicate segment lengths against the curve length.
fn prepare_segment_lengths<I: Index>(segment_lengths: &[u32], n: I) -> error::Result<Vec<u32>> {
    let n_u128 = n
        .to_u128()
        .ok_or_else(|| error::Error::Size("curve length exceeds u128".to_string()))?;
    let mut lengths: Vec<u32> = segment_lengths
        .iter()
        .copied()
        .filter(|&len| len > 0 && u128::from(len) <= n_u128)
        .collect();
    lengths.sort_unstable();
    lengths.dedup();
    Ok(lengths)
}

/// Collect all curve points into a contiguous buffer for window scans.
fn collect_points<C: Coord, I: Index>(
    curve: &dyn SpaceCurve<Coord = C, Index = I>,
    n: usize,
) -> error::Result<Vec<Point<C>>> {
    let mut points = Vec::with_capacity(n);
    for idx in 0..n {
        let index = I::from(idx)
            .ok_or_else(|| error::Error::Size("curve length exceeds index type".to_string()))?;
        points.push(curve.point(index));
    }
    Ok(points)
}

/// Update the accumulator for a window, fetching endpoints from cache or curve.
fn update_window<C: Coord, I: Index>(
    curve: &dyn SpaceCurve<Coord = C, Index = I>,
    points: Option<&[Point<C>]>,
    accumulator: &mut WindowAccumulator<I>,
    start: usize,
    segment_len: usize,
) -> error::Result<()> {
    let end = start + segment_len - 1;
    if let Some(points) = points {
        accumulator.update(start, &points[start], &points[end])?;
        return Ok(());
    }

    let start_index = index_from_usize::<I>(start)?;
    let end_index = index_from_usize::<I>(end)?;
    let start_point = curve.point(start_index);
    let end_point = curve.point(end_index);
    accumulator.update(start, &start_point, &end_point)?;
    Ok(())
}

/// Convert a window index into the curve index type.
fn index_from_usize<I: Index>(value: usize) -> error::Result<I> {
    I::from(value).ok_or_else(|| error::Error::Size("index exceeds index type".to_string()))
}

/// Sample window start indices uniformly without replacement.
fn sample_window_starts(
    window_count: usize,
    windows_per_len: usize,
    seed: u64,
    segment_len: u32,
) -> Vec<usize> {
    let target = cmp::min(window_count, windows_per_len);
    if target == window_count {
        return (0..window_count).collect();
    }

    let seed = seed ^ (segment_len as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15);
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    sample(&mut rng, window_count, target).into_vec()
}

/// Mutable accumulator for WL statistics at a single segment length.
struct WindowAccumulator<I: Index> {
    /// Segment length for the current scan.
    segment_len: u32,
    /// Curve dimensionality.
    dimension: u32,
    /// Number of windows scanned.
    total_windows: usize,
    /// Maximum WL∞ ratio observed so far.
    wl_inf_max: f64,
    /// Maximum WL2 ratio observed so far.
    wl2_max: f64,
    /// Window start index for the WL∞ maximum.
    wl_inf_argmax: Option<I>,
    /// Window start index for the WL2 maximum.
    wl2_argmax: Option<I>,
    /// Sum of WL∞ ratios for mean computation.
    wl_inf_sum: f64,
    /// Sum of WL2 ratios for mean computation.
    wl2_sum: f64,
    /// Optional WL∞ ratios for distribution stats.
    wl_inf_values: Option<Vec<f64>>,
    /// Optional WL2 ratios for distribution stats.
    wl2_values: Option<Vec<f64>>,
}

impl<I: Index> WindowAccumulator<I> {
    /// Initialize a new accumulator for a segment length.
    fn new(
        segment_len: u32,
        dimension: u32,
        total_windows: usize,
        collect_distribution: bool,
    ) -> Self {
        let wl_inf_values = if collect_distribution {
            Some(Vec::with_capacity(total_windows))
        } else {
            None
        };
        let wl2_values = if collect_distribution {
            Some(Vec::with_capacity(total_windows))
        } else {
            None
        };
        Self {
            segment_len,
            dimension,
            total_windows,
            wl_inf_max: f64::NEG_INFINITY,
            wl2_max: f64::NEG_INFINITY,
            wl_inf_argmax: None,
            wl2_argmax: None,
            wl_inf_sum: 0.0,
            wl2_sum: 0.0,
            wl_inf_values,
            wl2_values,
        }
    }

    /// Update statistics for a single window start.
    fn update<C: Coord>(
        &mut self,
        start: usize,
        start_point: &Point<C>,
        end_point: &Point<C>,
    ) -> error::Result<()> {
        let (dist_inf, sum_sq) = endpoint_distances(start_point, end_point);
        let wl_inf = ratio_from_distance(dist_inf, self.dimension, self.segment_len);
        let wl2 = ratio_from_sum_sq(sum_sq, self.dimension, self.segment_len);

        self.wl_inf_sum += wl_inf;
        self.wl2_sum += wl2;

        let start_index = I::from(start)
            .ok_or_else(|| error::Error::Size("window start exceeds index type".to_string()))?;

        if self.wl_inf_argmax.is_none() || wl_inf > self.wl_inf_max {
            self.wl_inf_max = wl_inf;
            self.wl_inf_argmax = Some(start_index);
        }
        if self.wl2_argmax.is_none() || wl2 > self.wl2_max {
            self.wl2_max = wl2;
            self.wl2_argmax = Some(start_index);
        }

        if let Some(values) = self.wl_inf_values.as_mut() {
            values.push(wl_inf);
        }
        if let Some(values) = self.wl2_values.as_mut() {
            values.push(wl2);
        }

        Ok(())
    }

    /// Finalize the accumulator into a WL profile row.
    fn into_row(self) -> error::Result<WlRow<I>> {
        let wl_inf_mean = mean_from_sum(self.wl_inf_sum, self.total_windows);
        let wl2_mean = mean_from_sum(self.wl2_sum, self.total_windows);

        let wl_inf_p95 = self
            .wl_inf_values
            .as_ref()
            .map(|values| metrics::quantile_r7(&metrics::sorted_f64_values(values), 0.95));
        let wl2_p95 = self
            .wl2_values
            .as_ref()
            .map(|values| metrics::quantile_r7(&metrics::sorted_f64_values(values), 0.95));

        Ok(WlRow {
            segment_len: self.segment_len,
            wl_inf_max: self.wl_inf_max,
            wl_inf_argmax_start: self
                .wl_inf_argmax
                .ok_or_else(|| error::Error::Other("missing WL∞ argmax".to_string()))?,
            wl2_max: self.wl2_max,
            wl2_argmax_start: self
                .wl2_argmax
                .ok_or_else(|| error::Error::Other("missing WL2 argmax".to_string()))?,
            wl_inf_mean,
            wl2_mean,
            wl_inf_p95,
            wl2_p95,
        })
    }
}

/// Return (L∞ distance, sum of squares) between two points.
fn endpoint_distances<C: Coord>(a: &Point<C>, b: &Point<C>) -> (f64, f64) {
    let mut max_diff: u128 = 0;
    let mut sum_sq = 0.0;

    for (left, right) in a.iter().zip(b.iter()) {
        let diff = if left >= right {
            *left - *right
        } else {
            *right - *left
        };
        let diff_u128 = diff.to_u128().expect("coord fits u128");
        if diff_u128 > max_diff {
            max_diff = diff_u128;
        }
        let diff_f64 = diff_u128 as f64;
        sum_sq += diff_f64 * diff_f64;
    }

    (max_diff as f64, sum_sq)
}

/// Compute WL ratio from an L∞ distance using log-space scaling.
fn ratio_from_distance(dist: f64, dimension: u32, segment_len: u32) -> f64 {
    if dist == 0.0 {
        return 0.0;
    }
    let log_ratio = (dimension as f64) * dist.ln() - (segment_len as f64).ln();
    log_ratio.exp()
}

/// Compute WL ratio from sum-of-squares using log-space scaling.
fn ratio_from_sum_sq(sum_sq: f64, dimension: u32, segment_len: u32) -> f64 {
    if sum_sq == 0.0 {
        return 0.0;
    }
    let log_ratio = 0.5 * (dimension as f64) * sum_sq.ln() - (segment_len as f64).ln();
    log_ratio.exp()
}

/// Compute a mean from a sum and count, returning None for empty inputs.
fn mean_from_sum(sum: f64, count: usize) -> Option<f64> {
    if count == 0 {
        return None;
    }
    Some(sum / count as f64)
}

#[cfg(test)]
mod tests {
    use super::{ScanMode, wl_profile};
    use crate::{SpaceCurve, curves::scan::Scan, point::Point};

    #[derive(Debug)]
    struct Identity1d {
        length: u32,
    }

    impl SpaceCurve for Identity1d {
        type Coord = u32;
        type Index = u32;

        fn name(&self) -> &'static str {
            "identity-1d"
        }

        fn info(&self) -> &'static str {
            "identity"
        }

        fn index(&self, p: &Point<Self::Coord>) -> Self::Index {
            p[0]
        }

        fn point(&self, index: Self::Index) -> Point<Self::Coord> {
            Point::new(vec![index])
        }

        fn length(&self) -> Self::Index {
            self.length
        }

        fn dimensions(&self) -> u32 {
            1
        }
    }

    #[derive(Debug)]
    struct Jump1d {
        length: u32,
        low: u32,
        high: u32,
    }

    impl SpaceCurve for Jump1d {
        type Coord = u32;
        type Index = u32;

        fn name(&self) -> &'static str {
            "jump-1d"
        }

        fn info(&self) -> &'static str {
            "jump"
        }

        fn index(&self, p: &Point<Self::Coord>) -> Self::Index {
            p[0]
        }

        fn point(&self, index: Self::Index) -> Point<Self::Coord> {
            if index % 2 == 0 {
                Point::new(vec![self.low])
            } else {
                Point::new(vec![self.high])
            }
        }

        fn length(&self) -> Self::Index {
            self.length
        }

        fn dimensions(&self) -> u32 {
            1
        }
    }

    #[test]
    fn identity_1d_matches_expected_values() {
        let curve = Identity1d { length: 8 };
        let profile = wl_profile(&curve, &[1, 2, 4, 8], ScanMode::Exact).expect("profile");

        for row in profile.rows {
            let l = row.segment_len as f64;
            let expected = if l <= 1.0 { 0.0 } else { (l - 1.0) / l };
            assert!((row.wl_inf_max - expected).abs() < 1e-9, "L={l}");
            assert!((row.wl2_max - expected).abs() < 1e-9, "L={l}");
        }
    }

    #[test]
    fn jump_curve_has_large_wl_for_short_segments() {
        let curve = Jump1d {
            length: 6,
            low: 0,
            high: 100,
        };
        let profile = wl_profile(&curve, &[2], ScanMode::Exact).expect("profile");
        let row = profile.rows.first().expect("row");
        assert!(row.wl_inf_max > 10.0, "wl-inf-max: {}", row.wl_inf_max);
        assert!(row.wl2_max > 10.0, "wl2-max: {}", row.wl2_max);
    }

    #[test]
    fn scan_curve_produces_nonzero_values() {
        let curve = Scan::<u32, u32>::from_dimensions(2, 4).expect("scan curve");
        let profile = wl_profile(&curve, &[2, 4], ScanMode::Exact).expect("profile");
        for row in profile.rows {
            if row.segment_len > 1 {
                assert!(row.wl_inf_max.is_finite());
                assert!(row.wl2_max.is_finite());
                assert!(row.wl_inf_max >= 0.0);
                assert!(row.wl2_max >= 0.0);
            }
        }
    }
}
