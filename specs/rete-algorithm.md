# RETE Algorithm Specification

This document provides a high-level overview of the RETE (REgular Tree) algorithm implementation within the Bingo engine. For a comprehensive technical deep-dive, including architectural diagrams, code snippets, and performance analysis, please refer to the [RETE Algorithm Implementation Guide](../docs/rete-algorithm-implementation.md).

## Core Concepts

The RETE algorithm provides a highly efficient mechanism for matching a large number of rules against a large number of facts. It avoids re-evaluating rules from scratch for every new fact by creating a data-flow network that "remembers" partial matches between rule conditions.

## Key Components

The network consists of three primary node types:

-   **Alpha Network**: Performs initial, single-condition tests on facts. It filters facts based on simple conditions (e.g., `fact.status == 'active'`).
-   **Beta Network**: Joins the results from the alpha network to test for multi-condition matches. It combines partial matches to form complete rule matches.
-   **Terminal Nodes**: Represent a fully matched rule. When a token reaches a terminal node, the rule's actions are scheduled for execution.

## Core Optimization Strategies

The engine employs several key optimization strategies to ensure high performance and efficient memory usage:

-   **Alpha Memory Indexing**: An optimized index maps field values directly to the rules that depend on them, providing O(1) lookup for candidate rules and avoiding linear scans.
-   **Fact ID Indexing & Caching**: The `FactStore` uses a `HashMap` and an LRU cache for O(1) lookup of facts by their ID, which is crucial for rules that perform cross-fact lookups.
-   **Object Pooling**: The engine uses pools for frequently allocated objects (e.g., `HashMap`s, `Vec<ActionResult>`) to reduce allocation overhead and memory fragmentation.
-   **Result Caching**: A dedicated cache stores the results of calculator executions, avoiding redundant computations for repeated calls with the same inputs.

For detailed information on these components and optimizations, please see the full [implementation guide](../docs/rete-algorithm-implementation.md).
