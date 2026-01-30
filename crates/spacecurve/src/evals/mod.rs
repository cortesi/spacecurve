//! Evaluation metrics for space-filling curves.
//!
//! Evaluations compute metrics over curves for a shared grid specification and
//! return comparable results. The CLI consumes these metrics to render tables or
//! JSON output.

use std::fmt;

use rand::{SeedableRng, seq::index::sample};
use rand_chacha::ChaCha8Rng;

use crate::{
    SpaceCurve, error,
    point::Point,
    spec::GridSpec,
    types::{Coord, Index},
};

/// Distribution helpers shared across evaluations.
mod metrics;
/// Nearest-neighbor stretch evaluation.
mod nns;

pub use metrics::QUANTILE_METHOD_R7;

/// Whether lower or higher metric values are better.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MetricDirection {
    /// Lower values are better.
    Minimize,
    /// Higher values are better.
    Maximize,
}

impl MetricDirection {
    /// Return a lowercase label for display or JSON output.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Minimize => "minimize",
            Self::Maximize => "maximize",
        }
    }

    /// Compare two candidate values, returning true when `candidate` is better.
    pub fn is_better(self, candidate: f64, best: f64) -> bool {
        match self {
            Self::Minimize => candidate < best,
            Self::Maximize => candidate > best,
        }
    }
}

impl fmt::Display for MetricDirection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Metric metadata published by an evaluation.
#[derive(Clone, Copy, Debug)]
pub struct MetricDef {
    /// Metric name (stable identifier for CLI and JSON).
    pub name: &'static str,
    /// Human-readable description of the metric.
    pub description: &'static str,
    /// Whether the metric is minimized or maximized.
    pub direction: MetricDirection,
}

/// A computed metric value.
#[derive(Clone, Copy, Debug)]
pub struct MetricValue {
    /// Metric name (matches a [`MetricDef`]).
    pub name: &'static str,
    /// Metric value.
    pub value: f64,
}

/// Common parameters for evaluations.
#[derive(Clone, Copy, Debug)]
pub struct EvalParams<C: Coord, I: Index> {
    /// Validated grid specification.
    pub spec: GridSpec<C, I>,
    /// Optional sample count (None uses the default sampling strategy).
    pub samples: Option<usize>,
    /// RNG seed for reproducible sampling.
    pub seed: u64,
}

/// Results from running an evaluation on a single curve.
#[derive(Clone, Debug)]
pub struct EvalResult {
    /// Curve key.
    pub curve: &'static str,
    /// Metric values for the curve.
    pub metrics: Vec<MetricValue>,
}

/// Registry of available evaluations.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Evaluation {
    /// Nearest-neighbor stretch.
    Nns,
}

impl Evaluation {
    /// Machine-readable key for this evaluation (for CLI and JSON).
    pub fn key(self) -> &'static str {
        match self {
            Self::Nns => "nns",
        }
    }

    /// Human-readable title for this evaluation.
    pub fn title(self) -> &'static str {
        match self {
            Self::Nns => "Nearest-Neighbor Stretch",
        }
    }

    /// Metric definitions this evaluation produces, in output order.
    pub fn metric_defs(self) -> &'static [MetricDef] {
        match self {
            Self::Nns => nns::METRIC_DEFS,
        }
    }

    /// Optional quantile method label for evaluations that compute percentiles.
    pub fn quantile_method(self) -> Option<&'static str> {
        match self {
            Self::Nns => Some(QUANTILE_METHOD_R7),
        }
    }

    /// Run the evaluation on a single curve, returning metric values.
    pub fn run<C: Coord, I: Index>(
        self,
        curve: &dyn SpaceCurve<Coord = C, Index = I>,
        params: &EvalParams<C, I>,
    ) -> error::Result<Vec<MetricValue>> {
        validate_eval_params(curve, params)?;
        match self {
            Self::Nns => nns::run(curve, params),
        }
    }
}

/// Validate that the curve matches the evaluation parameters.
fn validate_eval_params<C: Coord, I: Index>(
    curve: &dyn SpaceCurve<Coord = C, Index = I>,
    params: &EvalParams<C, I>,
) -> error::Result<()> {
    if curve.dimensions() != params.spec.dimension() {
        return Err(error::Error::Shape(format!(
            "curve dimension {} does not match spec dimension {}",
            curve.dimensions(),
            params.spec.dimension()
        )));
    }
    let curve_length = curve.length();
    let spec_length = params.spec.length();
    if curve_length != spec_length {
        return Err(error::Error::Size(format!(
            "curve length {:?} does not match spec length {:?}",
            curve_length, spec_length
        )));
    }
    Ok(())
}

/// All supported evaluations in stable ordering.
const EVALUATIONS: [Evaluation; 1] = [Evaluation::Nns];

/// Return all supported evaluations.
pub fn evaluations() -> &'static [Evaluation] {
    &EVALUATIONS
}

/// Default cap on samples when `samples` is unset.
const DEFAULT_SAMPLE_LIMIT: usize = 10_000;

/// Compute the effective sample count for a curve of length `length`.
pub fn effective_sample_count<I: Index>(length: I, samples: Option<usize>) -> error::Result<usize> {
    let length = length
        .to_usize()
        .ok_or_else(|| error::Error::Size("curve length exceeds usize".to_string()))?;
    match samples {
        Some(0) => Err(error::Error::Size("sample count must be >= 1".to_string())),
        Some(count) => Ok(count.min(length)),
        None if length <= DEFAULT_SAMPLE_LIMIT => Ok(length),
        None => Ok(DEFAULT_SAMPLE_LIMIT.min(length)),
    }
}

/// Sample curve indices without replacement using a deterministic RNG.
pub(crate) fn sample_indices<I: Index>(
    length: I,
    samples: Option<usize>,
    seed: u64,
) -> error::Result<Vec<I>> {
    let length_usize = length
        .to_usize()
        .ok_or_else(|| error::Error::Size("curve length exceeds usize".to_string()))?;
    let target = effective_sample_count(length, samples)?;

    if target == length_usize {
        return Ok((0..length_usize)
            .map(|idx| I::from(idx).expect("index fits target type"))
            .collect());
    }

    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let indices = sample(&mut rng, length_usize, target);
    Ok(indices
        .into_vec()
        .into_iter()
        .map(|idx| I::from(idx).expect("index fits target type"))
        .collect())
}

/// Return in-bounds L1 neighbors for a point in a `size^dimension` grid.
pub(crate) fn grid_neighbors<C: Coord>(point: &Point<C>, size: C) -> Vec<Point<C>> {
    let dimension = point.dimension() as usize;
    let mut neighbors = Vec::with_capacity(dimension.saturating_mul(2));

    for axis in 0..dimension {
        let coord = point[axis];
        if coord > C::zero() {
            let mut coords = point.0.clone();
            coords[axis] = coord - C::one();
            neighbors.push(Point::new_with_dimension(dimension as u32, coords));
        }
        if coord + C::one() < size {
            let mut coords = point.0.clone();
            coords[axis] = coord + C::one();
            neighbors.push(Point::new_with_dimension(dimension as u32, coords));
        }
    }

    neighbors
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{curves::scan::Scan, error::Error};

    #[test]
    fn effective_sample_count_rejects_zero() {
        let err = effective_sample_count(8_u32, Some(0)).unwrap_err();
        assert!(matches!(err, Error::Size(_)));
    }

    #[test]
    fn effective_sample_count_defaults_to_limit() -> error::Result<()> {
        let count = effective_sample_count(10_001_u32, None)?;
        assert_eq!(count, DEFAULT_SAMPLE_LIMIT);
        Ok(())
    }

    #[test]
    fn sample_indices_returns_full_range_when_unlimited() -> error::Result<()> {
        let indices = sample_indices(8_u32, None, 0)?;
        assert_eq!(indices, (0_u32..8).collect::<Vec<_>>());
        Ok(())
    }

    #[test]
    fn sample_indices_rejects_zero_samples() {
        let err = sample_indices(8_u32, Some(0), 0).unwrap_err();
        assert!(matches!(err, Error::Size(_)));
    }

    #[test]
    fn grid_neighbors_match_expected_2d() {
        let mut neighbors: Vec<Vec<u32>> = grid_neighbors(&Point::new(vec![0_u32, 0_u32]), 3_u32)
            .into_iter()
            .map(Vec::from)
            .collect();
        neighbors.sort();
        assert_eq!(neighbors, vec![vec![0, 1], vec![1, 0]]);

        let mut neighbors: Vec<Vec<u32>> = grid_neighbors(&Point::new(vec![1_u32, 1_u32]), 3_u32)
            .into_iter()
            .map(Vec::from)
            .collect();
        neighbors.sort();
        assert_eq!(
            neighbors,
            vec![vec![0, 1], vec![1, 0], vec![1, 2], vec![2, 1]]
        );
    }

    #[test]
    fn evaluation_rejects_dimension_mismatch() {
        let curve = Scan::<u32, u32>::from_dimensions(2, 2).expect("scan curve");
        let spec = GridSpec::<u32, u32>::new(3, 2).expect("grid spec");
        let params = EvalParams {
            spec,
            samples: None,
            seed: 0,
        };
        let err = Evaluation::Nns.run(&curve, &params).unwrap_err();
        assert!(matches!(err, Error::Shape(_)));
    }

    #[test]
    fn evaluation_rejects_length_mismatch() {
        let curve = Scan::<u32, u32>::from_dimensions(2, 2).expect("scan curve");
        let spec = GridSpec::<u32, u32>::new(2, 3).expect("grid spec");
        let params = EvalParams {
            spec,
            samples: None,
            seed: 0,
        };
        let err = Evaluation::Nns.run(&curve, &params).unwrap_err();
        assert!(matches!(err, Error::Size(_)));
    }
}
