#![allow(missing_docs, clippy::tests_outside_test_module)]

use std::{fs::File, io::Write, path::PathBuf, process::Command};

use assert_cmd::{
    assert::{Assert, OutputAssertExt},
    cargo::CommandCargoExt,
};
use image::{DynamicImage, GenericImageView, Rgba};
use spacecurve::pattern_from_name;
use tempfile::tempdir;

fn first_last_coords(width: u32, pattern: &str) -> (u32, u32, u32, u32) {
    let pat = pattern_from_name(pattern, 2, width).expect("pattern ok");
    let p0 = pat.point(0);
    let p_last = pat.point(pat.length() - 1);
    (p0[0], p0[1], p_last[0], p_last[1])
}

fn read_image(path: &PathBuf) -> DynamicImage {
    image::open(path).expect("image decodes")
}

fn write_bytes(path: &PathBuf, bytes: &[u8]) {
    let mut f = File::create(path).expect("create file");
    f.write_all(bytes).expect("write bytes");
}

fn rgba_eq(a: Rgba<u8>, b: Rgba<u8>) -> bool {
    a.0 == b.0
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

#[test]
fn empty_file_is_rejected() {
    let td = tempdir().expect("tmp");
    let input = td.path().join("empty.bin");
    // create truly empty file
    File::create(&input).expect("create empty");
    let output = td.path().join("out.png");

    run_vis(&input, &output, 16, "hilbert").failure();
}

#[test]
fn single_byte_file_maps_all_pixels() {
    let td = tempdir().expect("tmp");
    let input = td.path().join("one.bin");
    write_bytes(&input, &[0xff]);
    let output = td.path().join("one.png");

    run_vis(&input, &output, 16, "hilbert").success();

    let (x0, y0, xl, yl) = first_last_coords(16, "hilbert");
    let img = read_image(&output);
    let p0 = img.get_pixel(x0, y0);
    let plast = img.get_pixel(xl, yl);
    let white = Rgba([0xff, 0xff, 0xff, 0xff]);
    assert!(
        rgba_eq(p0, white) && rgba_eq(plast, white),
        "all pixels are white for 0xff byte"
    );
}

#[test]
fn two_byte_file_first_and_last_pixels() {
    let td = tempdir().expect("tmp");
    let input = td.path().join("two.bin");
    // first byte black (0x00), last white (0xff)
    write_bytes(&input, &[0x00, 0xff]);
    let output = td.path().join("two.png");

    run_vis(&input, &output, 16, "hilbert").success();

    let (x0, y0, xl, yl) = first_last_coords(16, "hilbert");
    let img = read_image(&output);
    let p0 = img.get_pixel(x0, y0);
    let plast = img.get_pixel(xl, yl);
    let black = Rgba([0x00, 0x00, 0x00, 0xff]);
    let white = Rgba([0xff, 0xff, 0xff, 0xff]);
    assert!(rgba_eq(p0, black), "first pixel maps first byte");
    assert!(rgba_eq(plast, white), "last pixel maps second byte");
}

#[test]
fn equal_length_file_maps_last_byte() {
    // When file length equals curve length, last pixel must map to last byte.
    let width = 16u32; // length == 256
    let plen = (width * width) as usize;
    let td = tempdir().expect("tmp");
    let input = td.path().join("equal.bin");
    let mut data = vec![0x10u8; plen];
    data[0] = 0x00; // first pixel black
    data[plen - 1] = 0xff; // last pixel white
    write_bytes(&input, &data);
    let output = td.path().join("equal.png");

    run_vis(&input, &output, width, "hilbert").success();

    let (x0, y0, xl, yl) = first_last_coords(width, "hilbert");
    let img = read_image(&output);
    let p0 = img.get_pixel(x0, y0);
    let plast = img.get_pixel(xl, yl);
    let black = Rgba([0x00, 0x00, 0x00, 0xff]);
    let white = Rgba([0xff, 0xff, 0xff, 0xff]);
    assert!(rgba_eq(p0, black), "first pixel maps first byte");
    assert!(
        rgba_eq(plast, white),
        "last pixel maps last byte when lengths match"
    );
}

#[test]
fn large_file_no_oob_and_dimensions_ok() {
    // Large file relative to curve length should render successfully with correct dimensions.
    let width = 32u32; // length 1024
    let plen = (width * width) as usize;
    let mlen = plen * 32 + 3; // much larger than plen, but small enough for CI
    let td = tempdir().expect("tmp");
    let input = td.path().join("large.bin");
    // Progressive content to exercise scaling without relying on exact final mapping
    let mut data = vec![0x00u8; mlen];
    data[0] = 0x00;
    data[mlen - 1] = 0xff; // ensure high value present
    write_bytes(&input, &data);
    let output = td.path().join("large.png");

    run_vis(&input, &output, width, "hilbert").success();

    let img = read_image(&output);
    assert_eq!(img.width(), width);
    assert_eq!(img.height(), width);
}
