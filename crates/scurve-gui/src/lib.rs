//! GUI application for exploring space‑filling curves using egui/eframe.

use std::{fs::File, io::BufWriter, path::PathBuf, sync::Arc};

use anyhow::Result;

/// Canonical application name used across the GUI.
pub const APP_NAME: &str = "spacecurve";

/// Primary repository URL for the application.
pub const APP_REPO_URL: &str = "https://github.com/cortesi/spacecurve";

/// Represents the currently active view pane.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Pane {
    /// The 2D curve visualization pane.
    #[default]
    TwoD,
    /// The 3D curve visualization pane.
    ThreeD,
}

/// Screenshot target specifying which UI state to capture.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScreenshotTarget {
    /// Capture the 2D pane.
    TwoD,
    /// Capture the 3D pane.
    ThreeD,
    /// Capture the About dialog.
    About,
    /// Capture the settings dropdown (on 2D pane).
    Settings,
    /// Capture the settings dropdown on the 3D pane.
    Settings3D,
}

/// Configuration for screenshot mode.
#[derive(Debug, Clone)]
pub struct ScreenshotConfig {
    /// Which UI element to screenshot.
    pub target: ScreenshotTarget,
    /// Output file path for the PNG.
    pub output_path: PathBuf,
}

#[derive(Debug)]
struct ActiveScreenshot {
    /// Destination path for the PNG output.
    output_path: PathBuf,
    /// Whether we've already requested a frame capture.
    requested: bool,
}

/// Launch configuration for the GUI.
#[derive(Debug, Clone, Default)]
pub struct GuiOptions {
    /// Include experimental curves in selectors when true.
    pub include_experimental_curves: bool,
    /// Optional screenshot capture settings.
    pub screenshot: Option<ScreenshotConfig>,
    /// Enable developer overlay (frame timing, etc.).
    pub show_dev_overlay: bool,
}

/// About dialog contents and helpers.
pub mod about;
/// Shared selection/cache helpers for 2D and 3D panes.
pub mod selection;
/// Shared helpers for snake overlays.
pub mod snake;
/// State management logic.
pub mod state;
/// Centralized theme constants (colors, fonts, spacing).
pub mod theme;
/// 3D view and interactions.
pub mod threed;
/// 2D view and interactions.
pub mod twod;
/// Reusable GUI widgets.
pub mod widgets;

pub use selection::{Selected3DCurve, SelectedCurve};
use state::AnimationController;
use threed::show_3d_pane;
use twod::show_2d_pane;

/// Settings shared between the 2D and 3D views.
pub struct SharedSettings {
    /// Opacity of the main curve rendering (0.0–1.0).
    pub curve_opacity: f32,
    /// Whether to draw long-jump segments in the curve.
    pub show_long_jumps: bool,
    /// Enable the animated snake overlay.
    pub snake_enabled: bool,
    /// Snake length as a percentage of curve length (0–50).
    pub snake_length: f32, // Percentage of curve length (0-50%)
    /// Snake speed, measured in segments per second.
    pub snake_speed: f32,
    /// Rotation speed of the 3D view (0–100 scale).
    pub spin_speed: f32,
}

impl Default for SharedSettings {
    fn default() -> Self {
        Self {
            curve_opacity: 0.35, // Default to 35% opacity
            show_long_jumps: false,
            snake_enabled: true,
            snake_length: 5.0, // Default to 5% of curve length
            snake_speed: 30.0, // Default snake speed (segments per second)
            spin_speed: 50.0,  // Default rotation speed (0-100 scale)
        }
    }
}

/// Mutable application state used by the GUI.
pub struct AppState {
    /// Currently selected pane.
    pub current_pane: Pane,
    /// Accumulated animation time in seconds.
    pub animation_time: f32,
    /// Global pause state for animations.
    pub paused: bool,
    /// Current rotation angle for the 3D view (radians).
    pub rotation_angle: f32,
    /// Whether the user is currently dragging in the 3D view.
    pub mouse_dragging: bool,
    /// Last X coordinate recorded during a drag gesture.
    pub last_mouse_x: f32,
    /// Accumulated time used to advance the snake animation.
    pub snake_time: f32,
    /// Reusable buffer for 2D snake segment indices.
    pub snake_segments_2d: Vec<usize>,
    /// Reusable buffer for 3D snake segment indices.
    pub snake_segments_3d: Vec<usize>,
    /// Reusable membership mask for 2D snake lookups.
    pub snake_mask_2d: Vec<bool>,
    /// Reusable membership mask for 3D snake lookups.
    pub snake_mask_3d: Vec<bool>,
    /// Reusable inclusion mask for visible 3D snake segments.
    pub snake_included_3d: Vec<bool>,
    /// Whether the settings dropdown is currently open.
    pub settings_dropdown_open: bool,
    /// Persisted position for the settings dropdown to avoid frame-to-frame jitter.
    pub settings_dropdown_pos: Option<egui::Pos2>,
    /// Whether the About dialog is currently open.
    pub about_open: bool,
    /// Smoothed frame time in milliseconds (for dev overlay).
    pub frame_time_ms: Option<f32>,
    /// Latched frame time used for the UI (updates slowly for readability).
    pub frame_time_display_ms: Option<f32>,
    /// Last time (seconds) the display value was latched.
    pub frame_time_last_display_s: Option<f64>,
    /// Latest canvas rect for positioning overlays relative to the view.
    pub last_canvas_rect: Option<egui::Rect>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            current_pane: Pane::TwoD,
            animation_time: 0.0,
            paused: false,
            rotation_angle: 0.0,
            mouse_dragging: false,
            last_mouse_x: 0.0,
            snake_time: 0.0,
            snake_segments_2d: Vec::new(),
            snake_segments_3d: Vec::new(),
            snake_mask_2d: Vec::new(),
            snake_mask_3d: Vec::new(),
            snake_included_3d: Vec::new(),
            settings_dropdown_open: false,
            settings_dropdown_pos: None,
            about_open: false,
            frame_time_ms: None,
            frame_time_display_ms: None,
            frame_time_last_display_s: None,
            last_canvas_rect: None,
        }
    }
}

/// Root eframe application.
pub struct ScurveApp {
    /// 2D selection and cache state.
    selected_curve: SelectedCurve,
    /// 3D selection and cache state.
    selected_3d_curve: Selected3DCurve,
    /// Curves available for selection in this run.
    available_curves: Vec<&'static str>,
    /// Mutable app state shared across panes.
    app_state: AppState,
    /// Settings shared between panes.
    shared_settings: SharedSettings,
    /// Active screenshot request state (when running in screenshot mode).
    screenshot: Option<ActiveScreenshot>,
    /// Last frame time used to compute deltas.
    last_time: Option<f64>,
    /// CommonMark cache for the About dialog.
    commonmark_cache: egui_commonmark::CommonMarkCache,
    /// Whether to show developer diagnostics overlay.
    show_dev_overlay: bool,
}

impl ScurveApp {
    /// Construct a new app instance.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        Self::with_options(cc, GuiOptions::default())
    }

    /// Construct a new app instance with optional screenshot configuration.
    pub fn with_screenshot_config(
        cc: &eframe::CreationContext<'_>,
        screenshot_config: Option<ScreenshotConfig>,
    ) -> Self {
        Self::with_options(
            cc,
            GuiOptions {
                screenshot: screenshot_config,
                ..GuiOptions::default()
            },
        )
    }

    /// Construct a new app instance with explicit launch options.
    pub fn with_options(cc: &eframe::CreationContext<'_>, options: GuiOptions) -> Self {
        // Configure visuals with our custom terminal theme
        theme::configure_visuals(&cc.egui_ctx);

        let include_experimental = options.include_experimental_curves;
        let mut available_curves = spacecurve::curve_names(include_experimental);
        if available_curves.is_empty() {
            // Ensure we always have something to show even if filters change.
            available_curves = spacecurve::curve_names(true);
        }

        let default_curve = available_curves
            .first()
            .copied()
            .unwrap_or(spacecurve::CURVE_NAMES[0]);

        let mut app_state = AppState::default();
        let screenshot_config = options.screenshot;
        let mut screenshot_runtime = screenshot_config.as_ref().map(|cfg| ActiveScreenshot {
            output_path: cfg.output_path.clone(),
            requested: false,
        });

        // Configure initial state based on screenshot target
        if let Some(config) = screenshot_config {
            match config.target {
                ScreenshotTarget::TwoD => {
                    app_state.current_pane = Pane::TwoD;
                }
                ScreenshotTarget::ThreeD => {
                    app_state.current_pane = Pane::ThreeD;
                }
                ScreenshotTarget::About => {
                    app_state.current_pane = Pane::TwoD;
                    app_state.about_open = true;
                }
                ScreenshotTarget::Settings => {
                    app_state.current_pane = Pane::TwoD;
                    app_state.settings_dropdown_open = true;
                }
                ScreenshotTarget::Settings3D => {
                    app_state.current_pane = Pane::ThreeD;
                    app_state.settings_dropdown_open = true;
                }
            }
            // Pause animations for consistent screenshots
            app_state.paused = true;
        }

        Self {
            selected_curve: SelectedCurve::with_name(default_curve),
            selected_3d_curve: Selected3DCurve::with_name(default_curve),
            available_curves,
            app_state,
            shared_settings: Default::default(),
            screenshot: screenshot_runtime.take(),
            last_time: None,
            commonmark_cache: Default::default(),
            show_dev_overlay: options.show_dev_overlay,
        }
    }

    /// Render the top menu bar with title, tabs, and About button.
    fn show_menu_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("menu_bar")
            .frame(egui::Frame::new().inner_margin(egui::Margin {
                left: theme::menu_bar::PADDING_HORIZONTAL as i8,
                right: theme::menu_bar::PADDING_HORIZONTAL as i8,
                top: theme::menu_bar::PADDING_VERTICAL as i8,
                bottom: theme::menu_bar::PADDING_VERTICAL as i8,
            }))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    // Title on the far left that links to GitHub
                    if ui
                        .link(
                            egui::RichText::new(APP_NAME)
                                .size(theme::font_size::TITLE)
                                .strong()
                                .color(theme::TEXT_HEADING),
                        )
                        .clicked()
                        && let Err(e) = webbrowser::open(APP_REPO_URL)
                    {
                        eprintln!("Failed to open browser: {e}");
                    }

                    ui.add_space(theme::menu_bar::TITLE_SPACING);

                    // Tab buttons with more visual weight
                    let tab_text_size = 15.0;
                    if ui
                        .selectable_label(
                            self.app_state.current_pane == Pane::TwoD,
                            egui::RichText::new("2D").size(tab_text_size),
                        )
                        .clicked()
                    {
                        self.app_state.current_pane = Pane::TwoD;
                    }
                    ui.add_space(theme::menu_bar::TAB_SPACING);
                    if ui
                        .selectable_label(
                            self.app_state.current_pane == Pane::ThreeD,
                            egui::RichText::new("3D").size(tab_text_size),
                        )
                        .clicked()
                    {
                        self.app_state.current_pane = Pane::ThreeD;
                    }

                    // Right-aligned About button with padding
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.add_space(theme::menu_bar::BUTTON_PADDING);
                        if ui.button("About").clicked() {
                            self.app_state.about_open = !self.app_state.about_open;
                        }
                    });
                });
            });
    }

    /// Handle multi-frame screenshot capture and saving to disk.
    fn handle_screenshot(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let Some(screenshot) = self.screenshot.as_mut() else {
            return;
        };

        // Request a screenshot on the second frame to ensure overlays are fully drawn.
        if !screenshot.requested {
            screenshot.requested = true;
            ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(Default::default()));
            ctx.request_repaint();
            return;
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
            if let Err(err) = save_color_image(&screenshot.output_path, &image) {
                eprintln!("Failed to save screenshot: {err}");
            }
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        } else {
            // Keep driving frames until the platform delivers the screenshot event.
            ctx.request_repaint();
        }
    }

    /// Smooth and store the latest frame time (ms) for dev overlay.
    fn update_frame_time(&mut self, delta_seconds: f32, now_seconds: f64) {
        const DISPLAY_INTERVAL_S: f64 = 0.25;

        if !self.show_dev_overlay {
            return;
        }

        let ms = delta_seconds * 1000.0;
        let smoothed = match self.app_state.frame_time_ms {
            Some(prev) => prev * 0.85 + ms * 0.15,
            None => ms,
        };
        self.app_state.frame_time_ms = Some(smoothed);

        // Latch the display value at a slower cadence for readability
        let should_update = match self.app_state.frame_time_last_display_s {
            Some(last) => now_seconds - last >= DISPLAY_INTERVAL_S,
            None => true,
        };

        if should_update {
            self.app_state.frame_time_display_ms = Some(smoothed);
            self.app_state.frame_time_last_display_s = Some(now_seconds);
        }
    }

    /// Render a lightweight developer overlay showing smoothed frame time.
    fn show_frame_time_overlay(&self, ctx: &egui::Context) {
        let Some(ms) = self
            .app_state
            .frame_time_display_ms
            .or(self.app_state.frame_time_ms)
        else {
            return;
        };
        let fps = if ms > 0.0 { 1000.0 / ms } else { 0.0 };

        let pos = if let Some(rect) = self.app_state.last_canvas_rect {
            egui::pos2(rect.max.x - 12.0, rect.min.y + 12.0)
        } else {
            // Fallback to top-right of the window if no canvas was drawn yet
            let screen_rect = ctx.viewport_rect();
            egui::pos2(screen_rect.max.x - 12.0, screen_rect.min.y + 12.0)
        };

        egui::Area::new(egui::Id::new("dev_frame_time_overlay"))
            .order(egui::Order::Tooltip)
            .fixed_pos(pos)
            .show(ctx, |ui| {
                egui::Frame::new()
                    .fill(theme::PANEL_BACKGROUND)
                    .stroke(egui::Stroke::new(1.0, theme::BORDER))
                    .corner_radius(egui::CornerRadius::same(4))
                    .inner_margin(egui::Margin::symmetric(8, 6))
                    .show(ui, |ui| {
                        ui.set_min_width(130.0);
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new(format!("{ms:.1} ms"))
                                    .color(theme::TEXT_PRIMARY)
                                    .size(theme::font_size::INFO),
                            );
                            ui.add_space(6.0);
                            ui.label(
                                egui::RichText::new(format!("{fps:.1} fps"))
                                    .color(theme::TEXT_PRIMARY)
                                    .size(theme::font_size::INFO),
                            );
                        });
                    });
            });
    }
}

impl eframe::App for ScurveApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        // Compute delta time using egui input time
        let now = ctx.input(|i| i.time);
        if let Some(prev) = self.last_time {
            let delta = (now - prev) as f32;
            let clamped_delta = delta.max(0.0);
            self.update_frame_time(clamped_delta, now);
            AnimationController::update(
                clamped_delta,
                &mut self.app_state,
                &self.shared_settings,
                &mut self.selected_curve,
                &mut self.selected_3d_curve,
            );
        }
        self.last_time = Some(now);

        // Only request a repaint when there is time-based animation to show
        let needs_repaint = self.shared_settings.snake_enabled
            || (self.app_state.current_pane == Pane::ThreeD
                && (!self.app_state.paused || self.app_state.mouse_dragging));
        if needs_repaint {
            ctx.request_repaint();
        }

        self.show_menu_bar(ctx);

        // Show About dialog if open
        if self.app_state.about_open {
            about::show_about_dialog(
                ctx,
                &mut self.app_state.about_open,
                &mut self.commonmark_cache,
            );
        }

        egui::CentralPanel::default().show(ctx, |ui| match self.app_state.current_pane {
            Pane::TwoD => {
                show_2d_pane(
                    ui,
                    &mut self.app_state,
                    &mut self.selected_curve,
                    &self.available_curves,
                    &mut self.shared_settings,
                );
            }
            Pane::ThreeD => {
                show_3d_pane(
                    ui,
                    &mut self.app_state,
                    &mut self.selected_3d_curve,
                    &self.available_curves,
                    &mut self.shared_settings,
                );
            }
        });

        // Synchronize selection between panes based on the active pane
        AnimationController::sync_panes(
            self.app_state.current_pane,
            &mut self.selected_curve,
            &mut self.selected_3d_curve,
            &self.available_curves,
        );

        self.handle_screenshot(ctx, frame);

        if self.show_dev_overlay {
            self.show_frame_time_overlay(ctx);
        }
    }
}

/// Persist an egui `ColorImage` to disk as a PNG file.
fn save_color_image(path: &PathBuf, image: &egui::ColorImage) -> anyhow::Result<()> {
    use png::{BitDepth, ColorType, Encoder};

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

/// Launch the native GUI application (non‑wasm targets).
#[cfg(not(target_arch = "wasm32"))]
pub fn gui() -> Result<()> {
    gui_with_options(GuiOptions::default())
}

/// Launch the native GUI application with optional screenshot configuration.
///
/// When `screenshot_config` is provided, the app will:
/// 1. Set the `EFRAME_SCREENSHOT_TO` environment variable
/// 2. Initialize the UI to show the requested target
/// 3. Render one frame and save the screenshot
/// 4. Exit automatically (when compiled with `__screenshot` feature)
#[cfg(not(target_arch = "wasm32"))]
pub fn gui_with_screenshot(screenshot_config: Option<ScreenshotConfig>) -> Result<()> {
    gui_with_options(GuiOptions {
        screenshot: screenshot_config,
        ..GuiOptions::default()
    })
}

/// Launch the native GUI with custom options, including dev/experimental curves.
#[cfg(not(target_arch = "wasm32"))]
pub fn gui_with_options(options: GuiOptions) -> Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size(theme::window::DEFAULT_SIZE)
            .with_title(format!("{APP_NAME} gui")),
        ..Default::default()
    };

    let options_clone = options;

    eframe::run_native(
        &format!("{APP_NAME} gui"),
        native_options,
        Box::new(move |cc| Ok(Box::new(ScurveApp::with_options(cc, options_clone)))),
    )
    .map_err(|e| anyhow::anyhow!(e.to_string()))?;

    Ok(())
}

/// Stub entrypoint used by the wasm target; the real start is in `src/web.rs`.
#[cfg(target_arch = "wasm32")]
pub fn gui() -> Result<()> {
    // Web is launched from src/web.rs using eframe's WebRunner
    Ok(())
}
