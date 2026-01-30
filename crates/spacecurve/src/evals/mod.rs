//! Evaluation metrics for space-filling curves.
//!
//! Evaluations compute metrics over curves for a shared grid specification and
//! return comparable results. The CLI consumes these metrics to render tables or
//! JSON output.

use std::fmt;

use rand::{SeedableRng, seq::index::sample};
use rand_chacha::ChaCha8Rng;

use crate::{SpaceCurve, point::Point, spec::GridSpec};

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
pub struct EvalParams {
    /// Validated grid specification.
    pub spec: GridSpec,
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
    pub fn run(self, curve: &dyn SpaceCurve, params: &EvalParams) -> Vec<MetricValue> {
        match self {
            Self::Nns => nns::run(curve, params),
        }
    }
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
pub fn effective_sample_count(length: u32, samples: Option<usize>) -> usize {
    let length = length as usize;
    match samples {
        Some(count) => count.min(length),
        None if length <= DEFAULT_SAMPLE_LIMIT => length,
        None => DEFAULT_SAMPLE_LIMIT.min(length),
    }
}

/// Sample curve indices without replacement using a deterministic RNG.
pub(crate) fn sample_indices(length: u32, samples: Option<usize>, seed: u64) -> Vec<u32> {
    let length = length as usize;
    let target = effective_sample_count(length as u32, samples);

    if target == length {
        return (0..length as u32).collect();
    }

    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let indices = sample(&mut rng, length, target);
    indices
        .into_vec()
        .into_iter()
        .map(|idx| idx as u32)
        .collect()
}

/// Return in-bounds L1 neighbors for a point in a `size^dimension` grid.
pub(crate) fn grid_neighbors(point: &Point, size: u32) -> Vec<Point> {
    let dimension = point.dimension() as usize;
    let mut neighbors = Vec::with_capacity(dimension.saturating_mul(2));

    for axis in 0..dimension {
        let coord = point[axis];
        if coord > 0 {
            let mut coords = point.0.clone();
            coords[axis] = coord - 1;
            neighbors.push(Point::new_with_dimension(dimension as u32, coords));
        }
        if coord + 1 < size {
            let mut coords = point.0.clone();
            coords[axis] = coord + 1;
            neighbors.push(Point::new_with_dimension(dimension as u32, coords));
        }
    }

    neighbors
}
