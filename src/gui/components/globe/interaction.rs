use std::f32::consts::PI;

use eframe::egui::{Rect, Response};

use super::state::{Drag, DragKind, GlobeState, MAX_ALTITUDE, MIN_ALTITUDE};

/// Exponential zoom sensitivity: fraction of altitude to add/remove per scroll unit.
const SCROLL_SENS: f32 = 0.01;
/// Bearing radians per pixel of horizontal orbit drag.
/// Full viewport-width drag → 180° of bearing rotation.
const BEARING_RAD_PER_PX: f32 = PI; // divided by viewport.width() each frame
/// Tilt radians per pixel of vertical orbit drag.
/// Full viewport-height drag → 60° of tilt.
const TILT_RAD_PER_PX: f32 = PI / 3.0; // divided by viewport.height() each frame
const MAX_TILT: f32 = PI / 2.5;

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
            viewport.contains(cursor)
        } else {
            response.is_pointer_button_down_on() || response.contains_pointer()
        };
        if !can_start {
            return;
        }

        let kind = desired.unwrap();
        let pan_anchor = if kind == DragKind::Pan {
            Some(state.camera.screen_to_world_clamped(cursor, viewport))
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
                state.camera.pan_to(anchor, cursor, viewport);
            }
        }
        DragKind::Orbit => {
            // Horizontal: spin the bearing around screen centre.
            state.camera.bearing += delta.x * BEARING_RAD_PER_PX / viewport.width();
            // Vertical: tilt the view toward the horizon.
            state.camera.tilt =
                (state.camera.tilt + delta.y * TILT_RAD_PER_PX / viewport.height())
                    .clamp(0.0, MAX_TILT);
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
        state.camera.screen_to_world(c, viewport).map(|w| (c, w))
    });

    // Multiplicative on altitude so zoom feels exponential in height above surface.
    let factor = (1.0 - scroll * SCROLL_SENS).clamp(0.5, 2.0);
    state.camera.altitude =
        (state.camera.altitude * factor).clamp(MIN_ALTITUDE, MAX_ALTITUDE);

    if let Some((cursor, world_pt)) = pinned {
        state.camera.pan_to(world_pt, cursor, viewport);
    }
}

