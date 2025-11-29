//! Debug helper to preview `egui-img` centering and optionally write a screenshot.

#![allow(missing_docs)]

use std::{env, path::PathBuf};

use egui_img::{view_image, view_image_with_screenshot};
use image::ImageReader;

fn main() -> anyhow::Result<()> {
    let mut args = env::args().skip(1).collect::<Vec<_>>();
    if args.is_empty() {
        anyhow::bail!("usage: debug_viewer <image_path> [--screenshot <out.png>]");
    }

    let mut screenshot: Option<PathBuf> = None;
    if let Some(idx) = args.iter().position(|a| a == "--screenshot") {
        if idx + 1 >= args.len() {
            anyhow::bail!("--screenshot requires a path");
        }
        screenshot = Some(PathBuf::from(args.remove(idx + 1)));
        args.remove(idx);
    }

    let image_path = PathBuf::from(args.remove(0));
    let title = image_path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("viewer")
        .to_string();
    let image = ImageReader::open(&image_path)?.decode()?.to_rgba8();

    match screenshot {
        Some(path) => view_image_with_screenshot(&title, image, &path),
        None => view_image(&title, image),
    }
}
