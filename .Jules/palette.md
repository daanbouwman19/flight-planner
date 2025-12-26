## 2024-05-24 - Visual Consistency in Popups
**Learning:** Users expect visual cues (like color-coded flight rules) to be consistent across the entire application. If the main table uses green for VFR, the popup details should also use green for VFR.
**Action:** When implementing detail views or popups, reuse the visual logic (colors, icons) from the list/table views to reduce cognitive load and improve consistency. Refactoring shared rendering logic into helpers is key.
## 2024-05-24 - Transient UI State
**Learning:** For transient UI state like "show password" toggles that doesn't need to persist in the application model, use `ui.data_mut()` with a unique ID (e.g., `ui.make_persistent_id("...")`). This keeps the ViewModel clean and focused on business logic.
**Action:** Use `ui.data()` for temporary view-only state flags instead of polluting the global state or ViewModel.

## 2024-05-24 - Consistent Search Inputs
**Learning:** Search inputs in `egui` feel much better with a built-in clear button (×) and a magnifying glass icon. This pattern should be consistent across all search fields, not just the reusable ones.
**Action:** When implementing ad-hoc search fields, replicate the pattern from `SearchableDropdown` (horizontal layout, 🔍 label, and clear button) to maintain UX consistency.
