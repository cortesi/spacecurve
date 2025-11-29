//! Command handlers for the `scurve` CLI.
//!
//! These functions implement the top‑level subcommands and write the resulting
//! images to disk.

use std::{fs::File, ops::Range, path::Path};

use anyhow::{Result, anyhow, bail};
use gif::{Encoder, Frame, Repeat};
use spacecurve::{pattern_from_name, registry};

use crate::map::{
    MapPalette, StrokeOptions, draw_chunk_overlay, render_chunk_image, render_map_image,
};

/// Black color for 0x00.
const COLOR_BLACK: image::Rgba<u8> = image::Rgba([0, 0, 0, 0xff]);
/// White color for 0xFF.
const COLOR_WHITE: image::Rgba<u8> = image::Rgba([0xff, 0xff, 0xff, 0xff]);
/// Green color for control characters (low ASCII).
const COLOR_GREEN: image::Rgba<u8> = image::Rgba([0x4d, 0xaf, 0x4a, 0xff]);
/// Blue color for printable characters.
const COLOR_BLUE: image::Rgba<u8> = image::Rgba([0x10, 0x72, 0xb8, 0xff]);
/// Red color for extended/other characters.
const COLOR_RED: image::Rgba<u8> = image::Rgba([0xe4, 0x1a, 0x1c, 0xff]);

/// Map a byte value to a representative RGBA color used by `vis`.
fn byte_to_color(byte: u8) -> image::Rgba<u8> {
    match byte {
        0x00 => COLOR_BLACK,
        0xff => COLOR_WHITE,
        // Low ASCII control chars approx range
        b if b < 31 => COLOR_GREEN,
        // Printable ASCII approx range
        b if (32..127).contains(&b) => COLOR_BLUE,
        // Extended ASCII / unprintable
        _ => COLOR_RED,
    }
}

/// Map a file into memory for read‑only access.
///
/// Safety rationale: the mapping is read‑only and the `File` is not mutated
/// for the lifetime of the returned map.
fn mmap_readonly(file: &File) -> Result<memmap2::Mmap> {
    // SAFETY: We create a read‑only mapping and only access it immutably.
    let map = unsafe { memmap2::MmapOptions::new().map(file)? };
    Ok(map)
}

/// Visualize a file by mapping each byte through a space‑filling curve.
///
/// The returned image is square with the requested `width`.
pub fn vis(input: &Path, width: u32, pattern_name: &str) -> Result<image::RgbaImage> {
    let file = File::open(input)?;
    let mmap = mmap_readonly(&file)?;

    if mmap.is_empty() {
        bail!("input file is empty");
    }

    let pattern = pattern_from_name(pattern_name, 2, width)?;

    let mut imgbuf = image::ImageBuffer::new(width, width);

    let plen = pattern.length() as u128;
    let mlen = mmap.len() as u128;
    for i in 0..pattern.length() {
        let p = pattern.point(i);
        // Integer scaling avoids float rounding that could produce idx == mlen.
        let idx = ((i as u128) * mlen / plen) as usize;
        let byte = mmap[idx.min(mmap.len() - 1)];
        imgbuf.put_pixel(p[0], p[1], byte_to_color(byte));
    }
    Ok(imgbuf)
}

/// Result of rendering a map image.
pub struct MapRender {
    /// The rendered image buffer.
    pub image: image::RgbaImage,
    /// Actual curve dimension (side length) used for the grid.
    pub side: u32,
    /// Whether the requested dimension had to be adjusted upward to satisfy curve constraints.
    pub adjusted: bool,
}

/// Result of rendering a snake animation.
pub struct SnakeRender {
    /// Actual curve dimension (side length) used for the grid.
    pub side: u32,
    /// Whether the requested dimension had to be adjusted upward to satisfy curve constraints.
    pub adjusted: bool,
}

/// Parameters controlling snake animation rendering.
pub struct SnakeOptions<'a> {
    /// Output image size in pixels.
    pub size: u32,
    /// Requested logical curve dimension (side length).
    pub curve_dimension: u32,
    /// Pattern name for the curve.
    pub pattern_name: &'a str,
    /// Segment range to animate.
    pub chunk: Range<u32>,
    /// Frames per second for the GIF.
    pub fps: u16,
    /// Stroke styling used for the snake overlay.
    pub stroke: StrokeOptions,
    /// Output GIF path.
    pub output: &'a Path,
    /// Optional color for rendering the full curve beneath the snake overlay.
    pub full_curve: Option<image::Rgba<u8>>,
}

/// Find the smallest curve dimension ≥ `requested_side` that satisfies the pattern constraints.
fn resolve_curve_dimension(pattern_name: &str, requested_side: u32) -> Result<(u32, bool)> {
    const DIMENSION: u32 = 2;

    if requested_side == 0 {
        bail!("curve dimension must be >= 1");
    }

    let initial_validation = registry::validate(pattern_name, DIMENSION, requested_side);
    if initial_validation.is_ok() {
        return Ok((requested_side, false));
    }

    let mut last_err = initial_validation.unwrap_err();

    let mut candidate = requested_side
        .checked_next_power_of_two()
        .and_then(|p| {
            if p > requested_side {
                Some(p)
            } else {
                p.checked_mul(2)
            }
        })
        .ok_or_else(|| {
            anyhow!(
                "could not find a valid curve dimension >= {} for '{}': {}",
                requested_side,
                pattern_name,
                last_err
            )
        })?;

    while candidate > requested_side {
        match registry::validate(pattern_name, DIMENSION, candidate) {
            Ok(()) => return Ok((candidate, true)),
            Err(err) => {
                last_err = err;
                candidate = match candidate.checked_mul(2) {
                    Some(next) if next > candidate => next,
                    _ => break,
                };
            }
        }
    }

    Err(anyhow!(
        "could not find a valid curve dimension >= {} for '{}': {}",
        requested_side,
        pattern_name,
        last_err
    ))
}

/// Render a map of a curve using a requested grid dimension.
///
/// - `size`: Output image width/height in pixels.
/// - `curve_dimension`: Requested side length for the curve grid (renders `dimension×dimension` points).
/// - `pattern_name`: Curve name.
/// - `chunk`: Optional [start, end) offsets limiting which part of the curve is drawn.
/// - `stroke`: Stroke rendering options.
pub fn map(
    size: u32,
    curve_dimension: u32,
    pattern_name: &str,
    chunk: Option<Range<u32>>,
    stroke: StrokeOptions,
) -> Result<MapRender> {
    if stroke.line_width == 0 {
        bail!("line width must be >= 1");
    }

    let (side, adjusted) = resolve_curve_dimension(pattern_name, curve_dimension)?;
    let pattern = pattern_from_name(pattern_name, 2, side)?;
    let length = pattern.length();
    let chunk = chunk.unwrap_or(0..length);

    if chunk.start >= chunk.end {
        bail!("chunk start must be less than chunk end");
    }

    if chunk.end > length {
        bail!(
            "chunk end {} exceeds curve length {} for pattern '{}'",
            chunk.end,
            length,
            pattern_name
        );
    }

    let imgbuf = render_map_image(size, side, chunk, stroke, &*pattern);
    Ok(MapRender {
        image: imgbuf,
        side,
        adjusted,
    })
}

/// Generate an animated snake GIF where a chunk of the curve marches across all offsets.
pub fn snake(options: SnakeOptions<'_>) -> Result<SnakeRender> {
    let SnakeOptions {
        size,
        curve_dimension,
        pattern_name,
        chunk,
        fps,
        stroke,
        output,
        full_curve,
    } = options;

    if stroke.line_width == 0 {
        bail!("line width must be >= 1");
    }

    if size > u16::MAX as u32 {
        bail!("size {} exceeds GIF limits ({}).", size, u16::MAX);
    }

    let (side, adjusted) = resolve_curve_dimension(pattern_name, curve_dimension)?;
    let pattern = pattern_from_name(pattern_name, 2, side)?;
    let length = pattern.length();

    if chunk.start >= chunk.end {
        bail!("chunk start must be less than chunk end");
    }

    if chunk.end > length {
        bail!(
            "chunk end {} exceeds curve length {} for pattern '{}'",
            chunk.end,
            length,
            pattern_name
        );
    }

    let chunk_len = chunk.end - chunk.start;
    if chunk_len < 2 {
        bail!("chunk must span at least two points for animation");
    }

    let mut file = File::create(output)?;
    let mut encoder = Encoder::new(&mut file, size as u16, size as u16, &[])?;
    encoder.set_repeat(Repeat::Infinite)?;

    let frame_delay = frame_delay_from_fps(fps);

    let base_frame = full_curve.map(|foreground| {
        let palette = StrokeOptions {
            palette: MapPalette {
                foreground,
                background: stroke.palette.background,
            },
            ..stroke
        };
        render_map_image(size, side, 0..length, palette, &*pattern)
    });

    for offset in 0..length {
        let start = (chunk.start + offset) % length;
        let mut frame_image = base_frame
            .clone()
            .unwrap_or_else(|| render_chunk_image(size, side, start, chunk_len, stroke, &*pattern));

        if base_frame.is_some() {
            draw_chunk_overlay(
                &mut frame_image,
                size,
                side,
                start,
                chunk_len,
                stroke,
                &*pattern,
            );
        }

        let mut raw = frame_image.into_raw();
        let mut frame = Frame::from_rgba_speed(size as u16, size as u16, &mut raw, 10);
        frame.delay = frame_delay;
        encoder.write_frame(&frame)?;
    }

    Ok(SnakeRender { side, adjusted })
}

/// Convert frames-per-second into a GIF frame delay (hundredths of a second).
fn frame_delay_from_fps(fps: u16) -> u16 {
    // GIF delays are centiseconds; clamp to at least 1cs to avoid zero-delay frames.
    let fps = fps.max(1);
    ((100 + (fps / 2)) / fps).max(1)
}

/// Generate a 4096×4096 image containing every RGB color exactly once.
///
/// The pixels are laid out following `pattern_name`; the colors are chosen by
/// walking `colormap_name` in RGB space.
pub fn allrgb(pattern_name: &str, colormap_name: &str) -> Result<image::RgbaImage> {
    let width = 4096;
    let pattern = pattern_from_name(pattern_name, 2, width)?;
    let mut imgbuf: image::RgbaImage = image::ImageBuffer::new(width, width);
    let colormap = pattern_from_name(colormap_name, 3, 256)?;

    let mut pb = pbr::ProgressBar::new(4096);
    pb.format("╢▌▌░╟");

    for i in 0..pattern.length() {
        let p = pattern.point(i);
        let c = colormap.point(i);
        if i % 4096 == 0 {
            pb.inc();
        }
        imgbuf.put_pixel(
            p[0],
            p[1],
            image::Rgba([c[0] as u8, c[1] as u8, c[2] as u8, 255]),
        );
    }

    pb.finish();
    Ok(imgbuf)
}
