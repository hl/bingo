# Performance Specification

This document outlines the high-level performance characteristics and optimization strategies of the Bingo Rules Engine. For detailed, up-to-date benchmark results and instructions on running the performance test suite, please refer to the [Performance Test Suite Documentation](../docs/performance-tests.md).

## Memory Management & Optimization

-   **Arena-based `FactStore`**: The default fact store uses an arena allocator (`bumpalo`) for extremely fast, contiguous memory allocation of facts, minimizing pointer chasing and improving cache locality.
-   **Object Pooling**: To combat allocation overhead in a high-throughput environment, the engine maintains pools for:
    -   `HashMap`s used for calculator inputs.
    -   `Vec<ActionResult>` used to store rule execution results.
-   **Result Caching**: A dedicated `CalculatorResultCache` stores the outcomes of calculator function calls, providing significant speedups when the same calculations are repeated.
-   **Compiled Rule Cache**: The API layer maintains a cache of compiled `ReteNetwork` instances. This is the most significant optimization for production workloads, as it avoids the cost of parsing and building the rule network on every request for static rule sets.
-   **Streaming API**: For very large result sets, the API can stream responses as NDJSON, keeping memory usage low and constant regardless of the number of rule firings.
