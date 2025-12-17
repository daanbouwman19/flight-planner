## 2024-05-24 - Transient UI State
**Learning:** For transient UI state like "show password" toggles that doesn't need to persist in the application model, use `ui.data_mut()` with a unique ID (e.g., `ui.make_persistent_id("...")`). This keeps the ViewModel clean and focused on business logic.
**Action:** Use `ui.data()` for temporary view-only state flags instead of polluting the global state or ViewModel.
