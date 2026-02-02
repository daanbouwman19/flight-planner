## 2026-02-02 - Accessible Popups in Egui
**Learning:** `egui::Window` close button does not have a keyboard shortcut by default. Explicitly handling `Esc` and adding a tooltip improves accessibility significantly.
**Action:** When adding modal windows, always check `ctx.input(|i| i.key_pressed(egui::Key::Escape))` to manually close the window and add a hint to the close button.
