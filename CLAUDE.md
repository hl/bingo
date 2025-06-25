# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Bingo is a production-ready high-performance RETE rules engine built in Rust 2024 edition. It processes large-scale datasets with complex business rules, capable of handling 3 million facts against 2,000 rules efficiently with sub-second evaluation times and <300MB RSS memory usage.

## Development Commands

### Basic Operations
```bash
# Build the project
cargo build

# Run all tests
cargo test

# Run benchmarks
cargo bench

# Code quality checks
cargo check
cargo clippy

# Start web server (default on http://127.0.0.1:3000)
cargo run --bin bingo

# Show engine explanation
cargo run --bin bingo explain
```

### Testing Specific Components
```bash
# Test specific crate
cargo test -p bingo-core
cargo test -p bingo-rete
cargo test -p bingo-api

# Run specific test
cargo test test_name

# Run tests with output
cargo test -- --nocapture
```

### Performance and Benchmarking
```bash
# Run all benchmarks with HTML reports
cargo bench

# Run specific benchmark
cargo bench --bench engine_bench
cargo bench --bench million_fact_bench

# Memory profiling (requires additional setup)
cargo run --release --features memory-profiling
```

### API Testing
```bash
# Health check
curl http://localhost:3000/health

# Evaluate facts
curl -X POST http://localhost:3000/evaluate \
  -H "Content-Type: application/json" \
  -d '{"facts": [{"id": 1, "data": {"fields": {"employee_id": 12345, "hours_worked": 42.5}}}]}'
```

## Architecture Overview

### Workspace Structure
- **bingo-core**: Core RETE engine implementation with optimization modules
- **bingo-rete**: Low-level RETE algorithm implementation (nodes, token propagation)
- **bingo-api**: HTTP API server with Axum web framework

### Key Components

#### Core Engine (`bingo-core`)
- **RETE Network**: Alpha/beta nodes, token propagation, pattern matching
- **Fact Store**: Multiple backends (Vec, Cached, Partitioned) with adaptive selection
- **Calculator DSL**: Business-friendly expression language for rules
- **Memory Management**: Arena allocation, memory pools, LRU caching
- **Performance Optimization**: Token sharing, incremental processing, bloom filters

#### Performance Features
- **Adaptive Backends**: Automatic selection of optimal fact storage strategy
- **Memory Optimizations**: Token sharing with Arc, LRU caching, fact partitioning
- **Distributed Processing**: Multi-node RETE networks with fault tolerance
- **Stream Processing**: Windowing and aggregation for real-time data

#### API Layer (`bingo-api`)
- **REST Endpoints**: `/health`, `/evaluate` for fact processing
- **OpenAPI Integration**: Auto-generated documentation with Swagger UI
- **Structured Logging**: JSON output with tracing integration

### Rule Types
1. **Built-in Rules**: Hardcoded in Rust for maximum performance
2. **JSON Rules with Calculator DSL**: Business-friendly syntax embedded in JSON

### Calculator DSL Capabilities
- Arithmetic, logical, and string operations
- Conditional set logic with first-match semantics
- Built-in functions (math, string, utility)
- Type-safe evaluation with comprehensive error handling
- Variable extraction for dependency analysis

## Performance Characteristics

### Validated Targets
- **Throughput**: 1M facts in <30 seconds (enterprise production target)
- **Memory**: <4GB RSS for 1M facts, <1.3GB for 500K facts
- **Latency**: P95 < 500ms for rule evaluation
- **Scale**: 2,000 rules without performance degradation

### Optimization Features
- Hash-based fact indexing for O(1) lookups
- RETE node memory pooling to reduce allocations
- Incremental fact processing to avoid full network traversal
- Batch processing mode for improved throughput
- Advanced field indexing with bitmap optimizations

## Configuration

### Environment Variables
- `BINGO_HOST`: Server host (default: 127.0.0.1)
- `BINGO_PORT`: Server port (default: 3000)
- `RUST_LOG`: Logging level (use "bingo=debug,info")

### Clippy Configuration
Custom clippy.toml with performance-focused lints:
- Cognitive complexity threshold: 25
- Type complexity threshold: 250
- Memory management optimizations enabled

## Development Guidelines

### Code Organization
- Modular architecture with clear separation of concerns
- Comprehensive tracing and metrics throughout
- Memory-safe patterns with arena allocation
- Error handling with anyhow/thiserror

### Testing Strategy
- Unit tests for individual components
- Integration tests for end-to-end workflows
- Performance benchmarks with Criterion
- Property-based testing with proptest for edge cases
- **BSSN Principle**: You MUST NOT implement any placeholder or future code, ONLY implement and suggest code that is needed for now
- **Testing Non-Negotiables**: When it comes to tests, linters, code style checks, there is ZERO compromises, everything MUST pass

### Performance Considerations
- All hot paths are instrumented with tracing
- Memory allocations are minimized in critical sections
- Benchmark regressions are caught with automated testing
- Profiling tools integrated for memory and CPU analysis

## Key Files and Modules

### Core Implementation
- `crates/bingo-core/src/engine.rs`: Main engine orchestration
- `crates/bingo-core/src/rete_network.rs`: RETE network implementation
- `crates/bingo-core/src/fact_store.rs`: Fact storage abstractions
- `crates/bingo-core/src/calculator/`: Calculator DSL implementation

### Performance Modules
- `crates/bingo-core/src/memory_pools.rs`: Token and memory pooling
- `crates/bingo-core/src/optimization_coordinator.rs`: Performance optimization
- `crates/bingo-core/src/unified_memory_coordinator.rs`: Memory management

### API and Integration
- `crates/bingo-api/src/lib.rs`: HTTP API implementation
- `crates/bingo-api/src/types.rs`: API type definitions

## Troubleshooting

### Common Issues
- **Memory exhaustion**: Check fact store backend selection and enable partitioning
- **Performance degradation**: Run benchmarks to identify bottlenecks
- **Rule compilation errors**: Validate Calculator DSL syntax and type safety

### Debugging Features
- Comprehensive tracing with structured JSON output
- Debug hooks for step-by-step execution analysis
- Memory profiler for allocation tracking
- Performance regression testing suite