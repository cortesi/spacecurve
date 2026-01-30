use std::cmp::Ordering;

use crate::types::Index;

/// Quantile method label for R-7 (linear interpolation between closest ranks).
pub const QUANTILE_METHOD_R7: &str = "r7";

/// Compute the arithmetic mean of the provided values.
pub fn mean<I: Index>(values: &[I]) -> f64 {
    if values.is_empty() {
        return f64::NAN;
    }

    let sum: f64 = values
        .iter()
        .map(|value| value.to_f64().expect("value fits f64"))
        .sum();
    sum / values.len() as f64
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

/// Return sorted values for percentile computation.
pub fn sorted_f64_values(values: &[f64]) -> Vec<f64> {
    let mut sorted = values.to_vec();
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mean_handles_large_values_without_overflow() {
        let values = [u128::MAX - 1, u128::MAX - 1];
        let mean_value = mean(&values);
        let expected = (u128::MAX - 1) as f64;
        assert!(mean_value.is_finite());
        assert!((mean_value - expected).abs() < 1.0);
    }

    #[test]
    fn sorted_f64_orders_values() {
        let values = [5_u32, 2, 4, 1, 3];
        let sorted = sorted_f64(&values);
        assert_eq!(sorted, vec![1.0, 2.0, 3.0, 4.0, 5.0]);
    }

    #[test]
    fn quantile_r7_matches_reference_values() {
        let sorted = vec![1.0, 2.0, 3.0, 4.0];
        assert!((quantile_r7(&sorted, 0.25) - 1.75).abs() < 1e-9);
        assert!((quantile_r7(&sorted, 0.5) - 2.5).abs() < 1e-9);
        assert!((quantile_r7(&sorted, 0.75) - 3.25).abs() < 1e-9);
    }
}
