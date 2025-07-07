# Bingo Rules Engine Documentation

This directory contains comprehensive documentation for the Bingo RETE Rules Engine.

## Core Documentation

### API & Integration
- **[Client Setup Guide](client-setup.md)** - Step-by-step client setup for multiple languages
- **[gRPC Deployment Guide](grpc-deployment-guide.md)** - Production deployment instructions

### Performance & Optimization
- **[Performance Tests](performance-tests.md)** - Benchmark results and testing methodologies
- **[Cache Lifecycle](cache-lifecycle.md)** - Caching system documentation

### Domain Applications
- **[Payroll Engine](payroll-engine.md)** - Payroll processing implementation
- **[Compliance Engine](compliance-engine.md)** - Compliance rule processing
- **[Wage Cost Estimation Engine](wage-cost-estimation-engine.md)** - Cost estimation features
- **[TRONC Engine](tronc-engine.md)** - TRONC system integration

## Quick Start

1. **For API Users**: Start with the [gRPC API Specification](../specs/grpc-api.md)
2. **For Client Development**: Follow [Client Setup Guide](client-setup.md)
3. **For Deployment**: See [gRPC Deployment Guide](grpc-deployment-guide.md)
4. **For Performance**: Review [Performance Tests](performance-tests.md)

## Architecture Overview

The Bingo Rules Engine implements the RETE algorithm for efficient rule processing with:

- **High-performance gRPC streaming API**
- **Stateless architecture with session-based rule compilation**
- **Memory-optimized data structures**
- **Advanced caching and indexing**
- **Real-time fact processing**

For technical details, see the [Specifications](../SPECS.md).
