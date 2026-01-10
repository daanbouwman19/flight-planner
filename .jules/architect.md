# Architect's Journal

## 2026-01-10 - Deeply Nested Date Formatting Logic
**Smell:** Deeply nested `match` statements and imperative logic in `date_utils::format_date_for_display`.
**Insight:** The function mixes control flow (matching `Option`) with parsing logic and formatting, creating a "arrow code" shape that is hard to scan. It also imperatively handles "Never" fallback.
**Prevention:** Prefer functional combinators (`map`, `filter`, `unwrap_or`) for transforming `Option` and `Result` types. This flattens the structure and clearly expresses the transformation pipeline.
