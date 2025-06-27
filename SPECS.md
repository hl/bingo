# Bingo RETE Rules Engine - Specifications

## Overview

Bingo is a high-performance stateless RETE rules engine built in Rust 2024 edition (1.87.0). The system delivers exceptional performance that exceeds enterprise targets by 30-60x margins, processing 1M facts in 693ms with <3GB memory usage.

**Stateless Architecture Design:**
- **Mandatory Rules and Facts**: Each evaluation request must include both rules and facts (validation enforced)
- **Per-Request Engine Instances**: Fresh engine instance created for every `/evaluate` request
- **No Shared State**: Zero lock contention enables perfect horizontal scaling
- **Rules-with-Facts Pattern**: Rules provided alongside facts in each evaluation request
- **Hardcoded Calculators**: Built-in calculators compiled into the engine for maximum performance
- **Unlimited Concurrency**: Stateless design supports unlimited parallel requests

## Performance Characteristics

**Validated Performance:**
- **100K facts**: 57ms (53x faster than 3s target)
- **200K facts**: 104ms (58x faster than 6s target)  
- **500K facts**: 312ms (32x faster than 10s target)
- **1M facts**: 693ms (43x faster than 30s target)

**System Capabilities:**
- **Direct Vec Indexing**: O(1) fact access eliminates HashMap overhead
- **Smart Memory Management**: Adaptive backends with capacity pre-allocation  
- **Linear Scaling**: Validated performance from 100K to 1M+ facts
- **Thread Safety**: Full `Send + Sync` compliance throughout
- **Zero Configuration**: Simplified build system, no feature flags
- **Comprehensive Testing**: 189+ unit tests + 16 performance tests

## Architecture

- **Language**: Rust 2024 edition (1.87.0) with latest features
- **Build System**: Simplified - no feature flags, direct dependencies
- **Performance**: Direct Vec indexing architecture for O(1) access
- **Memory**: <3GB for 1M facts, <500MB for CI environments
- **Thread Safety**: Full `Send + Sync` compliance for modern concurrency
- **Testing**: Quality/performance test separation for CI optimization

## Technical Specifications

### Core Components

| Domain | Document | Description |
|--------|----------|-------------|
| **Core Architecture** | [architecture.md](specs/architecture.md) | System design, component relationships, and data flow |
| **RETE Algorithm** | [rete-algorithm.md](specs/rete-algorithm.md) | RETE network implementation, nodes, and pattern matching |
| **Performance** | [performance.md](specs/performance.md) | Memory management, benchmarking, and optimization strategies |
| **Web API** | [web-api.md](specs/web-api.md) | HTTP endpoints, request/response formats, and error handling |

### Implementation Details

#### Core RETE Engine
- **High Performance**: Direct Vec indexing with O(1) fact access
- **Memory Optimized**: Arena-based allocation with capacity pre-allocation
- **Scalable Architecture**: Linear scaling from 100K to 1M+ facts
- **Enterprise Ready**: Deterministic node IDs, comprehensive error handling

#### Calculator DSL
- **Business-Friendly Syntax**: Intuitive expressions for domain experts
- **Complete Language**: Arithmetic, logic, strings, functions, conditionals
- **Type Safety**: Comprehensive validation and error reporting
- **Integration**: Seamless embedding in JSON rules via actions

#### Stateless JSON API with OpenAPI
- **Primary Endpoint**: `/evaluate` endpoint for rules-with-facts processing (rules and facts mandatory)
- **Stateless Endpoints**: `/health` and `/engine/stats` for monitoring (no persistent state)
- **OpenAPI 3.0**: Auto-generated documentation with Swagger UI reflecting stateless architecture
- **Type-Safe**: Native JSON types with mandatory field validation
- **Production Ready**: Error handling, logging, structured responses, request validation
- **Perfect Concurrency**: No shared state enables unlimited parallel requests

#### Advanced Memory Optimizations
- **Token Sharing**: Arc-based FactIdSet reduces memory duplication
- **LRU Caching**: Intelligent caching of frequently accessed facts
- **Fact Partitioning**: Distributed storage for very large datasets
- **Memory Pooling**: Token pools reduce allocation overhead

## Functional Requirements

### Core Functionality
- **Stateless Processing**: Process business rules against structured data with per-request engines
- **Mandatory Input Validation**: Rules and facts must be provided in each evaluation request (enforced)
- **Rules-with-Facts Pattern**: JSON rules with calculator DSL sent alongside facts in every request
- **Per-Request Scaling**: Handle large datasets (1M+ facts per request) with fresh engine instances
- **Single Evaluation Endpoint**: `/evaluate` HTTP endpoint for complete stateless rule processing
- **Built-in Calculators**: Hardcoded calculators compiled into engine for maximum performance

### Performance Requirements
- **Performance**: 1M facts processed in <1 second
- **Memory**: <3GB RSS for 1M facts
- **Throughput**: 1.4M+ facts/second sustained
- **Scalability**: Linear scaling characteristics
- **Observability**: Comprehensive tracing and metrics
- **Reliability**: Memory-safe Rust implementation

## Development Architecture

### Test Organization
- **Quality Tests**: Fast execution (<60s) with `cargo test --workspace`
- **Performance Tests**: Comprehensive validation with `cargo test --release -- --ignored`
- **Separation**: Quality and performance tests separated for CI efficiency
- **Zero Tolerance**: All quality tests must pass for code acceptance

### Built-in Calculators
- **threshold_checker**: Validate values against thresholds with compliance reporting
- **limit_validator**: Multi-tier validation with warning/critical/max levels
- **hours_between_datetime**: Calculate hours between datetime values
- **Extensible**: Framework for adding domain-specific calculators

### Error Handling
- **Structured Errors**: Comprehensive error types with context
- **Graceful Degradation**: Continue processing despite non-critical errors
- **Error Tracing**: All errors captured in structured logging
- **API Errors**: HTTP-specific errors with proper status codes

### Configuration Management
- **Environment Variables**: Runtime configuration via environment
- **Default Values**: Sensible defaults for all parameters
- **Validation**: Configuration validation at startup
- **Documentation**: All options documented in OpenAPI

## Development Guidelines

### Code Quality
- **Zero Warnings**: Strict `-D warnings` enforcement
- **British English**: Consistent language usage in code and comments
- **Project Conventions**: Follow established patterns and practices
- **Memory Safety**: Leverage Rust guarantees with smart allocation strategies

### Performance Considerations
- **Release Mode**: Performance tests require `--release` for accuracy
- **CI Resource Awareness**: Heavy tests marked `#[ignore]` for CI environments
- **Linear Scaling**: Performance characteristics scale predictably
- **Memory Efficiency**: Sub-3GB usage for enterprise scale workloads