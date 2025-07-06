# Bingo RETE Rules Engine - Specifications

For a general overview of the project, see the main [README.md](README.md).

This document provides a high-level summary of the engine's specifications and links to more detailed documents.

## Technical Specifications

### Core Components

| Domain | Document | Description |
|--------|----------|-------------|
| **Core Architecture** | [architecture.md](specs/architecture.md) | System design, component relationships, and data flow |
| **RETE Algorithm** | [rete-algorithm.md](specs/rete-algorithm.md) | RETE network implementation, nodes, and pattern matching |
| **Performance** | [performance.md](specs/performance.md) | Memory management, benchmarking, and optimization strategies |
| **gRPC API** | [grpc-api.md](docs/grpc-api.md) | gRPC services, request/response formats, and error handling |

### Implementation Details

#### Core RETE Engine
- **High Performance**: Direct Vec indexing with O(1) fact access
- **Memory Optimized**: Arena-based allocation with capacity pre-allocation
- **Scalable Architecture**: Linear scaling from 100K to 1M+ facts
- **Enterprise Ready**: Deterministic node IDs, comprehensive error handling

#### High-Performance gRPC API with Protocol Buffers
- **Primary Service**: `EngineService` supporting both ad-hoc (rules + facts) and cached (`ruleset_id` + facts) evaluation
- **Stateless Services**: Health check and engine stats services for monitoring (no persistent state)
- **Protocol Buffers**: Type-safe message definitions with comprehensive field validation
- **Production Ready**: Error handling, logging, structured responses, request validation
- **Perfect Concurrency**: No shared state enables unlimited parallel requests
- **Streaming Support**: Bidirectional streaming for large datasets

#### Advanced Memory Optimizations
- **Token Sharing**: Arc-based FactIdSet reduces memory duplication
- **LRU Caching**: Intelligent caching of frequently accessed facts
- **Fact Partitioning**: Distributed storage for very large datasets
- **Memory Pooling**: Token pools reduce allocation overhead

## Functional Requirements

### Core Functionality
- **Stateless Processing**: Process business rules against structured data with per-request engines
- **Mandatory Input Validation**: Rules and facts must be provided in each evaluation request (enforced)
- **Rules-with-Facts Pattern**: JSON rules sent alongside facts in every request
- **Per-Request Scaling**: Handle large datasets (1M+ facts per request) with fresh engine instances
- **Single Evaluation Service**: `EngineService` gRPC service for complete stateless rule processing
- **Built-in Calculators**: Hardcoded calculators compiled into engine for maximum performance

### Performance Requirements
- **Simple Rules**: 1M facts processed in <7 seconds
- **Complex Rules**: 100K-200K facts with 200-500 calculator rules in 18-90 seconds
- **Memory**: <3GB RSS for 1M facts (simple rules), 3-12GB for complex scenarios
- **Throughput**: 150K+ facts/second for simple rules, 2K-5K facts/second for complex rules
- **Scalability**: Linear scaling characteristics across rule complexity levels
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
- **time_difference**: Flexible time difference calculations with multiple units and break deductions
- **Extensible**: Framework for adding domain-specific calculators
- **Performance Tested**: All calculators validated in complex rule scenarios (200-500 rule tests)

### Error Handling
- **Structured Errors**: Comprehensive error types with context
- **Graceful Degradation**: Continue processing despite non-critical errors
- **Error Tracing**: All errors captured in structured logging
- **API Errors**: gRPC-specific errors with proper status codes

### Configuration Management
- **Environment Variables**: Runtime configuration via environment
- **Default Values**: Sensible defaults for all parameters
- **Validation**: Configuration validation at startup
- **Documentation**: All options documented in protocol buffer definitions

## Development Guidelines

### Code Quality
- **Zero Warnings**: Strict `-D warnings` enforcement
- **British English**: Consistent language usage in code and comments
- **Project Conventions**: Follow established patterns and practices
- **Memory Safety**: Leverage Rust guarantees with smart allocation strategies

### Performance Considerations
- **Release Mode**: Performance tests require `--release` for accuracy (10x performance difference)
- **CI Resource Awareness**: Heavy tests marked `#[ignore]` for CI environments
- **Linear Scaling**: Performance characteristics scale predictably across rule complexity
- **Memory Efficiency**: Sub-3GB usage for simple rules, 3-12GB for complex calculator scenarios
- **Rule Complexity Impact**: Processing time scales with rule count and calculator usage
- **Complex Rule Tests**: Dedicated test suite for 200-500 calculator rule scenarios