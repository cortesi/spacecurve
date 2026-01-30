use egui::{
    self,
    epaint::{PathShape, Stroke},
};

use super::widgets;
use crate::{
    AppState,
    selection::SelectedCurve,
    snake::{fill_snake_segments, is_adjacent_2d, snake_membership_mask},
    theme::{self, curve_glow_color, curve_glow_color_alpha},
};

/// Render the 2D pane, including controls and the curve canvas.
pub fn show_2d_pane(
    ui: &mut egui::Ui,
    app_state: &mut AppState,
    render_cache: &mut crate::RenderCache,
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

    draw_2d_canvas(ui, render_cache, selected_curve, shared_settings);
}

/// Render the 2D drawing canvas and overlays.
fn draw_2d_canvas(
    ui: &mut egui::Ui,
    render_cache: &mut crate::RenderCache,
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
    render_cache.last_canvas_rect = Some(drawing_rect);
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

        build_screen_points(
            curve_points,
            drawing_rect,
            scale,
            margin,
            &mut render_cache.cache_2d_screen,
        );
        let screen_points = &render_cache.cache_2d_screen;

        let line_color = theme::curve_color_with_brightness(1.0, shared_settings.curve_opacity);
        let line_width = theme::canvas_2d::LINE_WIDTH;

        if shared_settings.curve_opacity > 0.0 && screen_points.len() > 1 {
            draw_main_curve_segments(
                &painter,
                curve_points,
                screen_points,
                line_width,
                line_color,
                shared_settings.curve_long_jumps,
                &mut render_cache.cache_2d_run,
            );
        }

        if shared_settings.snake_enabled && curve_points.len() > 1 {
            let curve_len = curve_points.len() as f32;
            let snake_len = ((shared_settings.snake_length / 100.0) * curve_len)
                .round()
                .max(1.0);

            // Calculate interpolated tail position
            // When we snap for a long jump, we update both segment and frac to the snapped position
            let tail_pos = snake_offset % curve_len;
            let raw_tail_segment = tail_pos.floor() as usize % curve_points.len();
            let raw_tail_frac = tail_pos.fract();
            let tail_next = (raw_tail_segment + 1) % curve_points.len();
            let tail_adjacent =
                is_adjacent_2d(&curve_points[raw_tail_segment], &curve_points[tail_next]);

            // Effective tail position: either interpolated or snapped to next point
            // When PARTWAY through a long jump with snake_long_jumps=false, skip to the end
            // (If exactly at a point, don't snap - the body naturally starts there)
            let (tail_segment, tail_frac, tail_screen) =
                if raw_tail_frac > 0.0 && !tail_adjacent && !shared_settings.snake_long_jumps {
                    // Long jump with snake_long_jumps=false: snap to END of segment
                    (tail_next, 0.0, screen_points[tail_next])
                } else if raw_tail_frac > 0.0 {
                    // Smooth interpolation
                    let p1 = screen_points[raw_tail_segment];
                    let p2 = screen_points[tail_next];
                    let interp = egui::pos2(
                        p1.x + (p2.x - p1.x) * raw_tail_frac,
                        p1.y + (p2.y - p1.y) * raw_tail_frac,
                    );
                    (raw_tail_segment, raw_tail_frac, interp)
                } else {
                    (raw_tail_segment, 0.0, screen_points[raw_tail_segment])
                };

            // Calculate interpolated head position
            let head_pos = (snake_offset + snake_len) % curve_len;
            let raw_head_segment = head_pos.floor() as usize % curve_points.len();
            let raw_head_frac = head_pos.fract();
            let head_next = (raw_head_segment + 1) % curve_points.len();
            let head_adjacent =
                is_adjacent_2d(&curve_points[raw_head_segment], &curve_points[head_next]);

            // Effective head position: either interpolated or snapped to next point
            // When PARTWAY through a long jump with snake_long_jumps=false, skip to the end
            // (If exactly at a point, don't snap - the body naturally ends there)
            let (head_segment, head_frac, head_screen) =
                if raw_head_frac > 0.0 && !head_adjacent && !shared_settings.snake_long_jumps {
                    // Long jump with snake_long_jumps=false: snap to END of segment
                    (head_next, 0.0, screen_points[head_next])
                } else if raw_head_frac > 0.0 {
                    // Smooth interpolation
                    let p1 = screen_points[raw_head_segment];
                    let p2 = screen_points[head_next];
                    let interp = egui::pos2(
                        p1.x + (p2.x - p1.x) * raw_head_frac,
                        p1.y + (p2.y - p1.y) * raw_head_frac,
                    );
                    (raw_head_segment, raw_head_frac, interp)
                } else {
                    (raw_head_segment, 0.0, screen_points[raw_head_segment])
                };

            let Ok(curve_length) = u64::try_from(curve_points.len()) else {
                return;
            };
            fill_snake_segments(
                &mut render_cache.snake_segments_2d,
                snake_offset,
                shared_settings.snake_length,
                curve_length,
            );
            let snake_segments = &render_cache.snake_segments_2d;

            let snake_mask: &[bool] = if shared_settings.snake_long_jumps {
                &[]
            } else {
                snake_membership_mask(
                    snake_segments,
                    curve_points.len(),
                    &mut render_cache.snake_mask_2d,
                )
            };

            let snake_color = theme::snake_color_with_brightness(1.0);
            let snake_width = line_width * theme::canvas_2d::SNAKE_WIDTH_MULTIPLIER;
            let snake_stroke = Stroke::new(snake_width, snake_color);

            draw_snake_overlay(
                &painter,
                curve_points,
                screen_points,
                snake_segments,
                snake_mask,
                snake_stroke,
                shared_settings.snake_long_jumps,
                &mut render_cache.cache_2d_run,
                tail_segment,
                tail_frac,
                tail_screen,
                head_segment,
                head_frac,
                head_screen,
            );

            // Draw glowing head marker at the front of the snake
            draw_head_marker_at(&painter, head_screen);
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
    out: &mut Vec<egui::Pos2>,
) {
    out.clear();
    out.reserve(curve_points.len());
    for p in curve_points {
        out.push(egui::Pos2 {
            x: drawing_rect.min.x + margin + p[0] as f32 * scale,
            y: drawing_rect.min.y + margin + p[1] as f32 * scale,
        });
    }
}

/// Draw the main curve segments and half‑segments for isolated nodes.
fn draw_main_curve_segments(
    painter: &egui::Painter,
    curve_points: &[[u32; 2]],
    screen_points: &[egui::Pos2],
    line_width: f32,
    line_color: egui::Color32,
    show_long_jumps: bool,
    run: &mut Vec<egui::Pos2>,
) {
    if show_long_jumps {
        painter.add(PathShape::line(
            screen_points.to_vec(),
            Stroke::new(line_width, line_color),
        ));
        return;
    }

    run.clear();
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
        painter.add(PathShape::line(run.clone(), stroke));
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

/// Draw the animated snake overlay with smooth interpolation at tail and head.
///
/// The snake path is built from `tail_screen` to `head_screen`, including all
/// intermediate integer points. This ensures smooth motion as the fractional
/// parts of tail and head advance.
#[allow(clippy::too_many_arguments, clippy::cognitive_complexity)]
fn draw_snake_overlay(
    painter: &egui::Painter,
    curve_points: &[[u32; 2]],
    screen_points: &[egui::Pos2],
    _snake_segments: &[usize],
    _snake_mask: &[bool],
    snake_stroke: Stroke,
    show_long_jumps: bool,
    current_run: &mut Vec<egui::Pos2>,
    tail_segment: usize,
    tail_frac: f32,
    tail_screen: egui::Pos2,
    head_segment: usize,
    head_frac: f32,
    head_screen: egui::Pos2,
) {
    let n = curve_points.len();
    if n < 2 {
        return;
    }

    // Build the list of integer point indices from tail to head.
    // The path goes: tail_screen -> point[first_int] -> ... -> point[last_int] -> head_screen
    // where first_int is the first integer point AFTER the tail interpolation position
    // and last_int is the last integer point BEFORE the head interpolation position.
    let first_int = if tail_frac > 0.0 {
        (tail_segment + 1) % n
    } else {
        tail_segment
    };
    let last_int = head_segment;

    // Collect all integer point indices from first_int to last_int (inclusive)
    let mut int_points: Vec<usize> = Vec::new();
    if first_int <= last_int {
        // No wrap-around
        for i in first_int..=last_int {
            int_points.push(i);
        }
    } else {
        // Wrap-around case: e.g., tail at index 62, head at index 3 (out of 64)
        for i in first_int..n {
            int_points.push(i);
        }
        for i in 0..=last_int {
            int_points.push(i);
        }
    }

    if show_long_jumps {
        // Build single continuous path
        let mut snake_path = Vec::with_capacity(int_points.len() + 2);

        // Start with interpolated tail (if it's not exactly at an integer point)
        if tail_frac > 0.0 {
            snake_path.push(tail_screen);
        }

        // Add all integer points
        for &i in &int_points {
            snake_path.push(screen_points[i]);
        }

        // End with interpolated head (if it's not exactly at an integer point)
        if head_frac > 0.0 {
            snake_path.push(head_screen);
        }

        if snake_path.len() >= 2 {
            painter.add(PathShape::line(snake_path, snake_stroke));
        }
        return;
    }

    // For show_long_jumps=false, build runs of adjacent segments
    current_run.clear();

    // Start the path
    if tail_frac > 0.0 {
        // Check if tail segment is adjacent (for interpolation decision)
        let tail_next = (tail_segment + 1) % n;
        let tail_adjacent = is_adjacent_2d(&curve_points[tail_segment], &curve_points[tail_next]);
        if tail_adjacent {
            current_run.push(tail_screen);
        }
        // If not adjacent, we'll start fresh at the first integer point
    }

    // Process all integer points
    for (idx, &i) in int_points.iter().enumerate() {
        let prev_i = if idx == 0 {
            if tail_frac > 0.0 {
                Some(tail_segment)
            } else {
                None
            }
        } else {
            Some(int_points[idx - 1])
        };

        let is_adjacent_to_prev =
            prev_i.is_some_and(|p| is_adjacent_2d(&curve_points[p], &curve_points[i]));

        if !is_adjacent_to_prev && !current_run.is_empty() {
            // End current run and start a new one
            if current_run.len() >= 2 {
                painter.add(PathShape::line(current_run.clone(), snake_stroke));
            }
            current_run.clear();
        }

        current_run.push(screen_points[i]);
    }

    // Finish with interpolated head
    if head_frac > 0.0 {
        let head_next = (head_segment + 1) % n;
        let head_adjacent = is_adjacent_2d(&curve_points[head_segment], &curve_points[head_next]);
        if head_adjacent && !current_run.is_empty() {
            current_run.push(head_screen);
        } else if !current_run.is_empty() {
            // Non-adjacent: end run at head_segment, don't add interpolated point
        }
    }

    // Draw final run
    if current_run.len() >= 2 {
        painter.add(PathShape::line(current_run.clone(), snake_stroke));
    }
}

/// Draw a glowing marker at the given screen position.
fn draw_head_marker_at(painter: &egui::Painter, pos: egui::Pos2) {
    let brightness = 1.0; // Full brightness in 2D (no depth)

    // Draw outer glow (larger, semi-transparent)
    let glow_radius = theme::canvas_3d::HEAD_MARKER_GLOW_RADIUS * (0.7 + 0.3 * brightness);
    let glow_color = curve_glow_color_alpha(brightness, theme::canvas_3d::HEAD_MARKER_GLOW_ALPHA);
    painter.circle_filled(pos, glow_radius, glow_color);

    // Draw inner core (smaller, solid)
    let core_radius = theme::canvas_3d::HEAD_MARKER_RADIUS * (0.7 + 0.3 * brightness);
    let core_color = curve_glow_color(brightness);
    painter.circle_filled(pos, core_radius, core_color);
}
