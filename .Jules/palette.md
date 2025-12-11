## 2024-05-24 - Search Controls Improvement
**Learning:** `egui`'s `text_edit_singleline` builder doesn't expose `hint_text` directly, so one must use `ui.add(egui::TextEdit::singleline(..).hint_text(..))` for placeholders. This pattern is essential for space-constrained UIs where labels might be skipped.
**Action:** When adding text inputs in `egui`, always prefer the `ui.add(egui::TextEdit::...)` pattern over `ui.text_edit_singleline(...)` if any customization (hints, width, id) is needed.
