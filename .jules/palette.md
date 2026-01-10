## 2024-05-23 - [Selection Clear Buttons]
**Learning:** Users often select an item (like an airport or aircraft) and then want to "reset" their choice to a random or unspecified state. Requiring them to open the dropdown and find a "None" or "Unspecified" option is tedious. A direct "Clear" (X) button next to the selection provides immediate control and visibility of the "unselected" state.
**Action:** When designing selection components, always consider the "deselect" workflow. If a selection can be cleared, provide a visible, one-click action to do so without opening a menu.

## 2024-05-24 - [Contextual Header Tooltips]
**Learning:** Technical acronyms (like ICAO) and domain-specific terms (like Flight Rules) in data tables can be barriers for new users. Tooltips on headers allow providing definitions and context right where the user looks, without cluttering the visual design.
**Action:** Add explanatory tooltips to all table headers that use acronyms or domain-specific terminology.

## 2024-05-25 - [Dropdown Affordances]
**Learning:** Text-only buttons that trigger dropdowns can be mistaken for immediate actions. Adding a standard visual indicator like a chevron (▾) significantly improves affordance, signaling that the element opens a menu. Dynamically flipping the chevron (▴) when open provides subtle but effective state feedback.
**Action:** Always include a visual direction indicator (chevron/arrow) on buttons that toggle the visibility of other UI elements (menus, popups, accordions).

## 2024-05-26 - [Text Interaction Affordances]
**Learning:** Users often need to extract data like ICAO codes from tables for use in other applications. Static text labels block this workflow, creating friction.
**Action:** Make key identifiers (like ICAO codes) clickable to copy, providing a visual cue (pointing hand cursor) and a tooltip to explain the interaction.
## 2026-01-08 - [Enhanced Dialog Actions]
**Learning:** Standardizing dialog buttons (Save, Cancel, Add) with consistent emojis and descriptive tooltips significantly improves visual scanning and clarifies intent without cluttering the UI.
**Action:** Apply this pattern (Emoji + Label + Tooltip) to all future modal dialog actions to maintain consistency and accessibility.

## 2026-01-09 - Standardized 'Copy' Feedback Pattern
**Learning:** Implementing visual feedback for copy actions (like '✅ Copied!') significantly improves user confidence, but managing the transient state for each item (via `ui.data()`) requires unique IDs. Abstracting this into a reusable helper prevents code duplication and ensures consistency.
**Action:** Use `render_copyable_label` from `src/gui/components/common.rs` for any text that users might want to copy.
