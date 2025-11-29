//! State management for the GUI application.

use crate::{
    Pane, Selected3DCurve, SelectedCurve, SharedSettings, snake::advance_snake_offset, theme,
};

/// Logic controller for updating application state.
pub struct AnimationController;

impl AnimationController {
    /// Advance time-based state by `delta` seconds and update animations.
    pub fn update(
        delta: f32,
        app_state: &mut crate::AppState,
        shared_settings: &SharedSettings,
        selected_curve: &mut SelectedCurve,
        selected_3d_curve: &mut Selected3DCurve,
    ) {
        // Skip when paused or user is dragging in 3D view
        if app_state.paused || app_state.mouse_dragging {
            return;
        }

        app_state.animation_time += delta;

        // Convert 0-100 scale to actual rotation speed using base speed
        let actual_rotation_speed =
            theme::animation::BASE_ROTATION_SPEED * (shared_settings.spin_speed / 100.0);
        app_state.rotation_angle += delta * actual_rotation_speed;

        // Update snake animation timing
        app_state.snake_time += delta;

        // Snake animation speed from settings
        let snake_increment = delta * shared_settings.snake_speed;

        // Update snake offsets for both 2D and 3D
        if shared_settings.snake_enabled {
            selected_curve.snake_offset = advance_snake_offset(
                selected_curve.snake_offset,
                snake_increment,
                selected_curve.ensure_curve_length(),
            );
            selected_3d_curve.snake_offset = advance_snake_offset(
                selected_3d_curve.snake_offset,
                snake_increment,
                selected_3d_curve.ensure_curve_length(),
            );
        }
    }

    /// Synchronize selection between 2D and 3D panes.
    ///
    /// Propagates the selection from the active pane to the inactive pane,
    /// provided the curve name is valid in the target context.
    pub fn sync_panes(
        current_pane: Pane,
        selected_curve: &mut SelectedCurve,
        selected_3d_curve: &mut Selected3DCurve,
        available_curves: &[&str],
    ) {
        let is_supported = |name: &str| available_curves.contains(&name);

        match current_pane {
            Pane::TwoD => {
                if selected_3d_curve.name != selected_curve.name {
                    // Ensure name is valid for 3D
                    if is_supported(&selected_curve.name) {
                        selected_3d_curve.name = selected_curve.name.clone();
                    }
                }
            }
            Pane::ThreeD => {
                if selected_curve.name != selected_3d_curve.name {
                    // Ensure name is valid for 2D
                    if is_supported(&selected_3d_curve.name) {
                        selected_curve.name = selected_3d_curve.name.clone();
                    }
                }
            }
        }
    }
}
