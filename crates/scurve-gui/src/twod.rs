use egui::{
    self,
    epaint::{PathShape, Stroke},
};

use super::widgets;
use crate::{
    AppState,
    selection::SelectedCurve,
    snake::{fill_snake_segments, is_adjacent_2d, snake_mask_contains, snake_membership_mask},
    theme,
};

/// Render the 2D pane, including controls and the curve canvas.
pub fn show_2d_pane(
    ui: &mut egui::Ui,
    app_state: &mut AppState,
    selected_curve: &mut SelectedCurve,
    available_curves: &[&str],
    shared_settings: &mut crate::SharedSettings,
) {
    // Secondary control bar with lighter visual weight
    egui::Frame::new()
        .inner_margin(egui::Margin {
            left: theme::control_bar::PADDING_HORIZONTAL as i8,
            right: theme::control_bar::PADDING_HORIZONTAL as i8,
            top: theme::control_bar::PADDING_VERTICAL as i8,
            bottom: theme::control_bar::PADDING_VERTICAL as i8,
        })
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                // Use smaller, dimmer text for control labels
                ui.label(
                    egui::RichText::new("Curve:")
                        .size(theme::font_size::INFO)
                        .color(theme::TEXT_DIM),
                );
                widgets::curve_selector_combo(
                    ui,
                    &mut selected_curve.name,
                    available_curves,
                    "curve_selector",
                    &mut selected_curve.info_open,
                    2,
                    selected_curve.size,
                );

                ui.separator();

                ui.label(
                    egui::RichText::new("Size:")
                        .size(theme::font_size::INFO)
                        .color(theme::TEXT_DIM),
                );
                widgets::size_selector_2d(ui, &mut selected_curve.size, "size_selector");

                // Push pause and settings buttons to the far right
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    widgets::settings_dropdown(
                        ui,
                        &mut app_state.settings_dropdown_open,
                        &mut app_state.settings_dropdown_pos,
                        shared_settings,
                        false,
                    );
                    ui.add_space(theme::spacing::SMALL);
                    widgets::pause_play_button(ui, &mut app_state.paused);
                });
            });
        });

    ui.separator();

    draw_2d_canvas(ui, app_state, selected_curve, shared_settings);
}

/// Render the 2D drawing canvas and overlays.
fn draw_2d_canvas(
    ui: &mut egui::Ui,
    app_state: &mut AppState,
    selected_curve: &mut SelectedCurve,
    shared_settings: &crate::SharedSettings,
) {
    let bg = theme::CANVAS_BACKGROUND;
    let available_rect = ui.available_rect_before_wrap();
    let drawing_size = (available_rect.width().min(available_rect.height())
        * theme::canvas_2d::SIZE_FRACTION)
        .max(theme::canvas_2d::MIN_SIZE);
    let drawing_rect =
        egui::Rect::from_center_size(available_rect.center(), egui::Vec2::splat(drawing_size));
    app_state.last_canvas_rect = Some(drawing_rect);
    let painter = ui.painter_at(available_rect);
    painter.rect_filled(available_rect, 0.0, bg);

    let curve_size = selected_curve.size;
    let snake_offset = selected_curve.snake_offset;
    if let Some(curve_points) = selected_curve.ensure_cached_points() {
        let painter = ui.painter_at(drawing_rect);
        painter.rect_filled(drawing_rect, 5.0, bg);

        let margin = theme::canvas_2d::MARGIN;
        let inner_size = drawing_size - margin * 2.0;
        let scale = inner_size / (curve_size - 1) as f32;

        let screen_points = build_screen_points(curve_points, drawing_rect, scale, margin);

        let line_color = theme::curve_color_with_brightness(1.0, shared_settings.curve_opacity);
        let line_width = theme::canvas_2d::LINE_WIDTH;

        if shared_settings.curve_opacity > 0.0 && screen_points.len() > 1 {
            draw_main_curve_segments(
                &painter,
                curve_points,
                &screen_points,
                line_width,
                line_color,
                shared_settings.show_long_jumps,
            );
        }

        if shared_settings.snake_enabled && curve_points.len() > 1 {
            fill_snake_segments(
                &mut app_state.snake_segments_2d,
                snake_offset,
                shared_settings.snake_length,
                curve_points.len() as u32,
            );
            let snake_segments = &app_state.snake_segments_2d;

            let snake_mask: &[bool] = if shared_settings.show_long_jumps {
                &[]
            } else {
                snake_membership_mask(
                    snake_segments,
                    curve_points.len(),
                    &mut app_state.snake_mask_2d,
                )
            };

            let snake_color = theme::snake_color_with_brightness(1.0);
            let snake_width = line_width * theme::canvas_2d::SNAKE_WIDTH_MULTIPLIER;
            let snake_stroke = Stroke::new(snake_width, snake_color);

            draw_snake_overlay(
                &painter,
                curve_points,
                &screen_points,
                snake_segments,
                snake_mask,
                snake_stroke,
                shared_settings.show_long_jumps,
            );
        }
    }

    ui.allocate_rect(drawing_rect, egui::Sense::hover());
}

/// Convert integer curve points to screen positions within the drawing rect.
fn build_screen_points(
    curve_points: &[[u32; 2]],
    drawing_rect: egui::Rect,
    scale: f32,
    margin: f32,
) -> Vec<egui::Pos2> {
    curve_points
        .iter()
        .map(|p| egui::Pos2 {
            x: drawing_rect.min.x + margin + p[0] as f32 * scale,
            y: drawing_rect.min.y + margin + p[1] as f32 * scale,
        })
        .collect()
}

/// Draw the main curve segments and half‑segments for isolated nodes.
fn draw_main_curve_segments(
    painter: &egui::Painter,
    curve_points: &[[u32; 2]],
    screen_points: &[egui::Pos2],
    line_width: f32,
    line_color: egui::Color32,
    show_long_jumps: bool,
) {
    if show_long_jumps {
        painter.add(PathShape::line(
            screen_points.to_vec(),
            Stroke::new(line_width, line_color),
        ));
        return;
    }

    let mut run: Vec<egui::Pos2> = Vec::new();
    let stroke = Stroke::new(line_width, line_color);
    for i in 0..curve_points.len() - 1 {
        if is_adjacent_2d(&curve_points[i], &curve_points[i + 1]) {
            if run.is_empty() {
                run.push(screen_points[i]);
            }
            run.push(screen_points[i + 1]);
        } else if !run.is_empty() {
            if run.len() >= 2 {
                painter.add(PathShape::line(run.clone(), stroke));
            }
            run.clear();
        }
    }
    if !run.is_empty() && run.len() >= 2 {
        painter.add(PathShape::line(run, stroke));
    }

    for i in 0..curve_points.len() {
        let has_adjacent_prev = i > 0 && is_adjacent_2d(&curve_points[i - 1], &curve_points[i]);
        let has_adjacent_next =
            i < curve_points.len() - 1 && is_adjacent_2d(&curve_points[i], &curve_points[i + 1]);
        if !has_adjacent_prev && !has_adjacent_next {
            let current_pos = screen_points[i];
            let segment_end = if i == curve_points.len() - 1 && i > 0 {
                let prev_pos = screen_points[i - 1];
                egui::Pos2 {
                    x: current_pos.x + (current_pos.x - prev_pos.x) * 0.5,
                    y: current_pos.y + (current_pos.y - prev_pos.y) * 0.5,
                }
            } else if i < curve_points.len() - 1 {
                let next_pos = screen_points[i + 1];
                egui::Pos2 {
                    x: current_pos.x + (next_pos.x - current_pos.x) * 0.5,
                    y: current_pos.y + (next_pos.y - current_pos.y) * 0.5,
                }
            } else {
                continue;
            };
            painter.line_segment(
                [current_pos, segment_end],
                Stroke::new(line_width, line_color),
            );
        }
    }
}

/// Draw the animated snake overlay, honoring long‑jump visibility.
fn draw_snake_overlay(
    painter: &egui::Painter,
    curve_points: &[[u32; 2]],
    screen_points: &[egui::Pos2],
    snake_segments: &[usize],
    snake_mask: &[bool],
    snake_stroke: Stroke,
    show_long_jumps: bool,
) {
    if show_long_jumps {
        let mut snake_path = Vec::new();
        for &i in snake_segments {
            if i < screen_points.len() {
                snake_path.push(screen_points[i]);
            }
        }
        if snake_path.len() >= 2 {
            painter.add(PathShape::line(snake_path, snake_stroke));
        }
        return;
    }

    let mut current_run: Vec<egui::Pos2> = Vec::new();
    for &i in snake_segments {
        if i >= curve_points.len() {
            continue;
        }
        let has_prev = i > 0
            && snake_mask_contains(snake_mask, i - 1)
            && is_adjacent_2d(&curve_points[i - 1], &curve_points[i]);
        let has_next = i < curve_points.len() - 1
            && snake_mask_contains(snake_mask, i + 1)
            && is_adjacent_2d(&curve_points[i], &curve_points[i + 1]);
        if !has_prev && !has_next {
            // Isolated point handled below
        } else if !has_prev {
            current_run = vec![screen_points[i]];
        } else if has_prev && !current_run.is_empty() {
            current_run.push(screen_points[i]);
            if !has_next {
                if current_run.len() >= 2 {
                    painter.add(PathShape::line(current_run.clone(), snake_stroke));
                }
                current_run.clear();
            }
        }
    }
    if current_run.len() >= 2 {
        painter.add(PathShape::line(current_run, snake_stroke));
    }

    for &i in snake_segments {
        if i >= curve_points.len() {
            continue;
        }
        let has_prev = i > 0
            && snake_mask_contains(snake_mask, i - 1)
            && is_adjacent_2d(&curve_points[i - 1], &curve_points[i]);
        let has_next = i < curve_points.len() - 1
            && snake_mask_contains(snake_mask, i + 1)
            && is_adjacent_2d(&curve_points[i], &curve_points[i + 1]);
        if !has_prev && !has_next {
            let current_pos = screen_points[i];
            let segment_end = if i == curve_points.len() - 1 && i > 0 {
                let prev_pos = screen_points[i - 1];
                egui::Pos2 {
                    x: current_pos.x + (current_pos.x - prev_pos.x) * 0.5,
                    y: current_pos.y + (current_pos.y - prev_pos.y) * 0.5,
                }
            } else if i < curve_points.len() - 1 {
                let next_pos = screen_points[i + 1];
                egui::Pos2 {
                    x: current_pos.x + (next_pos.x - current_pos.x) * 0.5,
                    y: current_pos.y + (next_pos.y - current_pos.y) * 0.5,
                }
            } else {
                continue;
            };
            painter.line_segment([current_pos, segment_end], snake_stroke);
        }
    }
}
