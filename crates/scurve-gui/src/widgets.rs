use egui::{
    self, Response, Slider,
    epaint::{Shadow, Stroke},
};
use spacecurve::pattern_from_name;

use crate::theme;

/// Add a slider with themed rail and fill colors for better visibility.
pub fn themed_slider(ui: &mut egui::Ui, slider: Slider<'_>) -> Response {
    // Override widget visuals for the slider
    let old_noninteractive_bg = ui.visuals().widgets.noninteractive.bg_fill;
    let old_inactive_bg = ui.visuals().widgets.inactive.bg_fill;
    let old_inactive_fg = ui.visuals().widgets.inactive.fg_stroke.color;

    ui.visuals_mut().widgets.noninteractive.bg_fill = theme::SLIDER_RAIL;
    ui.visuals_mut().widgets.inactive.bg_fill = theme::SLIDER_RAIL;
    ui.visuals_mut().widgets.inactive.fg_stroke.color = theme::SLIDER_FILL;

    let response = ui.add(slider);

    // Restore original visuals
    ui.visuals_mut().widgets.noninteractive.bg_fill = old_noninteractive_bg;
    ui.visuals_mut().widgets.inactive.bg_fill = old_inactive_bg;
    ui.visuals_mut().widgets.inactive.fg_stroke.color = old_inactive_fg;

    response
}

/// Checkbox with a distinct neon-backed card for better contrast.
pub fn neon_checkbox(ui: &mut egui::Ui, checked: &mut bool, label: &str) -> Response {
    let fill = if *checked {
        theme::TOGGLE_BG_ACTIVE
    } else {
        theme::TOGGLE_BG
    };

    egui::Frame::new()
        .fill(fill)
        .stroke(Stroke::new(1.0, theme::BORDER))
        .inner_margin(egui::Margin::symmetric(8, 6))
        .corner_radius(egui::CornerRadius::same(3))
        .show(ui, |ui| ui.checkbox(checked, label))
        .inner
}

/// Minimal heading used inside settings sections.
fn section_header(ui: &mut egui::Ui, title: &str) {
    ui.label(
        egui::RichText::new(title)
            .strong()
            .color(theme::TEXT_HEADING)
            .size(theme::font_size::LABEL),
    );
}

/// Slider row with aligned label and themed slider control.
fn slider_row(ui: &mut egui::Ui, label: &str, slider: Slider<'_>) -> Response {
    const LABEL_WIDTH: f32 = 74.0;

    ui.horizontal(|ui| {
        ui.add_sized(
            [LABEL_WIDTH, 0.0],
            egui::Label::new(
                egui::RichText::new(label)
                    .color(theme::TEXT_BODY)
                    .size(theme::font_size::LABEL),
            ),
        );
        ui.add_space(theme::spacing::SMALL);
        themed_slider(ui, slider)
    })
    .inner
}

/// Slider row that shows a fixed-width value label to prevent layout jitter.
fn slider_row_with_value(
    ui: &mut egui::Ui,
    label: &str,
    slider: Slider<'_>,
    value: impl Into<String>,
) -> Response {
    const LABEL_WIDTH: f32 = 74.0;
    const VALUE_WIDTH: f32 = 80.0;

    ui.horizontal(|ui| {
        ui.add_sized(
            [LABEL_WIDTH, 0.0],
            egui::Label::new(
                egui::RichText::new(label)
                    .color(theme::TEXT_BODY)
                    .size(theme::font_size::LABEL),
            ),
        );

        let slider_width = (ui.available_width() - VALUE_WIDTH - theme::spacing::SMALL).max(80.0);
        let response = ui.add_sized([slider_width, 0.0], slider.show_value(false));

        ui.add_space(theme::spacing::SMALL);
        ui.add_sized(
            [VALUE_WIDTH, 0.0],
            egui::Label::new(
                egui::RichText::new(value.into())
                    .monospace()
                    .color(theme::TEXT_SECONDARY),
            ),
        );

        response
    })
    .inner
}

/// Common curve selector widget with label included.
pub fn curve_selector(
    ui: &mut egui::Ui,
    curve_name: &mut String,
    available_curves: &[&str],
    id_salt: &str,
    info_open: &mut bool,
    dim: u32,
    size: u32,
) {
    ui.label("Curve:");
    curve_selector_combo(
        ui,
        curve_name,
        available_curves,
        id_salt,
        info_open,
        dim,
        size,
    );
}

/// Curve selector combo box only (without label).
/// Use this when you want to style the label separately.
pub fn curve_selector_combo(
    ui: &mut egui::Ui,
    curve_name: &mut String,
    available_curves: &[&str],
    id_salt: &str,
    info_open: &mut bool,
    dim: u32,
    size: u32,
) {
    // Track if any curve was selected
    let mut curve_was_selected = false;

    let combo_response = egui::ComboBox::from_id_salt(id_salt)
        .selected_text(&*curve_name)
        .show_ui(ui, |ui| {
            for &name in available_curves {
                if ui
                    .selectable_value(curve_name, name.to_string(), name)
                    .clicked()
                {
                    curve_was_selected = true;
                }
            }
        });

    // Info button with better styling
    let info_button = ui.add(
        egui::Button::new("ℹ")
            .min_size(egui::vec2(20.0, 20.0))
            .fill(if *info_open {
                ui.visuals().widgets.hovered.bg_fill
            } else {
                egui::Color32::TRANSPARENT
            }),
    );
    if info_button.clicked() {
        *info_open = !*info_open;
    }

    if *info_open {
        draw_curve_info_pane(
            ui.ctx(),
            InfoPaneArgs {
                id_salt,
                info_open,
                curve_name,
                dim,
                size,
                curve_was_selected,
                combo_response: &combo_response.response,
                info_button: &info_button,
            },
        );
    }
}

/// Arguments for the curve info pane helper.
struct InfoPaneArgs<'a> {
    /// Unique salt used for UI ids tied to this selector.
    id_salt: &'a str,
    /// Mutable flag controlling whether the pane is open.
    info_open: &'a mut bool,
    /// Currently selected curve name.
    curve_name: &'a str,
    /// Dimensionality of the curve (2 or 3).
    dim: u32,
    /// Grid size used when querying pattern info.
    size: u32,
    /// Whether a selection just occurred in the combo box.
    curve_was_selected: bool,
    /// Response for the combo box area (used for outside‑click detection).
    combo_response: &'a egui::Response,
    /// Response for the info button (used for positioning and outside‑click detection).
    info_button: &'a egui::Response,
}

/// Render the floating curve info pane and handle its interactions.
fn draw_curve_info_pane(ctx: &egui::Context, args: InfoPaneArgs<'_>) {
    let InfoPaneArgs {
        id_salt,
        info_open,
        curve_name,
        dim,
        size,
        curve_was_selected,
        combo_response,
        info_button,
    } = args;
    let button_rect = info_button.rect;
    let anchor_pos = egui::pos2(
        button_rect.max.x + theme::popup::ANCHOR_OFFSET,
        button_rect.max.y + theme::popup::ANCHOR_OFFSET,
    );

    let area = egui::Area::new(egui::Id::new(format!("{}_info_pane", id_salt)))
        .movable(false)
        .order(egui::Order::Foreground)
        .pivot(egui::Align2::LEFT_TOP)
        .constrain_to(ctx.content_rect())
        .fixed_pos(anchor_pos)
        .show(ctx, |ui| {
            egui::Frame::popup(ui.style())
                .inner_margin(egui::Margin::same(theme::popup::INNER_MARGIN))
                .shadow(Shadow {
                    offset: theme::shadow::OFFSET,
                    blur: theme::shadow::BLUR,
                    spread: theme::shadow::SPREAD,
                    color: egui::Color32::from_black_alpha(theme::POPUP_SHADOW_ALPHA),
                })
                .corner_radius(egui::CornerRadius::same(theme::popup::CORNER_RADIUS))
                .show(ui, |ui| {
                    ui.set_width(theme::popup::INFO_PANE_WIDTH);
                    render_info_popup_contents(ui, curve_name, dim, size, info_open);
                });
        });

    if !curve_was_selected {
        let pointer_pos = ctx.input(|i| i.pointer.interact_pos());
        let combo_dropdown_open = egui::Popup::is_id_open(ctx, egui::Id::new(id_salt));
        if ctx.input(|i| i.pointer.primary_clicked())
            && !combo_dropdown_open
            && let Some(pos) = pointer_pos
        {
            let inside_button = info_button.rect.contains(pos);
            let inside_pane = area.response.rect.contains(pos);
            let inside_combo = combo_response.rect.contains(pos);
            if !inside_button && !inside_pane && !inside_combo {
                *info_open = false;
            }
        }
    }
    if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
        *info_open = false;
    }
}

/// Body content for the floating curve info popup.
fn render_info_popup_contents(
    ui: &mut egui::Ui,
    curve_name: &str,
    dim: u32,
    size: u32,
    info_open: &mut bool,
) {
    if let Ok(curve) = pattern_from_name(curve_name, dim, size) {
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new(curve.name()).heading().strong());
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui
                    .add(
                        egui::Button::new(egui::RichText::new("×").size(theme::font_size::LABEL))
                            .fill(egui::Color32::TRANSPARENT)
                            .frame(false),
                    )
                    .clicked()
                {
                    *info_open = false;
                }
            });
        });
        ui.add_space(theme::spacing::SMALL);
        ui.add(egui::Separator::default().spacing(theme::spacing::MEDIUM));
        ui.add_space(theme::spacing::SMALL + 2.0);
        let processed_text = curve
            .info()
            .lines()
            .map(|line| line.trim())
            .collect::<Vec<_>>()
            .join(" ");
        egui::Frame::new()
            .inner_margin(egui::Margin::symmetric(4, 2))
            .show(ui, |ui| {
                ui.label(
                    egui::RichText::new(processed_text)
                        .size(theme::font_size::INFO)
                        .color(ui.visuals().text_color().gamma_multiply(0.9)),
                );
            });
    } else {
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Curve Info").heading().strong());
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui
                    .add(
                        egui::Button::new(egui::RichText::new("×").size(theme::font_size::LABEL))
                            .fill(egui::Color32::TRANSPARENT)
                            .frame(false),
                    )
                    .clicked()
                {
                    *info_open = false;
                }
            });
        });
        ui.add_space(theme::spacing::SMALL);
        ui.add(egui::Separator::default().spacing(theme::spacing::MEDIUM));
        ui.add_space(theme::spacing::SMALL + 2.0);
        ui.label(
            egui::RichText::new("Unable to construct curve for info.")
                .italics()
                .color(ui.visuals().warn_fg_color),
        );
    }
}

/// Common size selector widget for 2D curves
pub fn size_selector_2d(ui: &mut egui::Ui, size: &mut u32, id_salt: &str) {
    egui::ComboBox::from_id_salt(id_salt)
        .selected_text(format!("{size}×{size}"))
        .show_ui(ui, |ui| {
            for &s in &[4, 8, 16, 32, 64, 128] {
                ui.selectable_value(size, s, format!("{s}×{s}"));
            }
        });
}

/// Common size selector widget for 3D curves
pub fn size_selector_3d(ui: &mut egui::Ui, size: &mut u32, id_salt: &str) {
    egui::ComboBox::from_id_salt(id_salt)
        .selected_text(format!("{size}×{size}×{size}"))
        .show_ui(ui, |ui| {
            for &s in &[4, 8, 16, 32] {
                // Smaller max size for 3D due to cubic growth
                ui.selectable_value(size, s, format!("{s}×{s}×{s}"));
            }
        });
}

/// Common pause/play button widget
pub fn pause_play_button(ui: &mut egui::Ui, paused: &mut bool) -> bool {
    let (fill, border, glyph) = if *paused {
        (theme::BUTTON_PLAY, theme::TEXT_LINK, "▶")
    } else {
        (theme::BUTTON_PAUSE, theme::TEXT_HEADING, "⏸")
    };

    let clicked = ui
        .add(
            egui::Button::new(
                egui::RichText::new(glyph)
                    .color(theme::TEXT_PRIMARY)
                    .size(theme::font_size::TITLE),
            )
            .min_size(egui::vec2(34.0, 28.0))
            .fill(fill)
            .stroke(Stroke::new(1.5, border)),
        )
        .clicked();

    if clicked {
        *paused = !*paused;
    }

    clicked
}

/// Render the settings panel content (called from within the dropdown frame).
fn settings_panel_content(
    ui: &mut egui::Ui,
    shared: &mut crate::SharedSettings,
    show_spin_speed: bool,
) {
    // Logarithmic opacity slider constant - maps opacity (0.01 to 1.0) to log scale (0 to 100)
    const LOG_MIN: f32 = -4.605;

    ui.spacing_mut().item_spacing.y = theme::spacing::MEDIUM - 2.0;

    // Curve controls (no top-level heading per request)

    let mut log_value = if shared.curve_opacity <= 0.0 {
        0.0
    } else {
        ((shared.curve_opacity.ln() - LOG_MIN) / (0.0 - LOG_MIN)) * 100.0
    };

    let response = slider_row(
        ui,
        "Opacity",
        egui::Slider::new(&mut log_value, 0.0..=100.0)
            .step_by(1.0)
            .suffix("%")
            .custom_formatter(|n, _| {
                if n <= 0.0 {
                    "0".to_string()
                } else {
                    let opacity = (LOG_MIN + (n as f32 / 100.0) * (0.0 - LOG_MIN)).exp();
                    format!("{:.0}", opacity * 100.0)
                }
            }),
    );

    if response.changed() {
        shared.curve_opacity = if log_value <= 0.0 {
            0.0
        } else {
            (LOG_MIN + (log_value / 100.0) * (0.0 - LOG_MIN)).exp()
        };
    }

    neon_checkbox(ui, &mut shared.show_long_jumps, "Long jumps");

    ui.add_space(theme::spacing::MEDIUM - 2.0);
    ui.add(egui::Separator::default().spacing(theme::spacing::SMALL));

    section_header(ui, "Snake");

    neon_checkbox(ui, &mut shared.snake_enabled, "Enable snake overlay");

    let snake_length_value = shared.snake_length;
    slider_row_with_value(
        ui,
        "Length",
        egui::Slider::new(&mut shared.snake_length, 0.0..=50.0).step_by(0.5),
        format!("{:>6.1}%", snake_length_value),
    );
    let snake_value = shared.snake_speed;
    slider_row_with_value(
        ui,
        "Speed",
        egui::Slider::new(&mut shared.snake_speed, 1.0..=200.0).step_by(1.0),
        format!("{:>6.0} seg/s", snake_value.round()),
    );

    if show_spin_speed {
        ui.add_space(theme::spacing::MEDIUM - 2.0);
        ui.add(egui::Separator::default().spacing(theme::spacing::SMALL));
        section_header(ui, "3D rotation");
        let spin_value = shared.spin_speed;
        slider_row_with_value(
            ui,
            "Speed",
            egui::Slider::new(&mut shared.spin_speed, 0.0..=100.0).step_by(1.0),
            format!("{:>5.0}%", spin_value.round()),
        );
    }
}

/// Settings dropdown widget that appears as an overlay.
///
/// When `show_spin_speed` is true (3D view), the rotation speed slider is displayed.
pub fn settings_dropdown(
    ui: &mut egui::Ui,
    settings_open: &mut bool,
    settings_pos: &mut Option<egui::Pos2>,
    shared: &mut crate::SharedSettings,
    show_spin_speed: bool,
) {
    let button_response = ui.button("⚙");
    if button_response.clicked() {
        *settings_open = !*settings_open;
        if *settings_open {
            *settings_pos = None; // force re-anchor on open
        }
    }

    if !*settings_open {
        *settings_pos = None;
        return;
    }

    // Position the dropdown relative to the button
    let button_rect = button_response.rect;
    let anchor_pos = settings_pos.get_or_insert_with(|| {
        egui::pos2(
            button_rect.max.x + theme::popup::SETTINGS_OFFSET_X,
            button_rect.max.y + theme::popup::SETTINGS_OFFSET_Y,
        )
    });

    let area_response = egui::Area::new(egui::Id::new("settings_dropdown"))
        .movable(false)
        .order(egui::Order::Foreground)
        .pivot(egui::Align2::RIGHT_TOP)
        .constrain_to(ui.ctx().content_rect())
        .fixed_pos(*anchor_pos)
        .show(ui.ctx(), |ui| {
            egui::Frame::new()
                .fill(theme::SETTINGS_PANEL_BG)
                .stroke(Stroke::new(1.0, theme::BORDER))
                .inner_margin(egui::Margin::same(theme::popup::SETTINGS_MARGIN))
                .corner_radius(egui::CornerRadius::same(theme::popup::CORNER_RADIUS))
                .shadow(Shadow {
                    offset: theme::shadow::OFFSET,
                    blur: theme::shadow::BLUR,
                    spread: theme::shadow::SPREAD,
                    color: egui::Color32::from_rgba_unmultiplied(
                        theme::accent_color::R,
                        theme::accent_color::G,
                        theme::accent_color::B,
                        theme::POPUP_SHADOW_ALPHA,
                    ),
                })
                .show(ui, |ui| {
                    ui.set_width(theme::popup::SETTINGS_WIDTH);
                    ui.set_min_width(theme::popup::SETTINGS_WIDTH);
                    ui.spacing_mut().slider_width = theme::popup::SETTINGS_WIDTH - 90.0;
                    ui.vertical(|ui| settings_panel_content(ui, shared, show_spin_speed));
                });
        });

    // Close dropdown if user clicks outside
    let pointer_pos = ui.input(|i| i.pointer.interact_pos());
    if ui.input(|i| i.pointer.primary_clicked())
        && let Some(pos) = pointer_pos
    {
        let inside_dropdown = area_response.response.rect.contains(pos);
        let inside_button = button_response.rect.contains(pos);
        if !inside_dropdown && !inside_button {
            *settings_open = false;
            *settings_pos = None;
        }
    }

    if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
        *settings_open = false;
        *settings_pos = None;
    }
}
