## 2024-05-24 - Accessible Disabled Tooltips
**Learning:** In `egui`, `.on_hover_text()` may not reliably display on disabled widgets depending on the specific integration or version. The best practice is to use `.on_disabled_hover_text()` explicitly for disabled states, or chain both if the same message applies to both. This ensures the user always knows *why* an interaction is unavailable.
**Action:** When adding tooltips to buttons that can be disabled, always consider adding `.on_disabled_hover_text("Reason...")` to improve accessibility and user understanding.
