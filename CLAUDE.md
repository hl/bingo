# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Development Commands

### Quality Assurance (Zero-Tolerance Policy)
All checks must pass before making changes:
```bash
cargo fmt --check && cargo clippy -- -D warnings && cargo check --workspace && cargo test --workspace
```

**Critical**: This project maintains a zero-tolerance policy for failing checks. All warnings are treated as errors.

### Core Development Commands
- **Format Code**: `cargo fmt`
- **Lint (Strict)**: `cargo clippy -- -D warnings`
- **Type Check**: `cargo check --workspace`
- **Run Tests**: `cargo test --workspace`
- **Run Single Test**: `cargo test <test_name>`
- **Performance Tests**: `cargo test --release -- --ignored`
- **Build Release**: `cargo build --release`
- **Run gRPC Server**: `cargo run --release --bin bingo`

### Benchmarking
- **Engine Benchmarks**: `cargo bench --package bingo-core`
- **Million Fact Benchmark**: `cargo bench --bin million_fact_bench`

## Architecture Overview

Bingo is a high-performance RETE rules engine with a multi-crate workspace architecture:

### Core Components

```
bingo-core/     - RETE engine implementation, fact stores, memory optimization
bingo-api/      - gRPC API server with Protocol Buffers and streaming
bingo-calculator/ - Plugin-based calculator system for business logic
bingo-types/    - Shared type definitions and FactValue system
bingo-web/      - Web interface for management and monitoring
bingo-performance-test/ - Performance testing utilities and benchmarks
```

### Key Architectural Concepts

#### RETE Algorithm Implementation
- **Alpha Memory**: Hash-indexed single-condition fact matching (O(1) lookups)
- **Beta Network**: Multi-condition token propagation and join operations
- **Working Memory**: Incremental fact lifecycle with O(Δfacts) complexity
- **Conflict Resolution**: Priority-based with multiple strategies (Priority, Salience, Specificity, Lexicographic)

#### Performance Optimizations
- **Arena Allocation**: Minimizes memory fragmentation in `arena_store.rs`
- **Object Pooling**: Reuses frequently allocated objects in `memory_pools.rs`
- **Parallel Processing**: Multi-threaded RETE with work-stealing queues in `parallel_rete.rs`
- **Rule Optimization**: Automatic condition reordering in `rule_optimizer.rs`

#### Business Engine Support
- **Multi-Domain**: Unified system for Compliance, Payroll, and TRONC engines
- **Calculator System**: Extensible plugin architecture for domain-specific calculations
- **Dependency Analysis**: Kahn's algorithm for rule execution ordering in `rule_dependency.rs`

## Code Organization

### Critical Files by Function

#### Core Engine (`bingo-core/src/`)
- `engine.rs` - Main BingoEngine orchestrator
- `rete_network.rs` - RETE network construction and execution
- `rete_nodes.rs` - Individual RETE node implementations
- `alpha_memory.rs` - Single-condition fact indexing
- `beta_network.rs` - Multi-condition rule processing
- `conflict_resolution.rs` - Rule execution prioritization
- `rule_optimizer.rs` - Performance optimization strategies
- `rule_dependency.rs` - Dependency analysis and topological sorting

#### Performance & Monitoring
- `profiler.rs` - Performance profiling and metrics
- `enhanced_monitoring.rs` - Comprehensive observability
- `parallel_rete.rs` - Multi-threaded processing
- `memory_pools.rs` - Memory management optimization

#### API Layer (`bingo-api/src/`)
- `grpc/service.rs` - gRPC service implementation
- `grpc/conversions.rs` - Protocol buffer conversions
- `streaming.rs` - Bidirectional streaming support
- `cache/` - Redis and in-memory caching

#### Calculator System (`bingo-calculator/src/`)
- `calculator.rs` - Calculator trait and plugin system
- `built_in/` - Pre-built calculators for common operations
- `plugin_manager.rs` - Plugin loading and management

## Development Guidelines

### Code Quality Standards
- **Rust Edition**: 2024 (toolchain: 1.88.0)
- **Formatting**: Use rustfmt with project-specific `.rustfmt.toml`
- **Linting**: Clippy with custom `clippy.toml` configuration
- **Documentation**: Comprehensive inline docs required for public APIs
- **Testing**: 361+ tests with 100% success rate requirement

### Performance Considerations
- **Memory Management**: Prefer arena allocation for frequently allocated objects
- **Concurrent Access**: All components must be `Send + Sync`
- **RETE Optimization**: Consider alpha memory sharing and beta network efficiency
- **Benchmarking**: Use criterion for performance testing

### Error Handling
- **Result Types**: Use `BingoResult<T>` for all fallible operations
- **Error Propagation**: Comprehensive error context with `ResultExt`
- **Diagnostics**: Enhanced error diagnostics in `error_diagnostics.rs`

## Testing Strategy

### Test Categories
- **Unit Tests**: Located in `src/` files and `tests/` directories
- **Integration Tests**: End-to-end scenarios in `tests/` directories
- **Performance Tests**: Benchmarks with `#[ignore]` attribute
- **Concurrency Tests**: Thread safety validation

### Running Specific Test Categories
```bash
# Unit tests only
cargo test --lib

# Integration tests
cargo test --test integration

# Performance tests (ignored by default)
cargo test --release -- --ignored

# Specific test file
cargo test --test comprehensive_rete_benchmark
```

## Business Domain Context

### Supported Engines
- **Compliance Engine**: Regulatory compliance monitoring (student visa work hours, etc.)
- **Payroll Engine**: Payroll processing with overtime calculations
- **TRONC Engine**: Tip and gratuity distribution with weighted calculations

### Key Calculator Types
- `weighted_average` - Weighted average calculations for roles
- `proportional_allocator` - Proportional distribution by metrics
- `limit_validator` - Multi-tiered threshold validation
- `percentage_deduct` - Percentage deduction calculations before distribution
- `add` - Addition operations
- `multiply` - Multiplication operations
- `percentage_add` - Percentage addition calculations
- `time_between_datetime` - Time duration calculations

## Performance Expectations

### Throughput Targets
- **Basic Processing**: 560K facts/sec
- **Alpha Memory**: 462K facts/sec with optimization
- **Beta Network**: 407K facts/sec with token propagation
- **Incremental Updates**: 1.2M facts/sec for working memory

### Scalability Characteristics
- **Memory Scaling**: Linear ~1.6GB per 1M facts
- **Rule Independence**: Performance independent of non-matching rules
- **Enterprise Scale**: Validated up to 2M facts (1.4M facts/sec)

## Common Development Patterns

### Adding New Rules
1. Define rule conditions using `Condition` enum
2. Specify actions with `ActionType` variants
3. Use `BingoEngine::add_rule()` for compilation
4. Test with various fact patterns

### Implementing Calculators
1. Implement `Calculator` trait in `bingo-calculator`
2. Add to `built_in/` directory with comprehensive tests
3. Register in plugin system via `plugin_manager.rs`
4. Document calculation logic and error cases

### Performance Optimization
1. Profile using `EngineProfiler` for bottleneck identification
2. Consider rule condition reordering for selectivity
3. Optimize memory usage with arena allocation
4. Validate with comprehensive benchmarks

## Configuration Files

- `.rustfmt.toml` - Code formatting rules (2024 edition, 100 char width)
- `clippy.toml` - Linting configuration with performance thresholds
- `rust-toolchain.toml` - Rust 1.88.0 with required components
- `Cargo.toml` - Workspace configuration with optimized release profile

## Key Dependencies

- **Tokio**: Async runtime for API layer
- **Tonic**: gRPC implementation
- **Serde**: Serialization framework
- **Tracing**: Structured logging and observability
- **Criterion**: Performance benchmarking
- **Proptest**: Property-based testing