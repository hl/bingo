# RETE Algorithm Implementation

This document describes the implementation of the RETE (REgular Tree) algorithm in the Bingo engine.

## Core Concepts

The RETE algorithm provides a highly efficient mechanism for matching a large number of rules against a large number of facts. It avoids re-evaluating rules from scratch for every fact by creating a data-flow network that "remembers" partial matches.

## Key Components

-   **Alpha Network**: Performs initial, intra-condition tests. It filters facts based on simple conditions (e.g., `fact.status == 'active'`).
    -   **`AlphaMemory`**: An optimized index that maps field values to the rules that depend on them. This provides an O(1) lookup for candidate rules for a given fact, drastically reducing the number of rules to evaluate.

-   **Beta Network**: Joins the results of the alpha network to test for inter-condition matches. It combines partial matches to form complete rule matches.
    -   **`BetaMemory`**: Tracks partial matches for multi-condition rules. When a fact satisfies one condition, it creates or extends a `PartialMatch` object, which is stored until the rule is fully matched or the partial match expires.

-   **Terminal Nodes**: Represent a fully matched rule. When a token reaches a terminal node, the rule's actions are scheduled for execution.

## Optimizations

-   **Fact ID Indexing**: The `FactStore` uses a `HashMap` to provide O(1) lookup of facts by their ID, which is crucial for rules that perform cross-fact lookups.
-   **Object Pooling**: The engine uses pools for frequently allocated objects like `HashMap`s (for calculator inputs) and `Vec<ActionResult>` to reduce allocation overhead and memory fragmentation.
-   **Result Caching**: The `CalculatorResultCache` stores the results of calculator executions. If the same calculator is called with the same inputs, the cached result is returned instantly, avoiding redundant computation.