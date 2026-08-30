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
        EvalParams, EvalResult, Evaluation, MetricDef, MetricValue, QUANTILE_METHOD_R7,
        effective_sample_count, evaluations, wl,
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

/// Run `scurve evals all`.
pub fn handle_all(options: &EvalsCommonOptions) -> Result<()> {
    if options.json {
        bail!("evals all only supports table output");
    }
    if options.metrics.is_some() {
        bail!("evals all does not support metric filtering");
    }

    let nns_size = 64;
    let nns_dims = [2_u32, 3, 4, 6];
    let wl_size = 64;
    let wl_dims = [2_u32, 3, 6];
    let wl_segments = "8,16,32".to_string();
    let wl_mode = crate::WlScanModeArg::Sample;
    let wl_windows_per_len = 512;

    let mut first = true;
    for dim in nns_dims {
        if !first {
            println!();
        }
        handle_nns(options, nns_size, dim, None)?;
        first = false;
    }

    for dim in wl_dims {
        if !first {
            println!();
        }
        handle_wl(
            options,
            wl_size,
            dim,
            Some(&wl_segments),
            wl_mode,
            wl_windows_per_len,
        )?;
        first = false;
    }

    Ok(())
}

/// Run `scurve evals list`.
pub fn handle_list(options: &EvalsCommonOptions) -> Result<()> {
    if options.json {
        let mut definitions: Vec<JsonEvalDefinition> = evaluations()
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
            .collect();
        definitions.push(JsonEvalDefinition {
            evaluation: wl::EVAL_KEY,
            title: wl::EVAL_TITLE,
            metrics: wl::METRIC_DEFS.iter().map(JsonMetricDef::from).collect(),
        });
        let output = JsonEvalList {
            evaluations: definitions,
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

    for metric in wl::METRIC_DEFS {
        builder.push_record([
            wl::EVAL_KEY,
            metric.name,
            metric.direction.as_str(),
            metric.description,
        ]);
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

/// Run `scurve evals wl`.
pub fn handle_wl(
    options: &EvalsCommonOptions,
    size: u32,
    dimension: u32,
    segments: Option<&String>,
    mode: crate::WlScanModeArg,
    windows_per_len: u32,
) -> Result<()> {
    let curve_list = parse_csv_list(&options.curves, "curve")?;
    let metric_list = parse_csv_list(&options.metrics, "metric")?;

    let selected_metrics = resolve_metric_defs(wl::METRIC_DEFS, metric_list.as_deref())?;
    let _metric_selection = WlMetricSelection::from_defs(&selected_metrics);

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

    let segment_lengths = match parse_csv_u32_list(segments, "segment length")? {
        Some(values) => values,
        None => default_segment_lengths(curves[0].spec.length())?,
    };

    let scan_mode = match mode {
        crate::WlScanModeArg::Exact => wl::ScanMode::Exact,
        crate::WlScanModeArg::Sample => wl::ScanMode::Sample {
            windows_per_len: windows_per_len as usize,
            seed: options.seed,
        },
    };

    let mut results = Vec::new();
    let mut segment_lengths_used = None;
    for selection in curves {
        let curve = (selection.entry.ctor)(&selection.spec)
            .with_context(|| format!("failed to construct curve '{}'", selection.entry.key))?;
        let profile = wl::wl_profile(&*curve, &segment_lengths, scan_mode)
            .with_context(|| format!("evaluation failed for '{}'", selection.entry.key))?;
        if profile.rows.is_empty() {
            bail!(
                "no valid segment lengths for curve '{}' (length {})",
                selection.entry.key,
                selection.spec.length()
            );
        }
        if segment_lengths_used.is_none() {
            segment_lengths_used = Some(
                profile
                    .rows
                    .iter()
                    .map(|row| row.segment_len)
                    .collect::<Vec<_>>(),
            );
        }
        results.push(WlEvalResult {
            curve: selection.entry.key,
            profile,
        });
    }

    let segment_lengths_used = segment_lengths_used.unwrap_or_else(|| segment_lengths.clone());

    if options.json {
        let output = JsonWlOutput::new(
            WlOutputParams {
                size,
                dimension,
                seed: options.seed,
                mode: scan_mode,
                segment_lengths: segment_lengths_used,
            },
            &selected_metrics,
            results,
            if used_default { skipped } else { Vec::new() },
        );
        println!("{}", serde_json::to_string_pretty(&output)?);
        return Ok(());
    }

    render_wl_table(size, dimension, scan_mode, &selected_metrics, &results)?;
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

/// Parse a comma-separated list of u32 values, rejecting duplicates and empty
/// values.
fn parse_csv_u32_list(input: Option<&String>, label: &str) -> Result<Option<Vec<u32>>> {
    let Some(raw) = input else {
        return Ok(None);
    };

    let mut items = Vec::new();
    for item in raw.split(',') {
        let trimmed = item.trim();
        if trimmed.is_empty() {
            continue;
        }
        let value: u32 = trimmed
            .parse()
            .with_context(|| format!("invalid {label} '{trimmed}'"))?;
        items.push(value);
    }

    if items.is_empty() {
        bail!("{label} list is empty");
    }

    let mut seen = BTreeSet::new();
    let mut unique = Vec::new();
    for value in items {
        if !seen.insert(value) {
            bail!("duplicate {label} '{value}'");
        }
        unique.push(value);
    }

    Ok(Some(unique))
}

/// Compute default segment lengths as powers of two up to the curve length.
fn default_segment_lengths(length: Index) -> Result<Vec<u32>> {
    let length_u128 = u128::from(length);
    if length_u128 == 0 {
        return Ok(Vec::new());
    }

    let mut lengths = Vec::new();
    let mut value: u32 = 1;
    let max_len = length_u128.min(u128::from(u32::MAX)) as u32;
    while value <= max_len {
        lengths.push(value);
        match value.checked_mul(2) {
            Some(next) => value = next,
            None => break,
        }
    }
    if let Some(last) = lengths.last().copied()
        && last != max_len
    {
        lengths.push(max_len);
    }

    Ok(lengths)
}

#[derive(Clone, Copy, Debug)]
/// Selection flags for WL metric columns.
struct WlMetricSelection {
    /// Include WL∞ max values (and argmax column).
    wl_inf_max: bool,
    /// Include WL∞ mean values.
    wl_inf_mean: bool,
    /// Include WL∞ p95 values.
    wl_inf_p95: bool,
    /// Include WL2 max values (and argmax column).
    wl2_max: bool,
    /// Include WL2 mean values.
    wl2_mean: bool,
    /// Include WL2 p95 values.
    wl2_p95: bool,
}

impl WlMetricSelection {
    /// Build a selection from a metric definition list.
    fn from_defs(defs: &[&MetricDef]) -> Self {
        let mut selection = Self {
            wl_inf_max: false,
            wl_inf_mean: false,
            wl_inf_p95: false,
            wl2_max: false,
            wl2_mean: false,
            wl2_p95: false,
        };
        for def in defs {
            match def.name {
                "wl-inf-max" => selection.wl_inf_max = true,
                "wl-inf-mean" => selection.wl_inf_mean = true,
                "wl-inf-p95" => selection.wl_inf_p95 = true,
                "wl2-max" => selection.wl2_max = true,
                "wl2-mean" => selection.wl2_mean = true,
                "wl2-p95" => selection.wl2_p95 = true,
                _ => {}
            }
        }
        selection
    }
}

#[derive(Clone, Debug)]
/// WL profile output for a single curve.
struct WlEvalResult {
    /// Curve key.
    curve: &'static str,
    /// WL profile data.
    profile: wl::WlProfile<Index>,
}

#[derive(Clone, Debug)]
/// Parameters used to shape WL output payloads.
struct WlOutputParams {
    /// Grid side length.
    size: u32,
    /// Number of dimensions.
    dimension: u32,
    /// RNG seed for sampling.
    seed: u64,
    /// Scan mode.
    mode: wl::ScanMode,
    /// Segment lengths in the profile.
    segment_lengths: Vec<u32>,
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

/// Render WL profile results as a table.
fn render_wl_table(
    size: u32,
    dimension: u32,
    mode: wl::ScanMode,
    metrics: &[&MetricDef],
    results: &[WlEvalResult],
) -> Result<()> {
    let mode_label = match mode {
        wl::ScanMode::Exact => "exact".to_string(),
        wl::ScanMode::Sample {
            windows_per_len, ..
        } => {
            format!("sample (windows_per_len={windows_per_len})")
        }
    };

    let segment_lengths = wl_segment_lengths(results);
    for (idx, segment_len) in segment_lengths.iter().enumerate() {
        println!(
            "{} (size={}, dim={}, mode={}, L={})",
            wl::EVAL_TITLE,
            size,
            dimension,
            mode_label,
            segment_len
        );

        let mut builder = Builder::default();
        let mut header = Vec::new();
        header.push("Curve".to_string());
        for def in metrics {
            header.push(wl_metric_label(def.name));
        }
        builder.push_record(header);

        let mut rows = Vec::new();
        for result in results {
            if let Some(row) = result
                .profile
                .rows
                .iter()
                .find(|row| row.segment_len == *segment_len)
            {
                let mut record = Vec::new();
                let mut row_metrics = Vec::with_capacity(metrics.len());
                record.push(result.curve.to_string());
                for def in metrics {
                    let value = wl_metric_value(row, def.name);
                    record.push(format_optional_metric(value));
                    row_metrics.push(value.unwrap_or(f64::NAN));
                }
                builder.push_record(record);
                rows.push(RowValues {
                    metrics: row_metrics,
                });
            }
        }

        let mut table = builder.build();
        table.with(Style::modern());
        if io::stdout().is_terminal() {
            highlight_best_values(&mut table, metrics, &rows);
        }
        println!("{table}");
        if idx + 1 < segment_lengths.len() {
            println!();
        }
    }
    Ok(())
}

/// Collect sorted segment lengths across WL results.
fn wl_segment_lengths(results: &[WlEvalResult]) -> Vec<u32> {
    let mut lengths = BTreeSet::new();
    for result in results {
        for row in &result.profile.rows {
            lengths.insert(row.segment_len);
        }
    }
    lengths.into_iter().collect()
}

/// Display label for WL metric columns.
fn wl_metric_label(name: &str) -> String {
    match name {
        "wl-inf-max" => "WL∞ Max".to_string(),
        "wl-inf-mean" => "WL∞ Mean".to_string(),
        "wl-inf-p95" => "WL∞ P95".to_string(),
        "wl2-max" => "WL2 Max".to_string(),
        "wl2-mean" => "WL2 Mean".to_string(),
        "wl2-p95" => "WL2 P95".to_string(),
        _ => name.to_string(),
    }
}

/// Extract a WL metric value from a profile row.
fn wl_metric_value(row: &wl::WlRow<Index>, name: &str) -> Option<f64> {
    match name {
        "wl-inf-max" => Some(row.wl_inf_max),
        "wl-inf-mean" => row.wl_inf_mean,
        "wl-inf-p95" => row.wl_inf_p95,
        "wl2-max" => Some(row.wl2_max),
        "wl2-mean" => row.wl2_mean,
        "wl2-p95" => row.wl2_p95,
        _ => None,
    }
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

/// Format optional metric values for table display.
fn format_optional_metric(value: Option<f64>) -> String {
    value.map(format_metric).unwrap_or_else(|| "-".to_string())
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

#[derive(Serialize)]
/// JSON payload for WL evaluation parameters.
struct JsonWlParams {
    /// Grid side length.
    size: Coord,
    /// Number of dimensions.
    dimension: u32,
    /// Segment lengths included in the profile.
    segment_lengths: Vec<u32>,
    /// Scan mode label.
    scan_mode: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Windows sampled per segment length (sample mode only).
    windows_per_len: Option<usize>,
    /// RNG seed used for sampling.
    seed: u64,
}

#[derive(Serialize)]
/// JSON payload for a single WL profile row.
struct JsonWlRow {
    /// Segment length for this row.
    segment_len: u32,
    /// Metric values keyed by name.
    metrics: BTreeMap<String, f64>,
}

#[derive(Serialize)]
/// JSON payload for a single curve's WL profile.
struct JsonWlResult {
    /// Curve key.
    curve: &'static str,
    /// WL profile rows.
    rows: Vec<JsonWlRow>,
}

#[derive(Serialize)]
/// JSON payload for WL evaluation output.
struct JsonWlOutput {
    /// Evaluation key.
    evaluation: &'static str,
    /// Evaluation title.
    title: &'static str,
    /// Evaluation parameters.
    parameters: JsonWlParams,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Quantile method label for percentile-producing evaluations.
    quantile_method: Option<&'static str>,
    /// Metric definitions included in the output.
    metric_definitions: Vec<JsonMetricDef>,
    /// Evaluation results.
    results: Vec<JsonWlResult>,
    /// Curves skipped during default selection.
    skipped: Vec<SkippedCurve>,
}

impl JsonWlOutput {
    /// Build a JSON payload from WL profile results.
    fn new(
        params: WlOutputParams,
        metrics: &[&MetricDef],
        results: Vec<WlEvalResult>,
        skipped: Vec<SkippedCurve>,
    ) -> Self {
        let (scan_mode, windows_per_len) = match params.mode {
            wl::ScanMode::Exact => ("exact", None),
            wl::ScanMode::Sample {
                windows_per_len, ..
            } => ("sample", Some(windows_per_len)),
        };
        let parameters = JsonWlParams {
            size: params.size,
            dimension: params.dimension,
            segment_lengths: params.segment_lengths,
            scan_mode,
            windows_per_len,
            seed: params.seed,
        };

        let metric_definitions = metrics
            .iter()
            .map(|def| JsonMetricDef::from(*def))
            .collect::<Vec<_>>();

        let metric_selection = WlMetricSelection::from_defs(metrics);
        let quantile_method = if metric_selection.wl_inf_p95 || metric_selection.wl2_p95 {
            Some(QUANTILE_METHOD_R7)
        } else {
            None
        };

        let results = results
            .into_iter()
            .map(|result| JsonWlResult {
                curve: result.curve,
                rows: result
                    .profile
                    .rows
                    .into_iter()
                    .map(|row| JsonWlRow {
                        segment_len: row.segment_len,
                        metrics: wl_metrics_map(&row, metrics),
                    })
                    .collect(),
            })
            .collect();

        Self {
            evaluation: wl::EVAL_KEY,
            title: wl::EVAL_TITLE,
            parameters,
            quantile_method,
            metric_definitions,
            results,
            skipped,
        }
    }
}

/// Build a metrics map for a WL row using the selected definitions.
fn wl_metrics_map(row: &wl::WlRow<Index>, metrics: &[&MetricDef]) -> BTreeMap<String, f64> {
    let mut map = BTreeMap::new();
    for def in metrics {
        let value = match def.name {
            "wl-inf-max" => Some(row.wl_inf_max),
            "wl-inf-mean" => row.wl_inf_mean,
            "wl-inf-p95" => row.wl_inf_p95,
            "wl2-max" => Some(row.wl2_max),
            "wl2-mean" => row.wl2_mean,
            "wl2-p95" => row.wl2_p95,
            _ => None,
        };
        map.insert(def.name.to_string(), value.unwrap_or(f64::NAN));
    }
    map
}
