use std::sync::OnceLock;

use egui::epaint::Shadow;
use egui_commonmark::CommonMarkViewer;
use eguidev::{DevUiExt, WidgetMeta, WidgetRole};

use crate::{APP_NAME, theme};

/// Show the modal About dialog overlay, handling open/close interactions.
pub fn show_about_dialog(
    ctx: &egui::Context,
    about_open: &mut bool,
    cache: &mut egui_commonmark::CommonMarkCache,
) {
    let (was_just_opened, dialog_opened_id) = track_dialog_open(ctx);
    draw_dim_background(ctx);

    let screen_rect = ctx.content_rect();
    let dialog_size = egui::vec2(
        theme::window::ABOUT_DIALOG_SIZE.0,
        theme::window::ABOUT_DIALOG_SIZE.1,
    );
    let center_pos = screen_rect.center() - dialog_size * 0.5;

    let mut should_close = false;
    let response = show_about_area(ctx, cache, dialog_size, center_pos, &mut should_close);
    eguidev::track_response_full(
        "dialog.about",
        &response.response,
        WidgetMeta {
            role: WidgetRole::Window,
            label: Some("About".to_string()),
            visible: true,
            ..Default::default()
        },
    );

    if !was_just_opened
        && ctx.input(|i| i.pointer.primary_clicked())
        && let Some(pos) = ctx.input(|i| i.pointer.interact_pos())
        && !response.response.rect.contains(pos)
    {
        *about_open = false;
    }

    if should_close || ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
        *about_open = false;
        clear_dialog_open(ctx, dialog_opened_id);
    }

    if !*about_open {
        clear_dialog_open(ctx, dialog_opened_id);
    }
}

/// Track the About dialog open flag; returns (just_opened, storage_id).
fn track_dialog_open(ctx: &egui::Context) -> (bool, egui::Id) {
    let id = egui::Id::new("about_dialog_opened");
    let was_just_opened = !ctx.data(|d| d.get_temp::<bool>(id).unwrap_or(false));
    ctx.data_mut(|d| d.insert_temp(id, true));
    (was_just_opened, id)
}

/// Clear the About dialog open flag stored in the Egui context.
fn clear_dialog_open(ctx: &egui::Context, id: egui::Id) {
    ctx.data_mut(|d| d.remove::<bool>(id));
}

/// Paint a dimmed full‑screen background behind the modal dialog.
fn draw_dim_background(ctx: &egui::Context) {
    let screen_rect = ctx.content_rect();
    // Use Middle order so the dim appears below the Foreground dialog
    let painter = ctx.layer_painter(egui::LayerId::new(
        egui::Order::Middle,
        egui::Id::new("about_background"),
    ));
    painter.rect_filled(
        screen_rect,
        egui::CornerRadius::ZERO,
        egui::Color32::from_black_alpha(theme::MODAL_DIM_ALPHA),
    );
}

/// Create and render the About dialog window contents.
fn show_about_area(
    ctx: &egui::Context,
    cache: &mut egui_commonmark::CommonMarkCache,
    dialog_size: egui::Vec2,
    center_pos: egui::Pos2,
    should_close: &mut bool,
) -> egui::InnerResponse<()> {
    egui::Area::new(egui::Id::new("about_dialog"))
        .fixed_pos(center_pos)
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            egui::Frame::popup(ui.style())
                .shadow(Shadow {
                    offset: theme::shadow::OFFSET,
                    blur: theme::shadow::BLUR,
                    spread: theme::shadow::SPREAD,
                    color: egui::Color32::from_black_alpha(theme::DIALOG_SHADOW_ALPHA),
                })
                .show(ui, |ui| {
                    ui.set_width(dialog_size.x);
                    ui.set_height(dialog_size.y);
                    eguidev::container(ui, "dialog.about.body", |ui| {
                        ui.horizontal(|ui| {
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                                if ui
                                    .dev_button(
                                        "dialog.about.close",
                                        egui::RichText::new("×")
                                            .size(theme::font_size::CLOSE_BUTTON),
                                    )
                                    .clicked()
                                {
                                    *should_close = true;
                                }
                            });
                        });

                        egui::Frame::new()
                            .inner_margin(egui::Margin {
                                left: 16,
                                right: 16,
                                top: 0,
                                bottom: 16,
                            })
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.vertical(|ui| {
                                        ui.dev_label(
                                            "dialog.about.heading",
                                            egui::RichText::new(APP_NAME)
                                                .size(theme::font_size::HEADING_LARGE)
                                                .color(theme::TEXT_HEADING)
                                                .strong(),
                                        );
                                        ui.add_space(2.0);
                                        ui.dev_label(
                                            "dialog.about.subtitle",
                                            egui::RichText::new("Space-filling curve playground")
                                                .size(theme::font_size::LABEL)
                                                .color(theme::TEXT_SECONDARY),
                                        );
                                        ui.add_space(2.0);
                                        ui.horizontal(|ui| {
                                            ui.label(
                                                egui::RichText::new("by")
                                                    .size(theme::font_size::INFO)
                                                    .color(theme::TEXT_DIM),
                                            );
                                            ui.hyperlink_to(
                                                egui::RichText::new("Aldo Cortesi")
                                                    .size(theme::font_size::INFO)
                                                    .color(theme::TEXT_LINK),
                                                "https://corte.si",
                                            );
                                        });
                                    });

                                    ui.with_layout(
                                        egui::Layout::right_to_left(egui::Align::TOP),
                                        |ui| {
                                            ui.dev_label(
                                                "dialog.about.version",
                                                egui::RichText::new(format!(
                                                    "v{}",
                                                    env!("CARGO_PKG_VERSION")
                                                ))
                                                .size(theme::font_size::VERSION)
                                                .color(theme::TEXT_DIM),
                                            );
                                        },
                                    );
                                });

                                ui.add_space(theme::spacing::LARGE);
                                ui.add(egui::Separator::default().spacing(12.0));
                                ui.add_space(theme::spacing::LARGE);

                                egui::ScrollArea::vertical()
                                    .max_height(theme::window::ABOUT_SCROLL_HEIGHT)
                                    .show(ui, |ui| {
                                        // Override visuals for readable markdown content
                                        ui.visuals_mut().override_text_color =
                                            Some(theme::TEXT_BODY);
                                        CommonMarkViewer::new().show(ui, cache, about_content());
                                    });
                            });
                    });
                });
        })
}

/// Static markdown content shown in the About dialog.
const ABOUT_CONTENT_BODY: &str = r#"

This interactive playground lets you explore various **space-filling curves** in both 2D and 3D. Space-filling curves are continuous paths that visit every point in a space, providing fascinating mathematical and practical properties.

---

*Built with Rust & egui.*
"#;

/// Markdown content buffer built once for the About dialog.
static ABOUT_CONTENT: OnceLock<String> = OnceLock::new();

/// Return the rendered About markdown, initializing it on first use.
fn about_content() -> &'static str {
    ABOUT_CONTENT
        .get_or_init(|| format!("## Welcome to {APP_NAME}{ABOUT_CONTENT_BODY}"))
        .as_str()
}
