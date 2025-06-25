# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Bingo is a production-ready high-performance RETE rules engine built in **Rust 2024 edition (1.87.0)**. It delivers exceptional performance that exceeds enterprise targets by **4-5x margins**, processing 1M facts in 6.59s (vs 30s target) with <3GB memory usage. Designed for simplicity in the design stage with zero configuration complexity.

## Development Commands

### Basic Operations
```bash
# Build the project (optimized for release)
cargo build --release

# Run all unit tests (167 tests)
cargo test --lib

# Run performance tests (CI-appropriate)
cargo test --release

# Run heavy performance tests (manual only)
cargo test --ignored --release

# Code quality checks (zero tolerance)
cargo fmt --check
cargo clippy -- -D warnings
cargo check --workspace

# Start web server (http://127.0.0.1:3000)
cargo run --bin bingo

# Show engine explanation
cargo run --bin bingo explain
```

### Performance Testing
```bash
# CI-appropriate scaling tests (100K, 200K facts)
cargo test --package bingo-core --test scaling_validation_test --release

# Manual heavy tests (500K, 1M facts)
cargo test --package bingo-core --test scaling_validation_test --ignored --release

# Comprehensive benchmarks
cargo bench

# Specific benchmarks
cargo bench --bench engine_bench
cargo bench --bench million_fact_bench
```

### API Testing
```bash
# Health check
curl http://localhost:3000/health

# Evaluate facts
curl -X POST http://localhost:3000/evaluate \
  -H "Content-Type: application/json" \
  -d '{"facts": [{"id": 1, "data": {"fields": {"employee_id": 12345, "hours_worked": 42.5}}}]}'

# OpenAPI documentation
open http://localhost:3000/swagger-ui/
```

## Architecture Overview

### Workspace Structure
- **bingo-core**: Core RETE engine with direct Vec indexing optimization
- **bingo-rete**: Low-level RETE algorithm implementation  
- **bingo-api**: HTTP API server with Axum + OpenAPI documentation

### Key Components

#### Core Engine (`bingo-core`)
- **RETE Network**: Optimized alpha/beta nodes with token sharing
- **Fact Store**: ArenaFactStore with O(1) direct Vec indexing (always available)
- **Calculator DSL**: Business-friendly expression language for rules
- **Memory Management**: Smart pre-allocation with capacity hints
- **Performance Optimization**: Direct indexing eliminates HashMap overhead

#### Performance Features
- **Direct Vec Indexing**: O(1) fact access using fact.id as Vec index
- **Memory Pre-allocation**: Capacity hints for large datasets (1M+ facts)
- **Field Indexing**: Optimized for enterprise patterns (entity_id, status, etc.)
- **Parallel Processing**: Rayon-based parallel fact processing (always available)
- **Linear Scaling**: Validated performance characteristics from 100K to 1M facts

#### API Layer (`bingo-api`)
- **REST Endpoints**: `/health`, `/evaluate` for fact processing
- **OpenAPI Integration**: Auto-generated documentation with Swagger UI
- **Structured Logging**: JSON output with tracing integration

### Rule Types
1. **Built-in Rules**: Hardcoded in Rust for maximum performance
2. **JSON Rules with Calculator DSL**: Business-friendly syntax embedded in JSON

## Performance Characteristics

### Validated Performance (Release Mode)
- **100K facts**: 635ms (4.7x faster than 3s target)
- **200K facts**: 1.16s (5.2x faster than 6s CI target)  
- **500K facts**: 2.16s (4.6x faster than 10s target)
- **1M facts**: 6.59s (4.6x faster than 30s target)

### Memory Efficiency
- **CI environments**: <500MB for 200K facts
- **Enterprise scale**: <3GB for 1M facts (well under 4GB target)
- **Linear growth**: Sub-linear memory scaling confirmed

### Optimization Features
- **Direct Vec indexing**: Eliminates HashMap lookup overhead
- **Memory pre-allocation**: Capacity hints prevent reallocation
- **Field indexing**: Optimized for common enterprise patterns
- **Batch processing**: Efficient handling of large fact sets

## Configuration

### Environment Variables
- `BINGO_HOST`: Server host (default: 127.0.0.1)
- `BINGO_PORT`: Server port (default: 3000)
- `RUST_LOG`: Logging level (use "bingo=debug,info")

### Build System (Simplified)
- **No feature flags**: Everything available by default (design stage appropriate)
- **Direct dependencies**: bumpalo and rayon always available
- **Zero configuration**: Simple `cargo build` workflow
- **Thread safety**: Full `Send + Sync` compliance throughout

## Development Guidelines

### Code Organization
- **Design Stage Focused**: Simple, clean architecture without premature optimization
- **Performance First**: Direct Vec indexing architecture for exceptional speed
- **Memory Safe**: Rust guarantees with smart allocation strategies
- **Zero Warnings**: Strict `-D warnings` enforcement

### Testing Strategy
- **Unit Tests**: 167 comprehensive tests for all components
- **Performance Tests**: CI-appropriate (100K, 200K) + manual heavy tests (500K, 1M)
- **CI Optimization**: Resource-appropriate tests for reliable automation
- **Release Mode**: Performance tests MUST run in release mode for accuracy
- **Zero Tolerance**: All tests, linters, and checks MUST pass

### Performance Considerations
- **Release Mode Required**: Performance tests are 10x slower in debug mode
- **CI Resource Awareness**: Heavy tests marked `#[ignore]` for CI environments
- **Linear Scaling**: Performance characteristics scale predictably
- **Memory Efficiency**: Sub-3GB usage for enterprise scale workloads

## Key Files and Modules

### Core Implementation
- `crates/bingo-core/src/engine.rs`: Main engine orchestration
- `crates/bingo-core/src/fact_store.rs`: ArenaFactStore with direct Vec indexing
- `crates/bingo-core/src/calculator/`: Calculator DSL implementation

### Performance Validation
- `crates/bingo-core/tests/scaling_validation_test.rs`: Enterprise performance validation
- `crates/bingo-core/benches/`: Criterion-based benchmarking

### API and Integration
- `crates/bingo-api/src/lib.rs`: HTTP API implementation with OpenAPI
- `crates/bingo-api/src/types.rs`: API type definitions

## Troubleshooting

### Performance Issues
- **Slow tests**: Ensure running with `--release` flag for performance tests
- **Memory usage**: Use capacity hints for large datasets (500K+ facts)
- **CI failures**: Heavy tests marked `#[ignore]` - run manually with `--ignored`

### Development Issues
- **Build failures**: Run `cargo check --workspace` for comprehensive validation
- **Test failures**: All 167 unit tests must pass - zero tolerance policy
- **Format issues**: Run `cargo fmt` before committing

### Debugging Features
- **Structured logging**: Use `RUST_LOG=bingo=debug` for detailed output
- **Performance tracing**: Built-in tracing for hot paths
- **Memory tracking**: MemoryTracker for allocation analysis