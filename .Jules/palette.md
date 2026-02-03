## 2024-05-23 - [Egui Keyboard Accessibility]
**Learning:** Egui components like Windows, TextEdits, and custom Dropdowns do not handle the Escape key by default, leaving keyboard users trapped or forced to use the mouse.
**Action:** Manually implement Escape key handlers for every modal, dropdown, and focusable input to ensure standard desktop accessibility behavior.
