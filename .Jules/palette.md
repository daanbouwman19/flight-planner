## 2026-02-02 - Accessible Popups in Egui
**Learning:** `egui::Window` close button does not have a keyboard shortcut by default. Explicitly handling `Esc` and adding a tooltip improves accessibility significantly.
**Action:** When adding modal windows, ensure they can be closed with the `Esc` key. A good pattern is to combine this check with the 'Cancel' or 'Close' button's click handler: `if ui.button(...).clicked() || ui.input(|i| i.key_pressed(egui::Key::Escape)) { /* close */ }`. Remember to add a hint like `(Esc)` to the button's tooltip.

## 2025-05-23 - Detailed Dropdown Tooltips
**Learning:** `egui::selectable_label` returns a `Response` that can be augmented with `on_hover_text`. This is a powerful way to add secondary information (like elevation, coordinates, or aircraft specs) to dropdown items without cluttering the list view.
**Action:** When implementing lists where items represent complex objects, consider adding a `tooltip_formatter` closure to reveal details on hover. This keeps the UI clean while remaining informative.
