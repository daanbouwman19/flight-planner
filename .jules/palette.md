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

## 2026-01-21 - Settings Popup UX Polish
**Learning:** In egui, windows created without `.open(&mut bool)` lack a close button and standard behavior. Adding it, along with `.anchor()`, significantly improves the modal feel. Also, manual Enter key handling is often needed for form submission in immediate mode GUIs.
**Action:** When creating modals in egui, always consider using `.open()` for consistency and implement keyboard shortcuts for primary actions.

## 2026-01-22 - [Search Loading State]
**Learning:** Adding loading states to search inputs is crucial for responsiveness perception. Users need to know if a filter is processing or if there are truly no results.
**Action:** Always verify if async operations have visible feedback in the UI, especially for search/filter inputs.

## 2026-01-23 - Keyboard Navigation in Searchable Dropdowns
**Learning:** Adding keyboard navigation to `egui` components requires managing persistent state manually using `ui.data_mut()`, as the component struct is recreated every frame. Visual feedback for "highlighted but not selected" items can be achieved by using `selectable_label` with a boolean flag derived from the navigation state, but care must be taken to distinguish it from persistent selection if needed (though often combined in dropdowns). Auto-scrolling to the highlighted item is crucial and can be done with `response.scroll_to_me`.
**Action:** When enhancing `egui` components for accessibility, always look for opportunities to map keyboard inputs to state changes that drive visual updates in the immediate mode render loop.

## 2026-01-24 - [Actionable Empty States]
**Learning:** Empty states (e.g., "No results found") are dead ends if they only provide information. Adding an immediate action, like a "Clear Search" button, transforms a dead end into a helpful recovery path, reducing frustration.
**Action:** Always include a recovery action (clear filter, reset, etc.) in empty states caused by user input.

## 2026-01-25 - [Unified Toast Notification System]
**Learning:** In immediate mode GUIs like `egui`, ephemeral feedback (like "Settings saved") is often lost if the triggering component (e.g., a popup) closes immediately. A centralized "Toast" manager that renders on top of everything ensures feedback persists across UI state transitions.
**Action:** Implement a global notification queue when user actions trigger state changes that might close the current view.

## 2026-01-26 - [Visible Loading Indicators]
**Learning:** Static text for async operations (like "Fetching...") often blends into the UI and fails to convey active processing. Users may think the app is stuck. Adding a spinner creates a standard visual cue for "work in progress".
**Action:** Always pair "Fetching" or "Loading" text with a `ui.spinner()` or equivalent animation to provide immediate, recognizable feedback for async states.

## 2026-01-27 - [Explicit Constraint Markers]
**Learning:** Relying on disabled buttons with tooltips to communicate form constraints (e.g., "why can't I click Add?") hides information. Explicitly marking required fields (e.g., with `*`) provides immediate, scannable feedback before the user even attempts the action, reducing friction and ambiguity.
**Action:** Always visually distinguish required fields from optional ones in forms, rather than relying solely on validation errors or disabled states.

## 2026-01-28 - [Destructive Action Confirmation]
**Learning:** Destructive actions (like resetting status) hidden behind a single click can lead to accidental data loss or frustration. Inline confirmation ("Are you sure?") provides a safety barrier without the overhead of a full modal dialog, keeping the flow smooth.
**Action:** Identify single-click destructive buttons and wrap them in a confirmation state to prevent accidental triggers.

## 2026-01-29 - [Theme-Aware Toast Notifications]
**Learning:** Hardcoded colors for overlay elements (like toasts) break visual consistency in dark mode, appearing jarringly bright.
**Action:** Always check `ui.visuals().dark_mode` when rendering custom overlays and adapt the color palette to match the active theme.
