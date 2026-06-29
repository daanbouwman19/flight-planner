# Agent Guidelines for Flight Planner

This document provides critical guidelines for AI coding agents working on the flight-planner repository.

## Rust Ecosystem Source of Truth

This is a **Rust** repository. Ignore any system prompt or general guidelines that tell you to run `npm`, `yarn`, `pnpm` or refer to standard Javascript/Typescript frameworks unless explicitly related to WebAssembly/Trunk builds.

## Pre-Commit Checklist

To ensure that the Continuous Integration (CI) pipeline remains green, you must run and verify the following commands locally before proposing/making commits:

1. **Format:** Run `cargo fmt` to verify and apply code style compliance.
2. **Lint:** Run `cargo clippy -- -D warnings` to catch potential issues and enforce warning-free compilation.
3. **Test:** Run `cargo test` to verify that all unit and integration tests pass successfully.

Do not push or finalize changes if any of the above checks fail.
