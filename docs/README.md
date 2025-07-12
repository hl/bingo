# Bingo Rules Engine Documentation

This directory contains comprehensive documentation for the Bingo RETE Rules Engine.

## 📖 Master Documentation Guides

### Essential Reading
- **[📋 Comprehensive Guide](COMPREHENSIVE_GUIDE.md)** - Complete documentation index and navigation guide
- **[🔧 API Reference](API_REFERENCE.md)** - Complete API documentation with examples and performance notes  
- **[👨‍💻 Developer Guide](DEVELOPER_GUIDE.md)** - In-depth development guide with best practices and workflows

## 🚀 Quick Start Paths

### For New Users
1. **Overview**: Start with the [Project README](../README.md) for system overview and features
2. **Getting Started**: Follow the [Comprehensive Guide](COMPREHENSIVE_GUIDE.md) for guided learning
3. **API Usage**: Use the [API Reference](API_REFERENCE.md) for implementation details

### For Developers  
1. **Development Setup**: Follow the [Developer Guide](DEVELOPER_GUIDE.md) environment setup
2. **Code Architecture**: Review the [Architecture Specification](../specs/architecture.md)
3. **Contributing**: See the [Developer Guide](DEVELOPER_GUIDE.md) workflow and standards

### For API Users
1. **gRPC API**: Start with the [gRPC API Specification](../specs/grpc-api.md)
2. **Client Setup**: Follow [Client Setup Guide](client-setup.md) 
3. **Deployment**: See [gRPC Deployment Guide](grpc-deployment-guide.md)

### For Performance Analysis
1. **Benchmarks**: Review [Performance Tests](performance-tests.md)
2. **Testing Framework**: See [Performance Testing Framework](performance-testing.md)
3. **Optimization**: Check the [Performance section](API_REFERENCE.md#performance-monitoring-api) in API Reference

## 📁 Detailed Documentation

### API & Integration
- **[Client Setup Guide](client-setup.md)** - Step-by-step client setup for multiple languages
- **[gRPC Deployment Guide](grpc-deployment-guide.md)** - Production deployment instructions
- **[Request Lifecycle](request-lifecycle.md)** - End-to-end request flow documentation

### Performance & Optimization  
- **[Performance Tests](performance-tests.md)** - Benchmark results and testing methodologies
- **[Performance Testing Framework](performance-testing.md)** - Adaptive performance testing documentation
- **[Testing Settings](testing-settings.md)** - Environment-specific testing configurations
- **[Cache Lifecycle](cache-lifecycle.md)** - Caching system documentation

### Domain Applications
- **[Payroll Engine](payroll-engine.md)** - Payroll processing implementation
- **[Compliance Engine](compliance-engine.md)** - Compliance rule processing  
- **[Wage Cost Estimation Engine](wage-cost-estimation-engine.md)** - Cost estimation features
- **[TRONC Engine](tronc-engine.md)** - TRONC system integration

## 📋 Technical Specifications

For detailed technical specifications, see the [specs/](../specs/) directory:
- **[Architecture](../specs/architecture.md)** - System design and component relationships
- **[RETE Algorithm](../specs/rete-algorithm-implementation.md)** - Implementation details
- **[gRPC API](../specs/grpc-api.md)** - Complete protocol specifications
- **[Built-in Calculators](../specs/built-in-calculators.md)** - Calculator reference
- **[Rule Specification](../specs/rule-specification.md)** - Rule design patterns

## Architecture Overview

The Bingo Rules Engine implements the RETE algorithm for efficient rule processing with:

- **High-performance gRPC streaming API**
- **Plugin-based calculator system with built-in business calculators**
- **Shared type system eliminating circular dependencies**
- **Web interface for management and monitoring**
- **Stateless architecture with session-based rule compilation**
- **Memory-optimized data structures**
- **Advanced caching and indexing**
- **Real-time fact processing**

For technical details, see the [Specifications](../SPECS.md).
