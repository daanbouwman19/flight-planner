use eframe::egui::{self, Color32, Context, Frame, Margin, RichText, Stroke, Vec2};
use std::time::{Duration, Instant};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ToastKind {
    Info,
    Success,
    Warning,
    Error,
}

#[derive(Clone, Debug)]
pub struct Toast {
    pub message: String,
    pub kind: ToastKind,
    pub created_at: Instant,
    pub duration: Duration,
}

#[derive(Default)]
pub struct ToastManager {
    toasts: Vec<Toast>,
}

impl ToastManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, message: String, kind: ToastKind) {
        self.toasts.push(Toast {
            message,
            kind,
            created_at: Instant::now(),
            duration: Duration::from_secs(4),
        });
    }

    pub fn render(&mut self, ctx: &Context) {
        let now = Instant::now();
        self.toasts
            .retain(|t| now.duration_since(t.created_at) < t.duration);

        if self.toasts.is_empty() {
            return;
        }

        // Request repaint to handle expiration and potential animations
        ctx.request_repaint();

        // We want to stack toasts from bottom to top.
        // Since egui lays out top-to-bottom, we can cheat by using a fixed position
        // that moves up for each toast, OR use an Area anchored at bottom-right
        // and hope egui handles the "growth" upwards? No, egui Layout is top-down.

        // Strategy: Iterate reversed, calculate offset manually, or just put them in a vertical layout
        // anchored at TOP_RIGHT with a large Y offset? No.

        // Simplest valid way: Anchor at RIGHT_BOTTOM, but render them in an order that looks okay.
        // If we anchor at RIGHT_BOTTOM, the area's bottom-right is at the anchor.
        // The content is laid out inside. If we use `ui.vertical`, it goes down.
        // This means the first item is at the top of the area (which is somewhere above the anchor),
        // and the last item is at the bottom (at the anchor).
        // Wait, if anchor is RIGHT_BOTTOM, does the area expand UPWARDS?
        // Egui documentation says: "The area will expand to fit the content."
        // "The position of the area is determined by the anchor and the content size."
        // So yes, if anchor is RIGHT_BOTTOM, the content ends at the anchor point.
        // So a vertical layout will result in the last item being at the bottom.

        egui::Area::new("toast_area".into())
            .anchor(egui::Align2::RIGHT_BOTTOM, Vec2::new(-20.0, -20.0))
            .order(egui::Order::Foreground)
            .interactable(false) // Don't block clicks? Maybe we want to?
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    ui.set_max_width(300.0);
                    for toast in self.toasts.iter() {
                        Self::render_toast(ui, toast, now);
                        ui.add_space(8.0);
                    }
                });
            });
    }

    fn render_toast(ui: &mut egui::Ui, toast: &Toast, now: Instant) {
        let (bg_color, icon) = match toast.kind {
            ToastKind::Info => (Color32::from_rgb(225, 245, 254), "ℹ"),
            ToastKind::Success => (Color32::from_rgb(232, 245, 233), "✅"),
            ToastKind::Warning => (Color32::from_rgb(255, 248, 225), "⚠️"),
            ToastKind::Error => (Color32::from_rgb(255, 235, 238), "❌"),
        };

        let text_color = Color32::BLACK; // Simple contrast

        // Calculate opacity for fade out
        let elapsed = now.duration_since(toast.created_at);
        let remaining = toast.duration.saturating_sub(elapsed);
        let opacity = if remaining.as_millis() < 500 {
            remaining.as_millis() as f32 / 500.0
        } else {
            1.0
        };

        let frame = Frame::default()
            .fill(bg_color.linear_multiply(opacity))
            .stroke(Stroke::new(
                1.0,
                Color32::from_black_alpha(20).linear_multiply(opacity),
            ))
            .corner_radius(4.0)
            .inner_margin(Margin::same(10));

        frame.show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new(icon).color(text_color.linear_multiply(opacity)));
                ui.label(
                    RichText::new(&toast.message)
                        .color(text_color.linear_multiply(opacity))
                        .strong(),
                );
            });
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_toast_manager_add() {
        let mut manager = ToastManager::new();
        manager.add("Test".to_string(), ToastKind::Info);
        assert_eq!(manager.toasts.len(), 1);
        assert_eq!(manager.toasts[0].message, "Test");
        assert_eq!(manager.toasts[0].kind, ToastKind::Info);
    }

    #[test]
    fn test_toast_expiration() {
        let mut manager = ToastManager::new();
        manager.add("Short".to_string(), ToastKind::Info);
        manager.toasts[0].duration = Duration::from_millis(10);
        manager.toasts[0].created_at = Instant::now() - Duration::from_millis(20);

        // We can't easily test render() as it requires Context, but we can verify retain logic
        // by manually simulating what render does:
        let now = Instant::now();
        manager.toasts.retain(|t| now.duration_since(t.created_at) < t.duration);

        assert_eq!(manager.toasts.len(), 0);
    }
}
