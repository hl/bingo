# Bingo RETE Rules Engine

A high-performance RETE rules engine built in Rust, designed to process large-scale datasets with complex business rules. Capable of processing 3 million facts against 2,000 rules efficiently.

## Features

- **High Performance**: Built for processing 3M facts with 2,000 rules efficiently
- **Aggregation Support**: First-class incremental aggregations integrated with RETE network
- **Multi-Phase Processing**: Support for complex analytical workflows
- **Memory Efficient**: Arena allocation and optimised data structures
- **Comprehensive Observability**: Full tracing, metrics, and logging with `tracing`
- **JSON API**: RESTful HTTP interface for rule evaluation
- **Modular Architecture**: Clean separation of concerns across crates

## Quick Start

### Run the "explain" command (prints Hello World)
```bash
cargo run --bin bingo explain
```

### Start the web server
```bash
cargo run --bin bingo
```

The server will start on `http://127.0.0.1:3000`

### Test the API
```bash
# Health check
curl http://localhost:3000/health

# Evaluate facts (example)
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
            "department": "Engineering"
          }
        }
      }
    ]
  }'
```

## Architecture

- **bingo-core**: High-level engine API and business logic
- **bingo-rete**: RETE algorithm implementation
- **bingo-web**: HTTP API server with Axum

## Performance Targets

- **Throughput**: 3M facts processed in <2 seconds
- **Memory Usage**: <300MB RSS for target dataset
- **Latency**: P95 < 500ms for rule evaluation
- **Rule Capacity**: 2,000 rules without performance degradation

## Development

### Run tests
```bash
cargo test
```

### Run benchmarks
```bash
cargo bench
```

### Check code
```bash
cargo check
cargo clippy
```

## Documentation

See the [specifications](SPECS.md) for detailed technical documentation.

## License

TBD