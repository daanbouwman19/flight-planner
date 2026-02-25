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
**Learning:** Empty states (e.g., "No results found") are dead ends if they only provide information. Adding an immediate action, like a "Clear Search" button, transforms a dead end into a helpful recovery path, reducing friction and ambiguity.
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

## 2026-01-31 - [Theme-Aware Toast Notifications]
**Learning:** Hardcoded colors for overlay elements (like toasts) break visual consistency in dark mode, appearing jarringly bright.
**Action:** Always check `ui.visuals().dark_mode` when rendering custom overlays and adapt the color palette to match the active theme.

## 2026-02-01 - Grouped Filter Reset
**Learning:** When users have multiple independent filters (like Aircraft and Departure Airport), clearing them individually is tedious. A grouped "Clear All" action near the section header significantly reduces clicks and friction.
**Action:** Look for other grouped inputs (like form sections) that could benefit from a section-level reset or clear action.

## 2026-02-02 - Accessible Popups in Egui
**Learning:** `egui::Window` close button does not have a keyboard shortcut by default. Explicitly handling `Esc` and adding a tooltip improves accessibility significantly.
**Action:** When adding modal windows, ensure they can be closed with the `Esc` key. A good pattern is to combine this check with the 'Cancel' or 'Close' button's click handler: `if ui.button(...).clicked() || ui.input(|i| i.key_pressed(egui::Key::Escape)) { /* close */ }`. Remember to add a hint like `(Esc)` to the button's tooltip.

## 2026-02-04 - Detailed Dropdown Tooltips
**Learning:** `egui::selectable_label` returns a `Response` that can be augmented with `on_hover_text`. This is a powerful way to add secondary information (like elevation, coordinates, or aircraft specs) to dropdown items without cluttering the list view.
**Action:** When implementing lists where items represent complex objects, consider adding a `tooltip_formatter` closure to reveal details on hover. This keeps the UI clean while remaining informative.

## 2026-02-06 - Context-Aware Actions
**Learning:** Users often want to perform actions (like 'Mark as Flown') on data items regardless of how they found them (e.g., via random generation vs. specific filtering). Restricting actions based on the *source* view can be frustrating and unintuitive.
**Action:** Always evaluate if an action is valid for the *data item itself*, rather than the *view mode* it's currently displayed in. If the data supports the action, make it available.

## 2026-02-07 - Micro-UX: Structured Statistics
**Learning:** Raw data tables are hard to scan. Grouping statistics into logical categories with icons and tooltips makes them much more engaging and readable.
**Action:** Always look for opportunities to add visual hierarchy and context to data displays.

## 2026-02-08 - [External Context Links]
**Learning:** Adding direct links to external tools (SkyVector, Google Maps) in context (Route Popup) significantly enhances the utility for flight simmers without cluttering the UI, thanks to `egui`'s compact `hyperlink_to`.
**Action:** Look for other places where context-specific external links can be added (e.g., aircraft details -> Wikipedia/Skybrary).

## 2026-02-09 - [Sidebar Button States]
**Learning:** `egui`'s `Button` widget has a `.selected(bool)` method that is perfect for indicating active navigation states without custom CSS or styling.
**Action:** Use `.selected(vm.current_mode == DisplayMode::...)` for all sidebar navigation buttons to provide visual feedback.

## 2026-02-11 - [Form Submission Shortcuts]
**Learning:** Users expect standard shortcuts like Ctrl+Enter to submit forms in modals, not just clicking the button. This is especially important for power users who prefer to keep their hands on the keyboard.
**Action:** Implement `Ctrl+Enter` (Cmd+Enter on Mac) for primary actions in forms and update button tooltips to reflect the shortcut.

## 2026-02-13 - [Validation Feedback Timing]
**Learning:** Displaying validation errors for empty required fields immediately upon opening a form feels aggressive and scolding. Users prefer to fill out the form first.
**Action:** Distinguish between "missing information" (rely on `*` labels and disabled button tooltips) and "logic errors" (show inline error alerts). Only show the alert box for actual invalid state, not just incomplete state.

## 2026-02-14 - [Consistent Iconography for Common Actions]
**Learning:** Using unicode characters (like "×" or "🔍") instead of the established icon system (Phosphor) creates subtle visual inconsistencies and can look cheap or broken on some platforms. Standardizing on the icon font ensures a cohesive and polished look.
**Action:** Always check `icons.rs` for existing icons before using unicode alternatives. If a common action (like "Add" or "Clear") is missing an icon, add it to the system rather than using a unicode fallback.

## 2026-02-15 - [Discoverable Empty State Actions]
**Learning:** Hiding "Random" or "Lucky" actions inside a dropdown limits their discoverability. Exposing them as a top-level button when the field is empty encourages exploration.
**Action:** When a selection field is empty, consider replacing the "Clear" button with a "Random" or "Suggest" button to reduce friction for undecided users.

## 2026-02-16 - [Copyable Data Formatting]
**Learning:** Flight simmers frequently need to transfer numerical data (like coordinates) to other tools. Displaying this data in a monospace font distinguishes it as "raw data" and making it click-to-copy significantly reduces friction compared to manual transcription.
**Action:** When displaying coordinate pairs or other technical data, use monospace formatting and wrap them in a copyable component to enhance utility without cluttering the UI.

## 2026-02-19 - Immediate Mode State Management
**Learning:** In this `egui` application, ViewModels (like `RoutePopupViewModel`) are reconstructed every frame in `Gui::update`. To persist UI state (like a button being disabled after a click) while keeping the window open, the state must be stored in the underlying service (e.g., `PopupService`) and passed to the ViewModel, rather than trying to store it in the component itself.
**Action:** When adding interactive state to a component that needs to persist beyond a single frame, always add the state field to the corresponding Service struct and expose it to the ViewModel construction in `Gui::update`.

## 2026-02-20 - [Tooltip-Based Data Augmentation]
**Learning:** Tables often suffer from lack of space for secondary information. Augmenting primary data (like distance) with calculated secondary data (like estimated flight time) via tooltips keeps the UI clean while providing valuable context on demand.
**Action:** Identify where calculated derived data can enhance understanding of primary data without cluttering the view.

## 2026-02-21 - [Copyable Route String]
**Learning:** Flight simmers often need the raw route string (e.g., `KLAX DCT KJFK`) to paste into various tools (vPilot, FMC, etc.). While a human-readable summary is nice, it requires manual editing to be machine-readable. Providing a dedicated "Copy Route" button that formats the data exactly as downstream tools expect it removes a friction point.
**Action:** When displaying structured data that is commonly used as input for other domain-specific tools, provide a "Copy" action that formats it specifically for that purpose, not just for human readability.
