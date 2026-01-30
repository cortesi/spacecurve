use crate::{
    SpaceCurve,
    evals::{
        EvalParams, MetricDef, MetricDirection, MetricValue, grid_neighbors, metrics,
        sample_indices,
    },
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
pub fn run(curve: &dyn SpaceCurve, params: &EvalParams) -> Vec<MetricValue> {
    let stretches = collect_stretches(curve, params);
    if stretches.is_empty() {
        return METRIC_DEFS
            .iter()
            .map(|metric| MetricValue {
                name: metric.name,
                value: f64::NAN,
            })
            .collect();
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

    vec![
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
    ]
}

/// Collect stretch values for unique neighbor edges.
fn collect_stretches(curve: &dyn SpaceCurve, params: &EvalParams) -> Vec<u32> {
    let spec = params.spec;
    let sample_indices = sample_indices(spec.length(), params.samples, params.seed);
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

    stretches
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::{collect_stretches, run};
    use crate::{curves::scan::Scan, evals::EvalParams, spec::GridSpec};

    fn metric_value(metrics: &[super::MetricValue], name: &str) -> f64 {
        metrics
            .iter()
            .find(|metric| metric.name == name)
            .expect("metric present")
            .value
    }

    #[test]
    fn scan_2d_size2_metrics_match_expected_values() {
        let curve = Scan::from_dimensions(2, 2).expect("scan curve");
        let spec = GridSpec::new(2, 2).expect("grid spec");
        let params = EvalParams {
            spec,
            samples: None,
            seed: 0,
        };

        let metrics = run(&curve, &params);

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

    proptest! {
        #[test]
        fn stretches_are_positive(dimension in 1u32..=4, size in 2u32..=6) {
            let curve = Scan::from_dimensions(dimension, size).expect("scan curve");
            let spec = GridSpec::new(dimension, size).expect("grid spec");
            let params = EvalParams {
                spec,
                samples: None,
                seed: 0,
            };

            let stretches = collect_stretches(&curve, &params);
            prop_assert!(!stretches.is_empty());
            prop_assert!(stretches.iter().all(|value| *value >= 1));
        }

        #[test]
        fn percentiles_are_monotonic(dimension in 1u32..=4, size in 2u32..=6) {
            let curve = Scan::from_dimensions(dimension, size).expect("scan curve");
            let spec = GridSpec::new(dimension, size).expect("grid spec");
            let params = EvalParams {
                spec,
                samples: None,
                seed: 0,
            };

            let metrics = run(&curve, &params);
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
