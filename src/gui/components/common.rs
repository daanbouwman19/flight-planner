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
