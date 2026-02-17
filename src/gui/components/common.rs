use crate::gui::icons;
use egui::{CursorIcon, Sense, Ui};
use std::borrow::Cow;

/// Renders a label that copies its text (or specific content) to the clipboard when clicked.
///
/// It provides visual feedback by changing the tooltip to "✅ Copied!" for a short duration
/// after the click.
///
/// # Arguments
///
/// * `ui` - The `egui::Ui` context.
/// * `display_text` - The text to display on the label.
/// * `copy_text` - The text to be copied to the clipboard (if different from display_text).
/// * `default_tooltip` - The tooltip text to show when not in the "Copied!" state.
/// * `monospace` - Whether to render the text in monospace font.
#[cfg(not(tarpaulin_include))]
pub fn render_copyable_label(
    ui: &mut Ui,
    display_text: &str,
    copy_text: &str,
    default_tooltip: &str,
    monospace: bool,
) {
    let text = if monospace {
        egui::RichText::new(display_text).monospace()
    } else {
        egui::RichText::new(display_text)
    };

    let response = ui.add(egui::Label::new(text).sense(Sense::click()));
    let id = response.id;
    let now = ui.input(|i| i.time);
    let copied_at: Option<f64> = ui.data(|d| d.get_temp(id));

    let tooltip: Cow<str> = if let Some(t) = copied_at {
        if now - t < 2.0 {
            Cow::Owned(format!("{} Copied!", icons::ICON_CHECK))
        } else {
            Cow::Borrowed(default_tooltip)
        }
    } else {
        Cow::Borrowed(default_tooltip)
    };

    let response = response
        .on_hover_cursor(CursorIcon::PointingHand)
        .on_hover_text(tooltip);

    if response.clicked() {
        ui.output_mut(|o| {
            o.commands
                .push(egui::OutputCommand::CopyText(copy_text.to_string()))
        });
        ui.data_mut(|d| d.insert_temp(id, now));
    }
}

/// Renders a label that copies its text (or specific content) to the clipboard when clicked,
/// with a specific color.
///
/// # Arguments
///
/// * `ui` - The `egui::Ui` context.
/// * `display_text` - The text to display on the label.
/// * `copy_text` - The text to be copied to the clipboard (if different from display_text).
/// * `default_tooltip` - The tooltip text to show when not in the "Copied!" state.
/// * `color` - The color of the text.
#[cfg(not(tarpaulin_include))]
pub fn render_copyable_label_with_color(
    ui: &mut Ui,
    display_text: &str,
    copy_text: &str,
    default_tooltip: &str,
    color: egui::Color32,
) {
    let text = egui::RichText::new(display_text).color(color);

    let response = ui.add(egui::Label::new(text).sense(Sense::click()));
    let id = response.id;
    let now = ui.input(|i| i.time);
    let copied_at: Option<f64> = ui.data(|d| d.get_temp(id));

    let tooltip: Cow<str> = if let Some(t) = copied_at {
        if now - t < 2.0 {
            Cow::Owned(format!("{} Copied!", icons::ICON_CHECK))
        } else {
            Cow::Borrowed(default_tooltip)
        }
    } else {
        Cow::Borrowed(default_tooltip)
    };

    let response = response
        .on_hover_cursor(CursorIcon::PointingHand)
        .on_hover_text(tooltip);

    if response.clicked() {
        ui.output_mut(|o| {
            o.commands
                .push(egui::OutputCommand::CopyText(copy_text.to_string()))
        });
        ui.data_mut(|d| d.insert_temp(id, now));
    }
}

/// Renders a heading that copies its text (or specific content) to the clipboard when clicked.
///
/// Similar to `render_copyable_label` but uses heading styling.
///
/// # Arguments
///
/// * `ui` - The `egui::Ui` context.
/// * `display_text` - The text to display on the heading.
/// * `copy_text` - The text to be copied to the clipboard (if different from display_text).
/// * `default_tooltip` - The tooltip text to show when not in the "Copied!" state.
#[cfg(not(tarpaulin_include))]
pub fn render_copyable_heading(
    ui: &mut Ui,
    display_text: &str,
    copy_text: &str,
    default_tooltip: &str,
) {
    let text = egui::RichText::new(display_text).heading();

    let response = ui.add(egui::Label::new(text).sense(Sense::click()));
    let id = response.id;
    let now = ui.input(|i| i.time);
    let copied_at: Option<f64> = ui.data(|d| d.get_temp(id));

    let tooltip: Cow<str> = if let Some(t) = copied_at {
        if now - t < 2.0 {
            Cow::Owned(format!("{} Copied!", icons::ICON_CHECK))
        } else {
            Cow::Borrowed(default_tooltip)
        }
    } else {
        Cow::Borrowed(default_tooltip)
    };

    let response = response
        .on_hover_cursor(CursorIcon::PointingHand)
        .on_hover_text(tooltip);

    if response.clicked() {
        ui.output_mut(|o| {
            o.commands
                .push(egui::OutputCommand::CopyText(copy_text.to_string()))
        });
        ui.data_mut(|d| d.insert_temp(id, now));
    }
}

/// A widget that renders a button with an icon and text, minimizing allocation overhead.
///
/// This widget lays out the icon and text separately, avoiding the creation of a combined
/// `String` (which happens with `format!`).
pub struct IconButton<'a> {
    icon: &'a str,
    text: &'a str,
    small: bool,
}

impl<'a> IconButton<'a> {
    /// Creates a new `IconButton`.
    pub fn new(icon: &'a str, text: &'a str) -> Self {
        Self {
            icon,
            text,
            small: false,
        }
    }

    /// Sets the button to be small.
    pub fn small(mut self) -> Self {
        self.small = true;
        self
    }
}

impl egui::Widget for IconButton<'_> {
    fn ui(self, ui: &mut Ui) -> egui::Response {
        let padding = if self.small {
            ui.spacing().button_padding - egui::vec2(2.0, 2.0)
        } else {
            ui.spacing().button_padding
        };

        let spacing = ui.spacing().item_spacing.x;

        let font_id = egui::TextStyle::Button.resolve(ui.style());

        // Layout icon and text separately
        let icon_galley = ui.painter().layout_no_wrap(
            self.icon.to_string(),
            font_id.clone(),
            ui.visuals().text_color(),
        );
        let text_galley =
            ui.painter()
                .layout_no_wrap(self.text.to_string(), font_id, ui.visuals().text_color());

        let content_size = egui::vec2(
            icon_galley.size().x + spacing + text_galley.size().x,
            icon_galley.size().y.max(text_galley.size().y),
        );

        let desired_size = content_size + 2.0 * padding;

        let (rect, response) = ui.allocate_exact_size(desired_size, Sense::click());

        if ui.is_rect_visible(rect) {
            let visuals = ui.style().interact(&response);

            // Draw background and frame
            ui.painter().rect(
                rect.expand(visuals.expansion),
                visuals.corner_radius,
                visuals.bg_fill,
                visuals.bg_stroke,
                egui::StrokeKind::Inside,
            );

            // Calculate positions
            let content_rect = rect.shrink2(padding);
            let center_y = content_rect.center().y;

            let icon_pos = egui::pos2(content_rect.min.x, center_y - icon_galley.size().y / 2.0);

            let text_pos = egui::pos2(
                content_rect.min.x + icon_galley.size().x + spacing,
                center_y - text_galley.size().y / 2.0,
            );

            // Draw content with correct interaction color
            ui.painter()
                .galley(icon_pos, icon_galley, visuals.text_color());
            ui.painter()
                .galley(text_pos, text_galley, visuals.text_color());
        }

        response
    }
}
