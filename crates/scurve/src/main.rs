//! Command‑line entry point for the `scurve` tool.
//!
//! Provides subcommands to render curve maps, visualize files, and launch the
//! GUI.

use std::{
    fmt::Display,
    ops::Range,
    path::{Path, PathBuf},
    process,
    str::FromStr,
};

use anyhow::Result;
use clap::{Parser, Subcommand};
use colornames::Color;
use image::{Rgba, RgbaImage};
use spacecurve::{CURVE_NAMES, registry};

/// CLI command implementations.
mod cmd;
/// Rendering helpers shared by the CLI.
mod map;

use crate::map::MapPalette;

/// Half-open range of curve offsets parsed from `--chunk`.
#[derive(Clone, Copy, Debug)]
struct ChunkOffsets {
    /// Inclusive start offset for rendering.
    start: u32,
    /// Exclusive end offset for rendering.
    end: u32,
}

impl ChunkOffsets {
    /// Convert the offsets into a standard half-open range.
    fn into_range(self) -> Range<u32> {
        self.start..self.end
    }
}

impl FromStr for ChunkOffsets {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let (start, end) = value
            .split_once(':')
            .ok_or_else(|| "chunk must be in START:END form".to_string())?;

        let parse_bound = |label: &str, bound: &str| -> Result<u32, String> {
            bound.trim().parse::<u32>().map_err(|_| {
                format!("invalid {label} offset '{bound}': expected a non-negative integer")
            })
        };

        let start = parse_bound("start", start)?;
        let end = parse_bound("end", end)?;

        if start >= end {
            return Err(format!(
                "chunk start ({start}) must be less than end ({end})"
            ));
        }

        Ok(Self { start, end })
    }
}

/// Validate a curve name against the known set.
fn parse_curve_name(s: &str) -> Result<String, String> {
    if CURVE_NAMES.contains(&s) {
        Ok(s.to_string())
    } else {
        Err(format!(
            "Invalid curve name '{}'. Valid options: {}",
            s,
            CURVE_NAMES.join(", ")
        ))
    }
}

/// Parse a named or hex color into an `Rgba` value (alpha defaults to 0xff).
///
/// Supports CSS color names via `colornames`, short/long hex (RGB/RRGGBB),
/// and optional alpha (RGBA/RRGGBBAA) with or without a leading `#`.
fn parse_rgba_color(input: &str) -> Result<Rgba<u8>, String> {
    fn parse_hex_rgba(hex: &str) -> Option<Rgba<u8>> {
        use std::ops::Range;

        let raw = hex.trim_start_matches('#');
        if !raw.as_bytes().iter().all(|b| b.is_ascii_hexdigit()) {
            return None;
        }

        let from_pair =
            |range: Range<usize>| -> Option<u8> { u8::from_str_radix(&raw[range], 16).ok() };
        let from_nibble = |idx: usize| -> Option<u8> {
            u8::from_str_radix(&raw[idx..idx + 1], 16)
                .ok()
                .map(|v| v * 17)
        };

        match raw.len() {
            3 => Some(Rgba([
                from_nibble(0)?,
                from_nibble(1)?,
                from_nibble(2)?,
                0xff,
            ])),
            4 => Some(Rgba([
                from_nibble(0)?,
                from_nibble(1)?,
                from_nibble(2)?,
                from_nibble(3)?,
            ])),
            6 => Some(Rgba([
                from_pair(0..2)?,
                from_pair(2..4)?,
                from_pair(4..6)?,
                0xff,
            ])),
            8 => Some(Rgba([
                from_pair(0..2)?,
                from_pair(2..4)?,
                from_pair(4..6)?,
                from_pair(6..8)?,
            ])),
            _ => None,
        }
    }

    let trimmed = input.trim();
    if let Some(rgba) = parse_hex_rgba(trimmed) {
        return Ok(rgba);
    }

    let color: Color = trimmed.try_into().map_err(|_| {
        format!(
            "invalid color '{input}': use a named color or hex (RGB/RRGGBB with optional alpha, leading '#' optional)"
        )
    })?;
    let (red, green, blue) = color.rgb();
    Ok(Rgba([red, green, blue, 0xff]))
}

#[derive(Parser)]
#[command(name = "scurve")]
#[command(version = env!("CARGO_PKG_VERSION"))]
/// Top‑level CLI options and subcommands.
struct Cli {
    /// Sets the level of verbosity (`-v`, `-vv`, ...).
    #[arg(short, action = clap::ArgAction::Count, help = "Sets the level of verbosity")]
    v: u8,

    /// Command to execute.
    #[command(subcommand)]
    command: Commands,
}

/// Screenshot target for the GUI.
#[derive(Clone, Copy, Debug, clap::ValueEnum)]
enum ScreenshotPane {
    /// The 2D curve visualization pane.
    #[value(name = "2d")]
    TwoD,
    /// The 3D curve visualization pane.
    #[value(name = "3d")]
    ThreeD,
    /// The About dialog.
    About,
    /// The settings dropdown.
    Settings,
    /// The settings dropdown in the 3D pane (includes rotation speed).
    #[value(name = "settings-3d")]
    Settings3D,
}

#[derive(Subcommand)]
/// Subcommands supported by the `scurve` tool.
enum Commands {
    #[command(about = "Generate a map of a pattern")]
    /// Generate a map of a pattern.
    Map {
        #[arg(short = 's', long = "size", help = "Square image size in pixels")]
        /// Image size in pixels (square output).
        size: Option<u32>,

        #[arg(
            short = 'd',
            long = "dimension",
            value_name = "SIDE",
            help = "Logical curve dimension (renders a SIDE×SIDE grid)"
        )]
        /// Side length of the curve grid (SIDE×SIDE points).
        curve_dimension: Option<u32>,

        #[arg(
            short = 'w',
            long = "line-width",
            value_name = "PIXELS",
            default_value_t = 1,
            value_parser = clap::value_parser!(u32).range(1..),
            help = "Line width in pixels for the curve stroke"
        )]
        /// Stroke width for the rendered curve.
        line_width: u32,

        #[arg(
            long = "fg",
            visible_alias = "foreground",
            value_parser = parse_rgba_color,
            default_value = "#8080ff",
            value_name = "HEX",
            help = "Foreground color (name or hex; RGB/RRGGBB with optional alpha, '#' optional)"
        )]
        /// Stroke color for the curve.
        foreground: Rgba<u8>,

        #[arg(
            long = "bg",
            visible_alias = "background",
            value_parser = parse_rgba_color,
            default_value = "#ffffff",
            value_name = "HEX",
            help = "Background color (name or hex; RGB/RRGGBB with optional alpha, '#' optional)"
        )]
        /// Background color for the map.
        background: Rgba<u8>,

        #[arg(
            long = "long",
            default_value_t = false,
            help = "Include long edges (segments longer than 1 unit)"
        )]
        /// Render long edges between non-adjacent points.
        long_edges: bool,

        #[arg(
            long = "chunk",
            value_name = "START:END",
            help = "Draw only the curve segment from START (inclusive) to END (exclusive)"
        )]
        /// Optional start/end offsets (START:END) for the rendered curve segment.
        chunk: Option<ChunkOffsets>,

        #[arg(help = &format!("Pattern name (options: {})", CURVE_NAMES.join(", ")), value_parser = parse_curve_name)]
        /// Pattern name.
        pattern: String,

        #[arg(help = "Optional output file path; opens a viewer when omitted")]
        /// Optional output file path (launches a viewer when not provided).
        output: Option<PathBuf>,
    },

    #[command(about = "Generate an animated snake GIF for a pattern")]
    /// Generate an animated snake GIF showing a moving curve segment.
    Snake {
        #[arg(short = 's', long = "size", help = "Square image size in pixels")]
        /// Image size in pixels (square output).
        size: Option<u32>,

        #[arg(
            short = 'd',
            long = "dimension",
            value_name = "SIDE",
            help = "Logical curve dimension (renders a SIDE×SIDE grid)"
        )]
        /// Side length of the curve grid (SIDE×SIDE points).
        curve_dimension: Option<u32>,

        #[arg(
            short = 'w',
            long = "line-width",
            value_name = "PIXELS",
            default_value_t = 1,
            value_parser = clap::value_parser!(u32).range(1..),
            help = "Line width in pixels for the curve stroke"
        )]
        /// Stroke width for the rendered curve.
        line_width: u32,

        #[arg(
            long = "fg",
            visible_alias = "foreground",
            value_parser = parse_rgba_color,
            default_value = "#8080ff",
            value_name = "HEX",
            help = "Foreground color (name or hex; RGB/RRGGBB with optional alpha, '#' optional)"
        )]
        /// Stroke color for the curve.
        foreground: Rgba<u8>,

        #[arg(
            long = "bg",
            visible_alias = "background",
            value_parser = parse_rgba_color,
            default_value = "#ffffff",
            value_name = "HEX",
            help = "Background color (name or hex; RGB/RRGGBB with optional alpha, '#' optional)"
        )]
        /// Background color for the map.
        background: Rgba<u8>,

        #[arg(
            long = "full",
            value_name = "COLOR",
            value_parser = parse_rgba_color,
            help = "Draw the full curve in COLOR beneath the animated snake"
        )]
        /// Optional full-curve color to render behind the snake overlay.
        full: Option<Rgba<u8>>,

        #[arg(
            long = "long",
            default_value_t = false,
            help = "Include long edges (segments longer than 1 unit)"
        )]
        /// Render long edges between non-adjacent points.
        long_edges: bool,

        #[arg(
            long = "chunk",
            value_name = "START:END",
            required = true,
            help = "Chunk to animate (START inclusive, END exclusive)"
        )]
        /// Mandatory start/end offsets (START:END) for the animated segment.
        chunk: ChunkOffsets,

        #[arg(
            long = "fps",
            default_value_t = 20,
            value_parser = clap::value_parser!(u16).range(1..=120),
            help = "Frames per second for the animated GIF"
        )]
        /// Frames per second for the animation (1-120).
        fps: u16,

        #[arg(help = &format!("Pattern name (options: {})", CURVE_NAMES.join(", ")), value_parser = parse_curve_name)]
        /// Pattern name.
        pattern: String,

        #[arg(help = "Output GIF file path")]
        /// Output GIF path (required).
        output: PathBuf,
    },

    #[command(
        about = "Generate a dense map of a pattern that contains one pixel for each RGB colour"
    )]
    /// Generate a dense map that contains one pixel for each RGB colour.
    Allrgb {
        #[arg(short = 'c', help = &format!("Pattern name for color map (options: {})", CURVE_NAMES.join(", ")), value_parser = parse_curve_name)]
        /// Optional pattern name for the color map (defaults to `pattern`).
        colormap: Option<String>,

        #[arg(help = &format!("Pattern name (options: {})", CURVE_NAMES.join(", ")), value_parser = parse_curve_name)]
        /// Pattern name for pixel layout.
        pattern: String,

        #[arg(help = "Optional output file path; opens a viewer when omitted")]
        /// Optional output file path (launches a viewer when not provided).
        output: Option<PathBuf>,
    },

    #[command(about = "visualise a file")]
    /// Visualise a file using a space‑filling curve.
    Vis {
        #[arg(short = 'p', help = &format!("Pattern name (options: {})", CURVE_NAMES.join(", ")), value_parser = parse_curve_name)]
        /// Optional pattern name (defaults to `hilbert`).
        pattern: Option<String>,

        #[arg(short = 'w', help = "Image width")]
        /// Output image width/height in pixels.
        width: Option<u32>,

        #[arg(help = "File to visualise")]
        /// Input file to visualise.
        input: PathBuf,

        #[arg(help = "Optional output file path; opens a viewer when omitted")]
        /// Optional output file path (launches a viewer when not provided).
        output: Option<PathBuf>,
    },

    #[command(about = "Open GUI window")]
    /// Launch the interactive GUI.
    Gui {
        #[arg(
            long = "dev",
            help = "Show experimental curves (e.g. Hairy Onion) in the GUI"
        )]
        /// Enable experimental curves in the GUI selectors.
        dev: bool,
    },

    #[command(about = "Take a screenshot of the GUI (requires --features screenshot)")]
    /// Capture a screenshot of a specific GUI pane.
    Screenshot {
        #[arg(
            short = 'p',
            long = "pane",
            value_enum,
            default_value = "2d",
            help = "Which pane to screenshot"
        )]
        /// Which pane to capture.
        pane: ScreenshotPane,

        #[arg(help = "Output PNG file path")]
        /// Output file path for the screenshot.
        output: PathBuf,
    },

    #[command(
        name = "list-curves",
        about = "List supported curve names and constraints"
    )]
    /// List supported curves and their constraints.
    ListCurves,
}

/// Print a success message or exit with an error.
fn report_ok<E: Display>(result: Result<(), E>, ok_msg: &str) {
    match result {
        Ok(()) => println!("{ok_msg}"),
        Err(e) => {
            eprintln!("{e}");
            process::exit(1);
        }
    }
}

/// Save an image to disk or show it in an egui viewer when no path is given.
fn deliver_image(image: RgbaImage, output: Option<&Path>, window_title: &str) -> Result<()> {
    if let Some(path) = output {
        image.save(path)?;
    } else {
        println!("No output file provided; opening viewer (close the window to finish)...");
        egui_img::view_image(window_title, image)?;
    }

    Ok(())
}

/// Handle the `vis` subcommand.
fn handle_vis(
    input: &Path,
    output: Option<&Path>,
    width: Option<u32>,
    pattern: Option<&str>,
) -> Result<()> {
    let width = width.unwrap_or(256);
    let pattern_name = pattern.unwrap_or("hilbert");
    let image = cmd::vis(input, width, pattern_name)?;
    deliver_image(image, output, &format!("vis: {pattern_name}"))
}

/// Handle the `map` subcommand.
fn handle_map(
    size: Option<u32>,
    curve_dimension: Option<u32>,
    pattern: &str,
    output: Option<&Path>,
    chunk: Option<ChunkOffsets>,
    stroke: map::StrokeOptions,
) -> Result<()> {
    let size = size.unwrap_or(512);
    // Default keeps behaviour similar to the previous 16×16 grid (256 points).
    let requested_dimension = curve_dimension.unwrap_or(16);
    let render = cmd::map(
        size,
        requested_dimension,
        pattern,
        chunk.map(ChunkOffsets::into_range),
        stroke,
    )?;
    if render.adjusted {
        eprintln!(
            "Requested curve dimension {} is not valid for pattern '{}'; using {} instead.",
            requested_dimension, pattern, render.side
        );
    }
    deliver_image(render.image, output, &format!("map: {pattern}"))
}

/// Parameters supplied by the CLI for the `snake` subcommand.
#[derive(Clone, Copy)]
struct SnakeInput<'a> {
    /// Requested output size in pixels (defaults to 512 when `None`).
    size: Option<u32>,
    /// Requested curve dimension (defaults to 16 when `None`).
    curve_dimension: Option<u32>,
    /// Curve pattern name.
    pattern: &'a str,
    /// Offset range for the animated segment.
    chunk: ChunkOffsets,
    /// Destination GIF path.
    output: &'a Path,
    /// Frames per second.
    fps: u16,
    /// Stroke styling options.
    stroke: map::StrokeOptions,
    /// Optional colour for the static full-curve layer.
    full_curve: Option<Rgba<u8>>,
}

/// Handle the `snake` subcommand.
fn handle_snake(input: SnakeInput<'_>) -> Result<()> {
    let SnakeInput {
        size,
        curve_dimension,
        pattern,
        chunk,
        output,
        fps,
        stroke,
        full_curve,
    } = input;

    let size = size.unwrap_or(512);
    let requested_dimension = curve_dimension.unwrap_or(16);
    let render = cmd::snake(cmd::SnakeOptions {
        size,
        curve_dimension: requested_dimension,
        pattern_name: pattern,
        chunk: chunk.into_range(),
        fps,
        stroke,
        output,
        full_curve,
    })?;

    if render.adjusted {
        eprintln!(
            "Requested curve dimension {} is not valid for pattern '{}'; using {} instead.",
            requested_dimension, pattern, render.side
        );
    }
    Ok(())
}

/// Handle the `allrgb` subcommand.
fn handle_allrgb(pattern: &str, colormap: Option<&str>, output: Option<&Path>) -> Result<()> {
    let colormap = colormap.unwrap_or(pattern);
    let image = cmd::allrgb(pattern, colormap)?;
    deliver_image(image, output, &format!("allrgb: {pattern}/{colormap}"))
}

/// Handle the `gui` subcommand.
fn handle_gui(dev: bool) {
    report_ok(
        scurve_gui::gui_with_options(scurve_gui::GuiOptions {
            include_experimental_curves: dev,
            show_dev_overlay: dev,
            ..scurve_gui::GuiOptions::default()
        }),
        "OK!",
    );
}

#[cfg(feature = "screenshot")]
/// Handle the `screenshot` subcommand when the feature is enabled.
fn handle_screenshot(pane: ScreenshotPane, output: PathBuf) {
    use scurve_gui::{ScreenshotConfig, ScreenshotTarget};

    let target = match pane {
        ScreenshotPane::TwoD => ScreenshotTarget::TwoD,
        ScreenshotPane::ThreeD => ScreenshotTarget::ThreeD,
        ScreenshotPane::About => ScreenshotTarget::About,
        ScreenshotPane::Settings => ScreenshotTarget::Settings,
        ScreenshotPane::Settings3D => ScreenshotTarget::Settings3D,
    };

    let config = ScreenshotConfig {
        target,
        output_path: output,
    };

    report_ok(
        scurve_gui::gui_with_screenshot(Some(config)),
        "Screenshot saved!",
    );
}

#[cfg(not(feature = "screenshot"))]
/// Handle the `screenshot` subcommand when the feature is disabled.
fn handle_screenshot(_pane: ScreenshotPane, _output: PathBuf) {
    eprintln!("Screenshot feature not enabled. Rebuild with: cargo build --features screenshot",);
    process::exit(1);
}

/// Handle the `list-curves` subcommand.
fn handle_list_curves() {
    println!("Supported curves (key — display — constraints):");
    for entry in registry::REGISTRY {
        println!(
            "- {} — {} — {}",
            entry.key, entry.display, entry.constraints
        );
    }
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Vis {
            input,
            output,
            width,
            pattern,
        } => report_ok(
            handle_vis(&input, output.as_deref(), width, pattern.as_deref()),
            "OK!",
        ),
        Commands::Map {
            pattern,
            size,
            curve_dimension,
            line_width,
            output,
            foreground,
            background,
            chunk,
            long_edges,
        } => report_ok(
            handle_map(
                size,
                curve_dimension,
                &pattern,
                output.as_deref(),
                chunk,
                map::StrokeOptions {
                    line_width,
                    long_edges,
                    palette: MapPalette {
                        foreground,
                        background,
                    },
                },
            ),
            "OK!",
        ),
        Commands::Allrgb {
            pattern,
            colormap,
            output,
        } => report_ok(
            handle_allrgb(&pattern, colormap.as_deref(), output.as_deref()),
            "OK!",
        ),
        Commands::Snake {
            pattern,
            size,
            curve_dimension,
            line_width,
            output,
            foreground,
            background,
            chunk,
            fps,
            long_edges,
            full,
        } => report_ok(
            handle_snake(SnakeInput {
                size,
                curve_dimension,
                pattern: &pattern,
                chunk,
                output: &output,
                fps,
                stroke: map::StrokeOptions {
                    line_width,
                    long_edges,
                    palette: MapPalette {
                        foreground,
                        background,
                    },
                },
                full_curve: full,
            }),
            "Saved snake GIF!",
        ),
        Commands::Gui { dev } => handle_gui(dev),
        Commands::Screenshot { pane, output } => handle_screenshot(pane, output),
        Commands::ListCurves => handle_list_curves(),
    }
}

#[cfg(test)]
mod tests {
    use super::ChunkOffsets;

    #[test]
    fn parses_chunk_offsets() {
        let chunk: ChunkOffsets = "1:5".parse().unwrap();
        assert_eq!(chunk.into_range(), 1..5);
    }

    #[test]
    fn rejects_invalid_chunks() {
        assert!("5:1".parse::<ChunkOffsets>().is_err());
        assert!("abc".parse::<ChunkOffsets>().is_err());
        assert!("1:".parse::<ChunkOffsets>().is_err());
    }
}
