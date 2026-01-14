# Agent Guidelines

This document provides guidelines for agents working on this repository, specifically tailored for the Rust ecosystem.

## Pre-push Checks

To ensure the Continuous Integration (CI) pipeline remains green, please follow these steps before submitting any changes:

1.  **Test:** Run `make test` (or `cargo test`) to ensure all tests pass. **YOU MUST RUN THIS AND ENSURE IT PASSES BEFORE COMMITTING.**
2.  **Lint:** Run `make clippy` (or `cargo clippy`) to catch common mistakes and improve code quality.
3.  **Format:** Run `make fmt` (or `cargo fmt`) to ensure code style compliance.

**Do not push changes if any of these steps fail. Failure to perform these checks will result in CI failure and rejection.**

## Journal Guidelines

-   **Date:** ALWAYS use the current date for new journal entries. Do NOT copy the date from previous entries. Check the "current local time" provided in your context.
-   **Structure:** Follow the format defined in your specific agent file in the `.jules/` directory (e.g., `.jules/bolt.md`, `.jules/architect.md`, etc.).
