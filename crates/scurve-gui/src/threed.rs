use egui::{
    self,
    epaint::{PathShape, Stroke, Vertex},
};
use eguidev::{WidgetMeta, WidgetRole, WidgetRoleMeta};

// pattern_from_name used in caching method only; no direct use here
use super::{AppState, widgets};
use crate::{
    selection::Selected3DCurve,
    snake::{fill_snake_segments, is_adjacent_3d, snake_mask_contains, snake_membership_mask},
    theme::{
        self, canvas_3d::CAP_SHORTEN_FACTOR, curve_color_opaque, curve_glow_color,
        curve_glow_color_alpha, isolated_point_brightness, isolated_point_line_width,
        segment_brightness, segment_line_width, snake_color_with_brightness,
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
    render_cache: &mut crate::RenderCache,
    selected_3d_curve: &mut Selected3DCurve,
    available_curves: &[&str],
    shared_settings: &mut crate::SharedSettings,
) {
    eguidev::container(ui, "pane.3d", |ui| {
        egui::Frame::new()
            .inner_margin(egui::Margin {
                left: theme::control_bar::PADDING_HORIZONTAL as i8,
                right: theme::control_bar::PADDING_HORIZONTAL as i8,
                top: theme::control_bar::PADDING_VERTICAL as i8,
                bottom: theme::control_bar::PADDING_VERTICAL as i8,
            })
            .show(ui, |ui| {
                eguidev::container(ui, "pane.3d.toolbar", |ui| {
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new("Curve:")
                                .size(theme::font_size::INFO)
                                .color(theme::TEXT_DIM),
                        );
                        widgets::curve_selector_combo(
                            ui,
                            &mut selected_3d_curve.name,
                            available_curves,
                            widgets::CurveSelectorConfig {
                                widget_id: "pane.3d.curve",
                                id_salt: "3d_curve_selector",
                                info_open: &mut selected_3d_curve.info_open,
                                dim: 3,
                                size: selected_3d_curve.size,
                            },
                        );

                        ui.separator();

                        ui.label(
                            egui::RichText::new("Size:")
                                .size(theme::font_size::INFO)
                                .color(theme::TEXT_DIM),
                        );
                        widgets::size_selector_3d(
                            ui,
                            &mut selected_3d_curve.size,
                            "3d_size_selector",
                            "pane.3d.size",
                        );

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            widgets::settings_dropdown(
                                ui,
                                &mut app_state.settings_dropdown_open,
                                &mut app_state.settings_dropdown_pos,
                                shared_settings,
                                true,
                                "pane.3d.settings.toggle",
                            );
                            ui.add_space(theme::spacing::SMALL);
                            widgets::pause_play_button(ui, &mut app_state.paused, "pane.3d.pause");
                        });
                    });
                });
            });

        ui.separator();

        let available_rect = ui.available_rect_before_wrap();
        render_cache.last_canvas_rect = Some(available_rect);
        let bg = theme::CANVAS_BACKGROUND;
        let painter = ui.painter_at(available_rect);
        painter.rect_filled(available_rect, 0.0, bg);

        let curve_size = selected_3d_curve.size;
        let snake_offset = selected_3d_curve.snake_offset;
        if let Some(points3d) = selected_3d_curve.ensure_cached_points() {
            draw_3d_space_curve(
                &painter,
                available_rect,
                app_state,
                render_cache,
                shared_settings,
                points3d,
                curve_size,
                snake_offset,
            );
        }

        let response = ui.allocate_rect(available_rect, egui::Sense::click_and_drag());
        eguidev::track_response(
            "pane.3d.canvas",
            &response,
            WidgetMeta {
                role: WidgetRoleMeta::Plain(WidgetRole::Unknown),
                label: Some("3D canvas".to_string()),
                visible: true,
                ..Default::default()
            },
        );

        if response.hovered() && ui.input(|i| i.pointer.primary_down()) {
            if !app_state.mouse_dragging {
                app_state.mouse_dragging = true;
                app_state.last_mouse_x = response.interact_pointer_pos().unwrap_or_default().x;
            }

            if response.dragged() {
                let current_mouse_x = response.interact_pointer_pos().unwrap_or_default().x;
                let delta_x = current_mouse_x - app_state.last_mouse_x;
                app_state.rotation_angle += delta_x * theme::canvas_3d::DRAG_SENSITIVITY;
                app_state.last_mouse_x = current_mouse_x;
            }
        } else if app_state.mouse_dragging {
            app_state.mouse_dragging = false;
        }
    });
}

/// Render the 3D curve and overlays into the given rect.
#[allow(clippy::too_many_arguments)]
fn draw_3d_space_curve(
    painter: &egui::Painter,
    rect: egui::Rect,
    app_state: &AppState,
    render_cache: &mut crate::RenderCache,
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

    // Use cached buffers
    project_points(
        original_curve_points,
        curve_size,
        rotation_x,
        rotation_y,
        center,
        scale,
        &mut render_cache.cache_3d_points,
        &mut render_cache.cache_3d_screen,
    );

    compute_connected(original_curve_points, &mut render_cache.cache_connected);
    compute_shorten_caps(&render_cache.cache_connected, &mut render_cache.cache_caps);
    build_segment_depths(
        &render_cache.cache_3d_points,
        &render_cache.cache_connected,
        shared_settings.curve_long_jumps,
        &mut render_cache.cache_depths,
    );

    // Sorted by depth binning inside draw_curve_segments
    draw_curve_segments(
        painter,
        &render_cache.cache_3d_screen,
        &render_cache.cache_depths,
        &render_cache.cache_caps,
        shared_settings.curve_opacity,
        &mut render_cache.cache_bins,
    );

    if shared_settings.snake_enabled && render_cache.cache_3d_screen.len() > 1 {
        let curve_len = original_curve_points.len() as f32;
        let snake_len = ((shared_settings.snake_length / 100.0) * curve_len)
            .round()
            .max(1.0);

        // Calculate interpolated tail position
        // When we snap for a long jump, we update both segment and frac to the snapped position
        let tail_pos = snake_offset % curve_len;
        let raw_tail_segment = tail_pos.floor() as usize % original_curve_points.len();
        let raw_tail_frac = tail_pos.fract();
        let tail_next = (raw_tail_segment + 1) % original_curve_points.len();
        let tail_adjacent = is_adjacent_3d(
            &original_curve_points[raw_tail_segment],
            &original_curve_points[tail_next],
        );

        // Effective tail position: either interpolated or snapped to next point
        // When PARTWAY through a long jump with snake_long_jumps=false, skip to the end
        // (If exactly at a point, don't snap - the body naturally starts there)
        let (tail_segment, tail_frac, tail_screen, tail_depth) =
            if raw_tail_frac > 0.0 && !tail_adjacent && !shared_settings.snake_long_jumps {
                // Long jump with snake_long_jumps=false: snap to END of segment
                (
                    tail_next,
                    0.0,
                    render_cache.cache_3d_screen[tail_next],
                    render_cache.cache_3d_points[tail_next][2],
                )
            } else if raw_tail_frac > 0.0 {
                // Smooth interpolation
                let p1 = render_cache.cache_3d_screen[raw_tail_segment];
                let p2 = render_cache.cache_3d_screen[tail_next];
                let d1 = render_cache.cache_3d_points[raw_tail_segment][2];
                let d2 = render_cache.cache_3d_points[tail_next][2];
                let interp = egui::pos2(
                    p1.x + (p2.x - p1.x) * raw_tail_frac,
                    p1.y + (p2.y - p1.y) * raw_tail_frac,
                );
                let depth_interp = d1 + (d2 - d1) * raw_tail_frac;
                (raw_tail_segment, raw_tail_frac, interp, depth_interp)
            } else {
                (
                    raw_tail_segment,
                    0.0,
                    render_cache.cache_3d_screen[raw_tail_segment],
                    render_cache.cache_3d_points[raw_tail_segment][2],
                )
            };

        // Calculate interpolated head position
        let head_pos = (snake_offset + snake_len) % curve_len;
        let raw_head_segment = head_pos.floor() as usize % original_curve_points.len();
        let raw_head_frac = head_pos.fract();
        let head_next = (raw_head_segment + 1) % original_curve_points.len();
        let head_adjacent = is_adjacent_3d(
            &original_curve_points[raw_head_segment],
            &original_curve_points[head_next],
        );

        // Effective head position: either interpolated or snapped to next point
        // When PARTWAY through a long jump with snake_long_jumps=false, skip to the end
        // (If exactly at a point, don't snap - the body naturally ends there)
        let (head_segment, head_frac, head_screen, head_depth) =
            if raw_head_frac > 0.0 && !head_adjacent && !shared_settings.snake_long_jumps {
                // Long jump with snake_long_jumps=false: snap to END of segment
                (
                    head_next,
                    0.0,
                    render_cache.cache_3d_screen[head_next],
                    render_cache.cache_3d_points[head_next][2],
                )
            } else if raw_head_frac > 0.0 {
                // Smooth interpolation
                let p1 = render_cache.cache_3d_screen[raw_head_segment];
                let p2 = render_cache.cache_3d_screen[head_next];
                let d1 = render_cache.cache_3d_points[raw_head_segment][2];
                let d2 = render_cache.cache_3d_points[head_next][2];
                let interp = egui::pos2(
                    p1.x + (p2.x - p1.x) * raw_head_frac,
                    p1.y + (p2.y - p1.y) * raw_head_frac,
                );
                let depth_interp = d1 + (d2 - d1) * raw_head_frac;
                (raw_head_segment, raw_head_frac, interp, depth_interp)
            } else {
                (
                    raw_head_segment,
                    0.0,
                    render_cache.cache_3d_screen[raw_head_segment],
                    render_cache.cache_3d_points[raw_head_segment][2],
                )
            };

        let Ok(curve_length) = u64::try_from(original_curve_points.len()) else {
            return;
        };
        fill_snake_segments(
            &mut render_cache.snake_segments_3d,
            snake_offset,
            shared_settings.snake_length,
            curve_length,
        );
        let snake_segments = &render_cache.snake_segments_3d;

        let snake_mask: &[bool] = if shared_settings.snake_long_jumps {
            &[]
        } else {
            snake_membership_mask(
                snake_segments,
                render_cache.cache_3d_screen.len(),
                &mut render_cache.snake_mask_3d,
            )
        };
        let snake_included = snake_included_mask(
            snake_segments,
            &render_cache.cache_connected,
            shared_settings.snake_long_jumps,
            &mut render_cache.snake_included_3d,
        );
        let draws = collect_snake_draws(
            &render_cache.cache_3d_screen,
            &render_cache.cache_3d_points,
            &render_cache.cache_connected,
            snake_included,
            &render_cache.cache_caps,
            snake_segments,
            shared_settings.snake_long_jumps,
            tail_segment,
            tail_frac,
            tail_screen,
            tail_depth,
            head_segment,
            head_frac,
            head_screen,
            head_depth,
        );
        // Sorted by depth binning inside draw_snake_draws
        draw_snake_draws(painter, &draws, &mut render_cache.cache_bins);

        if !shared_settings.snake_long_jumps {
            draw_isolated_snake_points(
                painter,
                original_curve_points,
                &render_cache.cache_3d_screen,
                &render_cache.cache_3d_points,
                snake_segments,
                snake_mask,
            );
        }

        // Draw glowing head marker
        draw_head_marker_at(painter, head_screen, head_depth);
    }

    if !shared_settings.curve_long_jumps {
        draw_isolated_points(
            painter,
            original_curve_points,
            &render_cache.cache_3d_screen,
            &render_cache.cache_3d_points,
        );
    }
}

/// Project integer 3D curve points into rotated 3D coordinates and 2D screen positions.
#[allow(clippy::too_many_arguments)]
fn project_points(
    original: &[[u32; 3]],
    curve_size: u32,
    rotation_x: f32,
    rotation_y: f32,
    center: egui::Pos2,
    scale: f32,
    pts3d: &mut Vec<[f32; 3]>,
    pts2d: &mut Vec<egui::Pos2>,
) {
    pts3d.clear();
    pts2d.clear();
    pts3d.reserve(original.len());
    pts2d.reserve(original.len());

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
}

/// Compute whether successive 3D points are adjacent (Manhattan distance <= 1).
fn compute_connected(original: &[[u32; 3]], connected: &mut Vec<bool>) {
    connected.clear();
    if original.len() < 2 {
        return;
    }
    let last_seg_idx = original.len() - 2;
    connected.reserve(last_seg_idx + 1);
    for i in 0..=last_seg_idx {
        connected.push(is_adjacent_3d(&original[i], &original[i + 1]));
    }
}

/// For each segment, decide whether to shorten start/end caps at exposed ends.
fn compute_shorten_caps(connected: &[bool], caps: &mut Vec<(bool, bool)>) {
    caps.clear();
    if connected.is_empty() {
        return;
    }
    let last = connected.len() - 1;
    caps.reserve(connected.len());
    for i in 0..=last {
        let prev_conn = if i == 0 { false } else { connected[i - 1] };
        let next_conn = if i == last { false } else { connected[i + 1] };
        caps.push((!prev_conn, !next_conn));
    }
}

/// Build a list of segment indices with their average depth for painter sorting.
fn build_segment_depths(
    pts3d: &[[f32; 3]],
    connected: &[bool],
    show_long_jumps: bool,
    segs: &mut Vec<(usize, f32)>,
) {
    segs.clear();
    segs.reserve(connected.len());
    for i in 0..connected.len() {
        let start_depth = pts3d[i][2];
        let end_depth = pts3d[i + 1][2];
        let avg_depth = (start_depth + end_depth) / 2.0;
        if show_long_jumps || connected[i] {
            segs.push((i, avg_depth));
        }
    }
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
    bins: &mut [Vec<usize>],
) {
    if opacity <= 0.0 {
        return;
    }

    for bin in bins.iter_mut() {
        bin.clear();
    }

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

/// Turn snake segments into depth‑sortable draw primitives with interpolated endpoints.
///
/// The snake path is built from `tail_screen` to `head_screen`, including all
/// intermediate integer points. This ensures smooth motion as the fractional
/// parts of tail and head advance.
#[allow(clippy::too_many_arguments)]
fn collect_snake_draws(
    pts2d: &[egui::Pos2],
    pts3d: &[[f32; 3]],
    connected: &[bool],
    _snake_included: &[bool],
    _shorten_caps: &[(bool, bool)],
    _snake_segments: &[usize],
    snake_long_jumps: bool,
    tail_segment: usize,
    tail_frac: f32,
    tail_screen: egui::Pos2,
    tail_depth: f32,
    head_segment: usize,
    head_frac: f32,
    head_screen: egui::Pos2,
    head_depth: f32,
) -> Vec<SnakeDraw> {
    let mut draws = Vec::new();
    let n = pts2d.len();
    if n < 2 {
        return draws;
    }

    // Build the list of integer point indices from tail to head.
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
        // Wrap-around case
        for i in first_int..n {
            int_points.push(i);
        }
        for i in 0..=last_int {
            int_points.push(i);
        }
    }

    if snake_long_jumps {
        // Build a single continuous path including long jumps
        let mut snake_pts: Vec<egui::Pos2> = Vec::with_capacity(int_points.len() + 2);
        let mut snake_depths: Vec<f32> = Vec::with_capacity(int_points.len() + 2);

        // Start with interpolated tail (if it's not exactly at an integer point)
        if tail_frac > 0.0 {
            snake_pts.push(tail_screen);
            snake_depths.push(tail_depth);
        }

        // Add all integer points
        for &i in &int_points {
            snake_pts.push(pts2d[i]);
            snake_depths.push(pts3d[i][2]);
        }

        // End with interpolated head (if it's not exactly at an integer point)
        if head_frac > 0.0 {
            snake_pts.push(head_screen);
            snake_depths.push(head_depth);
        }

        if snake_pts.len() >= 2 {
            let avg_depth: f32 = snake_depths.iter().sum::<f32>() / snake_depths.len() as f32;
            let brightness = segment_brightness(avg_depth);
            draws.push(SnakeDraw {
                depth: avg_depth,
                width: segment_line_width(brightness),
                color: snake_color_with_brightness(brightness),
                points: snake_pts,
                shorten: None,
            });
        }

        return draws;
    }

    // For snake_long_jumps=false, build runs of connected segments
    let mut current_pts: Vec<egui::Pos2> = Vec::new();
    let mut current_depths: Vec<f32> = Vec::new();

    // Start the path
    if tail_frac > 0.0 {
        // Check if tail segment is adjacent (for interpolation decision)
        let tail_adjacent = connected.get(tail_segment).copied().unwrap_or(false);
        if tail_adjacent {
            current_pts.push(tail_screen);
            current_depths.push(tail_depth);
        }
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

        // Check adjacency using the connected array
        let actually_adjacent = if let Some(p) = prev_i {
            if p < n - 1 && p + 1 == i {
                connected.get(p).copied().unwrap_or(false)
            } else if i < n - 1 && i + 1 == p {
                connected.get(i).copied().unwrap_or(false)
            } else {
                false
            }
        } else {
            false
        };

        if !actually_adjacent && !current_pts.is_empty() {
            // End current run and start a new one
            if current_pts.len() >= 2 {
                let avg_depth: f32 =
                    current_depths.iter().sum::<f32>() / current_depths.len() as f32;
                let brightness = segment_brightness(avg_depth);
                draws.push(SnakeDraw {
                    depth: avg_depth,
                    width: segment_line_width(brightness),
                    color: snake_color_with_brightness(brightness),
                    points: current_pts.clone(),
                    shorten: None,
                });
            }
            current_pts.clear();
            current_depths.clear();
        }

        current_pts.push(pts2d[i]);
        current_depths.push(pts3d[i][2]);
    }

    // Finish with interpolated head
    if head_frac > 0.0 {
        let head_adjacent = connected.get(head_segment).copied().unwrap_or(false);
        if head_adjacent && !current_pts.is_empty() {
            current_pts.push(head_screen);
            current_depths.push(head_depth);
        }
    }

    // Finalize last run
    if current_pts.len() >= 2 {
        let avg_depth: f32 = current_depths.iter().sum::<f32>() / current_depths.len() as f32;
        let brightness = segment_brightness(avg_depth);
        draws.push(SnakeDraw {
            depth: avg_depth,
            width: segment_line_width(brightness),
            color: snake_color_with_brightness(brightness),
            points: current_pts,
            shorten: None,
        });
    }

    draws
}

/// Render snake primitives with proper cap handling using depth binning.
///
/// Similar to `draw_curve_segments`, this batches the snake segments into meshes
/// to minimize draw calls. Continuous polyline paths (length >= 3) are still drawn
/// as paths because they are already efficient, but isolated segments are batched.
fn draw_snake_draws(painter: &egui::Painter, draws: &[SnakeDraw], bins: &mut [Vec<usize>]) {
    for bin in bins.iter_mut() {
        bin.clear();
    }

    for (i, d) in draws.iter().enumerate() {
        let normalized = theme::normalize_depth(d.depth);
        let bin_idx = (normalized * (NUM_DEPTH_BINS as f32 - 1.0)).round() as usize;
        if bin_idx < NUM_DEPTH_BINS {
            bins[bin_idx].push(i);
        }
    }

    for bin in bins {
        let mut mesh = egui::Mesh::default();

        for &i in bin.iter() {
            let d = &draws[i];
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

/// Draw a glowing marker at the given screen position with depth-based brightness.
fn draw_head_marker_at(painter: &egui::Painter, pos: egui::Pos2, depth: f32) {
    let brightness = segment_brightness(depth);

    // Draw outer glow (larger, semi-transparent)
    let glow_radius = theme::canvas_3d::HEAD_MARKER_GLOW_RADIUS * (0.7 + 0.3 * brightness);
    let glow_color = curve_glow_color_alpha(brightness, theme::canvas_3d::HEAD_MARKER_GLOW_ALPHA);
    painter.circle_filled(pos, glow_radius, glow_color);

    // Draw inner core (smaller, solid)
    let core_radius = theme::canvas_3d::HEAD_MARKER_RADIUS * (0.7 + 0.3 * brightness);
    let core_color = curve_glow_color(brightness);
    painter.circle_filled(pos, core_radius, core_color);
}
