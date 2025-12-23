## 2024-05-24 - Transient UI State
**Learning:** For transient UI state like "show password" toggles that doesn't need to persist in the application model, use `ui.data_mut()` with a unique ID (e.g., `ui.make_persistent_id("...")`). This keeps the ViewModel clean and focused on business logic.
**Action:** Use `ui.data()` for temporary view-only state flags instead of polluting the global state or ViewModel.

## 2024-05-24 - Consistent Search Inputs
**Learning:** Search inputs in `egui` feel much better with a built-in clear button (×) and a magnifying glass icon. This pattern should be consistent across all search fields, not just the reusable ones.
**Action:** When implementing ad-hoc search fields, replicate the pattern from `SearchableDropdown` (horizontal layout, 🔍 label, and clear button) to maintain UX consistency.
