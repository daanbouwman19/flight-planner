# Claude Code Guidelines

## Code Style

### Warning Suppression
- **Do not use `#[allow(...)]` attributes to suppress warnings** unless there is genuinely no alternative.
- Instead, fix the root cause. Common patterns:
  - `#[allow(unused_mut)]` on a variable that is only written to inside a `#[cfg(target_arch = "wasm32")]` block → gate the variable declaration itself with the same `#[cfg]` so the variable simply doesn't exist on other targets.
  - `let _ = variable;` silencers are also a code smell; prefer removing the variable or restructuring the code.

## Architecture

### Web / WASM Split
The project compiles to two targets:
- **Native binary** (`gui` + `server` features): egui desktop UI + axum REST backend.
- **WASM frontend** (`web` feature): egui in the browser, communicates with the native backend via REST.

Keep platform-conditional code minimal. Prefer shared abstractions over `#[cfg]` forks where practical.
