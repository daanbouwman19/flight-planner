# Agent Guidelines

This document provides guidelines for agents working on this repository, specifically tailored for the Rust ecosystem.

## Pre-push Checks

> [!CRITICAL] > **MANDATORY PRE-COMMIT CHECKLIST**
>
> You **MUST** run the following commands and ensure they pass before creating a PR.
> If you skip these, the CI **WILL FAIL** and your PR will be rejected.
>
> 1.  **Format:** Run `cargo fmt` locally to fix formatting issues.
>     - _Note:_ CI runs `cargo fmt --all -- --check` and will fail if you haven't run `cargo fmt`.
> 2.  **Lint:** Run `cargo clippy -- -D warnings` to catch common mistakes.
>     - _Note:_ Fix any warnings or errors reported.
> 3.  **Test:** Run `cargo test` to ensure all tests pass.
>
> **DO NOT PUSH if any of these produce errors or unformatted code.**

## Journal Guidelines

- **Date:** ALWAYS use the current date for new journal entries. Do NOT copy the date from previous entries. Check the "current local time" provided in your context.
- **Structure:** Follow the format defined in your specific agent file in the `.jules/` directory (e.g., `.jules/bolt.md`, `.jules/architect.md`, etc.).
