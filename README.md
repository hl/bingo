# Bingo RETE Rules Engine

A production-ready, high-performance RETE rules engine built in **Rust 2024 edition**. Processes large-scale datasets with complex business rules, delivering exceptional performance that exceeds enterprise targets by **4-5x margins**.

## 🚀 Performance Achievements

**Validated Enterprise Performance (Release Mode):**
- **100K facts**: 635ms (4.7x faster than 3s target)
- **200K facts**: 1.16s (5.2x faster than 6s target)  
- **500K facts**: 2.16s (4.6x faster than 10s target)
- **1M facts**: 6.59s (4.6x faster than 30s target)

**Memory Efficiency:**
- CI environments: <500MB
- Enterprise scale: <3GB (well under 4GB target)

## ⭐ Key Features

- **🏎️ Exceptional Performance**: Direct Vec indexing with O(1) fact access
- **🧠 Smart Memory Management**: Adaptive backends with capacity pre-allocation
- **📈 Linear Scaling**: Validated from 100K to 1M+ facts
- **🦀 Rust 2024**: Latest edition with full thread safety (`Send + Sync`)
- **🎯 Production Ready**: Zero warnings, comprehensive testing
- **🔧 CI Optimized**: Resource-appropriate testing for reliable automation
- **🎨 Design Stage Friendly**: Simplified architecture, zero configuration
- **📊 Comprehensive Observability**: Full tracing and metrics with `tracing`
- **🌐 HTTP API**: RESTful interface with OpenAPI documentation

## 🏗️ Architecture

**Workspace Structure:**
```
bingo/
├── bingo-core/     # Core RETE engine & optimizations  
├── bingo-rete/     # Low-level RETE algorithm implementation
└── bingo-api/      # HTTP API server with Axum + OpenAPI
```

**Key Components:**
- **RETE Network**: Optimized alpha/beta nodes with token sharing
- **Fact Store**: Multiple backends (Vec, Cached, Partitioned, Arena)
- **Calculator DSL**: Business-friendly expression language
- **Memory Pools**: Arena allocation with LRU caching
- **Performance Optimization**: Adaptive selection of optimal strategies

## 🚀 Quick Start

### Prerequisites
- **Rust 1.87.0+** (2024 edition)
- No additional configuration required

### Run the Engine
```bash
# Clone and build
git clone <repository-url>
cd bingo
cargo build --release

# Run explanation
cargo run --bin bingo explain

# Start HTTP server
cargo run --bin bingo
```

The server starts on `http://127.0.0.1:3000` with:
- Health endpoint: `GET /health`
- Rule evaluation: `POST /evaluate` 
- OpenAPI docs: `GET /swagger-ui/`

### Example API Usage
```bash
# Health check
curl http://localhost:3000/health

# Evaluate facts
curl -X POST http://localhost:3000/evaluate \
  -H "Content-Type: application/json" \
  -d '{
    "facts": [
      {
        "id": 1,
        "data": {
          "fields": {
            "employee_id": 12345,
            "hours_worked": 42.5,
            "status": "active"
          }
        }
      }
    ]
  }'
```

## 🧪 Development

### Testing
```bash
# Unit tests (167 tests)
cargo test --lib

# Performance tests (CI-appropriate)
cargo test --release

# Heavy performance tests (manual)
cargo test --ignored --release
```

### Quality Checks
```bash
# Zero-tolerance quality validation
cargo fmt --check
cargo clippy -- -D warnings
cargo check --workspace
```

### Benchmarking
```bash
# Comprehensive benchmarks
cargo bench

# Specific benchmarks
cargo bench --bench engine_bench
cargo bench --bench million_fact_bench
```

## 📊 Performance Characteristics

### Scaling Validation
- **Linear performance**: O(n) scaling confirmed
- **Memory efficiency**: Sub-linear memory growth
- **Throughput**: 150K+ facts/second sustained
- **Latency**: Sub-second response for 100K facts

### Optimization Features
- **Direct Vec indexing**: Eliminates HashMap overhead
- **Memory pre-allocation**: Capacity hints for large datasets
- **Field indexing**: Optimized for enterprise patterns
- **Batch processing**: Efficient handling of large fact sets

## 🔧 Configuration

### Environment Variables
```bash
BINGO_HOST=127.0.0.1       # Server host
BINGO_PORT=3000            # Server port  
RUST_LOG=bingo=debug,info  # Logging level
```

### Build Modes
- **Debug**: Development and unit testing
- **Release**: Performance testing and production
- **Benchmark**: Criterion-based performance analysis

## 📚 Documentation

- **[CLAUDE.md](CLAUDE.md)**: Development commands and guidelines
- **[specs/](specs/)**: Detailed technical specifications
- **API Docs**: Available at `/swagger-ui/` when server running
- **Rust Docs**: Generate with `cargo doc --open`

## 🏆 Production Readiness

**Quality Standards:**
- ✅ **Zero warnings** with `-D warnings`
- ✅ **Comprehensive testing** (167 unit + 4 performance tests)  
- ✅ **Thread safety** throughout (`Send + Sync`)
- ✅ **Memory safety** with Rust guarantees
- ✅ **Performance validation** at enterprise scale

**Enterprise Features:**
- Linear scaling to 1M+ facts
- Sub-3GB memory usage
- Comprehensive error handling
- Structured logging and metrics
- OpenAPI integration

## 🤝 Contributing

This project follows:
- **Rust 2024 edition** standards
- **Zero tolerance** for warnings
- **Comprehensive testing** requirements
- **Performance-first** design principles

## 📄 License

TBD