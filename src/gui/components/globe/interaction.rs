use eframe::egui::{Pos2, Rect, Response};

use super::camera::{Camera, MAX_DISTANCE, MIN_DISTANCE, ORBIT_SENS, PITCH_LIMIT, SCROLL_SENS};
use super::state::{Drag, DragKind, GlobeState};

pub fn update(state: &mut GlobeState, response: &Response, viewport: Rect) {
    handle_drag(state, response, viewport);
    handle_scroll(state, response, viewport);
}

fn handle_drag(state: &mut GlobeState, response: &Response, viewport: Rect) {
    let (pri_down, orbit_down, cursor) = response.ctx.input(|i| {
        let ctrl = i.modifiers.ctrl;
        (
            i.pointer.primary_down(),
            i.pointer.secondary_down()
                || i.pointer.middle_down()
                || (ctrl && i.pointer.primary_down()),
            // hover_pos works for all button states; interact_pos is primary-only.
            i.pointer.interact_pos().or_else(|| i.pointer.latest_pos()),
        )
    });

    let desired: Option<DragKind> = if orbit_down {
        Some(DragKind::Orbit)
    } else if pri_down {
        Some(DragKind::Pan)
    } else {
        None
    };

    if let Some(d) = state.drag {
        if Some(d.kind) != desired {
            state.drag = None;
        }
    }
    if desired.is_none() {
        state.drag = None;
        return;
    }

    let Some(cursor) = cursor else { return };

    if state.drag.is_none() {
        let can_start = if orbit_down {
            // contains_pointer() is unreliable for non-primary buttons (egui assigns
            // "widget under pointer" via primary-button sense only), so check directly.
            viewport.contains(cursor)
        } else {
            response.is_pointer_button_down_on() || response.contains_pointer()
        };
        if !can_start {
            return;
        }
        let kind = if orbit_down { DragKind::Orbit } else { DragKind::Pan };
        let world_pt = Some(state.camera.screen_to_world_clamped(cursor, viewport));
        state.drag = Some(Drag {
            kind,
            cursor_start: cursor,
            yaw_start: state.camera.yaw,
            pitch_start: state.camera.pitch,
            roll_start: state.camera.roll,
            world_pt,
        });
    }

    let Some(drag) = state.drag else { return };

    match drag.kind {
        DragKind::Pan => {
            if let Some(world_pt) = drag.world_pt {
                state.camera.rotate_to_pin(world_pt, cursor, viewport);
            }
        }
        DragKind::Orbit => {
            apply_orbit(state, &drag, cursor, viewport);
        }
    }
}

/// MapLibre-style orbit: horizontal drag changes yaw (bearing), vertical drag changes pitch (tilt).
/// No anchor point, no roll — the same model used by maps.kagi.com.
fn apply_orbit(state: &mut GlobeState, drag: &Drag, cursor: Pos2, viewport: Rect) {
    let dx = cursor.x - drag.cursor_start.x;
    let dy = cursor.y - drag.cursor_start.y;
    let f = Camera {
        yaw: drag.yaw_start,
        pitch: drag.pitch_start,
        roll: drag.roll_start,
        distance: state.camera.distance,
        fov_y: state.camera.fov_y,
    }
    .focal_pixels(viewport.height())
    .max(1.0);

    state.camera.yaw = drag.yaw_start + dx * ORBIT_SENS / f;
    state.camera.pitch =
        (drag.pitch_start - dy * ORBIT_SENS / f).clamp(-PITCH_LIMIT, PITCH_LIMIT);
    state.camera.roll = 0.0;
}

fn handle_scroll(state: &mut GlobeState, response: &Response, viewport: Rect) {
    let scroll = response.ctx.input(|i| i.smooth_scroll_delta.y);
    if scroll == 0.0 {
        return;
    }

    let pinned = response
        .hover_pos()
        .and_then(|c| state.camera.screen_to_world(c, viewport).map(|w| (c, w)));

    // Multiplicative on (distance - 1) so zoom feels exponential in height above surface.
    let factor = (1.0 - scroll * SCROLL_SENS).clamp(0.5, 2.0);
    state.camera.distance =
        (1.0 + (state.camera.distance - 1.0) * factor).clamp(MIN_DISTANCE, MAX_DISTANCE);

    let Some((cursor, world_pt)) = pinned else { return };
    state.camera.rotate_to_pin(world_pt, cursor, viewport);
}
