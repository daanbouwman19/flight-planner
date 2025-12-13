# Bolt's Journal

## 2024-05-22 - Parallel Iterator usage
**Learning:** Using `par_iter` (Rayon) for expensive filtering operations on large datasets is significantly faster than sequential iteration, especially when cloning `Arc`s or doing string comparisons.
**Action:** Look for `iter().filter(...)` patterns on large collections (like `aircrafts` or `airports`) where the predicate is non-trivial, and convert to `par_iter()`.

## 2024-05-23 - Lazy UI Generation
**Learning:** Generating UI elements (like `ListItem`s for a table) eagerly for the entire dataset causes massive startup lag and high memory usage.
**Action:** Use `egui`'s virtualization (e.g., `TableBody::rows`) effectively. However, even with virtualization, if the *data source* for the table is pre-calculated into UI-specific structs for *all* items, it's still slow. Defer the creation of display-ready structs until the last possible moment (filtering/searching), or use `par_iter` to speed up the transformation.

## 2024-05-24 - Database Connection Pooling
**Learning:** Opening a new SQLite connection for every small operation is slow.
**Action:** Ensure `r2d2` or similar pooling is used and correctly configured. Passed `Pool` around instead of creating new connections.

## 2024-05-24 - String formatting in hot loops
**Learning:** `format!` inside a tight loop (like rendering a table cell every frame) allocates.
**Action:** Pre-calculate display strings if they don't change often, or use `write!` to a reused buffer if possible (though harder in immediate mode GUIs like egui). Storing the formatted string in the view model during the "update" phase (or search phase) is better than formatting it in the "view" phase.
