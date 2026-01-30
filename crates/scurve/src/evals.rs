//! CLI evaluation handlers for curve metrics.

use std::{
    collections::{BTreeMap, BTreeSet},
    io::{self, IsTerminal},
};

use anyhow::{Context, Result, bail};
use serde::Serialize;
use spacecurve::{
    DefaultCoord, DefaultIndex,
    evals::{
        EvalParams, EvalResult, Evaluation, MetricDef, MetricValue, effective_sample_count,
        evaluations,
    },
    registry,
    spec::GridSpec,
};
use tabled::{
    Table,
    builder::Builder,
    settings::{Color, Style, object::Cell},
};

/// Coordinate type used by CLI evaluations.
type Coord = DefaultCoord;
/// Index type used by CLI evaluations.
type Index = DefaultIndex;
/// Grid specification alias for CLI evaluations.
type Spec = GridSpec<Coord, Index>;
/// Registry entry alias for CLI evaluations.
type Entry = registry::CurveEntry<Coord, Index>;

/// Common options shared across evaluation commands.
#[derive(Clone, Debug)]
pub struct EvalsCommonOptions {
    /// Optional comma-separated curve list.
    pub curves: Option<String>,
    /// Include experimental curves when selecting all curves.
    pub include_experimental: bool,
    /// Emit JSON output.
    pub json: bool,
    /// Optional comma-separated metric list.
    pub metrics: Option<String>,
    /// RNG seed.
    pub seed: u64,
}

/// Run `scurve evals list`.
pub fn handle_list(options: &EvalsCommonOptions) -> Result<()> {
    if options.json {
        let output = JsonEvalList {
            evaluations: evaluations()
                .iter()
                .map(|evaluation| JsonEvalDefinition {
                    evaluation: evaluation.key(),
                    title: evaluation.title(),
                    metrics: evaluation
                        .metric_defs()
                        .iter()
                        .map(JsonMetricDef::from)
                        .collect(),
                })
                .collect(),
        };
        println!("{}", serde_json::to_string_pretty(&output)?);
        return Ok(());
    }

    let mut builder = Builder::default();
    builder.push_record(["Evaluation", "Metric", "Direction", "Description"]);

    for evaluation in evaluations() {
        for metric in evaluation.metric_defs() {
            builder.push_record([
                evaluation.key(),
                metric.name,
                metric.direction.as_str(),
                metric.description,
            ]);
        }
    }

    let mut table = builder.build();
    table.with(Style::modern());
    println!("{table}");

    Ok(())
}

/// Run `scurve evals nns`.
pub fn handle_nns(
    options: &EvalsCommonOptions,
    size: u32,
    dimension: u32,
    samples: Option<u32>,
) -> Result<()> {
    if size < 2 {
        bail!("size must be >= 2 for nearest-neighbor evaluation");
    }

    let spec = GridSpec::<Coord, Index>::new(dimension, size)?;
    let eval = Evaluation::Nns;

    let curve_list = parse_csv_list(&options.curves, "curve")?;
    let metric_list = parse_csv_list(&options.metrics, "metric")?;

    let selected_metrics = resolve_metric_defs(eval.metric_defs(), metric_list.as_deref())?;
    let (mut curves, skipped, used_default) = select_curves(
        dimension,
        size,
        curve_list.as_deref(),
        options.include_experimental,
    )?;

    if curves.is_empty() {
        bail!("no curves available for dimension {dimension}, size {size}");
    }

    curves.sort_by(|left, right| left.entry.key.cmp(right.entry.key));

    if used_default {
        emit_skipped_warnings(&skipped);
    }

    let params = EvalParams {
        spec,
        samples: samples.map(|value| value as usize),
        seed: options.seed,
    };

    let mut results = Vec::new();
    for selection in curves {
        let curve = (selection.entry.ctor)(&selection.spec)
            .with_context(|| format!("failed to construct curve '{}'", selection.entry.key))?;
        let metrics = eval
            .run(&*curve, &params)
            .with_context(|| format!("evaluation failed for '{}'", selection.entry.key))?;
        let filtered = filter_metric_values(&metrics, &selected_metrics);
        results.push(EvalResult {
            curve: selection.entry.key,
            metrics: filtered,
        });
    }

    if options.json {
        let output = JsonEvalOutput::new(
            eval,
            &params,
            &selected_metrics,
            results,
            if used_default { skipped } else { Vec::new() },
        )?;
        println!("{}", serde_json::to_string_pretty(&output)?);
        return Ok(());
    }

    render_table(eval, &params, &selected_metrics, &results)?;
    Ok(())
}

/// Parse a comma-separated list, rejecting duplicates and empty values.
fn parse_csv_list(input: &Option<String>, label: &str) -> Result<Option<Vec<String>>> {
    let Some(raw) = input.as_ref() else {
        return Ok(None);
    };

    let items: Vec<String> = raw
        .split(',')
        .map(|item| item.trim())
        .filter(|item| !item.is_empty())
        .map(|item| item.to_string())
        .collect();

    if items.is_empty() {
        bail!("{label} list is empty");
    }

    let mut seen = BTreeSet::new();
    let mut unique = Vec::new();
    for item in items {
        if !seen.insert(item.clone()) {
            bail!("duplicate {label} '{item}'");
        }
        unique.push(item);
    }

    Ok(Some(unique))
}

/// Resolve requested metric names to their definitions.
fn resolve_metric_defs<'a>(
    defs: &'a [MetricDef],
    requested: Option<&[String]>,
) -> Result<Vec<&'a MetricDef>> {
    if let Some(requested) = requested {
        let mut by_name = BTreeMap::new();
        for def in defs {
            by_name.insert(def.name, def);
        }

        let mut selected = Vec::with_capacity(requested.len());
        for name in requested {
            let def = by_name
                .get(name.as_str())
                .with_context(|| format!("unknown metric '{name}'"))?;
            selected.push(*def);
        }
        return Ok(selected);
    }

    Ok(defs.iter().collect())
}

/// Filter metric values to match the selected metric definitions.
fn filter_metric_values(values: &[MetricValue], defs: &[&MetricDef]) -> Vec<MetricValue> {
    let mut by_name = BTreeMap::new();
    for metric in values {
        by_name.insert(metric.name, metric.value);
    }

    defs.iter()
        .map(|def| MetricValue {
            name: def.name,
            value: *by_name
                .get(def.name)
                .expect("metric values include definition"),
        })
        .collect()
}

/// Curve entry paired with a validated grid spec.
struct SelectedCurve {
    /// Curve registry entry.
    entry: Entry,
    /// Spec validated for this curve.
    spec: Spec,
}

/// Curve skipped due to unsupported grid parameters.
#[derive(Clone, Debug, Serialize)]
struct SkippedCurve {
    /// Curve key.
    curve: &'static str,
    /// Reason the curve was skipped.
    reason: String,
}

/// Select curves to evaluate and collect any skipped entries.
fn select_curves(
    dimension: u32,
    size: u32,
    curves: Option<&[String]>,
    include_experimental: bool,
) -> Result<(Vec<SelectedCurve>, Vec<SkippedCurve>, bool)> {
    if let Some(curves) = curves {
        let mut selected = Vec::with_capacity(curves.len());
        for curve in curves {
            let entry = registry::find::<Coord, Index>(curve)
                .with_context(|| format!("unknown curve '{curve}'"))?;
            let spec = (entry.build_spec)(dimension, size)
                .with_context(|| format!("curve '{curve}' does not support the grid"))?;
            selected.push(SelectedCurve { entry, spec });
        }
        return Ok((selected, Vec::new(), false));
    }

    let mut selected = Vec::new();
    let mut skipped = Vec::new();
    for key in registry::curve_names(include_experimental) {
        let entry = registry::find::<Coord, Index>(key)
            .with_context(|| format!("unknown curve '{key}'"))?;
        match (entry.build_spec)(dimension, size) {
            Ok(spec) => selected.push(SelectedCurve { entry, spec }),
            Err(err) => skipped.push(SkippedCurve {
                curve: entry.key,
                reason: err.to_string(),
            }),
        }
    }

    Ok((selected, skipped, true))
}

/// Print warnings for curves skipped during default selection.
fn emit_skipped_warnings(skipped: &[SkippedCurve]) {
    for entry in skipped {
        eprintln!("Skipping curve '{}': {}", entry.curve, entry.reason);
    }
}

/// Render evaluation results as a comparison table.
fn render_table(
    eval: Evaluation,
    params: &EvalParams<Coord, Index>,
    metrics: &[&MetricDef],
    results: &[EvalResult],
) -> Result<()> {
    let sample_count = effective_sample_count(params.spec.length(), params.samples)?;
    println!(
        "{} (size={}, dim={}, samples={})",
        eval.title(),
        params.spec.size(),
        params.spec.dimension(),
        sample_count
    );

    let mut builder = Builder::default();
    let mut header = Vec::with_capacity(metrics.len() + 1);
    header.push("Curve".to_string());
    header.extend(metrics.iter().map(|metric| metric.name.to_string()));
    builder.push_record(header);

    let mut rows = Vec::with_capacity(results.len());
    for result in results {
        let mut record = Vec::with_capacity(metrics.len() + 1);
        record.push(result.curve.to_string());
        let values = metrics
            .iter()
            .map(|def| {
                let value = result
                    .metrics
                    .iter()
                    .find(|metric| metric.name == def.name)
                    .expect("metric present")
                    .value;
                format_metric(value)
            })
            .collect::<Vec<_>>();
        record.extend(values);
        builder.push_record(record);
        rows.push(RowValues {
            metrics: metrics
                .iter()
                .map(|def| {
                    result
                        .metrics
                        .iter()
                        .find(|metric| metric.name == def.name)
                        .expect("metric present")
                        .value
                })
                .collect(),
        });
    }

    let mut table = builder.build();
    table.with(Style::modern());

    if io::stdout().is_terminal() {
        highlight_best_values(&mut table, metrics, &rows);
    }

    println!("{table}");
    Ok(())
}

#[derive(Clone, Debug)]
/// Numerical row values used to compute highlights.
struct RowValues {
    /// Metric values in column order.
    metrics: Vec<f64>,
}

/// Apply highlight styling to best metric values in the table.
fn highlight_best_values(table: &mut Table, metrics: &[&MetricDef], rows: &[RowValues]) {
    let highlight = Color::new("\u{1b}[1;32m", "\u{1b}[0m");

    let mut best_values = Vec::with_capacity(metrics.len());
    for (col_idx, def) in metrics.iter().enumerate() {
        let mut best: Option<f64> = None;
        for row in rows {
            let value = row.metrics[col_idx];
            if value.is_nan() {
                continue;
            }
            match best {
                None => best = Some(value),
                Some(current) => {
                    if def.direction.is_better(value, current) {
                        best = Some(value);
                    }
                }
            }
        }
        best_values.push(best);
    }

    for (col_idx, best) in best_values.into_iter().enumerate() {
        let Some(best) = best else {
            continue;
        };
        for (row_idx, row) in rows.iter().enumerate() {
            if (row.metrics[col_idx] - best).abs() < 1e-9 {
                table.modify(Cell::new(row_idx + 1, col_idx + 1), highlight.clone());
            }
        }
    }
}

/// Format metric values for table display.
fn format_metric(value: f64) -> String {
    if value.is_nan() {
        return "NaN".to_string();
    }

    let rounded = value.round();
    if (value - rounded).abs() < 1e-6 {
        format!("{rounded:.0}")
    } else {
        format!("{value:.2}")
    }
}

#[derive(Serialize)]
/// JSON payload for evaluation parameters.
struct JsonParams {
    /// Grid side length.
    size: Coord,
    /// Number of dimensions.
    dimension: u32,
    /// Effective sample count.
    samples: usize,
    /// RNG seed used for sampling.
    seed: u64,
}

#[derive(Serialize)]
/// JSON representation of a metric definition.
struct JsonMetricDef {
    /// Metric name.
    name: &'static str,
    /// Direction label.
    direction: &'static str,
    /// Metric description.
    description: &'static str,
}

impl From<&MetricDef> for JsonMetricDef {
    fn from(def: &MetricDef) -> Self {
        Self {
            name: def.name,
            direction: def.direction.as_str(),
            description: def.description,
        }
    }
}

#[derive(Serialize)]
/// JSON payload for a single curve's results.
struct JsonResult {
    /// Curve key.
    curve: &'static str,
    /// Metric values keyed by name.
    metrics: BTreeMap<String, f64>,
}

#[derive(Serialize)]
/// JSON payload for evaluation output.
struct JsonEvalOutput {
    /// Evaluation key.
    evaluation: &'static str,
    /// Evaluation title.
    title: &'static str,
    /// Evaluation parameters.
    parameters: JsonParams,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Quantile method label for percentile-producing evaluations.
    quantile_method: Option<&'static str>,
    /// Metric definitions included in the output.
    metric_definitions: Vec<JsonMetricDef>,
    /// Evaluation results.
    results: Vec<JsonResult>,
    /// Curves skipped during default selection.
    skipped: Vec<SkippedCurve>,
}

impl JsonEvalOutput {
    /// Build a JSON payload from evaluation results.
    fn new(
        eval: Evaluation,
        params: &EvalParams<Coord, Index>,
        metrics: &[&MetricDef],
        results: Vec<EvalResult>,
        skipped: Vec<SkippedCurve>,
    ) -> Result<Self> {
        let parameters = JsonParams {
            size: params.spec.size(),
            dimension: params.spec.dimension(),
            samples: effective_sample_count(params.spec.length(), params.samples)?,
            seed: params.seed,
        };

        let metric_definitions = metrics
            .iter()
            .map(|def| JsonMetricDef::from(*def))
            .collect();

        let results = results
            .into_iter()
            .map(|result| {
                let mut metrics_map = BTreeMap::new();
                for metric in result.metrics {
                    metrics_map.insert(metric.name.to_string(), metric.value);
                }
                JsonResult {
                    curve: result.curve,
                    metrics: metrics_map,
                }
            })
            .collect();

        Ok(Self {
            evaluation: eval.key(),
            title: eval.title(),
            parameters,
            quantile_method: eval.quantile_method(),
            metric_definitions,
            results,
            skipped,
        })
    }
}

#[derive(Serialize)]
/// JSON payload for a single evaluation definition.
struct JsonEvalDefinition {
    /// Evaluation key.
    evaluation: &'static str,
    /// Evaluation title.
    title: &'static str,
    /// Metric definitions for the evaluation.
    metrics: Vec<JsonMetricDef>,
}

#[derive(Serialize)]
/// JSON payload for the evaluation list command.
struct JsonEvalList {
    /// Available evaluations and their metrics.
    evaluations: Vec<JsonEvalDefinition>,
}
