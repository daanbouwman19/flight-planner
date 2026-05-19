use std::f32::consts::PI;

use eframe::egui::{Rect, Response};

use super::state::{Drag, DragKind, GlobeState, MAX_ALTITUDE, MIN_ALTITUDE};

/// Exponential zoom sensitivity: fraction of altitude to add/remove per scroll unit.
const SCROLL_SENS: f32 = 0.01;
/// Bearing radians per pixel of horizontal orbit drag.
/// Full viewport-width drag → 180° of bearing rotation.
const BEARING_RAD_PER_PX: f32 = PI; // divided by viewport.width() each frame

pub fn update(state: &mut GlobeState, response: &Response, viewport: Rect) {
    handle_drag(state, response, viewport);
    handle_scroll(state, response, viewport);
}

fn handle_drag(state: &mut GlobeState, response: &Response, viewport: Rect) {
    let (pri_down, orbit_down, cursor, delta) = response.ctx.input(|i| {
        let ctrl = i.modifiers.ctrl;
        (
            i.pointer.primary_down(),
            i.pointer.secondary_down()
                || i.pointer.middle_down()
                || (ctrl && i.pointer.primary_down()),
            i.pointer.interact_pos().or_else(|| i.pointer.latest_pos()),
            i.pointer.delta(),
        )
    });

    let desired: Option<DragKind> = if orbit_down {
        Some(DragKind::Orbit)
    } else if pri_down {
        Some(DragKind::Pan)
    } else {
        None
    };

    // Release or mode-switch cancels the active drag.
    if desired.is_none() || state.drag.map(|d| d.kind) != desired {
        state.drag = None;
    }
    if desired.is_none() {
        return;
    }

    let Some(cursor) = cursor else {
        return;
    };

    // Initialise drag on the first frame a button is held.
    if state.drag.is_none() {
        let can_start = if orbit_down {
            // egui only tracks "widget under pointer" for the primary button, so
            // check bounds directly for secondary / middle.
            viewport.contains(cursor)
        } else {
            response.is_pointer_button_down_on() || response.contains_pointer()
        };
        if !can_start {
            return;
        }

        let kind = desired.unwrap();
        let pan_anchor = if kind == DragKind::Pan {
            Some(
                state
                    .map_view
                    .to_camera()
                    .screen_to_world_clamped(cursor, viewport),
            )
        } else {
            None
        };
        state.drag = Some(Drag { kind, pan_anchor });
    }

    let Some(drag) = state.drag else {
        return;
    };

    match drag.kind {
        DragKind::Pan => {
            if let Some(anchor) = drag.pan_anchor {
                // Keep the anchor world-point under the cursor.
                let mut camera = state.map_view.to_camera();
                camera.rotate_to_pin(anchor, cursor, viewport);
                // rotate_to_pin changes yaw+pitch only; roll (= bearing) is preserved.
                state.map_view.sync_center_from_camera(&camera);
            }
        }
        DragKind::Orbit => {
            // Bearing rotation: horizontal drag spins the view around the screen centre.
            // Roll does not affect the nadir (the nadir is at camera-space [0,0,1], which
            // the roll rotation leaves unchanged), so the globe rotates around its own
            // centre on screen regardless of cursor position.
            state.map_view.bearing += delta.x * BEARING_RAD_PER_PX / viewport.width();
            // Vertical tilt requires the camera to look at the nadir rather than the
            // globe origin, which needs a different projection model. Vertical drag is
            // intentionally a no-op here so the camera location stays fixed.
        }
    }
}

fn handle_scroll(state: &mut GlobeState, response: &Response, viewport: Rect) {
    let scroll = response.ctx.input(|i| i.smooth_scroll_delta.y);
    if scroll == 0.0 {
        return;
    }

    // Record the world point under the cursor before zooming so we can re-pin it.
    let pinned = response.hover_pos().and_then(|c| {
        let camera = state.map_view.to_camera();
        camera.screen_to_world(c, viewport).map(|w| (c, w))
    });

    // Multiplicative on altitude so zoom feels exponential in height above surface.
    let factor = (1.0 - scroll * SCROLL_SENS).clamp(0.5, 2.0);
    state.map_view.altitude = (state.map_view.altitude * factor).clamp(MIN_ALTITUDE, MAX_ALTITUDE);

    if let Some((cursor, world_pt)) = pinned {
        let mut camera = state.map_view.to_camera();
        camera.rotate_to_pin(world_pt, cursor, viewport);
        state.map_view.sync_center_from_camera(&camera);
    }
}
