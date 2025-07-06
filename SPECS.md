# Specification Index

This document provides an index to all specification documents in the `specs/` directory. Each specification covers a specific aspect of the Bingo Rules Engine architecture and implementation.

## Specifications

| Document | Description | Purpose |
|----------|-------------|---------|
| [Architecture](specs/architecture.md) | System design, component relationships, and data flow | Understanding overall system structure and multi-crate workspace design |
| [Built-in Calculators](specs/built-in-calculators.md) | Predefined calculator system for high-performance calculations | Implementing calculator registry and business logic execution |
| [Performance](specs/performance.md) | Performance characteristics, benchmarks, and optimization strategies | Meeting performance requirements and optimization guidance |
| [RETE Algorithm](specs/rete-algorithm.md) | RETE network implementation details and pattern matching | Core rule evaluation engine implementation |
| [Rule Specification](specs/rule-specification.md) | Complete language implementation guide for rules engine | Cross-language implementation reference and compatibility requirements |
| [gRPC API](specs/web-api.md) | gRPC streaming API specification and service definitions | API integration and client implementation |

## Implementation Workflow

For implementing the Bingo Rules Engine, follow this recommended reading order:

1. **[Architecture](specs/architecture.md)** - Start here to understand the overall system design
2. **[RETE Algorithm](specs/rete-algorithm.md)** - Learn the core pattern matching implementation
3. **[Rule Specification](specs/rule-specification.md)** - Use as comprehensive implementation guide
4. **[Built-in Calculators](specs/built-in-calculators.md)** - Implement calculator system for business logic
5. **[Performance](specs/performance.md)** - Optimize implementation to meet performance targets
6. **[gRPC API](specs/web-api.md)** - Add API layer for client integration

## Quick Reference

- **New Implementation**: Start with [Rule Specification](specs/rule-specification.md) for complete cross-language guide
- **Performance Tuning**: See [Performance](specs/performance.md) for benchmarks and optimization strategies  
- **API Integration**: Refer to [gRPC API](specs/web-api.md) for service interface details
- **Calculator Development**: Check [Built-in Calculators](specs/built-in-calculators.md) for business logic patterns
- **Algorithm Details**: Consult [RETE Algorithm](specs/rete-algorithm.md) for pattern matching internals
- **System Design**: Review [Architecture](specs/architecture.md) for component relationships