# Bingo Rules Engine gRPC API Documentation

## Overview

The Bingo Rules Engine provides a high-performance gRPC streaming API for real-time rule processing using the RETE algorithm. This API offers superior performance for high-throughput rule evaluation scenarios with efficient memory usage and real-time processing capabilities.

## Architecture

### Core Components

1. **Rules Engine Service** (`RulesEngineServiceImpl`)
   - Stateless gRPC service implementation
   - Thread-safe processing with `Arc<AppState>`
   - Streaming support for real-time fact processing

2. **Protocol Buffers Schema** (`proto/rules_engine.proto`)
   - Defines all message types and service interfaces
   - Supports complex nested rule structures
   - Optimized for efficient serialization

3. **Type Conversions** (`src/grpc/conversions.rs`)
   - Bidirectional conversion between protobuf and core types
   - Handles complex value type mappings
   - Provides error handling for malformed data

## API Reference

### Service Interface

```protobuf
service RulesEngineService {
    // Phase 1: Compile and validate rules
    rpc CompileRules(CompileRulesRequest) returns (CompileRulesResponse);
    
    // Phase 2: Stream facts through compiled rules
    rpc ProcessFactsStream(stream ProcessFactsStreamRequest) returns (stream RuleExecutionResult);
    
    // Alternative: Single-call processing with validation
    rpc ProcessWithRulesStream(ProcessWithRulesRequest) returns (stream ProcessingResponse);
    
    // Batch processing for high-throughput scenarios
    rpc ProcessFactsBatch(ProcessFactsRequest) returns (stream ProcessingStatus);
    
    // Cached ruleset operations
    rpc RegisterRuleset(RegisterRulesetRequest) returns (RegisterRulesetResponse);
    rpc EvaluateRulesetStream(EvaluateRulesetRequest) returns (stream RuleExecutionResult);
    
    // Health and monitoring
    rpc HealthCheck(google.protobuf.Empty) returns (HealthResponse);
}
```

### Message Types

#### Rule Structure
```protobuf
message Rule {
    string id = 1;                    // Numeric string (e.g., "123")
    string name = 2;                  // Human-readable name
    string description = 3;           // Rule description
    repeated Condition conditions = 4; // Rule conditions (AND logic)
    repeated Action actions = 5;      // Actions to execute
    int32 priority = 6;              // Execution priority
    bool enabled = 7;                // Enable/disable flag
    repeated string tags = 8;        // Rule categorization
    int64 created_at = 9;           // Unix timestamp
    int64 updated_at = 10;          // Unix timestamp
}
```

#### Condition Types
```protobuf
message Condition {
    oneof condition_type {
        SimpleCondition simple = 1;     // Field comparison
        ComplexCondition complex = 2;   // Logical combinations
        AggregationCondition aggregation = 3; // Data aggregations
        StreamCondition stream = 4;     // Time-based conditions
    }
}

message SimpleCondition {
    string field = 1;                // Field name to evaluate
    SimpleOperator operator = 2;     // Comparison operator
    Value value = 3;                // Expected value
}
```

#### Action Types
```protobuf
message Action {
    oneof action_type {
        CreateFactAction create_fact = 1;           // Create new fact
        CallCalculatorAction call_calculator = 2;   // Invoke calculator
        FormulaAction formula = 3;                  // Execute formula
        UpdateFactAction update_fact = 4;           // Modify existing fact
        DeleteFactAction delete_fact = 5;           // Remove fact
    }
}
```

#### Value Types
```protobuf
message Value {
    oneof value {
        string string_value = 1;    // String data
        double number_value = 2;    // Floating-point numbers
        bool bool_value = 3;        // Boolean values
        int64 int_value = 4;        // Integer values
    }
}
```

#### Fact Structure
```protobuf
message Fact {
    string id = 1;                           // Unique identifier
    map<string, Value> data = 2;            // Key-value data
    int64 created_at = 3;                   // Unix timestamp
}
```

## Server Setup

### Prerequisites

- Rust 1.88+ with 2024 edition support
- Protocol Buffers compiler (`protoc`)
- gRPC dependencies (tonic, prost)

### Building the Server

```bash
# Clone repository
git clone <repository-url>
cd bingo

# Build the server
cargo build --release --bin bingo

# Run the server
./target/release/bingo
```

### Configuration

The server supports environment-based configuration:

```bash
# Server binding
export GRPC_HOST="0.0.0.0"
export GRPC_PORT="50051"

# Logging
export RUST_LOG="info"

# Performance tuning
export TOKIO_WORKER_THREADS="4"
```

### Docker Deployment

```dockerfile
FROM rust:1.88 as builder
WORKDIR /app
COPY . .
RUN cargo build --release --bin bingo

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/bingo /usr/local/bin/bingo
EXPOSE 50051
CMD ["bingo"]
```

## Client Setup

### Rust Client

#### Dependencies
```toml
[dependencies]
tonic = "0.13"
prost = "0.13"
tokio = { version = "1.46", features = ["full"] }
tokio-stream = "0.1"
```

#### Basic Client Example
```rust
use tonic::transport::Channel;
use tokio_stream::StreamExt;

// Generated code from proto file
pub mod rules_engine {
    tonic::include_proto!("rules_engine.v1");
}

use rules_engine::{
    rules_engine_service_client::RulesEngineServiceClient,
    CompileRulesRequest, Rule, Condition, SimpleCondition,
    SimpleOperator, Value, Action, CallCalculatorAction,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to server
    let channel = Channel::from_static("http://127.0.0.1:50051").connect().await?;
    let mut client = RulesEngineServiceClient::new(channel);

    // Create a simple rule
    let rule = Rule {
        id: "1".to_string(),
        name: "Overtime Detection".to_string(),
        description: "Detect when hours exceed 8".to_string(),
        conditions: vec![Condition {
            condition_type: Some(rules_engine::condition::ConditionType::Simple(
                SimpleCondition {
                    field: "hours".to_string(),
                    operator: SimpleOperator::GreaterThan as i32,
                    value: Some(Value {
                        value: Some(rules_engine::value::Value::NumberValue(8.0)),
                    }),
                }
            )),
        }],
        actions: vec![Action {
            action_type: Some(rules_engine::action::ActionType::CallCalculator(
                CallCalculatorAction {
                    calculator_name: "overtime_calculator".to_string(),
                    input_mapping: std::collections::HashMap::from([
                        ("hours".to_string(), "hours".to_string()),
                    ]),
                    output_field: "overtime_pay".to_string(),
                }
            )),
        }],
        priority: 100,
        enabled: true,
        tags: vec!["payroll".to_string()],
        created_at: chrono::Utc::now().timestamp(),
        updated_at: chrono::Utc::now().timestamp(),
    };

    // Compile rules
    let compile_request = tonic::Request::new(CompileRulesRequest {
        rules: vec![rule],
        session_id: "client_session_001".to_string(),
        options: None,
    });

    let compile_response = client.compile_rules(compile_request).await?;
    let session_id = compile_response.into_inner().session_id;
    
    println!("Rules compiled successfully! Session ID: {}", session_id);

    Ok(())
}
```

#### Streaming Facts Example
```rust
use futures_util::sink::SinkExt;
use tokio_stream::wrappers::UnboundedReceiverStream;

async fn stream_facts_example(
    mut client: RulesEngineServiceClient<Channel>,
    session_id: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
    let request_stream = UnboundedReceiverStream::new(rx);

    // Start streaming
    let mut response_stream = client
        .process_facts_stream(tonic::Request::new(request_stream))
        .await?
        .into_inner();

    // Send session ID
    tx.send(ProcessFactsStreamRequest {
        request: Some(process_facts_stream_request::Request::SessionId(session_id)),
    })?;

    // Send facts
    let fact = Fact {
        id: "fact_001".to_string(),
        data: std::collections::HashMap::from([
            ("hours".to_string(), Value {
                value: Some(rules_engine::value::Value::NumberValue(10.5)),
            }),
            ("employee_id".to_string(), Value {
                value: Some(rules_engine::value::Value::StringValue("EMP001".to_string())),
            }),
        ]),
        created_at: chrono::Utc::now().timestamp(),
    };

    tx.send(ProcessFactsStreamRequest {
        request: Some(process_facts_stream_request::Request::FactBatch(fact)),
    })?;

    // Process results
    while let Some(result) = response_stream.next().await {
        match result {
            Ok(execution_result) => {
                println!("Rule fired: {} for fact: {}", 
                    execution_result.rule_name,
                    execution_result.matched_fact.unwrap().id
                );
            }
            Err(e) => eprintln!("Stream error: {}", e),
        }
    }

    Ok(())
}
```

### Python Client

#### Dependencies
```bash
pip install grpcio grpcio-tools
```

#### Generate Python Code
```bash
python -m grpc_tools.protoc \
    --proto_path=proto \
    --python_out=. \
    --grpc_python_out=. \
    proto/rules_engine.proto
```

#### Python Client Example
```python
import grpc
import rules_engine_pb2
import rules_engine_pb2_grpc
from datetime import datetime
import time

def create_client():
    channel = grpc.insecure_channel('localhost:50051')
    return rules_engine_pb2_grpc.RulesEngineServiceStub(channel)

def compile_rules_example():
    client = create_client()
    
    # Create a rule
    rule = rules_engine_pb2.Rule(
        id="1",
        name="Temperature Alert",
        description="Alert when temperature exceeds threshold",
        conditions=[
            rules_engine_pb2.Condition(
                simple=rules_engine_pb2.SimpleCondition(
                    field="temperature",
                    operator=rules_engine_pb2.SimpleOperator.GREATER_THAN,
                    value=rules_engine_pb2.Value(number_value=25.0)
                )
            )
        ],
        actions=[
            rules_engine_pb2.Action(
                create_fact=rules_engine_pb2.CreateFactAction(
                    fields={
                        "alert_type": rules_engine_pb2.Value(string_value="temperature_high"),
                        "severity": rules_engine_pb2.Value(string_value="warning")
                    }
                )
            )
        ],
        priority=200,
        enabled=True,
        tags=["monitoring", "alerts"],
        created_at=int(time.time()),
        updated_at=int(time.time())
    )
    
    # Compile rules
    request = rules_engine_pb2.CompileRulesRequest(
        rules=[rule],
        session_id="python_client_session",
    )
    
    response = client.CompileRules(request)
    print(f"Compilation successful: {response.success}")
    print(f"Session ID: {response.session_id}")
    print(f"Rules compiled: {response.rules_compiled}")
    
    return response.session_id

def stream_facts_example(session_id):
    client = create_client()
    
    def generate_requests():
        # Send session ID first
        yield rules_engine_pb2.ProcessFactsStreamRequest(
            session_id=session_id
        )
        
        # Send facts
        for i in range(5):
            fact = rules_engine_pb2.Fact(
                id=f"sensor_reading_{i}",
                data={
                    "temperature": rules_engine_pb2.Value(number_value=20.0 + i * 2),
                    "sensor_id": rules_engine_pb2.Value(string_value=f"TEMP_{i:03d}"),
                    "location": rules_engine_pb2.Value(string_value="warehouse_a")
                },
                created_at=int(time.time())
            )
            
            yield rules_engine_pb2.ProcessFactsStreamRequest(fact_batch=fact)
            time.sleep(0.1)  # Small delay between facts
    
    # Process streaming response
    for response in client.ProcessFactsStream(generate_requests()):
        print(f"Rule executed: {response.rule_name}")
        print(f"Matched fact: {response.matched_fact.id}")
        print(f"Actions executed: {len(response.action_results)}")
        print("---")

if __name__ == "__main__":
    try:
        session_id = compile_rules_example()
        stream_facts_example(session_id)
    except grpc.RpcError as e:
        print(f"gRPC error: {e.code()} - {e.details()}")
```

### Node.js Client

#### Dependencies
```bash
npm install @grpc/grpc-js @grpc/proto-loader
```

#### JavaScript Client Example
```javascript
const grpc = require('@grpc/grpc-js');
const protoLoader = require('@grpc/proto-loader');
const path = require('path');

// Load proto file
const PROTO_PATH = path.join(__dirname, 'proto/rules_engine.proto');
const packageDefinition = protoLoader.loadSync(PROTO_PATH, {
    keepCase: true,
    longs: String,
    enums: String,
    defaults: true,
    oneofs: true
});

const rulesEngine = grpc.loadPackageDefinition(packageDefinition).rules_engine.v1;

class RulesEngineClient {
    constructor(address = 'localhost:50051') {
        this.client = new rulesEngine.RulesEngineService(
            address,
            grpc.credentials.createInsecure()
        );
    }

    async compileRules() {
        const rule = {
            id: "1",
            name: "Order Value Check",
            description: "Check if order value exceeds limit",
            conditions: [{
                simple: {
                    field: "order_value",
                    operator: "GREATER_THAN",
                    value: { number_value: 1000.0 }
                }
            }],
            actions: [{
                create_fact: {
                    fields: {
                        "requires_approval": { bool_value: true },
                        "approval_level": { string_value: "manager" }
                    }
                }
            }],
            priority: 150,
            enabled: true,
            tags: ["orders", "approval"],
            created_at: Math.floor(Date.now() / 1000),
            updated_at: Math.floor(Date.now() / 1000)
        };

        return new Promise((resolve, reject) => {
            this.client.CompileRules({
                rules: [rule],
                session_id: "nodejs_session",
                options: null
            }, (error, response) => {
                if (error) {
                    reject(error);
                } else {
                    resolve(response);
                }
            });
        });
    }

    async streamFacts(sessionId) {
        const stream = this.client.ProcessFactsStream();
        
        // Send session ID
        stream.write({
            session_id: sessionId
        });

        // Send facts
        const facts = [
            {
                id: "order_001",
                data: {
                    "order_value": { number_value: 1500.0 },
                    "customer_id": { string_value: "CUST_001" },
                    "product_count": { int_value: 3 }
                },
                created_at: Math.floor(Date.now() / 1000)
            },
            {
                id: "order_002", 
                data: {
                    "order_value": { number_value: 750.0 },
                    "customer_id": { string_value: "CUST_002" },
                    "product_count": { int_value: 1 }
                },
                created_at: Math.floor(Date.now() / 1000)
            }
        ];

        facts.forEach(fact => {
            stream.write({ fact_batch: fact });
        });

        stream.end();

        // Process responses
        stream.on('data', (response) => {
            console.log(`Rule fired: ${response.rule_name}`);
            console.log(`Matched fact: ${response.matched_fact.id}`);
            console.log(`Execution time: ${response.execution_time_ns}ns`);
        });

        stream.on('error', (error) => {
            console.error('Stream error:', error);
        });

        stream.on('end', () => {
            console.log('Stream ended');
        });
    }
}

// Usage example
async function main() {
    const client = new RulesEngineClient();
    
    try {
        const compileResponse = await client.compileRules();
        console.log('Rules compiled successfully!');
        console.log(`Session ID: ${compileResponse.session_id}`);
        
        await client.streamFacts(compileResponse.session_id);
    } catch (error) {
        console.error('Error:', error);
    }
}

main();
```

## Advanced Usage

### Complex Rule Examples

#### Multi-Condition Rules
```rust
// Rule with AND logic across multiple conditions
let complex_rule = Rule {
    id: "2".to_string(),
    name: "VIP Customer Discount".to_string(),
    description: "Apply discount for VIP customers with large orders".to_string(),
    conditions: vec![
        Condition {
            condition_type: Some(condition::ConditionType::Simple(SimpleCondition {
                field: "customer_tier".to_string(),
                operator: SimpleOperator::Equal as i32,
                value: Some(Value {
                    value: Some(value::Value::StringValue("VIP".to_string())),
                }),
            })),
        },
        Condition {
            condition_type: Some(condition::ConditionType::Simple(SimpleCondition {
                field: "order_total".to_string(),
                operator: SimpleOperator::GreaterThan as i32,
                value: Some(Value {
                    value: Some(value::Value::NumberValue(500.0)),
                }),
            })),
        },
    ],
    actions: vec![Action {
        action_type: Some(action::ActionType::Formula(FormulaAction {
            formula: "order_total * 0.9".to_string(), // 10% discount
            output_field: "discounted_total".to_string(),
        })),
    }],
    priority: 300,
    enabled: true,
    tags: vec!["discounts".to_string(), "vip".to_string()],
    created_at: chrono::Utc::now().timestamp(),
    updated_at: chrono::Utc::now().timestamp(),
};
```

### Error Handling

#### Client-Side Error Handling
```rust
match client.compile_rules(request).await {
    Ok(response) => {
        let response = response.into_inner();
        if response.success {
            println!("Compilation successful: {} rules", response.rules_compiled);
        } else {
            eprintln!("Compilation failed: {}", response.error_message);
        }
    }
    Err(status) => {
        match status.code() {
            tonic::Code::InvalidArgument => {
                eprintln!("Invalid rule format: {}", status.message());
            }
            tonic::Code::Unavailable => {
                eprintln!("Server unavailable: {}", status.message());
            }
            tonic::Code::DeadlineExceeded => {
                eprintln!("Request timeout: {}", status.message());
            }
            _ => {
                eprintln!("Unexpected error: {}", status);
            }
        }
    }
}
```

#### Stream Error Recovery
```rust
async fn robust_stream_processing(
    mut client: RulesEngineServiceClient<Channel>,
    facts: Vec<Fact>,
) -> Result<(), Box<dyn std::error::Error>> {
    let max_retries = 3;
    let mut retry_count = 0;

    while retry_count < max_retries {
        match process_fact_stream(&mut client, &facts).await {
            Ok(_) => return Ok(()),
            Err(e) if e.code() == tonic::Code::Unavailable => {
                retry_count += 1;
                if retry_count < max_retries {
                    println!("Server unavailable, retrying in 5 seconds...");
                    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                    
                    // Reconnect
                    let channel = Channel::from_static("http://127.0.0.1:50051")
                        .connect()
                        .await?;
                    client = RulesEngineServiceClient::new(channel);
                }
            }
            Err(e) => return Err(e.into()),
        }
    }

    Err("Max retries exceeded".into())
}
```

## Performance Optimization

### Client-Side Optimizations

#### Connection Pooling
```rust
use tonic::transport::{Channel, Endpoint};

async fn create_load_balanced_client() -> Result<RulesEngineServiceClient<Channel>, Box<dyn std::error::Error>> {
    let endpoints = vec![
        "http://127.0.0.1:50051",
        "http://127.0.0.1:50052", 
        "http://127.0.0.1:50053",
    ];

    let channel = Channel::balance_list(
        endpoints.into_iter().map(|addr| Endpoint::from_static(addr))
    );

    Ok(RulesEngineServiceClient::new(channel))
}
```

#### Batch Processing
```rust
async fn batch_fact_processing(
    mut client: RulesEngineServiceClient<Channel>,
    facts: Vec<Fact>,
    batch_size: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    for batch in facts.chunks(batch_size) {
        let request = ProcessWithRulesRequest {
            rules: vec![], // Use pre-compiled rules
            facts: batch.to_vec(),
            request_id: uuid::Uuid::new_v4().to_string(),
            options: None,
            validate_rules_only: false,
        };

        let mut response_stream = client
            .process_with_rules_stream(tonic::Request::new(request))
            .await?
            .into_inner();

        while let Some(response) = response_stream.next().await {
            match response? {
                ProcessingResponse { response: Some(processing_response::Response::Completion(completion)) } => {
                    println!("Batch completed: {} facts processed", completion.total_facts_processed);
                    break;
                }
                ProcessingResponse { response: Some(processing_response::Response::StatusUpdate(status)) } => {
                    println!("Processing: {}/{} facts", status.facts_processed, batch.len());
                }
                _ => {}
            }
        }
    }

    Ok(())
}
```

### Server-Side Configuration

#### Production Settings
```bash
# Environment variables for production deployment
export RUST_LOG="warn,bingo_api=info"
export TOKIO_WORKER_THREADS="8"
export GRPC_MAX_CONCURRENT_STREAMS="1000"
export GRPC_KEEPALIVE_TIME="30s"
export GRPC_KEEPALIVE_TIMEOUT="5s"
export GRPC_HTTP2_INITIAL_STREAM_WINDOW_SIZE="1048576"
export GRPC_HTTP2_INITIAL_CONNECTION_WINDOW_SIZE="1048576"
```

## Monitoring and Observability

### Health Checks
```rust
async fn health_check_example(mut client: RulesEngineServiceClient<Channel>) {
    match client.health_check(tonic::Request::new(())).await {
        Ok(response) => {
            let health = response.into_inner();
            println!("Server status: {}", health.status);
            println!("Version: {}", health.version);
            println!("Uptime: {} seconds", health.uptime_seconds);
        }
        Err(e) => {
            eprintln!("Health check failed: {}", e);
        }
    }
}
```

### Metrics Collection
```rust
use std::time::Instant;

async fn measure_compilation_performance(
    mut client: RulesEngineServiceClient<Channel>,
    rules: Vec<Rule>,
) -> Result<(), Box<dyn std::error::Error>> {
    let start = Instant::now();
    
    let request = tonic::Request::new(CompileRulesRequest {
        rules,
        session_id: "perf_test".to_string(),
        options: None,
    });

    let response = client.compile_rules(request).await?;
    let compilation_time = start.elapsed();
    
    let result = response.into_inner();
    
    println!("Performance Metrics:");
    println!("  Client-side time: {:?}", compilation_time);
    println!("  Server-side time: {}ms", result.compilation_time_ms);
    println!("  Rules compiled: {}", result.rules_compiled);
    println!("  Network nodes: {}", result.network_nodes_created);
    println!("  Throughput: {:.2} rules/ms", 
        result.rules_compiled as f64 / result.compilation_time_ms as f64);

    Ok(())
}
```

## Security Considerations

### TLS Configuration
```rust
use tonic::transport::{Certificate, ClientTlsConfig, Channel};

async fn create_secure_client() -> Result<RulesEngineServiceClient<Channel>, Box<dyn std::error::Error>> {
    let ca_cert = tokio::fs::read("ca-cert.pem").await?;
    let ca_cert = Certificate::from_pem(ca_cert);

    let tls_config = ClientTlsConfig::new()
        .ca_certificate(ca_cert)
        .domain_name("rules-engine.example.com");

    let channel = Channel::from_static("https://rules-engine.example.com:443")
        .tls_config(tls_config)?
        .connect()
        .await?;

    Ok(RulesEngineServiceClient::new(channel))
}
```

### Authentication
```rust
use tonic::{Request, Status};
use tonic::metadata::MetadataValue;

fn add_auth_token<T>(mut request: Request<T>, token: &str) -> Request<T> {
    let token_value = MetadataValue::from_str(&format!("Bearer {}", token))
        .expect("Invalid token format");
    
    request.metadata_mut().insert("authorization", token_value);
    request
}

// Usage
let request = add_auth_token(
    tonic::Request::new(CompileRulesRequest { /* ... */ }),
    "your-auth-token"
);
```

## Two-Phase Processing

### Overview

The gRPC API supports efficient two-phase processing:

1. **Phase 1: Rule Compilation** - Validate and compile rules into RETE network
2. **Phase 2: Fact Streaming** - Stream facts through pre-compiled rules

### Benefits

- **Memory Efficiency**: O(1) memory usage vs O(n) accumulation
- **Rule Validation**: Catch errors before processing facts
- **Session Reuse**: Compiled rules can process multiple fact streams
- **Early Exit**: Client can stop processing at any point

### Usage Patterns

#### Pattern 1: Two-Phase (Recommended for Large Datasets)
```rust
// 1. Compile rules once
let session_id = client.compile_rules(rules).await?;

// 2. Stream many fact batches through compiled rules
for fact_batch in large_fact_dataset.chunks(1000) {
    client.stream_facts_to_session(&session_id, fact_batch, |result| {
        // Process result immediately
        handle_result(result)
    }).await?;
}
```

#### Pattern 2: Single-Call (Convenient for Smaller Datasets)
```rust
// Rules compiled first, then facts streamed - all in one call
client.process_with_rules_streaming(facts, rules, |response| {
    match response.response {
        RulesCompiled(compilation) => println!("Rules ready!"),
        ExecutionResult(result) => handle_result(result),
        Completion(_) => println!("Done!"),
    }
}).await?;
```

## Troubleshooting

### Common Issues

#### Connection Refused
```bash
# Check if server is running
netstat -tlnp | grep :50051

# Test connectivity
grpc_cli call localhost:50051 rules_engine.v1.RulesEngineService.HealthCheck ""
```

#### Protocol Buffer Mismatches
```bash
# Ensure proto files are synchronized
diff client/proto/rules_engine.proto server/proto/rules_engine.proto

# Regenerate client code
protoc --rust_out=. --grpc_out=. --plugin=protoc-gen-grpc=`which grpc_rust_plugin` proto/rules_engine.proto
```

#### Performance Issues
- Monitor server logs for compilation errors
- Check memory usage during rule compilation
- Verify network latency between client and server
- Use connection pooling for high-throughput scenarios

### Debug Logging
```rust
// Enable debug logging
std::env::set_var("RUST_LOG", "debug");
env_logger::init();

// Add request tracing
let request = tonic::Request::new(your_request);
request.set_timeout(Duration::from_secs(30));
```

## Best Practices

### 1. Rule Design
- Use numeric string IDs for rules (e.g., "123")
- Keep rule conditions simple and focused
- Use appropriate priority values for execution order
- Tag rules for categorization and filtering

### 2. Fact Processing
- Process facts in batches for better performance
- Use streaming for large datasets
- Implement proper error handling and retry logic
- Monitor memory usage during processing

### 3. Session Management
- Compile rules once, reuse sessions for multiple fact streams
- Set appropriate session timeouts
- Clean up expired sessions periodically
- Use unique session IDs for isolation

### 4. Error Handling
- Implement comprehensive error handling for all RPC calls
- Use proper retry logic for transient failures
- Log errors with sufficient context for debugging
- Handle stream interruptions gracefully

### 5. Performance
- Use connection pooling for high-throughput scenarios
- Optimize batch sizes based on memory constraints
- Monitor latency and throughput metrics
- Scale horizontally when needed

This comprehensive documentation provides everything needed to understand, deploy, and use the Bingo Rules Engine gRPC API effectively in production environments.