## 2024-05-23 - History Data Loading Optimization
**Learning:** Passing large vectors around (like 40k+ airports) and rebuilding auxiliary structures (like HashMaps) on the fly is a performance anti-pattern in this codebase.
**Action:** Always check if a full O(N) map construction is necessary. Often, we only need to map a small subset of the data (O(M)). Filtering the dataset before building the map can significantly reduce memory allocation and improve performance.
