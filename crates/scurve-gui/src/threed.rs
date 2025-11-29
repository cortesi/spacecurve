use egui::{
    self,
    epaint::{PathShape, Stroke, Vertex},
};

// pattern_from_name used in caching method only; no direct use here
use super::{AppState, widgets};
use crate::{
    selection::Selected3DCurve,
    snake::{fill_snake_segments, is_adjacent_3d, snake_mask_contains, snake_membership_mask},
    theme::{
        self, canvas_3d::CAP_SHORTEN_FACTOR, curve_color_opaque, isolated_point_brightness,
        isolated_point_line_width, segment_brightness, segment_line_width,
        snake_color_with_brightness,
    },
};

/// Number of depth buckets for O(N) "sorting".
///
/// Instead of fully sorting 32k+ segments (O(N log N)), we bucket them into fixed depth slices.
/// All segments in a bucket share the same Z-depth for styling purposes, allowing us to
/// batch them into a single mesh. 128 bins provides smooth enough depth gradation that
/// the discrete steps are not noticeable.
const NUM_DEPTH_BINS: usize = 128;

/// Helper to tessellate a line segment into a mesh (as a simple quad).
///
/// We do this manually rather than using `painter.line_segment` to allow batching.
/// `egui`'s immediate mode painter handles thousands of individual line calls poorly,
/// as each one adds overhead. By manually pushing vertices to a single `Mesh`, we
/// reduce the overhead to essentially zero.
fn add_segment_to_mesh(
    mesh: &mut egui::Mesh,
    a: egui::Pos2,
    b: egui::Pos2,
    width: f32,
    color: egui::Color32,
    shorten_start: bool,
    shorten_end: bool,
) {
    let dx = b.x - a.x;
    let dy = b.y - a.y;
    let len_sq = dx * dx + dy * dy;
    if len_sq <= 0.000001 {
        return;
    }
    let len = len_sq.sqrt();

    let shorten = (width * CAP_SHORTEN_FACTOR).min(len * 0.25);
    let ux = dx / len;
    let uy = dy / len;

    let a2 = if shorten_start {
        egui::pos2(a.x + ux * shorten, a.y + uy * shorten)
    } else {
        a
    };
    let b2 = if shorten_end {
        egui::pos2(b.x - ux * shorten, b.y - uy * shorten)
    } else {
        b
    };

    // Normal vector for width expansion
    let nx = -uy * width * 0.5;
    let ny = ux * width * 0.5;

    let idx = mesh.vertices.len() as u32;

    // 0: a2 + normal
    mesh.vertices.push(Vertex {
        pos: egui::pos2(a2.x + nx, a2.y + ny),
        uv: egui::pos2(0.0, 0.0),
        color,
    });
    // 1: a2 - normal
    mesh.vertices.push(Vertex {
        pos: egui::pos2(a2.x - nx, a2.y - ny),
        uv: egui::pos2(0.0, 0.0),
        color,
    });
    // 2: b2 - normal
    mesh.vertices.push(Vertex {
        pos: egui::pos2(b2.x - nx, b2.y - ny),
        uv: egui::pos2(0.0, 0.0),
        color,
    });
    // 3: b2 + normal
    mesh.vertices.push(Vertex {
        pos: egui::pos2(b2.x + nx, b2.y + ny),
        uv: egui::pos2(0.0, 0.0),
        color,
    });

    // Triangle 1: 0-1-2
    mesh.indices.push(idx);
    mesh.indices.push(idx + 1);
    mesh.indices.push(idx + 2);

    // Triangle 2: 0-2-3
    mesh.indices.push(idx);
    mesh.indices.push(idx + 2);
    mesh.indices.push(idx + 3);
}

/// Helper for depth-sorted snake rendering in 3D.
struct SnakeDraw {
    /// Average depth used for painter ordering (smaller draws first).
    depth: f32,
    /// Stroke width for this segment/polyline.
    width: f32,
    /// Stroke color for this draw call.
    color: egui::Color32,
    /// Points to render (either a polyline or a single segment).
    points: Vec<egui::Pos2>,
    /// Optional cap-shortening flags for single segments.
    shorten: Option<(bool, bool)>,
}

/// Render the 3D pane, including controls and the curve canvas.
pub fn show_3d_pane(
    ui: &mut egui::Ui,
    app_state: &mut AppState,
    selected_3d_curve: &mut Selected3DCurve,
    available_curves: &[&str],
    shared_settings: &mut crate::SharedSettings,
) {
    // Repaints are requested conditionally from the app loop

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
                    &mut selected_3d_curve.name,
                    available_curves,
                    "3d_curve_selector",
                    &mut selected_3d_curve.info_open,
                    3,
                    selected_3d_curve.size,
                );

                ui.separator();

                ui.label(
                    egui::RichText::new("Size:")
                        .size(theme::font_size::INFO)
                        .color(theme::TEXT_DIM),
                );
                widgets::size_selector_3d(ui, &mut selected_3d_curve.size, "3d_size_selector");

                // Add pause button and settings on the right side of the controls
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    widgets::settings_dropdown(
                        ui,
                        &mut app_state.settings_dropdown_open,
                        &mut app_state.settings_dropdown_pos,
                        shared_settings,
                        true, // Include spin speed for 3D view
                    );
                    ui.add_space(theme::spacing::SMALL);
                    widgets::pause_play_button(ui, &mut app_state.paused);
                });
            });
        });

    ui.separator();

    let available_rect = ui.available_rect_before_wrap();
    app_state.last_canvas_rect = Some(available_rect);
    let bg = theme::CANVAS_BACKGROUND;
    let painter = ui.painter_at(available_rect);
    painter.rect_filled(available_rect, 0.0, bg);

    // Draw 3D space-filling curve using 2D painting, using cached points
    // Capture values that will be needed while we hold a borrow during caching
    let curve_size = selected_3d_curve.size;
    let snake_offset = selected_3d_curve.snake_offset;
    if let Some(points3d) = selected_3d_curve.ensure_cached_points() {
        draw_3d_space_curve(
            &painter,
            available_rect,
            app_state,
            shared_settings,
            points3d,
            curve_size,
            snake_offset,
        );
    }

    // Handle mouse interaction for manual rotation control
    let response = ui.allocate_rect(available_rect, egui::Sense::click_and_drag());

    if response.hovered() && ui.input(|i| i.pointer.primary_down()) {
        // Mouse button is down - pause rotation immediately
        if !app_state.mouse_dragging {
            app_state.mouse_dragging = true;
            app_state.last_mouse_x = response.interact_pointer_pos().unwrap_or_default().x;
        }

        // If dragging, apply manual rotation
        if response.dragged() {
            let current_mouse_x = response.interact_pointer_pos().unwrap_or_default().x;
            let delta_x = current_mouse_x - app_state.last_mouse_x;

            // Apply manual rotation (scale the mouse movement)
            app_state.rotation_angle += delta_x * theme::canvas_3d::DRAG_SENSITIVITY;
            app_state.last_mouse_x = current_mouse_x;
        }
    } else if app_state.mouse_dragging {
        // Mouse button released - resume automatic rotation
        app_state.mouse_dragging = false;
    }
}

/// Render the 3D curve and overlays into the given rect.
fn draw_3d_space_curve(
    painter: &egui::Painter,
    rect: egui::Rect,
    app_state: &mut AppState,
    shared_settings: &crate::SharedSettings,
    original_curve_points: &[[u32; 3]],
    curve_size: u32,
    snake_offset: f32,
) {
    let center = rect.center();
    let margin = theme::canvas_3d::MARGIN;
    let available_width = rect.width() - margin * 2.0;
    let available_height = rect.height() - margin * 2.0;
    let scale = (available_width.min(available_height) * theme::canvas_3d::SCALE_FACTOR)
        .max(theme::canvas_3d::MIN_SCALE);

    if original_curve_points.is_empty() {
        return;
    }

    let rotation_y = app_state.rotation_angle;
    let rotation_x = theme::canvas_3d::CAMERA_TILT;
    let (curve_points_3d, curve_points_2d) = project_points(
        original_curve_points,
        curve_size,
        rotation_x,
        rotation_y,
        center,
        scale,
    );

    let connected = compute_connected(original_curve_points);
    let shorten_caps = compute_shorten_caps(&connected);
    let segments_with_depth = build_segment_depths(
        &curve_points_3d,
        &connected,
        shared_settings.show_long_jumps,
    );
    // Sorted by depth binning inside draw_curve_segments

    draw_curve_segments(
        painter,
        &curve_points_2d,
        &segments_with_depth,
        &shorten_caps,
        shared_settings.curve_opacity,
    );

    if shared_settings.snake_enabled && curve_points_2d.len() > 1 {
        fill_snake_segments(
            &mut app_state.snake_segments_3d,
            snake_offset,
            shared_settings.snake_length,
            original_curve_points.len() as u32,
        );
        let snake_segments = &app_state.snake_segments_3d;

        let snake_mask: &[bool] = if shared_settings.show_long_jumps {
            &[]
        } else {
            snake_membership_mask(
                snake_segments,
                curve_points_2d.len(),
                &mut app_state.snake_mask_3d,
            )
        };
        let snake_included = snake_included_mask(
            snake_segments,
            &connected,
            shared_settings.show_long_jumps,
            &mut app_state.snake_included_3d,
        );
        let draws = collect_snake_draws(
            &curve_points_2d,
            &curve_points_3d,
            &connected,
            snake_included,
            &shorten_caps,
        );
        // Sorted by depth binning inside draw_snake_draws
        draw_snake_draws(painter, &draws);

        if !shared_settings.show_long_jumps {
            draw_isolated_snake_points(
                painter,
                original_curve_points,
                &curve_points_2d,
                &curve_points_3d,
                snake_segments,
                snake_mask,
            );
        }
    }

    if !shared_settings.show_long_jumps {
        draw_isolated_points(
            painter,
            original_curve_points,
            &curve_points_2d,
            &curve_points_3d,
        );
    }
}

/// Project integer 3D curve points into rotated 3D coordinates and 2D screen positions.
fn project_points(
    original: &[[u32; 3]],
    curve_size: u32,
    rotation_x: f32,
    rotation_y: f32,
    center: egui::Pos2,
    scale: f32,
) -> (Vec<[f32; 3]>, Vec<egui::Pos2>) {
    let mut pts3d = Vec::with_capacity(original.len());
    let mut pts2d = Vec::with_capacity(original.len());
    for p in original.iter() {
        let x = (p[0] as f32 / (curve_size - 1) as f32) * 2.0 - 1.0;
        let y = (p[1] as f32 / (curve_size - 1) as f32) * 2.0 - 1.0;
        let z = (p[2] as f32 / (curve_size - 1) as f32) * 2.0 - 1.0;
        let x_rot = x * rotation_y.cos() + z * rotation_y.sin();
        let z_rot = -x * rotation_y.sin() + z * rotation_y.cos();
        let y_tilt = y * rotation_x.cos() - z_rot * rotation_x.sin();
        let z_tilt = y * rotation_x.sin() + z_rot * rotation_x.cos();
        pts3d.push([x_rot, y_tilt, z_tilt]);
        let depth = theme::canvas_3d::PERSPECTIVE_DISTANCE - z_tilt;
        let perspective_scale = theme::canvas_3d::PERSPECTIVE_DISTANCE / depth;
        let screen_x = center.x + x_rot * scale * perspective_scale;
        let screen_y = center.y - y_tilt * scale * perspective_scale;
        pts2d.push(egui::Pos2::new(screen_x, screen_y));
    }
    (pts3d, pts2d)
}

/// Compute whether successive 3D points are adjacent (Manhattan distance <= 1).
fn compute_connected(original: &[[u32; 3]]) -> Vec<bool> {
    if original.len() < 2 {
        return Vec::new();
    }
    let last_seg_idx = original.len() - 2;
    let mut connected = Vec::with_capacity(last_seg_idx + 1);
    for i in 0..=last_seg_idx {
        connected.push(is_adjacent_3d(&original[i], &original[i + 1]));
    }
    connected
}

/// For each segment, decide whether to shorten start/end caps at exposed ends.
fn compute_shorten_caps(connected: &[bool]) -> Vec<(bool, bool)> {
    if connected.is_empty() {
        return Vec::new();
    }
    let last = connected.len() - 1;
    let mut caps = Vec::with_capacity(connected.len());
    for i in 0..=last {
        let prev_conn = if i == 0 { false } else { connected[i - 1] };
        let next_conn = if i == last { false } else { connected[i + 1] };
        caps.push((!prev_conn, !next_conn));
    }
    caps
}

/// Build a list of segment indices with their average depth for painter sorting.
fn build_segment_depths(
    pts3d: &[[f32; 3]],
    connected: &[bool],
    show_long_jumps: bool,
) -> Vec<(usize, f32)> {
    let mut segs = Vec::with_capacity(connected.len());
    for i in 0..connected.len() {
        let start_depth = pts3d[i][2];
        let end_depth = pts3d[i + 1][2];
        let avg_depth = (start_depth + end_depth) / 2.0;
        if show_long_jumps || connected[i] {
            segs.push((i, avg_depth));
        }
    }
    segs
}

/// Draw depth‑sorted curve segments using depth binning.
///
/// This function implements the core optimization:
/// 1. **Binning**: Distribute segments into `NUM_DEPTH_BINS` buckets based on depth.
/// 2. **Batching**: For each bin, generate a single `egui::Mesh` containing all segments.
///
/// This reduces the number of draw calls from O(N) (e.g., 32,000) to O(BINS) (128),
/// providing a massive performance boost.
fn draw_curve_segments(
    painter: &egui::Painter,
    pts2d: &[egui::Pos2],
    segments_with_depth: &[(usize, f32)],
    shorten_caps: &[(bool, bool)],
    opacity: f32,
) {
    if opacity <= 0.0 {
        return;
    }

    let mut bins: Vec<Vec<usize>> = vec![Vec::new(); NUM_DEPTH_BINS];
    for (i, depth) in segments_with_depth {
        let normalized = theme::normalize_depth(*depth);
        let bin_idx = (normalized * (NUM_DEPTH_BINS as f32 - 1.0)).round() as usize;
        if bin_idx < NUM_DEPTH_BINS {
            bins[bin_idx].push(*i);
        }
    }

    for (bin_idx, bin) in bins.iter().enumerate() {
        if bin.is_empty() {
            continue;
        }
        // Use the bin center to determine style for all segments in this bin
        let normalized_depth = bin_idx as f32 / (NUM_DEPTH_BINS as f32 - 1.0);
        let depth = theme::canvas_3d::DEPTH_MIN
            + normalized_depth * (theme::canvas_3d::DEPTH_MAX - theme::canvas_3d::DEPTH_MIN);
        let brightness = theme::segment_brightness(depth);
        let line_width = theme::segment_line_width(brightness);
        let color = theme::curve_color_with_brightness(brightness, opacity);
        // Stroke not needed for mesh, just width and color

        let mut mesh = egui::Mesh::default();

        for &i in bin {
            let start_pos = pts2d[i];
            let end_pos = pts2d[i + 1];
            let (shorten_start, shorten_end) = shorten_caps[i];
            add_segment_to_mesh(
                &mut mesh,
                start_pos,
                end_pos,
                line_width,
                color,
                shorten_start,
                shorten_end,
            );
        }

        if !mesh.vertices.is_empty() {
            painter.add(egui::Shape::Mesh(mesh.into()));
        }
    }
}

/// Build a membership mask for snake segments that should be included given visibility rules.
fn snake_included_mask<'a>(
    snake_segments: &[usize],
    connected: &[bool],
    show_long_jumps: bool,
    scratch: &'a mut Vec<bool>,
) -> &'a [bool] {
    let len = connected.len();
    if scratch.len() < len {
        scratch.resize(len, false);
    } else {
        scratch[..len].fill(false);
    }

    for &i in snake_segments {
        if i < len && (show_long_jumps || connected[i]) {
            scratch[i] = true;
        }
    }

    &scratch[..len]
}

/// Turn included snake segments into depth‑sortable draw primitives.
fn collect_snake_draws(
    pts2d: &[egui::Pos2],
    pts3d: &[[f32; 3]],
    connected: &[bool],
    snake_included: &[bool],
    shorten_caps: &[(bool, bool)],
) -> Vec<SnakeDraw> {
    let mut draws = Vec::new();
    let nsegs = connected.len();
    let mut i = 0usize;
    while i < nsegs {
        if snake_mask_contains(snake_included, i) && connected[i] {
            let mut pts: Vec<egui::Pos2> = Vec::new();
            pts.push(pts2d[i]);
            let mut j = i;
            while j < nsegs && snake_mask_contains(snake_included, j) && connected[j] {
                pts.push(pts2d[j + 1]);
                j += 1;
            }
            let mut sum = 0.0f32;
            let mut cnt = 0usize;
            for k in i..j {
                sum += (pts3d[k][2] + pts3d[k + 1][2]) / 2.0;
                cnt += 1;
            }
            let avg_depth = if cnt > 0 { sum / cnt as f32 } else { 0.0 };
            let brightness = segment_brightness(avg_depth);
            draws.push(SnakeDraw {
                depth: avg_depth,
                width: segment_line_width(brightness),
                color: snake_color_with_brightness(brightness),
                points: pts,
                shorten: None,
            });
            i = j;
        } else {
            if snake_mask_contains(snake_included, i) {
                let avg_depth = (pts3d[i][2] + pts3d[i + 1][2]) / 2.0;
                let brightness = segment_brightness(avg_depth);
                draws.push(SnakeDraw {
                    depth: avg_depth,
                    width: segment_line_width(brightness),
                    color: snake_color_with_brightness(brightness),
                    points: vec![pts2d[i], pts2d[i + 1]],
                    shorten: Some(shorten_caps[i]),
                });
            }
            i += 1;
        }
    }
    draws
}

/// Render snake primitives with proper cap handling using depth binning.
///
/// Similar to `draw_curve_segments`, this batches the snake segments into meshes
/// to minimize draw calls. Continuous polyline paths (length >= 3) are still drawn
/// as paths because they are already efficient, but isolated segments are batched.
fn draw_snake_draws(painter: &egui::Painter, draws: &[SnakeDraw]) {
    let mut bins: Vec<Vec<&SnakeDraw>> = vec![Vec::new(); NUM_DEPTH_BINS];

    for d in draws {
        let normalized = theme::normalize_depth(d.depth);
        let bin_idx = (normalized * (NUM_DEPTH_BINS as f32 - 1.0)).round() as usize;
        if bin_idx < NUM_DEPTH_BINS {
            bins[bin_idx].push(d);
        }
    }

    for bin in bins {
        let mut mesh = egui::Mesh::default();

        for d in bin {
            if d.points.len() >= 3 {
                painter.add(PathShape::line(
                    d.points.clone(),
                    Stroke::new(d.width, d.color),
                ));
            } else if d.points.len() == 2 {
                let (shorten_start, shorten_end) = d.shorten.unwrap_or((false, false));
                add_segment_to_mesh(
                    &mut mesh,
                    d.points[0],
                    d.points[1],
                    d.width,
                    d.color,
                    shorten_start,
                    shorten_end,
                );
            }
        }

        if !mesh.vertices.is_empty() {
            painter.add(egui::Shape::Mesh(mesh.into()));
        }
    }
}

/// Draw half‑segments for isolated snake nodes when long jumps are hidden.
fn draw_isolated_snake_points(
    painter: &egui::Painter,
    original: &[[u32; 3]],
    pts2d: &[egui::Pos2],
    pts3d: &[[f32; 3]],
    snake_segments: &[usize],
    snake_mask: &[bool],
) {
    let mut isolated = Vec::new();
    for &idx in snake_segments {
        if idx < original.len() {
            let has_adjacent_prev = idx > 0
                && snake_mask_contains(snake_mask, idx - 1)
                && is_adjacent_3d(&original[idx - 1], &original[idx]);
            let has_adjacent_next = idx < original.len() - 1
                && snake_mask_contains(snake_mask, idx + 1)
                && is_adjacent_3d(&original[idx], &original[idx + 1]);
            if !has_adjacent_prev && !has_adjacent_next {
                isolated.push((idx, pts3d[idx][2]));
            }
        }
    }
    isolated.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
    for (i, depth) in isolated.iter() {
        let current_pos = pts2d[*i];
        let segment_end = if *i == pts2d.len() - 1 && *i > 0 {
            let prev_pos = pts2d[*i - 1];
            egui::Pos2 {
                x: current_pos.x + (current_pos.x - prev_pos.x) * 0.5,
                y: current_pos.y + (current_pos.y - prev_pos.y) * 0.5,
            }
        } else if *i < pts2d.len() - 1 {
            let next_pos = pts2d[*i + 1];
            egui::Pos2 {
                x: current_pos.x + (next_pos.x - current_pos.x) * 0.5,
                y: current_pos.y + (next_pos.y - current_pos.y) * 0.5,
            }
        } else {
            continue;
        };
        let brightness = isolated_point_brightness(*depth);
        let line_width = isolated_point_line_width(brightness);
        let color = snake_color_with_brightness(brightness);
        painter.line_segment([current_pos, segment_end], Stroke::new(line_width, color));
    }
}

/// Draw half‑segments for isolated curve nodes when long jumps are hidden.
fn draw_isolated_points(
    painter: &egui::Painter,
    original: &[[u32; 3]],
    pts2d: &[egui::Pos2],
    pts3d: &[[f32; 3]],
) {
    let mut iso = Vec::new();
    for i in 0..original.len() {
        let has_adjacent_prev = i > 0 && is_adjacent_3d(&original[i - 1], &original[i]);
        let has_adjacent_next =
            i < original.len() - 1 && is_adjacent_3d(&original[i], &original[i + 1]);
        if !has_adjacent_prev && !has_adjacent_next {
            iso.push((i, pts3d[i][2]));
        }
    }
    iso.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
    for (i, depth) in iso.iter() {
        let current_pos = pts2d[*i];
        let segment_end = if *i == pts2d.len() - 1 && *i > 0 {
            let prev_pos = pts2d[*i - 1];
            egui::Pos2 {
                x: current_pos.x + (current_pos.x - prev_pos.x) * 0.5,
                y: current_pos.y + (current_pos.y - prev_pos.y) * 0.5,
            }
        } else if *i < pts2d.len() - 1 {
            let next_pos = pts2d[*i + 1];
            egui::Pos2 {
                x: current_pos.x + (next_pos.x - current_pos.x) * 0.5,
                y: current_pos.y + (next_pos.y - current_pos.y) * 0.5,
            }
        } else {
            continue;
        };
        let brightness = isolated_point_brightness(*depth);
        let line_width = isolated_point_line_width(brightness);
        let color = curve_color_opaque(brightness);
        painter.line_segment([current_pos, segment_end], Stroke::new(line_width, color));
    }
}
