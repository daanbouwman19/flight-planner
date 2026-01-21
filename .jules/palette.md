# Palette's Journal

## 2026-01-12 - [Visual Hierarchy in Error States]
**Learning:** Plain text errors in popups are easily missed. Users scan for colors (red) and icons (⚠️) to identify issues quickly. A consistent visual language for "Close" actions (e.g., using ❌) improves learnability across the application.
**Action:** Always couple error text with a distinct color and an icon to differentiate it from informational content. Ensure action buttons use consistent iconography.

## 2026-01-15 - [Dynamic Tooltips for Stateful Actions]
**Learning:** Static tooltips on toggle buttons (like "Show/Hide") force users to guess the current state or the result of the action. Dynamic tooltips (e.g., "Show" when hidden, "Hide" when visible) reduce cognitive load and provide clear confirmation of the next action.
**Action:** For any toggleable UI element (show/hide, mute/unmute, play/pause), always ensure the tooltip text reflects the *action that will happen* upon clicking, not just a description of the button's purpose.

## 2026-01-18 - [Copyable Headers for Data Identifiers]
**Learning:** Users often need to transfer key identifiers (like route strings "ICAO-ICAO") to other tools. When these are displayed as static headings, it creates friction. Making primary identifiers implicitly copyable reduces this friction without cluttering the UI with explicit "Copy" buttons.
**Action:** When displaying primary data keys (IDs, codes, routes) in headers or prominent labels, always consider making them click-to-copy, especially in data-heavy applications.

## 2026-01-19 - [Interaction Preservation in Custom Styling]
**Learning:** Applying custom visual effects (like fade-in opacity) in `egui` often requires manual reconstruction of standard widget behaviors. Specifically, maintaining utility features like "click-to-copy" on styled text prevents visual polish from degrading usability.
**Action:** Create reusable wrappers (like `render_copyable_label_with_color`) that combine custom styling inputs (color, opacity) with standard interaction patterns, rather than dropping interactions for the sake of visuals.

## 2024-05-23 - Settings Popup UX Polish
**Learning:** In egui, windows created without `.open(&mut bool)` lack a close button and standard behavior. Adding it, along with `.anchor()`, significantly improves the modal feel. Also, manual Enter key handling is often needed for form submission in immediate mode GUIs.
**Action:** When creating modals in egui, always consider using `.open()` for consistency and implement keyboard shortcuts for primary actions.
