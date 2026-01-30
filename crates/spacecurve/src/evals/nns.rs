use crate::{
    SpaceCurve, error,
    evals::{
        EvalParams, MetricDef, MetricDirection, MetricValue, grid_neighbors, metrics,
        sample_indices,
    },
    types::{Coord, Index},
};

/// Metric definitions for nearest-neighbor stretch.
pub const METRIC_DEFS: &[MetricDef] = &[
    MetricDef {
        name: "nns-mean",
        description: "Average stretch across all sampled neighbor pairs",
        direction: MetricDirection::Minimize,
    },
    MetricDef {
        name: "nns-max",
        description: "Maximum observed stretch",
        direction: MetricDirection::Minimize,
    },
    MetricDef {
        name: "nns-p25",
        description: "25th percentile of stretch distribution",
        direction: MetricDirection::Minimize,
    },
    MetricDef {
        name: "nns-p50",
        description: "Median stretch",
        direction: MetricDirection::Minimize,
    },
    MetricDef {
        name: "nns-p75",
        description: "75th percentile of stretch distribution",
        direction: MetricDirection::Minimize,
    },
    MetricDef {
        name: "nns-p90",
        description: "90th percentile of stretch distribution",
        direction: MetricDirection::Minimize,
    },
    MetricDef {
        name: "nns-p95",
        description: "95th percentile of stretch distribution",
        direction: MetricDirection::Minimize,
    },
    MetricDef {
        name: "nns-p99",
        description: "99th percentile of stretch distribution",
        direction: MetricDirection::Minimize,
    },
];

/// Run nearest-neighbor stretch evaluation for a single curve.
pub fn run<C: Coord, I: Index>(
    curve: &dyn SpaceCurve<Coord = C, Index = I>,
    params: &EvalParams<C, I>,
) -> error::Result<Vec<MetricValue>> {
    let stretches = collect_stretches(curve, params)?;
    if stretches.is_empty() {
        return Ok(METRIC_DEFS
            .iter()
            .map(|metric| MetricValue {
                name: metric.name,
                value: f64::NAN,
            })
            .collect());
    }

    let mean = metrics::mean(&stretches);
    let max = metrics::max(&stretches);
    let sorted = metrics::sorted_f64(&stretches);

    let p25 = metrics::quantile_r7(&sorted, 0.25);
    let p50 = metrics::quantile_r7(&sorted, 0.50);
    let p75 = metrics::quantile_r7(&sorted, 0.75);
    let p90 = metrics::quantile_r7(&sorted, 0.90);
    let p95 = metrics::quantile_r7(&sorted, 0.95);
    let p99 = metrics::quantile_r7(&sorted, 0.99);

    Ok(vec![
        MetricValue {
            name: "nns-mean",
            value: mean,
        },
        MetricValue {
            name: "nns-max",
            value: max,
        },
        MetricValue {
            name: "nns-p25",
            value: p25,
        },
        MetricValue {
            name: "nns-p50",
            value: p50,
        },
        MetricValue {
            name: "nns-p75",
            value: p75,
        },
        MetricValue {
            name: "nns-p90",
            value: p90,
        },
        MetricValue {
            name: "nns-p95",
            value: p95,
        },
        MetricValue {
            name: "nns-p99",
            value: p99,
        },
    ])
}

/// Collect stretch values for unique neighbor edges.
fn collect_stretches<C: Coord, I: Index>(
    curve: &dyn SpaceCurve<Coord = C, Index = I>,
    params: &EvalParams<C, I>,
) -> error::Result<Vec<I>> {
    let spec = params.spec;
    let sample_indices = sample_indices(spec.length(), params.samples, params.seed)?;
    let mut stretches = Vec::new();

    for index in sample_indices {
        let point = curve.point(index);
        for neighbor in grid_neighbors(&point, spec.size()) {
            let neighbor_index = curve.index(&neighbor);
            if neighbor_index > index {
                stretches.push(neighbor_index - index);
            }
        }
    }

    Ok(stretches)
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::{collect_stretches, run};
    use crate::{
        curves::scan::Scan, evals::EvalParams, point::Point, spacecurve::SpaceCurve, spec::GridSpec,
    };

    fn expected_edge_count(dimension: u32, size: u32) -> usize {
        let dimension = dimension as usize;
        let size = size as usize;
        dimension * (size - 1) * size.pow(dimension.saturating_sub(1) as u32)
    }

    fn brute_force_stretches(curve: &Scan<u32, u32>, dimension: u32, size: u32) -> Vec<u32> {
        fn visit(
            curve: &Scan<u32, u32>,
            dimension: usize,
            size: u32,
            coords: &mut Vec<u32>,
            stretches: &mut Vec<u32>,
        ) {
            if coords.len() == dimension {
                let idx = curve.index(&Point::new(coords.clone()));
                for axis in 0..dimension {
                    if coords[axis] + 1 < size {
                        let mut neighbor = coords.clone();
                        neighbor[axis] += 1;
                        let nidx = curve.index(&Point::new(neighbor));
                        let stretch = nidx.abs_diff(idx);
                        stretches.push(stretch);
                    }
                }
                return;
            }

            for value in 0..size {
                coords.push(value);
                visit(curve, dimension, size, coords, stretches);
                coords.pop();
            }
        }

        let mut stretches = Vec::new();
        let mut coords = Vec::with_capacity(dimension as usize);
        visit(curve, dimension as usize, size, &mut coords, &mut stretches);
        stretches
    }

    fn metric_value(metrics: &[super::MetricValue], name: &str) -> f64 {
        metrics
            .iter()
            .find(|metric| metric.name == name)
            .expect("metric present")
            .value
    }

    #[test]
    fn scan_2d_size2_metrics_match_expected_values() {
        let curve = Scan::<u32, u32>::from_dimensions(2, 2).expect("scan curve");
        let spec = GridSpec::<u32, u32>::new(2, 2).expect("grid spec");
        let params = EvalParams {
            spec,
            samples: None,
            seed: 0,
        };

        let metrics = run(&curve, &params).expect("metrics");

        let mean = metric_value(&metrics, "nns-mean");
        let max = metric_value(&metrics, "nns-max");
        let p25 = metric_value(&metrics, "nns-p25");
        let p50 = metric_value(&metrics, "nns-p50");
        let p75 = metric_value(&metrics, "nns-p75");
        let p90 = metric_value(&metrics, "nns-p90");
        let p95 = metric_value(&metrics, "nns-p95");
        let p99 = metric_value(&metrics, "nns-p99");

        assert!((mean - 1.5).abs() < 1e-6, "mean: {mean}");
        assert!((max - 3.0).abs() < 1e-6, "max: {max}");
        assert!((p25 - 1.0).abs() < 1e-6, "p25: {p25}");
        assert!((p50 - 1.0).abs() < 1e-6, "p50: {p50}");
        assert!((p75 - 1.5).abs() < 1e-6, "p75: {p75}");
        assert!((p90 - 2.4).abs() < 1e-6, "p90: {p90}");
        assert!((p95 - 2.7).abs() < 1e-6, "p95: {p95}");
        assert!((p99 - 2.94).abs() < 1e-6, "p99: {p99}");
    }

    #[test]
    fn collect_stretches_matches_bruteforce_for_scan() {
        let curve = Scan::<u32, u32>::from_dimensions(2, 3).expect("scan curve");
        let spec = GridSpec::<u32, u32>::new(2, 3).expect("grid spec");
        let params = EvalParams {
            spec,
            samples: None,
            seed: 0,
        };

        let mut actual = collect_stretches(&curve, &params).expect("stretches");
        let mut expected = brute_force_stretches(&curve, 2, 3);
        actual.sort_unstable();
        expected.sort_unstable();

        assert_eq!(actual, expected);
    }

    proptest! {
        #[test]
        fn stretches_are_positive(dimension in 1u32..=4, size in 2u32..=6) {
            let curve = Scan::<u32, u32>::from_dimensions(dimension, size).expect("scan curve");
            let spec = GridSpec::<u32, u32>::new(dimension, size).expect("grid spec");
            let params = EvalParams {
                spec,
                samples: None,
                seed: 0,
            };

            let stretches = collect_stretches(&curve, &params).expect("stretches");
            prop_assert!(!stretches.is_empty());
            prop_assert!(stretches.iter().all(|value| *value >= 1));
        }

        #[test]
        fn stretch_count_matches_grid_edges(dimension in 1u32..=4, size in 2u32..=6) {
            let curve = Scan::<u32, u32>::from_dimensions(dimension, size).expect("scan curve");
            let spec = GridSpec::<u32, u32>::new(dimension, size).expect("grid spec");
            let params = EvalParams {
                spec,
                samples: None,
                seed: 0,
            };

            let stretches = collect_stretches(&curve, &params).expect("stretches");
            let expected = expected_edge_count(dimension, size);
            prop_assert_eq!(stretches.len(), expected);
        }

        #[test]
        fn percentiles_are_monotonic(dimension in 1u32..=4, size in 2u32..=6) {
            let curve = Scan::<u32, u32>::from_dimensions(dimension, size).expect("scan curve");
            let spec = GridSpec::<u32, u32>::new(dimension, size).expect("grid spec");
            let params = EvalParams {
                spec,
                samples: None,
                seed: 0,
            };

            let metrics = run(&curve, &params).expect("metrics");
            let p25 = metric_value(&metrics, "nns-p25");
            let p50 = metric_value(&metrics, "nns-p50");
            let p75 = metric_value(&metrics, "nns-p75");
            let p90 = metric_value(&metrics, "nns-p90");
            let p95 = metric_value(&metrics, "nns-p95");
            let p99 = metric_value(&metrics, "nns-p99");

            prop_assert!(p25 <= p50);
            prop_assert!(p50 <= p75);
            prop_assert!(p75 <= p90);
            prop_assert!(p90 <= p95);
            prop_assert!(p95 <= p99);
            prop_assert!(p99 >= 0.0);
        }
    }
}
