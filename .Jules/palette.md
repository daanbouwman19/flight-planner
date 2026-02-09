## 2024-05-22 - [Sidebar Button States]
**Learning:** `egui`'s `Button` widget has a `.selected(bool)` method that is perfect for indicating active navigation states without custom CSS or styling.
**Action:** Use `.selected(vm.current_mode == DisplayMode::...)` for all sidebar navigation buttons to provide visual feedback.
