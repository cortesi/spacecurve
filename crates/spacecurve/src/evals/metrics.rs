use std::cmp::Ordering;

use crate::types::Index;

/// Quantile method label for R-7 (linear interpolation between closest ranks).
pub const QUANTILE_METHOD_R7: &str = "r7";

/// Compute the arithmetic mean of the provided values.
pub fn mean<I: Index>(values: &[I]) -> f64 {
    if values.is_empty() {
        return f64::NAN;
    }

    let sum: u128 = values
        .iter()
        .map(|value| value.to_u128().expect("value fits u128"))
        .sum();
    sum as f64 / values.len() as f64
}

/// Return the maximum value or NaN when the slice is empty.
pub fn max<I: Index>(values: &[I]) -> f64 {
    values
        .iter()
        .copied()
        .max()
        .and_then(|value| value.to_f64())
        .unwrap_or(f64::NAN)
}

/// Return sorted values as `f64` for percentile computation.
pub fn sorted_f64<I: Index>(values: &[I]) -> Vec<f64> {
    let mut sorted: Vec<f64> = values
        .iter()
        .map(|value| value.to_f64().expect("value fits f64"))
        .collect();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));
    sorted
}

/// Compute the R-7 quantile for the already-sorted values.
pub fn quantile_r7(sorted: &[f64], probability: f64) -> f64 {
    if sorted.is_empty() {
        return f64::NAN;
    }

    if sorted.len() == 1 {
        return sorted[0];
    }

    let clamped = probability.clamp(0.0, 1.0);
    let n = sorted.len() as f64;
    let h = (n - 1.0) * clamped + 1.0;
    let i = h.floor();
    let gamma = h - i;
    let idx = i as usize;

    if idx >= sorted.len() {
        return *sorted.last().expect("non-empty");
    }

    let lower = sorted[idx.saturating_sub(1)];
    let upper = sorted[idx];
    lower + gamma * (upper - lower)
}
