# Bingo RETE Rules Engine Documentation

This document provides a comprehensive index to all documentation for the Bingo RETE Rules Engine.

## 📖 Essential Documentation

### Core Guides
- **[📋 Comprehensive Guide](docs/comprehensive-guide.md)** - Complete documentation index and navigation guide
- **[🔧 API Reference](docs/api-reference.md)** - Complete API documentation with examples and performance notes  
- **[👨‍💻 Developer Guide](docs/developer-guide.md)** - In-depth development guide with best practices and workflows

### Quick Start Paths

#### For New Users
1. **Overview**: Start with the [Project README](README.md) for system overview and features
2. **Getting Started**: Follow the [Comprehensive Guide](docs/comprehensive-guide.md) for guided learning
3. **API Usage**: Use the [API Reference](docs/api-reference.md) for implementation details

#### For Developers  
1. **Development Setup**: Follow the [Developer Guide](docs/developer-guide.md) environment setup
2. **Code Architecture**: Review the [Architecture Specification](specs/architecture.md)
3. **Contributing**: See the [Developer Guide](docs/developer-guide.md) workflow and standards

#### For API Users
1. **gRPC API**: Start with the [gRPC API Specification](specs/grpc-api.md)
2. **Client Setup**: Follow [Client Setup Guide](docs/client-setup.md) 
3. **Deployment**: See [gRPC Deployment Guide](docs/grpc-deployment-guide.md)

## 📁 Documentation Categories

### API & Integration
- **[Client Setup Guide](docs/client-setup.md)** - Step-by-step client setup for multiple languages
- **[gRPC Deployment Guide](docs/grpc-deployment-guide.md)** - Production deployment instructions
- **[Request Lifecycle](docs/request-lifecycle.md)** - End-to-end request flow documentation

### Development & Testing
- **[Testing Guide](docs/testing-guide.md)** - Complete testing framework and methodology
- **[Test Categories](docs/test-categories.md)** - Detailed test categorization and execution
- **[Testing Settings](docs/testing-settings.md)** - Environment-specific testing configurations
- **[AI Contributor Guide](docs/ai-contributor-guide.md)** - Guidelines for AI assistants working on the codebase

### Performance & Optimization  
- **[Performance Tests](docs/performance-tests.md)** - Benchmark results and testing methodologies
- **[Performance Testing Framework](docs/performance-testing.md)** - Adaptive performance testing documentation
- **[Cache Lifecycle](docs/cache-lifecycle.md)** - Caching system documentation

### Production & Operations
- **[Production Deployment Guide](docs/production-deployment-guide.md)** - Complete production deployment instructions
- **[Production Deployment Checklist](docs/production-deployment-checklist.md)** - Comprehensive pre-production validation checklist
- **[Security Hardening Checklist](docs/security-hardening-checklist.md)** - Security configuration and hardening guide

### Business Domain Applications
- **[Payroll Engine](docs/payroll-engine.md)** - Payroll processing implementation
- **[Compliance Engine](docs/compliance-engine.md)** - Compliance rule processing  
- **[Wage Cost Estimation Engine](docs/wage-cost-estimation-engine.md)** - Cost estimation features
- **[TRONC Engine](docs/tronc-engine.md)** - TRONC system integration

## 🔬 Technical Specifications

For detailed technical specifications, see the [SPECS.md](SPECS.md) index and the [specs/](specs/) directory:

- **[Architecture](specs/architecture.md)** - System design and component relationships
- **[RETE Algorithm](specs/rete-algorithm.md)** - Core pattern matching algorithm
- **[RETE Algorithm Implementation](specs/rete-algorithm-implementation.md)** - Implementation details
- **[Rule Specification](specs/rule-specification.md)** - Rule design patterns and syntax
- **[Built-in Calculators](specs/built-in-calculators.md)** - Calculator reference and plugin system
- **[Calculator DSL Guide](specs/calculator-dsl-guide.md)** - Domain-specific language for calculations
- **[gRPC API](specs/grpc-api.md)** - Complete protocol specifications
- **[Performance](specs/performance.md)** - Performance characteristics and benchmarks

## 🏗️ Architecture Overview

The Bingo Rules Engine implements the RETE algorithm for efficient rule processing with:

- **High-performance gRPC streaming API** - Sub-millisecond response times
- **Plugin-based calculator system** - Extensible business logic framework
- **Shared type system** - Eliminates circular dependencies
- **Web interface** - Management and monitoring capabilities
- **Stateless architecture** - Session-based rule compilation
- **Memory-optimized data structures** - Arena allocation and object pooling
- **Advanced caching and indexing** - Multi-level caching strategy
- **Real-time fact processing** - Incremental O(Δfacts) complexity

## 🚀 Key Features

### RETE Algorithm Implementation
- **O(Δfacts) complexity** - Only processes new/changed facts
- **Alpha/Beta memory networks** - Efficient pattern matching
- **Incremental processing** - 12-18x performance improvements
- **Conflict resolution** - Multiple prioritization strategies

### Enterprise Performance
- **High throughput** - Up to 1.9M facts/sec for optimized workloads
- **Scalable architecture** - Supports datasets up to 2M+ facts
- **Memory efficiency** - Linear scaling ~1.6GB per 1M facts
- **Thread safety** - Full Send + Sync implementation

### Business Engine Support
- **Multi-domain architecture** - Compliance, Payroll, and TRONC engines
- **Advanced calculators** - Weighted aggregation, proportional allocation
- **Business logic framework** - Extensible plugin system
- **Rule templates** - Pre-configured business scenarios

## 📋 Quick Navigation

### By Task
- **Getting Started**: [README.md](README.md) → [Comprehensive Guide](docs/comprehensive-guide.md)
- **API Development**: [API Reference](docs/api-reference.md) → [Client Setup](docs/client-setup.md)
- **System Architecture**: [Architecture Spec](specs/architecture.md) → [RETE Algorithm](specs/rete-algorithm.md)
- **Performance Analysis**: [Performance Tests](docs/performance-tests.md) → [Performance Spec](specs/performance.md)
- **Production Deployment**: [Deployment Guide](docs/production-deployment-guide.md) → [Deployment Checklist](docs/production-deployment-checklist.md)
- **Testing**: [Testing Guide](docs/testing-guide.md) → [Test Categories](docs/test-categories.md)

### By Role
- **Developers**: [Developer Guide](docs/developer-guide.md) + [AI Contributor Guide](docs/ai-contributor-guide.md)
- **DevOps**: [Production Deployment Guide](docs/production-deployment-guide.md) + [Security Hardening](docs/security-hardening-checklist.md)
- **API Users**: [API Reference](docs/api-reference.md) + [gRPC API Spec](specs/grpc-api.md)
- **Business Users**: [Payroll Engine](docs/payroll-engine.md) + [Compliance Engine](docs/compliance-engine.md)

## 🎯 Performance Highlights

- **560K facts/sec** - Basic processing throughput
- **1.9M facts/sec** - Optimized workload processing
- **12-18x speedup** - RETE algorithm performance improvement
- **Linear scaling** - Memory usage scales predictably
- **Sub-millisecond** - Rule evaluation latency

For detailed performance analysis, see the [Performance Tests](docs/performance-tests.md) and [Performance Specification](specs/performance.md).