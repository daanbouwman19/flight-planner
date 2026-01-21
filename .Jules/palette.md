## 2024-05-23 - Settings Popup UX Polish
**Learning:** In egui, windows created without  lack a close button and standard behavior. Adding it, along with , significantly improves the modal feel. Also, manual Enter key handling is often needed for form submission in immediate mode GUIs.
**Action:** When creating modals in egui, always consider using  for consistency and implement keyboard shortcuts for primary actions.
## 2024-05-23 - Settings Popup UX Polish
**Learning:** In egui, windows created without `.open(&mut bool)` lack a close button and standard behavior. Adding it, along with `.anchor()`, significantly improves the modal feel. Also, manual Enter key handling is often needed for form submission in immediate mode GUIs.
**Action:** When creating modals in egui, always consider using `.open()` for consistency and implement keyboard shortcuts for primary actions.
