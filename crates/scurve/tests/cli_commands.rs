//! Integration tests for CLI commands: vis, map, allrgb, and error handling.

#![allow(missing_docs, clippy::tests_outside_test_module, deprecated)]

use std::{fs, fs::File, io::Write, path::PathBuf, process::Command};

use assert_cmd::{
    assert::{Assert, OutputAssertExt},
    cargo::CommandCargoExt,
};
use colornames::Color;
use image::DynamicImage;
use tempfile::tempdir;

fn rgba_from_hex(hex: &str) -> [u8; 4] {
    let raw = hex.trim_start_matches('#');
    match raw.len() {
        6 => [
            u8::from_str_radix(&raw[0..2], 16).expect("r"),
            u8::from_str_radix(&raw[2..4], 16).expect("g"),
            u8::from_str_radix(&raw[4..6], 16).expect("b"),
            0xff,
        ],
        8 => [
            u8::from_str_radix(&raw[0..2], 16).expect("r"),
            u8::from_str_radix(&raw[2..4], 16).expect("g"),
            u8::from_str_radix(&raw[4..6], 16).expect("b"),
            u8::from_str_radix(&raw[6..8], 16).expect("a"),
        ],
        _ => panic!("invalid test color: {hex}"),
    }
}

fn rgba_from_name(name: &str) -> [u8; 4] {
    let color: Color = name.try_into().expect("valid color name");
    let (r, g, b) = color.rgb();
    [r, g, b, 0xff]
}

fn write_bytes(path: &PathBuf, bytes: &[u8]) {
    let mut f = File::create(path).expect("create file");
    f.write_all(bytes).expect("write bytes");
}

fn read_image(path: &PathBuf) -> DynamicImage {
    image::open(path).expect("image decodes")
}

#[allow(deprecated)]
fn run_vis(input: &PathBuf, output: &PathBuf, width: u32, pattern: &str) -> Assert {
    let mut cmd = Command::cargo_bin("scurve").expect("binary exists");
    cmd.arg("vis")
        .arg("-p")
        .arg(pattern)
        .arg("-w")
        .arg(width.to_string())
        .arg(input)
        .arg(output);
    cmd.assert()
}

#[allow(deprecated)]
fn run_map(output: &PathBuf, pattern: &str, size: u32, dimension: u32) -> Assert {
    let mut cmd = Command::cargo_bin("scurve").expect("binary exists");
    cmd.arg("map")
        .arg("-s")
        .arg(size.to_string())
        .arg("-d")
        .arg(dimension.to_string())
        .arg(pattern)
        .arg(output);
    cmd.assert()
}

#[allow(deprecated)]
fn run_map_with_colors(
    output: &PathBuf,
    pattern: &str,
    size: u32,
    dimension: u32,
    fg: &str,
    bg: &str,
) -> Assert {
    let mut cmd = Command::cargo_bin("scurve").expect("binary exists");
    cmd.arg("map")
        .arg("-s")
        .arg(size.to_string())
        .arg("-d")
        .arg(dimension.to_string())
        .arg("--fg")
        .arg(fg)
        .arg("--bg")
        .arg(bg)
        .arg(pattern)
        .arg(output);
    cmd.assert()
}

#[allow(deprecated)]
fn run_map_with_line_width(
    output: &PathBuf,
    pattern: &str,
    size: u32,
    dimension: u32,
    line_width: u32,
) -> Assert {
    let mut cmd = Command::cargo_bin("scurve").expect("binary exists");
    cmd.arg("map")
        .arg("-s")
        .arg(size.to_string())
        .arg("-w")
        .arg(line_width.to_string())
        .arg("-d")
        .arg(dimension.to_string())
        .arg("--long")
        .arg(pattern)
        .arg(output);
    cmd.assert()
}

struct SnakeCmd<'a> {
    pattern: &'a str,
    size: u32,
    dimension: u32,
    chunk: &'a str,
    long_edges: bool,
    fps: Option<u16>,
    full: Option<&'a str>,
}

fn snake_cmd<'a>(pattern: &'a str, size: u32, dimension: u32, chunk: &'a str) -> SnakeCmd<'a> {
    SnakeCmd {
        pattern,
        size,
        dimension,
        chunk,
        long_edges: false,
        fps: None,
        full: None,
    }
}

#[allow(deprecated)]
fn run_snake(output: &PathBuf, opts: &SnakeCmd<'_>) -> Assert {
    let mut cmd = Command::cargo_bin("scurve").expect("binary exists");
    cmd.arg("snake")
        .arg("-s")
        .arg(opts.size.to_string())
        .arg("-d")
        .arg(opts.dimension.to_string())
        .arg("--chunk")
        .arg(opts.chunk)
        .arg(opts.pattern)
        .arg(output);
    if opts.long_edges {
        cmd.arg("--long");
    }
    if let Some(fps) = opts.fps {
        cmd.arg("--fps").arg(fps.to_string());
    }
    if let Some(full) = opts.full {
        cmd.arg("--full").arg(full);
    }
    cmd.assert()
}

// ============================================================================
// VIS command tests
// ============================================================================

#[test]
fn vis_produces_valid_png() {
    let td = tempdir().expect("tmp");
    let input = td.path().join("data.bin");
    let data: Vec<u8> = (0..256u16).map(|i| i as u8).collect();
    write_bytes(&input, &data);
    let output = td.path().join("out.png");

    run_vis(&input, &output, 16, "hilbert").success();

    let img = read_image(&output);
    assert_eq!(img.width(), 16);
    assert_eq!(img.height(), 16);
}

#[test]
fn vis_works_with_scan_pattern() {
    let td = tempdir().expect("tmp");
    let input = td.path().join("data.bin");
    write_bytes(&input, &[0x00, 0x80, 0xff]);
    let output = td.path().join("scan.png");

    run_vis(&input, &output, 8, "scan").success();

    let img = read_image(&output);
    assert_eq!(img.width(), 8);
    assert_eq!(img.height(), 8);
}

#[test]
fn vis_works_with_zorder_pattern() {
    let td = tempdir().expect("tmp");
    let input = td.path().join("data.bin");
    write_bytes(&input, &[0x55; 64]);
    let output = td.path().join("zorder.png");

    run_vis(&input, &output, 8, "zorder").success();

    let img = read_image(&output);
    assert_eq!(img.width(), 8);
    assert_eq!(img.height(), 8);
}

#[test]
fn vis_works_with_gray_pattern() {
    let td = tempdir().expect("tmp");
    let input = td.path().join("data.bin");
    write_bytes(&input, &[0xAA; 64]);
    let output = td.path().join("gray.png");

    run_vis(&input, &output, 8, "gray").success();

    let img = read_image(&output);
    assert_eq!(img.width(), 8);
    assert_eq!(img.height(), 8);
}

#[test]
fn vis_works_with_onion_pattern() {
    let td = tempdir().expect("tmp");
    let input = td.path().join("data.bin");
    write_bytes(&input, &[0x12; 100]);
    let output = td.path().join("onion.png");

    run_vis(&input, &output, 10, "onion").success();

    let img = read_image(&output);
    assert_eq!(img.width(), 10);
    assert_eq!(img.height(), 10);
}

#[test]
fn vis_works_with_hairyonion_pattern() {
    let td = tempdir().expect("tmp");
    let input = td.path().join("data.bin");
    write_bytes(&input, &[0x34; 49]);
    let output = td.path().join("hairyonion.png");

    run_vis(&input, &output, 7, "hairyonion").success();

    let img = read_image(&output);
    assert_eq!(img.width(), 7);
    assert_eq!(img.height(), 7);
}

#[test]
fn vis_works_with_hcurve_pattern() {
    let td = tempdir().expect("tmp");
    let input = td.path().join("data.bin");
    write_bytes(&input, &[0x56; 64]);
    let output = td.path().join("hcurve.png");

    run_vis(&input, &output, 8, "hcurve").success();

    let img = read_image(&output);
    assert_eq!(img.width(), 8);
    assert_eq!(img.height(), 8);
}

// ============================================================================
// MAP command tests
// ============================================================================

#[test]
fn map_produces_valid_png() {
    let td = tempdir().expect("tmp");
    let output = td.path().join("map.png");

    run_map(&output, "hilbert", 256, 8).success();

    let img = read_image(&output);
    assert_eq!(img.width(), 256);
    assert_eq!(img.height(), 256);
}

#[test]
fn map_with_various_dimensions() {
    let td = tempdir().expect("tmp");

    for dimension in [4, 8, 16] {
        let output = td.path().join(format!("map_{dimension}.png"));
        run_map(&output, "hilbert", 128, dimension).success();
        let img = read_image(&output);
        assert_eq!(img.width(), 128, "width for dimension {dimension}");
        assert_eq!(img.height(), 128, "height for dimension {dimension}");
    }
}

#[test]
fn map_with_scan_pattern() {
    let td = tempdir().expect("tmp");
    let output = td.path().join("scan_map.png");

    run_map(&output, "scan", 128, 10).success();

    let img = read_image(&output);
    assert_eq!(img.width(), 128);
    assert_eq!(img.height(), 128);
}

#[test]
fn map_with_zorder_pattern() {
    let td = tempdir().expect("tmp");
    let output = td.path().join("zorder_map.png");

    run_map(&output, "zorder", 128, 8).success();

    let img = read_image(&output);
    assert_eq!(img.width(), 128);
    assert_eq!(img.height(), 128);
}

#[test]
fn map_with_onion_pattern() {
    let td = tempdir().expect("tmp");
    let output = td.path().join("onion_map.png");

    run_map(&output, "onion", 128, 9).success();

    let img = read_image(&output);
    assert_eq!(img.width(), 128);
    assert_eq!(img.height(), 128);
}

#[test]
fn map_warns_when_rounding_dimension() {
    let td = tempdir().expect("tmp");
    let output = td.path().join("hilbert_round.png");

    let assert = run_map(&output, "hilbert", 128, 3).success();
    let stderr = String::from_utf8_lossy(&assert.get_output().stderr);
    assert!(
        stderr.contains("using 4"),
        "warning should mention rounded dimension: {stderr}"
    );

    let img = read_image(&output);
    assert_eq!(img.width(), 128);
    assert_eq!(img.height(), 128);
}

#[test]
fn map_respects_custom_colors() {
    let td = tempdir().expect("tmp");
    let output = td.path().join("map_colors.png");
    let fg = "#336699cc";
    let bg = "#0a0b0c11";

    run_map_with_colors(&output, "hilbert", 64, 8, fg, bg).success();

    let img = read_image(&output).to_rgba8();
    let bg_expected = rgba_from_hex(bg);
    assert_eq!(img.get_pixel(0, 0).0, bg_expected, "background matches");

    let fg_expected = rgba_from_hex(fg);
    let has_fg = img.pixels().any(|p| p.0 == fg_expected);
    assert!(has_fg, "foreground colour appears in rendered map");
}

#[test]
fn map_accepts_named_colors() {
    let td = tempdir().expect("tmp");
    let output = td.path().join("map_named.png");

    run_map_with_colors(&output, "hilbert", 64, 8, "red", "linen").success();

    let img = read_image(&output).to_rgba8();
    let bg_expected = rgba_from_name("linen");
    assert_eq!(
        img.get_pixel(0, 0).0,
        bg_expected,
        "background matches named colour"
    );

    let fg_expected = rgba_from_name("red");
    let has_fg = img.pixels().any(|p| p.0 == fg_expected);
    assert!(has_fg, "foreground named colour appears in rendered map");
}

#[test]
fn map_accepts_hex_without_hash() {
    let td = tempdir().expect("tmp");
    let output = td.path().join("map_nohash.png");

    run_map_with_colors(&output, "hilbert", 64, 8, "c0c0c0", "0a0b0c").success();

    let img = read_image(&output).to_rgba8();
    assert_eq!(
        img.get_pixel(0, 0).0,
        rgba_from_hex("0a0b0c"),
        "background matches hex without hash"
    );

    let fg_expected = rgba_from_hex("c0c0c0");
    let has_fg = img.pixels().any(|p| p.0 == fg_expected);
    assert!(
        has_fg,
        "foreground hex without hash appears in rendered map"
    );
}

#[test]
fn map_respects_line_width() {
    let td = tempdir().expect("tmp");
    let default_output = td.path().join("map_default.png");
    let thick_output = td.path().join("map_thick.png");

    run_map(&default_output, "hilbert", 128, 8).success();
    run_map_with_line_width(&thick_output, "hilbert", 128, 8, 3).success();

    let fg_expected = rgba_from_hex("#8080ff");
    let default_fg = read_image(&default_output)
        .to_rgba8()
        .pixels()
        .filter(|p| p.0 == fg_expected)
        .count();
    let thick_fg = read_image(&thick_output)
        .to_rgba8()
        .pixels()
        .filter(|p| p.0 == fg_expected)
        .count();

    assert!(
        thick_fg > default_fg,
        "larger line width renders more foreground pixels"
    );
}

#[test]
fn snake_produces_gif() {
    let td = tempdir().expect("tmp");
    let output = td.path().join("snake.gif");

    let cmd = SnakeCmd {
        long_edges: true,
        ..snake_cmd("hilbert", 32, 4, "0:4")
    };

    run_snake(&output, &cmd).success();

    let bytes = fs::read(&output).expect("gif exists");
    assert!(bytes.starts_with(b"GIF"));

    let img = read_image(&output);
    assert_eq!(img.width(), 32);
    assert_eq!(img.height(), 32);
}

#[test]
fn snake_respects_fps_setting() {
    let td = tempdir().expect("tmp");
    let output = td.path().join("snake_fps.gif");

    let cmd = SnakeCmd {
        fps: Some(10),
        ..snake_cmd("hilbert", 16, 4, "0:4")
    };

    run_snake(&output, &cmd).success();

    let mut decoder = gif::DecodeOptions::new();
    decoder.set_color_output(gif::ColorOutput::RGBA);
    let mut reader = decoder
        .read_info(File::open(&output).expect("open gif"))
        .expect("read gif");
    let frame = reader
        .read_next_frame()
        .expect("frame")
        .expect("frame exists");
    assert_eq!(frame.delay, 10); // 100/10 fps = 10 centiseconds
}

#[test]
fn snake_renders_full_curve_when_requested() {
    let td = tempdir().expect("tmp");
    let output = td.path().join("snake_full.gif");

    let cmd = SnakeCmd {
        size: 24,
        full: Some("lime"),
        ..snake_cmd("hilbert", 24, 4, "0:4")
    };

    run_snake(&output, &cmd).success();

    let img = read_image(&output).to_rgba8();
    let full_expected = rgba_from_name("lime");
    let snake_expected = rgba_from_hex("#8080ff");

    assert!(
        img.pixels().any(|p| p.0 == full_expected),
        "full curve colour should be visible"
    );
    assert!(
        img.pixels().any(|p| p.0 == snake_expected),
        "snake overlay colour should be visible"
    );
}

// ============================================================================
// ALLRGB command tests
// ============================================================================

#[test]
#[ignore = "slow: produces a 4096x4096 image; run with --ignored"]
fn allrgb_produces_correct_dimensions() {
    let td = tempdir().expect("tmp");
    let output = td.path().join("allrgb.png");

    let mut cmd = Command::cargo_bin("scurve").expect("binary exists");
    cmd.arg("allrgb").arg("hilbert").arg(&output);
    cmd.assert().success();

    let img = read_image(&output);
    assert_eq!(img.width(), 4096);
    assert_eq!(img.height(), 4096);
}

// ============================================================================
// Error handling tests
// ============================================================================

#[test]
fn vis_rejects_empty_file() {
    let td = tempdir().expect("tmp");
    let input = td.path().join("empty.bin");
    File::create(&input).expect("create empty");
    let output = td.path().join("out.png");

    run_vis(&input, &output, 16, "hilbert").failure();
}

#[test]
fn vis_rejects_invalid_pattern() {
    let td = tempdir().expect("tmp");
    let input = td.path().join("data.bin");
    write_bytes(&input, &[0x00; 16]);
    let output = td.path().join("out.png");

    let mut cmd = Command::cargo_bin("scurve").expect("binary exists");
    cmd.arg("vis")
        .arg("-p")
        .arg("nonexistent_pattern")
        .arg("-w")
        .arg("16")
        .arg(&input)
        .arg(&output);
    cmd.assert().failure();
}

#[test]
fn vis_rejects_nonexistent_input() {
    let td = tempdir().expect("tmp");
    let input = td.path().join("does_not_exist.bin");
    let output = td.path().join("out.png");

    run_vis(&input, &output, 16, "hilbert").failure();
}

#[test]
fn map_rejects_invalid_pattern() {
    let td = tempdir().expect("tmp");
    let output = td.path().join("map.png");

    let mut cmd = Command::cargo_bin("scurve").expect("binary exists");
    cmd.arg("map")
        .arg("-s")
        .arg("128")
        .arg("-d")
        .arg("8")
        .arg("invalid_curve_name")
        .arg(&output);
    cmd.assert().failure();
}

#[test]
fn map_rejects_zero_dimension() {
    let td = tempdir().expect("tmp");
    let output = td.path().join("map.png");

    run_map(&output, "hilbert", 128, 0).failure();
}

#[test]
fn map_rejects_invalid_color() {
    let td = tempdir().expect("tmp");
    let output = td.path().join("map.png");

    run_map_with_colors(&output, "hilbert", 64, 8, "not-a-color", "#ffffff").failure();
}

#[test]
fn allrgb_rejects_invalid_pattern() {
    let td = tempdir().expect("tmp");
    let output = td.path().join("allrgb.png");

    let mut cmd = Command::cargo_bin("scurve").expect("binary exists");
    cmd.arg("allrgb").arg("not_a_real_pattern").arg(&output);
    cmd.assert().failure();
}

#[test]
fn allrgb_rejects_invalid_colormap() {
    let td = tempdir().expect("tmp");
    let output = td.path().join("allrgb.png");

    let mut cmd = Command::cargo_bin("scurve").expect("binary exists");
    cmd.arg("allrgb")
        .arg("-c")
        .arg("not_a_real_pattern")
        .arg("hilbert")
        .arg(&output);
    cmd.assert().failure();
}
