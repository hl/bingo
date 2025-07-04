# Performance Specification

This document outlines the performance characteristics, benchmarks, and optimization strategies of the Bingo Rules Engine.

## Performance Benchmarks

The engine is benchmarked across various scales to ensure linear performance and efficient memory usage.

| Scale      | Processing Time | Facts/Second | Memory Usage |
|------------|-----------------|--------------|--------------|
| 1M facts   | 1.04s           | 962K/s       | <1GB         |
| 500K facts | 0.44s           | 1.1M/s       | <500MB       |
| 200K facts | 0.21s           | 952K/s       | <200MB       |
| 100K facts | 0.11s           | 909K/s       | <100MB       |

*Performance tests can be run via `cargo test --release -- --ignored`.*

## Memory Management & Optimization

-   **Arena-based `FactStore`**: The default fact store uses an arena allocator (`bumpalo`) for extremely fast, contiguous memory allocation of facts, minimizing pointer chasing and improving cache locality.
-   **Object Pooling**: To combat allocation overhead in a high-throughput environment, the engine maintains pools for:
    -   `HashMap`s used for calculator inputs.
    -   `Vec<ActionResult>` used to store rule execution results.
-   **Result Caching**: A dedicated `CalculatorResultCache` stores the outcomes of calculator function calls, providing significant speedups when the same calculations are repeated.
-   **Compiled Rule Cache**: The API layer maintains a cache of compiled `ReteNetwork` instances. This is the most significant optimization for production workloads, as it avoids the cost of parsing and building the rule network on every request for static rule sets.
-   **Streaming API**: For very large result sets, the API can stream responses as NDJSON, keeping memory usage low and constant regardless of the number of rule firings.