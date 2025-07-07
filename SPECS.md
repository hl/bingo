# Specification Index

This document provides an index to all specification documents in the `specs/` directory. Each specification covers a specific aspect of the Bingo Rules Engine architecture and implementation.

## Specifications

| Document | Description | Purpose |
|----------|-------------|---------|
| [Architecture](specs/architecture.md) | System design, component relationships, and data flow | Understanding overall system structure and multi-crate workspace design |
| [RETE Algorithm](specs/rete-algorithm.md) | High-level overview of the RETE algorithm | Core rule evaluation engine concepts |
| [RETE Algorithm Implementation](specs/rete-algorithm-implementation.md) | Detailed technical deep-dive into the RETE implementation | Understanding the core engine's implementation |
| [Rule Specification](specs/rule-specification.md) | Complete language implementation guide for rules engine | Cross-language implementation reference and compatibility requirements |
| [Built-in Calculators](specs/built-in-calculators.md) | Predefined calculator system for high-performance calculations | Implementing calculator registry and business logic execution |
| [Calculator DSL Guide](specs/calculator-dsl-guide.md) | Guide to the domain-specific language for calculators | Writing and understanding calculator expressions |
| [gRPC API](specs/grpc-api.md) | gRPC streaming API specification and service definitions | API integration and client implementation |
| [Performance](specs/performance.md) | Performance characteristics, benchmarks, and optimization strategies | Meeting performance requirements and optimization guidance |

## Implementation Workflow

For implementing the Bingo Rules Engine, follow this recommended reading order:

1. **[Architecture](specs/architecture.md)** - Start here to understand the overall system design
2. **[RETE Algorithm](specs/rete-algorithm.md)** - Learn the core pattern matching implementation
3. **[Rule Specification](specs/rule-specification.md)** - Use as comprehensive implementation guide
4. **[Built-in Calculators](specs/built-in-calculators.md)** - Implement calculator system for business logic
5. **[Performance](specs/performance.md)** - Optimize implementation to meet performance targets
6. **[gRPC API](specs/grpc-api.md)** - Add API layer for client integration

## Quick Reference

- **New Implementation**: Start with [Rule Specification](specs/rule-specification.md) for complete cross-language guide
- **Performance Tuning**: See [Performance](specs/performance.md) for benchmarks and optimization strategies  
- **API Integration**: Refer to [gRPC API](specs/grpc-api.md) for service interface details
- **Calculator Development**: Check [Built-in Calculators](specs/built-in-calculators.md) for business logic patterns
- **Algorithm Details**: Consult [RETE Algorithm Implementation](specs/rete-algorithm-implementation.md) for pattern matching internals
- **System Design**: Review [Architecture](specs/architecture.md) for component relationships
