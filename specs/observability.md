# Observability Specification

## Overview

Comprehensive observability through structured logging, distributed tracing, and metrics collection using the `tracing` ecosystem. **Advanced debugging and profiling capabilities have been implemented but are currently disabled due to compilation issues.**

## Tracing Architecture

### Span Hierarchy
```
bingo_main
├── serve_command
│   ├── create_app
│   └── request_handling
│       ├── evaluate_handler
│       │   ├── engine_processing
│       │   │   ├── rete_network_processing
│       │   │   ├── fact_insertion
│       │   │   └── rule_evaluation
│       │   └── response_serialisation
│       └── health_handler
└── explain_command
```

### Instrumentation Strategy
```rust
#[instrument(
    skip(self, facts),
    fields(
        fact_count = facts.len(),
        engine_id = %self.id
    )
)]
pub fn process_facts(&mut self, facts: Vec<Fact>) -> Result<Vec<Fact>> {
    // Implementation with automatic span creation
}
```

## Structured Logging

### Log Levels
- **ERROR**: System failures, unrecoverable errors
- **WARN**: Performance degradation, recoverable errors  
- **INFO**: Request lifecycle, major operations
- **DEBUG**: Detailed execution flow, intermediate results
- **TRACE**: Fine-grained debugging, internal state

### Log Format
```json
{
  "timestamp": "2024-01-15T10:30:45.123Z",
  "level": "INFO",
  "fields": {
    "message": "Facts processed successfully",
    "fact_count": 15000,
    "result_count": 342,
    "execution_time_ms": 45,
    "engine_id": "engine-001"
  },
  "span": {
    "name": "process_facts",
    "trace_id": "a1b2c3d4e5f6",
    "span_id": "1a2b3c4d"
  }
}
```

## Metrics Collection

### Performance Metrics
```rust
pub struct EngineMetrics {
    // Timing metrics
    pub evaluation_duration: Histogram,
    pub rule_compilation_duration: Histogram,
    
    // Throughput metrics  
    pub facts_processed_total: Counter,
    pub rules_fired_total: Counter,
    
    // Resource metrics
    pub memory_usage_bytes: Gauge,
    pub network_node_count: Gauge,
    
    // Error metrics
    pub evaluation_errors_total: Counter,
    pub rule_compilation_errors_total: Counter,
}
```

### Business Metrics
- **Rules Fired**: Count of successful rule executions
- **Fact Processing Rate**: Facts processed per second
- **Rule Coverage**: Percentage of rules that fired
- **Action Execution**: Count of actions triggered

### System Metrics
- **Memory Usage**: RSS, heap allocation, arena usage
- **CPU Utilisation**: Processing time, idle time
- **Network**: Request rate, response time, error rate
- **Threading**: Task count, queue depth, contention

## Distributed Tracing

### OpenTelemetry Integration
```rust
use tracing_opentelemetry::OpenTelemetryLayer;

let tracer = opentelemetry::global::tracer("bingo");
let telemetry_layer = tracing_opentelemetry::layer().with_tracer(tracer);

tracing_subscriber::registry()
    .with(telemetry_layer)
    .with(tracing_subscriber::fmt::layer())
    .init();
```

### Trace Context Propagation
- **HTTP Headers**: B3 or W3C trace context
- **Cross-Service**: Maintain trace across components  
- **Sampling**: Configurable sampling rates
- **Correlation**: Link related operations

## Error Tracking

### Error Categories
```rust
#[derive(thiserror::Error, Debug)]
pub enum BingoError {
    #[error("Rule compilation failed: {rule_id}")]
    RuleCompilation { rule_id: RuleId },
    
    #[error("Fact processing error: {message}")]
    FactProcessing { message: String },
    
    #[error("Memory allocation failed")]
    MemoryAllocation,
    
    #[error("Network error: {source}")]
    Network { source: Box<dyn std::error::Error> },
}
```

### Error Context
- **Error Spans**: Automatic error capture in tracing
- **Stack Traces**: Full error context with anyhow
- **Error Metrics**: Count and categorise errors
- **Alert Integration**: Trigger alerts on error thresholds

## Performance Monitoring

### Real-Time Monitoring
```rust
#[instrument(fields(memory_mb = Empty))]
fn monitor_memory_usage() {
    let memory_mb = get_memory_usage() / 1024 / 1024;
    Span::current().record("memory_mb", memory_mb);
    
    if memory_mb > 150 {
        warn!(memory_mb, "High memory usage detected");
    }
}
```

### Performance Alerts
- **Latency**: P95 > 200ms
- **Memory**: RSS > 200MB  
- **Throughput**: <500K facts/second
- **Error Rate**: >1% failure rate

### Dashboards
- **System Overview**: Key metrics and health status
- **Performance**: Latency, throughput, resource usage
- **Errors**: Error rates, types, and trends
- **Business**: Rule firing rates, fact processing

## Configuration

### Environment Variables
```bash
RUST_LOG=bingo=debug,tower_http=info
OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317
OTEL_RESOURCE_ATTRIBUTES=service.name=bingo,service.version=0.1.0
BINGO_METRICS_ENABLED=true
BINGO_TRACING_SAMPLE_RATE=0.1
```

### Runtime Configuration
```rust
pub struct ObservabilityConfig {
    pub log_level: LevelFilter,
    pub metrics_enabled: bool,
    pub tracing_endpoint: Option<String>,
    pub sample_rate: f64,
}
```

## Advanced Debugging (IMPLEMENTED - TEMPORARILY DISABLED)

### Debug Manager
Comprehensive debugging and profiling system has been implemented with the following capabilities:

#### Execution Tracing
- **TraceId-based tracking**: Each rule execution gets a unique trace identifier
- **Node-level execution tracking**: Detailed execution path through RETE network nodes
- **Performance profiling**: Execution time, memory usage, and bottleneck identification
- **Dependency analysis**: Track rule dependencies and execution flow

#### Breakpoint System
- **Conditional breakpoints**: Stop execution based on fact values or rule conditions
- **Step-by-step debugging**: Execute rules one step at a time
- **Variable inspection**: Examine fact values and token states during execution

#### Performance Profiling
- **Rule performance profiles**: Track execution times and memory usage per rule
- **Bottleneck identification**: Automatically identify performance hotspots
- **Optimization recommendations**: Generate suggestions for improving rule performance
- **Memory usage tracking**: Monitor memory allocation during rule execution

#### Debug Session Management
```rust
pub struct DebugManager {
    sessions: HashMap<DebugSessionId, DebugSession>,
    traces: HashMap<TraceId, ExecutionTrace>,
    profiles: HashMap<RuleId, RulePerformanceProfile>,
    config: DebugConfig,
    stats: DebugStats,
}
```

### Current Status
- ✅ **Fully Implemented**: All debugging infrastructure completed
- ⚠️ **Temporarily Disabled**: Due to compilation issues with Token creation methods
- 🔧 **Requires Fix**: Resolve Token::from_fact vs Token::new method signatures
- 📝 **Test Coverage**: Comprehensive test suite ready (debugging_test.rs.disabled)