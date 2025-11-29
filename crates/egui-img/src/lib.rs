#![warn(missing_docs)]

//! Tiny helper to show an RGBA image inside an egui window.

use std::{
    fs::File,
    io::BufWriter,
    path::{Path, PathBuf},
    sync::Arc,
};

use anyhow::{Result, anyhow};
use eframe::{NativeOptions, egui};
use egui::{Vec2, load::SizedTexture};
use image::RgbaImage;
use png::{BitDepth, ColorType, Encoder};

/// Simple egui app that shows a single texture with a zoom slider.
struct ImageViewer {
    /// Texture containing the displayed image.
    texture: egui::TextureHandle,
    /// Pixel dimensions of the image.
    image_size: [usize; 2],
    /// Current zoom multiplier.
    zoom: f32,
    /// Default zoom used for reset.
    base_zoom: f32,
    /// Optional screenshot capture state.
    screenshot: Option<ScreenshotState>,
    /// Window title shown in the header.
    title: String,
}

/// Layout constants for the viewer window.
const PADDING_PX: f32 = 24.0;
/// Minimum window size used for the viewer.
const MIN_WINDOW: Vec2 = Vec2::new(320.0, 240.0);
/// Maximum window size used for the viewer.
const MAX_WINDOW: Vec2 = Vec2::new(1200.0, 900.0);
/// Estimated vertical chrome (heading + controls) reserved in the window.
const UI_OVERHEAD_PX: f32 = 120.0;
/// Horizontal chrome allowance (panel padding/scrollbar reserve).
const UI_OVERHEAD_X_PX: f32 = 24.0;

/// Tracks pending screenshot capture for the debug helper.
#[derive(Clone)]
struct ScreenshotState {
    /// Whether a screenshot request has been issued.
    requested: bool,
    /// Destination path for the captured PNG.
    output_path: PathBuf,
}

/// Compute initial zoom and window size that keep a gap around the image while respecting caps.
fn initial_view(image_size: [usize; 2]) -> (f32, Vec2) {
    let img = Vec2::new(image_size[0] as f32, image_size[1] as f32);
    let usable_max = Vec2::new(
        (MAX_WINDOW.x - PADDING_PX * 2.0 - UI_OVERHEAD_X_PX).max(1.0),
        (MAX_WINDOW.y - PADDING_PX * 2.0 - UI_OVERHEAD_PX).max(1.0),
    );

    let fit_zoom = (usable_max.x / img.x)
        .min(usable_max.y / img.y)
        .clamp(0.1, 1.0);

    let window = Vec2::new(
        (img.x * fit_zoom + PADDING_PX * 2.0 + UI_OVERHEAD_X_PX).clamp(MIN_WINDOW.x, MAX_WINDOW.x),
        (img.y * fit_zoom + PADDING_PX * 2.0 + UI_OVERHEAD_PX).clamp(MIN_WINDOW.y, MAX_WINDOW.y),
    );

    (fit_zoom, window)
}

impl ImageViewer {
    /// Create an `ImageViewer` by uploading the provided `ColorImage` to a texture.
    fn new(
        cc: &eframe::CreationContext<'_>,
        title: String,
        color_image: egui::ColorImage,
        screenshot: Option<PathBuf>,
    ) -> Self {
        let image_size = color_image.size;
        let (base_zoom, _) = initial_view(image_size);
        let texture =
            cc.egui_ctx
                .load_texture(title.clone(), color_image, egui::TextureOptions::NEAREST);

        Self {
            texture,
            image_size,
            zoom: base_zoom,
            base_zoom,
            screenshot: screenshot.map(|output_path| ScreenshotState {
                requested: false,
                output_path,
            }),
            title,
        }
    }

    /// Pixel size of the image at the current zoom level.
    fn display_size(&self) -> Vec2 {
        Vec2::new(
            self.image_size[0] as f32 * self.zoom,
            self.image_size[1] as f32 * self.zoom,
        )
    }

    /// Render the texture into the given `ui` at `display_size`.
    fn paint_image(&self, ui: &mut egui::Ui, display_size: Vec2) {
        let sized_texture = SizedTexture::from_handle(&self.texture);

        ui.add(
            egui::Image::from_texture(sized_texture)
                .texture_options(egui::TextureOptions::NEAREST)
                .fit_to_exact_size(display_size),
        );
    }

    /// Kick off and save a screenshot if configured. Returns true when capture completes.
    fn handle_screenshot(&mut self, ctx: &egui::Context) -> bool {
        let Some(state) = self.screenshot.as_mut() else {
            return false;
        };

        if !state.requested {
            state.requested = true;
            ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(Default::default()));
            ctx.request_repaint();
            return false;
        }

        let mut captured: Option<Arc<egui::ColorImage>> = None;
        ctx.input(|input| {
            for event in &input.events {
                if let egui::Event::Screenshot { image, .. } = event {
                    captured = Some(image.clone());
                    break;
                }
            }
        });

        if let Some(image) = captured {
            if let Err(err) = save_color_image(&state.output_path, &image) {
                eprintln!("Failed to save screenshot: {err}");
            }
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return true;
        }

        ctx.request_repaint();
        false
    }
}

impl eframe::App for ImageViewer {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let title = self.title.clone();
        egui::CentralPanel::default().show(ctx, |ui| {
            if self.screenshot.is_none() {
                ui.heading(&title);
                ui.separator();

                ui.horizontal(|ui| {
                    ui.label(format!("{} × {}", self.image_size[0], self.image_size[1]));
                    ui.add(
                        egui::Slider::new(&mut self.zoom, 0.1..=8.0)
                            .logarithmic(true)
                            .text("Zoom"),
                    );
                    if ui.button("Reset").clicked() {
                        self.zoom = self.base_zoom;
                    }
                });

                ui.separator();
            }

            let display_size = self.display_size();
            let padded_size = Vec2::new(
                display_size.x + PADDING_PX * 2.0,
                display_size.y + PADDING_PX * 2.0,
            );
            let available = ui.available_size();
            let fits_without_scroll = padded_size.x <= available.x && padded_size.y <= available.y;

            if fits_without_scroll {
                if let Some(state) = &self.screenshot && !state.requested {
                    println!(
                        "[egui-img debug] available={:?} padded={:?} display={:?} (fits)",
                        available, padded_size, display_size
                    );
                }
                ui.allocate_ui_with_layout(
                    available,
                    egui::Layout::centered_and_justified(egui::Direction::TopDown),
                    |ui| {
                        ui.allocate_ui_with_layout(
                            padded_size,
                            egui::Layout::centered_and_justified(egui::Direction::TopDown),
                            |ui| self.paint_image(ui, display_size),
                        );
                    },
                );
            } else {
                egui::ScrollArea::both()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        let container = Vec2::new(
                            padded_size.x.max(ui.available_width()),
                            padded_size.y.max(ui.available_height()),
                        );
                        if let Some(state) = &self.screenshot && !state.requested {
                            println!(
                                "[egui-img debug] available={:?} padded={:?} display={:?} (scroll, container={:?})",
                                available, padded_size, display_size, container
                            );
                        }
                        ui.allocate_ui_with_layout(
                            container,
                            egui::Layout::centered_and_justified(egui::Direction::TopDown),
                            |ui| self.paint_image(ui, display_size),
                        );
                    });
            }
        });

        let _ = self.handle_screenshot(ctx);
    }
}

/// Suggest a window size that stays within a comfortable range for most screens.
fn initial_window_size(image_size: [usize; 2]) -> Vec2 {
    let (_, window) = initial_view(image_size);
    window
}

/// Show an RGBA image in a lightweight egui window.
///
/// This function blocks until the window is closed by the user.
/// The image is uploaded with nearest‑neighbour sampling to keep pixels crisp.
pub fn view_image(title: &str, image: RgbaImage) -> Result<()> {
    let size = [image.width() as usize, image.height() as usize];
    let mut color_image = Some(egui::ColorImage::from_rgba_unmultiplied(
        size,
        image.as_raw(),
    ));
    drop(image);
    let window_title = title.to_string();
    let app_title = window_title.clone();

    let native_options = NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size(initial_window_size(size))
            .with_title(window_title.clone()),
        ..Default::default()
    };

    eframe::run_native(
        &app_title,
        native_options,
        Box::new(move |cc| {
            let color_image = color_image
                .take()
                .expect("image should only be consumed once");

            Ok(Box::new(ImageViewer::new(
                cc,
                window_title.clone(),
                color_image,
                None,
            )))
        }),
    )
    .map_err(|err| anyhow!(err.to_string()))
}

/// View an image and emit a screenshot to `output` once the first frame is rendered.
///
/// This is intended for debugging layout/centering issues. The window closes after capture.
#[cfg(not(target_arch = "wasm32"))]
pub fn view_image_with_screenshot(title: &str, image: RgbaImage, output: &Path) -> Result<()> {
    let size = [image.width() as usize, image.height() as usize];
    let mut color_image = Some(egui::ColorImage::from_rgba_unmultiplied(
        size,
        image.as_raw(),
    ));
    let output_path = output.to_path_buf();
    drop(image);
    let window_title = title.to_string();
    let app_title = window_title.clone();

    let native_options = NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size(initial_window_size(size))
            .with_title(window_title.clone()),
        ..Default::default()
    };

    println!(
        "[egui-img debug] screenshot to {:?}, window {:?}, base_zoom {:.3}",
        output_path,
        initial_window_size(size),
        initial_view(size).0
    );

    eframe::run_native(
        &app_title,
        native_options,
        Box::new(move |cc| {
            let color_image = color_image
                .take()
                .expect("image should only be consumed once");

            Ok(Box::new(ImageViewer::new(
                cc,
                window_title.clone(),
                color_image,
                Some(output_path.clone()),
            )))
        }),
    )
    .map_err(|err| anyhow!(err.to_string()))
}

/// Persist an egui `ColorImage` to disk as a PNG file.
fn save_color_image(path: &Path, image: &egui::ColorImage) -> Result<()> {
    let file = File::create(path)?;
    let buffered_file = BufWriter::new(file);
    let mut encoder = Encoder::new(buffered_file, image.size[0] as u32, image.size[1] as u32);
    encoder.set_color(ColorType::Rgba);
    encoder.set_depth(BitDepth::Eight);
    let mut writer = encoder.write_header()?;

    let mut data = Vec::with_capacity(image.pixels.len() * 4);
    for color in &image.pixels {
        let [red, green, blue, alpha] = color.to_srgba_unmultiplied();
        data.extend_from_slice(&[red, green, blue, alpha]);
    }

    writer.write_image_data(&data)?;
    Ok(())
}
