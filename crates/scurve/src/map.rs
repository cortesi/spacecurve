//! Image rendering helpers used by the CLI.
//!
//! This module includes small drawing primitives and the function that renders
//! a sampled map for a given space‑filling curve.

use std::ops::Range;

use image::{Rgba, RgbaImage};
use spacecurve::spacecurve::SpaceCurve;

/// Colors used when rendering a map image.
#[derive(Clone, Copy, Debug)]
pub struct MapPalette {
    /// Color for the curve strokes.
    pub foreground: Rgba<u8>,
    /// Background fill color.
    pub background: Rgba<u8>,
}

/// Stroke styling and edge-handling options for rendering.
#[derive(Clone, Copy, Debug)]
pub struct StrokeOptions {
    /// Stroke width in pixels.
    pub line_width: u32,
    /// Whether to render non-adjacent edges (Manhattan distance > 1).
    pub long_edges: bool,
    /// Colors for foreground/background.
    pub palette: MapPalette,
}

/// Convert a map coordinate to image space.
fn scale(v: u32, margin: u32, side: u32, innerw: f64) -> f64 {
    if side <= 1 {
        return f64::from(margin);
    }

    let sc = innerw / f64::from(side - 1);
    f64::from(margin) + (f64::from(v) * sc)
}

/// Put a pixel if the coordinates are inside the image bounds.
fn put_pixel_safe(img: &mut RgbaImage, x: i64, y: i64, col: image::Rgba<u8>) {
    let w = i64::from(img.width());
    let h = i64::from(img.height());
    if x >= 0 && y >= 0 && x < w && y < h {
        img.put_pixel(x as u32, y as u32, col);
    }
}

/// Stamp a filled square centered on `(cx, cy)` with a given side length.
fn stamp_square(img: &mut RgbaImage, cx: i64, cy: i64, size: u32, col: image::Rgba<u8>) {
    let radius = (i64::from(size) - 1) / 2;
    let extra = if size.is_multiple_of(2) { 1 } else { 0 };
    let x_start = cx - radius;
    let x_end = cx + radius + i64::from(extra);
    let y_start = cy - radius;
    let y_end = cy + radius + i64::from(extra);

    for y in y_start..=y_end {
        for x in x_start..=x_end {
            put_pixel_safe(img, x, y, col);
        }
    }
}

/// Draw a 4‑connected Bresenham line into `img` with color `col`.
fn draw_line(
    img: &mut RgbaImage,
    mut x0: i64,
    mut y0: i64,
    x1: i64,
    y1: i64,
    col: image::Rgba<u8>,
    line_width: u32,
) {
    let dx = (x1 - x0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let dy = -(y1 - y0).abs();
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;
    loop {
        stamp_square(img, x0, y0, line_width, col);
        if x0 == x1 && y0 == y1 {
            break;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x0 += sx;
        }
        if e2 <= dx {
            err += dx;
            y0 += sy;
        }
    }
}

/// Render a square `size×size` image showing a sampled map of `pattern`.
///
/// `side` controls the logical grid size of the pattern (e.g. 16 for a 16×16 Hilbert
/// traversal). `chunk` limits which sequential points are drawn from the pattern using
/// half-open offsets `[start, end)`. `stroke` controls line width, long-edge handling,
/// and colors.
pub fn render_map_image(
    size: u32,
    side: u32,
    chunk: Range<u32>,
    stroke: StrokeOptions,
    pattern: &dyn SpaceCurve,
) -> RgbaImage {
    render_chunk_image(
        size,
        side,
        chunk.start,
        chunk.end.saturating_sub(chunk.start),
        stroke,
        pattern,
    )
}

/// Draw a contiguous curve segment starting at `start` with `len` points into `img`.
///
/// The segment wraps around the curve when `start + len` exceeds the curve length. Styling and
/// long-edge handling are controlled by `stroke`. The existing image contents are preserved and
/// the segment is painted on top.
fn draw_chunk(
    img: &mut RgbaImage,
    size: u32,
    side: u32,
    start: u32,
    len: u32,
    stroke: StrokeOptions,
    pattern: &dyn SpaceCurve,
) {
    let stroke_width = stroke.line_width.max(1);
    let margin = 10_u32.saturating_add(stroke_width / 2);
    let innerw = f64::from(size.saturating_sub(margin.saturating_mul(2))).max(1.0);

    let total_points = pattern.length();
    let len = len.min(total_points);

    debug_assert!(len <= total_points, "chunk length exceeds available points");

    if len < 2 || total_points < 2 {
        return;
    }

    let mut prev = pattern.point(start % total_points);
    for step in 1..len {
        let idx = (start + step) % total_points;
        let next = pattern.point(idx);
        if !stroke.long_edges {
            let dx = (prev[0] as i64 - next[0] as i64).abs();
            let dy = (prev[1] as i64 - next[1] as i64).abs();
            if dx + dy > 1 {
                prev = next;
                continue;
            }
        }
        let x0 = scale(prev[0], margin, side, innerw).round() as i64;
        let y0 = scale(prev[1], margin, side, innerw).round() as i64;
        let x1 = scale(next[0], margin, side, innerw).round() as i64;
        let y1 = scale(next[1], margin, side, innerw).round() as i64;
        draw_line(img, x0, y0, x1, y1, stroke.palette.foreground, stroke_width);
        prev = next;
    }
}

/// Render a square image showing a contiguous curve segment starting at `start` with `len` points.
///
/// The segment wraps around the curve when `start + len` exceeds the curve length. Styling and
/// long-edge handling are controlled by `stroke`.
pub fn render_chunk_image(
    size: u32,
    side: u32,
    start: u32,
    len: u32,
    stroke: StrokeOptions,
    pattern: &dyn SpaceCurve,
) -> RgbaImage {
    let mut imgbuf: RgbaImage =
        image::ImageBuffer::from_pixel(size, size, stroke.palette.background);

    draw_chunk(&mut imgbuf, size, side, start, len, stroke, pattern);
    imgbuf
}

/// Draw a curve segment onto an existing image without clearing it first.
pub fn draw_chunk_overlay(
    img: &mut RgbaImage,
    size: u32,
    side: u32,
    start: u32,
    len: u32,
    stroke: StrokeOptions,
    pattern: &dyn SpaceCurve,
) {
    draw_chunk(img, size, side, start, len, stroke, pattern);
}

#[cfg(test)]
mod tests {
    use image::Rgba;
    use spacecurve::{point::Point, spacecurve::SpaceCurve};

    use super::*;

    #[derive(Debug)]
    struct StubPattern {
        points: Vec<Point>,
    }

    impl StubPattern {
        fn new(coords: Vec<[u32; 2]>) -> Self {
            let points = coords
                .into_iter()
                .map(|c| Point::new(vec![c[0], c[1]]))
                .collect();
            Self { points }
        }
    }

    impl SpaceCurve for StubPattern {
        fn name(&self) -> &'static str {
            "stub"
        }

        fn info(&self) -> &'static str {
            "stub"
        }

        fn index(&self, p: &Point) -> u32 {
            self.points
                .iter()
                .position(|candidate| candidate == p)
                .expect("point not found") as u32
        }

        fn point(&self, index: u32) -> Point {
            self.points[index as usize].clone()
        }

        fn length(&self) -> u32 {
            self.points.len() as u32
        }

        fn dimensions(&self) -> u32 {
            2
        }
    }

    #[test]
    fn render_respects_chunk_range() {
        let pattern = StubPattern::new(vec![[0, 0], [1, 0], [1, 1]]);

        let stroke = StrokeOptions {
            line_width: 1,
            long_edges: true,
            palette: MapPalette {
                foreground: Rgba([1, 2, 3, 255]),
                background: Rgba([0, 0, 0, 0]),
            },
        };

        let full = render_map_image(32, 2, 0..pattern.length(), stroke, &pattern);
        let partial = render_map_image(32, 2, 0..2, stroke, &pattern);

        assert_eq!(partial.get_pixel(22, 10), &stroke.palette.foreground);
        assert_eq!(partial.get_pixel(22, 22), &stroke.palette.background);
        assert_eq!(full.get_pixel(22, 22), &stroke.palette.foreground);
    }

    #[test]
    fn render_wraps_chunk_across_boundary() {
        let pattern = StubPattern::new(vec![[0, 0], [1, 0], [1, 1], [0, 1]]);

        let stroke = StrokeOptions {
            line_width: 1,
            long_edges: true,
            palette: MapPalette {
                foreground: Rgba([9, 9, 9, 255]),
                background: Rgba([0, 0, 0, 0]),
            },
        };

        let wrapped = render_chunk_image(32, 2, 3, 3, stroke, &pattern);

        // Draw from index 3 -> 0 -> 1.
        assert_eq!(wrapped.get_pixel(10, 22), &stroke.palette.foreground);
        assert_eq!(wrapped.get_pixel(22, 10), &stroke.palette.foreground);
    }

    #[test]
    fn render_skips_long_edges_by_default() {
        let pattern = StubPattern::new(vec![[0, 0], [2, 0]]);
        let stroke_short = StrokeOptions {
            line_width: 1,
            long_edges: false,
            palette: MapPalette {
                foreground: Rgba([50, 60, 70, 255]),
                background: Rgba([0, 0, 0, 0]),
            },
        };
        let stroke_long = StrokeOptions {
            long_edges: true,
            ..stroke_short
        };

        let image = render_chunk_image(64, 3, 0, 2, stroke_short, &pattern);
        let with_long = render_chunk_image(64, 3, 0, 2, stroke_long, &pattern);

        let mid_pixel = image.get_pixel(32, 10);
        assert_eq!(mid_pixel, &stroke_short.palette.background);

        let mid_pixel_long = with_long.get_pixel(32, 10);
        assert_eq!(mid_pixel_long, &stroke_short.palette.foreground);
    }
}
