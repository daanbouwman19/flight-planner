## 2024-05-24 - Zero-Allocation Search in UTF-8 Strings
**Learning:** Checking `is_ascii()` on a haystack is O(N) and redundant if checking for an ASCII needle via byte comparison. An ASCII byte in a query can never match a UTF-8 continuation byte or a leading byte of a multi-byte character.
**Action:** When searching for an ASCII needle in a potentially non-ASCII haystack, skip `haystack.is_ascii()` and run the byte-scan loop directly. Only fall back to `to_lowercase()` (allocation) if the byte scan fails AND the haystack contains non-ASCII characters (to handle edge cases like Kelvin sign 'K' -> 'k').

## 2026-01-14 - Loop Fusion and Iterator Parity
**Learning:** When manually fusing `min_by_key` and `max_by_key` into a single loop for O(N) efficiency, remember that `min_by` typically takes the *first* match (`<`) while `max_by` takes the *last* match (`>=`) in Rust's standard library. Failing to respect this can break tie-breaking logic.
**Action:** Use strict inequality for min and inclusive inequality for max when manually unrolling iterator logic.
