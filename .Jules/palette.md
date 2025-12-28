## 2024-05-24 - Consistent Text Input Behavior
**Learning:** Users appreciate the ability to quickly clear text inputs, especially for long strings like API keys. Adding a conditional "Clear" button (×) is a simple delight. However, simply adding a button that appears/disappears can cause layout jitter in auto-sizing windows.
**Action:** When adding conditional buttons next to inputs, always reserve the space (e.g., using `ui.allocate_exact_size`) or use a fixed width for the input field to prevent the UI from "jumping" when the button state changes.
