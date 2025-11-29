//! Centralized theme constants for the scurve GUI.
//!
//! This module contains all visual styling constants including colors, fonts,
//! spacing, and rendering parameters. Centralizing these values makes it easy
//! to experiment with the visual presentation of the application.
//!
//! # Theme: "Neon Grid"
//!
//! Cyberpunk-retro styling inspired by phosphor CRTs and neon signage.
//! Electric cyan drives the curves and primary actions, while ultraviolet
//! magenta highlights secondary affordances. Panels sit on an inky midnight
//! background with subtle indigo strokes for a tech-noir vibe.

use egui::{Color32, FontData, FontDefinitions};

// =============================================================================
// COLORS - Neon Grid Theme
// =============================================================================

/// Inky midnight background that lets neon colors pop.
pub const CANVAS_BACKGROUND: Color32 = Color32::from_rgb(0x06, 0x08, 0x14);

/// Panel/UI background – a hair brighter than the canvas.
pub const PANEL_BACKGROUND: Color32 = Color32::from_rgb(0x0b, 0x0f, 0x22);

/// Primary/interactive color (electric cyan) used for curves and key controls.
/// These values are scaled for depth effects.
pub mod curve_color {
    /// Red component.
    pub const R: u8 = 0x1f;
    /// Green component.
    pub const G: u8 = 0xf2;
    /// Blue component.
    pub const B: u8 = 0xff;
}

/// Snake/accent color (ultraviolet magenta) for high-contrast highlights.
pub mod accent_color {
    /// Red component.
    pub const R: u8 = 0xff;
    /// Green component.
    pub const G: u8 = 0x4d;
    /// Blue component.
    pub const B: u8 = 0xf6;
}

/// Primary text color - crisp cool white.
pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(0xe6, 0xed, 0xff);

/// Secondary/muted text color - desaturated periwinkle.
pub const TEXT_SECONDARY: Color32 = Color32::from_rgb(0x9f, 0xb4, 0xe5);

/// Body text color - softly cool white.
pub const TEXT_BODY: Color32 = Color32::from_rgb(0xcb, 0xd7, 0xff);

/// Dim text color - muted dusk blue.
pub const TEXT_DIM: Color32 = Color32::from_rgb(0x69, 0x73, 0x92);

/// Heading color - neon magenta.
pub const TEXT_HEADING: Color32 = Color32::from_rgb(0xff, 0x5a, 0xf1);

/// Link color - sharp cyan.
pub const TEXT_LINK: Color32 = Color32::from_rgb(0x55, 0xf0, 0xff);

/// Widget background color - deep indigo.
pub const WIDGET_BACKGROUND: Color32 = Color32::from_rgb(0x16, 0x14, 0x28);

/// Widget background when hovered - brighter neon wash.
pub const WIDGET_HOVERED: Color32 = Color32::from_rgb(0x23, 0x20, 0x38);

/// Widget background when active/pressed - saturated ultraviolet.
pub const WIDGET_ACTIVE: Color32 = Color32::from_rgb(0x2f, 0x28, 0x4e);

/// Toggle background for idle checkboxes.
pub const TOGGLE_BG: Color32 = Color32::from_rgb(0x1c, 0x1a, 0x32);

/// Toggle background when checked.
pub const TOGGLE_BG_ACTIVE: Color32 = Color32::from_rgb(0x28, 0x22, 0x4b);

/// Settings panel fill with higher opacity for legibility.
pub const SETTINGS_PANEL_BG: Color32 = Color32::from_rgba_premultiplied(0x16, 0x14, 0x28, 250);

/// Selected/highlighted state - muted magenta fill.
pub const SELECTION: Color32 = Color32::from_rgb(0x25, 0x1e, 0x3a);

/// Border/separator color - indigo stroke.
pub const BORDER: Color32 = Color32::from_rgb(0x38, 0x35, 0x57);

/// Slider track/rail background color.
pub const SLIDER_RAIL: Color32 = Color32::from_rgb(0x2a, 0x27, 0x40);

/// Slider filled/active portion color - cyan to mirror the curve color.
pub const SLIDER_FILL: Color32 = Color32::from_rgb(0x29, 0xf0, 0xff);

/// Play button color - deep cyan fill that fits the neon grid palette.
pub const BUTTON_PLAY: Color32 = Color32::from_rgb(0x0f, 0x6e, 0xa8);

/// Pause button color - ultraviolet violet fill for strong contrast.
pub const BUTTON_PAUSE: Color32 = Color32::from_rgb(0x4a, 0x14, 0x63);

/// Dimming overlay for modal backgrounds.
pub const MODAL_DIM_ALPHA: u8 = 180;

/// Shadow color alpha for popups and dropdowns.
pub const POPUP_SHADOW_ALPHA: u8 = 140;

/// Shadow color alpha for the About dialog.
pub const DIALOG_SHADOW_ALPHA: u8 = 160;

// =============================================================================
// FONTS
// =============================================================================

/// Embedded Orbitron Regular font bytes (OFL licensed).
const FONT_ORBITRON_REGULAR: &[u8] = include_bytes!("../assets/fonts/Orbitron-Regular.ttf");

/// Embedded Orbitron Bold font bytes (OFL licensed).
const FONT_ORBITRON_BOLD: &[u8] = include_bytes!("../assets/fonts/Orbitron-Bold.ttf");

// =============================================================================
// FONTS & TEXT
// =============================================================================

/// Font sizes used throughout the application.
pub mod font_size {
    /// Title text in the menu bar.
    pub const TITLE: f32 = 18.0;

    /// Large heading text (e.g., "spacecurve" in About dialog).
    pub const HEADING_LARGE: f32 = 28.0;

    /// Close button text.
    pub const CLOSE_BUTTON: f32 = 18.0;

    /// Standard label text.
    pub const LABEL: f32 = 14.0;

    /// Small info text.
    pub const INFO: f32 = 13.0;

    /// Version text.
    pub const VERSION: f32 = 12.0;
}

// =============================================================================
// SPACING & LAYOUT
// =============================================================================

/// Spacing values used throughout the UI.
pub mod spacing {
    /// Small vertical space.
    pub const SMALL: f32 = 4.0;

    /// Medium vertical space.
    pub const MEDIUM: f32 = 8.0;

    /// Large vertical space.
    pub const LARGE: f32 = 16.0;
}

/// Menu bar styling constants.
pub mod menu_bar {
    /// Vertical padding for the top menu bar.
    pub const PADDING_VERTICAL: f32 = 6.0;

    /// Horizontal padding for the top menu bar.
    pub const PADDING_HORIZONTAL: f32 = 12.0;

    /// Space after the title before tabs.
    pub const TITLE_SPACING: f32 = 16.0;

    /// Space between tab items.
    pub const TAB_SPACING: f32 = 4.0;

    /// Padding around the About button.
    pub const BUTTON_PADDING: f32 = 8.0;
}

/// Control bar (secondary toolbar) styling constants.
pub mod control_bar {
    /// Vertical padding for the control bar.
    pub const PADDING_VERTICAL: f32 = 4.0;

    /// Horizontal padding for the control bar.
    pub const PADDING_HORIZONTAL: f32 = 8.0;
}

/// Window and dialog dimensions.
pub mod window {
    /// Default window size.
    pub const DEFAULT_SIZE: [f32; 2] = [800.0, 600.0];

    /// About dialog size.
    pub const ABOUT_DIALOG_SIZE: (f32, f32) = (550.0, 450.0);

    /// About dialog content scroll area max height.
    pub const ABOUT_SCROLL_HEIGHT: f32 = 300.0;
}

/// Popup and dropdown dimensions.
pub mod popup {
    /// Inner margin for popup frames.
    pub const INNER_MARGIN: i8 = 10;

    /// Settings dropdown inner margin.
    pub const SETTINGS_MARGIN: i8 = 10;

    /// Settings dropdown width.
    pub const SETTINGS_WIDTH: f32 = 220.0;

    /// Horizontal padding between the anchor button and panel to avoid overlap with canvas.
    pub const SETTINGS_OFFSET_X: f32 = 28.0;

    /// Curve info pane width.
    pub const INFO_PANE_WIDTH: f32 = 320.0;

    /// Corner radius for popup frames (small for technical look).
    pub const CORNER_RADIUS: u8 = 2;

    /// Offset from anchor button for popup positioning.
    pub const ANCHOR_OFFSET: f32 = 4.0;

    /// Vertical offset for settings dropdown from button.
    pub const SETTINGS_OFFSET_Y: f32 = 4.0;
}

/// Shadow parameters for UI elements.
pub mod shadow {
    /// Shadow offset (x, y) - subtle, technical.
    pub const OFFSET: [i8; 2] = [1, 2];

    /// Shadow blur radius.
    pub const BLUR: u8 = 6;

    /// Shadow spread.
    pub const SPREAD: u8 = 0;
}

// =============================================================================
// 2D RENDERING
// =============================================================================

/// 2D canvas rendering parameters.
pub mod canvas_2d {
    /// Margin inside the drawing rect.
    pub const MARGIN: f32 = 10.0;

    /// Drawing area as a fraction of available space.
    pub const SIZE_FRACTION: f32 = 0.85;

    /// Minimum drawing area size.
    pub const MIN_SIZE: f32 = 200.0;

    /// Line width for curve segments.
    pub const LINE_WIDTH: f32 = 2.5;

    /// Snake overlay width multiplier (relative to line width).
    pub const SNAKE_WIDTH_MULTIPLIER: f32 = 1.8;
}

// =============================================================================
// 3D RENDERING
// =============================================================================

/// 3D canvas rendering parameters.
pub mod canvas_3d {
    use std::f32::consts::PI;

    /// Margin around the 3D drawing area.
    pub const MARGIN: f32 = 50.0;

    /// Scale factor for responsive sizing.
    pub const SCALE_FACTOR: f32 = 0.25;

    /// Minimum scale value.
    pub const MIN_SCALE: f32 = 25.0;

    /// Mouse drag rotation sensitivity.
    pub const DRAG_SENSITIVITY: f32 = 0.01;

    /// Distance from camera to scene center in normalized coordinates.
    ///
    /// A value of 4.0 with a scene spanning [-1, 1] provides moderate perspective
    /// distortion that adds depth without excessive foreshortening.
    pub const PERSPECTIVE_DISTANCE: f32 = 4.0;

    /// Fixed tilt angle (radians) for X-axis rotation, giving a slight top-down view.
    ///
    /// PI/6 (30°) tilts the scene so the top face is partially visible while keeping
    /// the front face prominent.
    pub const CAMERA_TILT: f32 = PI / 6.0;

    /// Minimum depth value (front of scene) for brightness mapping.
    pub const DEPTH_MIN: f32 = -2.0;

    /// Maximum depth value (back of scene) for brightness mapping.
    pub const DEPTH_MAX: f32 = 2.0;

    /// Factor by which segment endpoints are shortened to avoid overlap at joints.
    ///
    /// A value of 0.6 times the stroke width provides clean separation between
    /// segments meeting at corners without creating visible gaps.
    pub const CAP_SHORTEN_FACTOR: f32 = 0.6;

    /// Base line width for curve segments.
    pub const BASE_LINE_WIDTH: f32 = 2.0;
}

// =============================================================================
// ANIMATION
// =============================================================================

/// Animation timing parameters.
pub mod animation {
    /// Base rotation speed (radians per second) when the speed slider is at 100%.
    ///
    /// At this rate, a full 360° rotation takes approximately 18 seconds, which
    /// provides a comfortable viewing speed for examining 3D curve structure.
    pub const BASE_ROTATION_SPEED: f32 = 0.35;
}

// =============================================================================
// HELPER FUNCTIONS
// =============================================================================

/// Create the primary curve color with brightness scaling and opacity.
#[inline]
pub fn curve_color_with_brightness(brightness: f32, opacity: f32) -> Color32 {
    Color32::from_rgba_unmultiplied(
        (curve_color::R as f32 * brightness) as u8,
        (curve_color::G as f32 * brightness) as u8,
        (curve_color::B as f32 * brightness) as u8,
        (255.0 * opacity) as u8,
    )
}

/// Create the primary curve color with brightness scaling (opaque).
#[inline]
pub fn curve_color_opaque(brightness: f32) -> Color32 {
    Color32::from_rgb(
        (curve_color::R as f32 * brightness) as u8,
        (curve_color::G as f32 * brightness) as u8,
        (curve_color::B as f32 * brightness) as u8,
    )
}

/// Create snake/accent color scaled by brightness.
#[inline]
pub fn snake_color_with_brightness(brightness: f32) -> Color32 {
    Color32::from_rgb(
        (accent_color::R as f32 * brightness) as u8,
        (accent_color::G as f32 * brightness) as u8,
        (accent_color::B as f32 * brightness) as u8,
    )
}

/// Calculate brightness for regular curve segments (range: 0.3 to 1.0).
///
/// Farther objects appear brighter to simulate depth-based atmosphere.
#[inline]
pub fn segment_brightness(depth: f32) -> f32 {
    0.3 + 0.7 * normalize_depth(depth)
}

/// Calculate brightness for isolated points (range: 0.4 to 1.0).
///
/// Uses a slightly higher base brightness for visibility of single points.
#[inline]
pub fn isolated_point_brightness(depth: f32) -> f32 {
    0.4 + 0.6 * normalize_depth(depth)
}

/// Calculate line width for regular segments based on brightness.
#[inline]
pub fn segment_line_width(brightness: f32) -> f32 {
    canvas_3d::BASE_LINE_WIDTH * (0.5 + 0.5 * brightness)
}

/// Calculate line width for isolated points based on brightness.
#[inline]
pub fn isolated_point_line_width(brightness: f32) -> f32 {
    canvas_3d::BASE_LINE_WIDTH * (0.6 + 0.4 * brightness)
}

/// Normalize a depth value to [0, 1] based on the scene depth range.
#[inline]
pub fn normalize_depth(depth: f32) -> f32 {
    ((depth - canvas_3d::DEPTH_MIN) / (canvas_3d::DEPTH_MAX - canvas_3d::DEPTH_MIN)).clamp(0.0, 1.0)
}

// =============================================================================
// EGUI VISUALS CONFIGURATION
// =============================================================================

/// Configure egui visuals with the terminal theme.
pub fn configure_visuals(ctx: &egui::Context) {
    use egui::{FontFamily, FontId, TextStyle, Visuals, epaint::Shadow};

    let mut visuals = Visuals::dark();

    // Window and panel backgrounds
    visuals.window_fill = PANEL_BACKGROUND;
    visuals.panel_fill = PANEL_BACKGROUND;
    visuals.extreme_bg_color = CANVAS_BACKGROUND;
    visuals.faint_bg_color = WIDGET_BACKGROUND;

    // Override text color - use readable body text as default
    visuals.override_text_color = Some(TEXT_BODY);

    // Widget colors
    visuals.widgets.noninteractive.bg_fill = WIDGET_BACKGROUND;
    visuals.widgets.noninteractive.fg_stroke.color = TEXT_BODY;
    visuals.widgets.noninteractive.bg_stroke.color = BORDER;

    visuals.widgets.inactive.bg_fill = WIDGET_BACKGROUND;
    visuals.widgets.inactive.fg_stroke.color = TEXT_PRIMARY;
    visuals.widgets.inactive.bg_stroke.color = BORDER;

    visuals.widgets.hovered.bg_fill = WIDGET_HOVERED;
    visuals.widgets.hovered.fg_stroke.color = TEXT_PRIMARY;
    visuals.widgets.hovered.bg_stroke.color = TEXT_SECONDARY;

    visuals.widgets.active.bg_fill = WIDGET_ACTIVE;
    visuals.widgets.active.fg_stroke.color = TEXT_PRIMARY;
    visuals.widgets.active.bg_stroke.color = TEXT_PRIMARY;

    visuals.widgets.open.bg_fill = WIDGET_ACTIVE;
    visuals.widgets.open.fg_stroke.color = TEXT_PRIMARY;
    visuals.widgets.open.bg_stroke.color = TEXT_PRIMARY;

    // Selection - amber tint for contrast
    visuals.selection.bg_fill = SELECTION;
    visuals.selection.stroke.color = TEXT_HEADING;

    // Hyperlinks - cyan for visibility
    visuals.hyperlink_color = TEXT_LINK;

    // Warning text - amber
    visuals.warn_fg_color = TEXT_HEADING;

    // Slider styling - show filled portion
    visuals.slider_trailing_fill = true;

    // Window styling - technical, minimal rounding
    visuals.window_corner_radius = egui::CornerRadius::same(2);
    visuals.menu_corner_radius = egui::CornerRadius::same(2);
    visuals.window_stroke.color = BORDER;

    // Popup shadow - magenta glow for a neon halo
    visuals.popup_shadow = Shadow {
        offset: [1, 2],
        blur: 10,
        spread: 0,
        color: Color32::from_rgba_unmultiplied(0xff, 0x5a, 0xf1, 110),
    };

    // Register embedded fonts (Orbitron) for a neon, sci‑fi tone
    let mut fonts = FontDefinitions::default();
    fonts.font_data.insert(
        "orbitron-regular".to_owned(),
        FontData::from_static(FONT_ORBITRON_REGULAR).into(),
    );
    fonts.font_data.insert(
        "orbitron-bold".to_owned(),
        FontData::from_static(FONT_ORBITRON_BOLD).into(),
    );

    // Build stacks that fall back to default proportional fonts so symbols/emojis still render.
    let mut orbitron_stack = fonts
        .families
        .get(&FontFamily::Proportional)
        .cloned()
        .unwrap_or_default();
    orbitron_stack.insert(0, "orbitron-regular".into());

    let mut orbitron_bold_stack = fonts
        .families
        .get(&FontFamily::Proportional)
        .cloned()
        .unwrap_or_default();
    orbitron_bold_stack.insert(0, "orbitron-bold".into());

    fonts
        .families
        .insert(FontFamily::Proportional, orbitron_stack.clone());
    fonts
        .families
        .insert(FontFamily::Name("Orbitron".into()), orbitron_stack);
    fonts.families.insert(
        FontFamily::Name("Orbitron-Bold".into()),
        orbitron_bold_stack,
    );

    ctx.set_fonts(fonts);
    ctx.set_visuals(visuals);

    // Configure text styles
    let mut style = (*ctx.style()).clone();

    // Use tightly spaced techno type with Orbitron
    style.text_styles = [
        (
            TextStyle::Small,
            FontId::new(11.0, FontFamily::Name("Orbitron".into())),
        ),
        (
            TextStyle::Body,
            FontId::new(13.0, FontFamily::Name("Orbitron".into())),
        ),
        (
            TextStyle::Button,
            FontId::new(13.0, FontFamily::Name("Orbitron".into())),
        ),
        (
            TextStyle::Heading,
            FontId::new(18.0, FontFamily::Name("Orbitron-Bold".into())),
        ),
        (
            TextStyle::Monospace,
            FontId::new(13.0, FontFamily::Monospace),
        ),
    ]
    .into();

    // Tighter spacing for a more compact, technical feel
    style.spacing.item_spacing = egui::vec2(6.0, 4.0);
    style.spacing.button_padding = egui::vec2(6.0, 3.0);
    style.spacing.indent = 16.0;

    ctx.set_style(style);
}
