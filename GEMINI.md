# Bingo Rules Engine: Gemini Development Guide

This document provides guidance for Gemini and other AI assistants working with the Bingo codebase.

## Project Overview

Bingo is a production-ready, high-performance RETE rules engine built in Rust (2024 edition). It is engineered for extreme speed and memory efficiency, capable of processing over 1 million facts in under 7 seconds, far exceeding typical enterprise performance targets. The engine features a sophisticated, multi-layered architecture that provides both a high-performance core and a user-friendly abstraction layer.

**Key Architectural Principles:**
- **Performance First:** The design prioritizes speed and memory efficiency through advanced techniques like direct Vec indexing, arena allocation, and data-parallelism.
- **Dual Accessibility:** The engine is designed for two primary audiences: developers who need maximum control via the core Rust API, and business users who can define complex logic using a simple, sandboxed Calculator DSL.
- **Production Grade:** The system is built with comprehensive observability (structured logging, metrics), a robust testing suite, and a stable, versioned API.

## Development Commands

### Primary Quality & Testing Workflow

To ensure the repository is in a clean state, run the full suite of quality checks and tests. **There is a zero-tolerance policy for any failing checks.**

```bash
# Run format check, clippy, workspace check, and all tests in release mode
cargo fmt --check && cargo clippy -- -D warnings && cargo check --workspace && cargo test --lib && cargo test --release
```

### Individual Commands

- **Build for Production:** `cargo build --release`
- **Run All Unit Tests:** `cargo test --lib`
- **Run Performance Tests:** `cargo test --release`
- **Run Heavy Performance Tests (Manual Only):** `cargo test --ignored --release`
- **Check Formatting:** `cargo fmt --check`
- **Linting (Strict):** `cargo clippy -- -D warnings`
- **Full Project Check:** `cargo check --workspace`

### Running the Application

- **Start Web Server:** `cargo run --bin bingo`
  - API will be available at `http://127.0.0.1:3000`.
  - Interactive Swagger documentation at `http://127.0.0.1:3000/swagger-ui/`.
- **Show Engine Explanation:** `cargo run --bin bingo explain`

## Architecture & Code Structure

- `crates/bingo-api`: The public-facing HTTP API built with Axum. This crate handles web requests, serialization, and provides OpenAPI documentation. It is the entry point for all external interactions.
- `crates/bingo-core`: The heart of the engine. This crate contains:
  - `engine.rs`: The main `BingoEngine` struct, which serves as the primary public API for the core library.
  - `rete_network.rs`: The orchestrator for the RETE algorithm, managing all nodes, memory, caches, and trackers. **This is the most complex part of the system.**
  - `rete_nodes.rs`: The implementation of the individual RETE nodes (Alpha, Beta, Terminal).
  - `fact_store.rs`: A critical abstraction (`FactStore` trait) with multiple backend implementations (`VecFactStore`, `ArenaFactStore`, `CachedFactStore`, `PartitionedFactStore`) for optimized fact storage.
  - `calculator/`: The full implementation of the Calculator DSL, including the parser, AST, and evaluator.
- `crates/bingo-rete`: A legacy crate containing foundational data structures. Its functionality has been largely integrated into `bingo-core`.

## Recent Refactoring (June 2025)

The codebase recently underwent a significant refactoring to improve concurrency and resolve critical bugs. Key changes include:

1.  **Thread-Safe Engine:** The `BingoEngine` and `ReteNetwork` were refactored to use `&self` for fact processing instead of `&mut self`. This was achieved by wrapping all mutable state in thread-safe locking primitives (`RwLock`, `Mutex`), allowing the API to process multiple requests concurrently.
2.  **True Parallelism:** The `process_facts_parallel` method in `engine.rs` was rewritten to use `rayon` to execute the RETE network evaluation across multiple threads for large fact sets.
3.  **Stable ID Generation:** The unstable `DefaultHasher` was replaced with a stable FNV-1a hash for converting string-based API rule and fact IDs into the `u64` IDs used by the core engine. This ensures state consistency across server restarts.
4.  **Fact ID Preservation:** The API now correctly preserves fact IDs through the request-response cycle, allowing clients to correlate output with input.

These changes have resolved the most significant architectural bottlenecks and correctness issues.
