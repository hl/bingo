# Bingo Rules Engine gRPC Client Setup Guide

## Quick Start

This guide provides step-by-step instructions for setting up clients to connect to the Bingo Rules Engine gRPC server in multiple programming languages.

## Prerequisites

- Access to a running Bingo Rules Engine gRPC server
- Protocol Buffers compiler (`protoc`) installed
- Language-specific gRPC libraries

## Server Information

- **Default Port**: 50051
- **Protocol**: gRPC over HTTP/2
- **Service Name**: `rules_engine.v1.RulesEngineService`
- **Proto File**: `proto/rules_engine.proto`

## Language-Specific Setup

### Rust Client

#### 1. Create a new Rust project
```bash
cargo new bingo-client
cd bingo-client
```

#### 2. Add dependencies to `Cargo.toml`
```toml
[package]
name = "bingo-client"
version = "0.1.0"
edition = "2021"

[dependencies]
tonic = "0.13"
prost = "0.13"
tokio = { version = "1.46", features = ["full"] }
tokio-stream = "0.1"
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1.17", features = ["v4"] }
anyhow = "1.0"

[build-dependencies]
tonic-build = "0.13"
```

#### 3. Create `build.rs`
```rust
fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .build_server(false)
        .build_client(true)
        .out_dir("src/generated")
        .compile(&["proto/rules_engine.proto"], &["proto/"])?;
    Ok(())
}
```

#### 4. Copy proto file
```bash
mkdir proto
# Copy rules_engine.proto from the server repository
cp /path/to/server/proto/rules_engine.proto proto/
```

#### 5. Create client code (`src/main.rs`)
```rust
mod generated {
    tonic::include_proto!("rules_engine.v1");
}

use generated::{
    rules_engine_service_client::RulesEngineServiceClient,
    *
};
use tonic::transport::Channel;
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to server
    let channel = Channel::from_static("http://127.0.0.1:50051")
        .connect()
        .await?;
    let mut client = RulesEngineServiceClient::new(channel);

    // Test health check
    let health_response = client.health_check(()).await?;
    println!("Server health: {:?}", health_response.into_inner());

    // Example rule compilation
    let rule = Rule {
        id: "1".to_string(),
        name: "Test Rule".to_string(),
        description: "A simple test rule".to_string(),
        conditions: vec![Condition {
            condition_type: Some(condition::ConditionType::Simple(
                SimpleCondition {
                    field: "temperature".to_string(),
                    operator: SimpleOperator::GreaterThan as i32,
                    value: Some(Value {
                        value: Some(value::Value::NumberValue(25.0)),
                    }),
                }
            )),
        }],
        actions: vec![Action {
            action_type: Some(action::ActionType::CreateFact(
                CreateFactAction {
                    fields: HashMap::from([
                        ("alert".to_string(), Value {
                            value: Some(value::Value::StringValue("high_temp".to_string())),
                        }),
                    ]),
                }
            )),
        }],
        priority: 100,
        enabled: true,
        tags: vec!["monitoring".to_string()],
        created_at: chrono::Utc::now().timestamp(),
        updated_at: chrono::Utc::now().timestamp(),
    };

    // Compile rules
    let compile_request = CompileRulesRequest {
        rules: vec![rule],
        session_id: "test_session".to_string(),
        options: None,
    };

    let compile_response = client.compile_rules(compile_request).await?;
    println!("Compilation result: {:?}", compile_response.into_inner());

    Ok(())
}
```

#### 6. Build and run
```bash
cargo build
cargo run
```

### Python Client

#### 1. Install dependencies
```bash
pip install grpcio grpcio-tools
```

#### 2. Generate Python code
```bash
# Create proto directory and copy proto file
mkdir proto
cp /path/to/server/proto/rules_engine.proto proto/

# Generate Python code
python -m grpc_tools.protoc \
    --proto_path=proto \
    --python_out=. \
    --grpc_python_out=. \
    proto/rules_engine.proto
```

#### 3. Create client script (`client.py`)
```python
import grpc
import rules_engine_pb2
import rules_engine_pb2_grpc
import time

def main():
    # Connect to server
    channel = grpc.insecure_channel('localhost:50051')
    client = rules_engine_pb2_grpc.RulesEngineServiceStub(channel)
    
    # Test health check
    try:
        health_response = client.HealthCheck(rules_engine_pb2.google_dot_protobuf_dot_empty__pb2.Empty())
        print(f"Server health: {health_response.status}")
    except grpc.RpcError as e:
        print(f"Health check failed: {e}")
        return
    
    # Create a test rule
    rule = rules_engine_pb2.Rule(
        id="1",
        name="Temperature Alert",
        description="Alert when temperature is too high",
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
                        "alert": rules_engine_pb2.Value(string_value="high_temp")
                    }
                )
            )
        ],
        priority=100,
        enabled=True,
        tags=["monitoring"],
        created_at=int(time.time()),
        updated_at=int(time.time())
    )
    
    # Compile rules
    compile_request = rules_engine_pb2.CompileRulesRequest(
        rules=[rule],
        session_id="python_test_session"
    )
    
    try:
        compile_response = client.CompileRules(compile_request)
        print(f"Compilation successful: {compile_response.success}")
        print(f"Session ID: {compile_response.session_id}")
        print(f"Rules compiled: {compile_response.rules_compiled}")
    except grpc.RpcError as e:
        print(f"Compilation failed: {e}")

if __name__ == "__main__":
    main()
```

#### 4. Run the client
```bash
python client.py
```

### Node.js Client

#### 1. Initialize project and install dependencies
```bash
npm init -y
npm install @grpc/grpc-js @grpc/proto-loader
```

#### 2. Copy proto file
```bash
mkdir proto
cp /path/to/server/proto/rules_engine.proto proto/
```

#### 3. Create client script (`client.js`)
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

const rulesEngineProto = grpc.loadPackageDefinition(packageDefinition);

async function main() {
    // Create client
    const client = new rulesEngineProto.rules_engine.v1.RulesEngineService(
        'localhost:50051',
        grpc.credentials.createInsecure()
    );

    // Test health check
    try {
        const healthResponse = await new Promise((resolve, reject) => {
            client.HealthCheck({}, (error, response) => {
                if (error) reject(error);
                else resolve(response);
            });
        });
        console.log(`Server health: ${healthResponse.status}`);
    } catch (error) {
        console.error(`Health check failed: ${error}`);
        return;
    }

    // Create test rule
    const rule = {
        id: "1",
        name: "Temperature Alert",
        description: "Alert when temperature is too high",
        conditions: [{
            simple: {
                field: "temperature",
                operator: "GREATER_THAN",
                value: { number_value: 25.0 }
            }
        }],
        actions: [{
            create_fact: {
                fields: {
                    "alert": { string_value: "high_temp" }
                }
            }
        }],
        priority: 100,
        enabled: true,
        tags: ["monitoring"],
        created_at: Math.floor(Date.now() / 1000),
        updated_at: Math.floor(Date.now() / 1000)
    };

    // Compile rules
    try {
        const compileResponse = await new Promise((resolve, reject) => {
            client.CompileRules({
                rules: [rule],
                session_id: "nodejs_test_session"
            }, (error, response) => {
                if (error) reject(error);
                else resolve(response);
            });
        });

        console.log(`Compilation successful: ${compileResponse.success}`);
        console.log(`Session ID: ${compileResponse.session_id}`);
        console.log(`Rules compiled: ${compileResponse.rules_compiled}`);
    } catch (error) {
        console.error(`Compilation failed: ${error}`);
    }
}

main().catch(console.error);
```

#### 4. Run the client
```bash
node client.js
```

### Go Client

#### 1. Initialize Go module
```bash
mkdir bingo-go-client
cd bingo-go-client
go mod init bingo-client
```

#### 2. Install dependencies
```bash
go get google.golang.org/grpc
go get google.golang.org/protobuf/cmd/protoc-gen-go
go get google.golang.org/grpc/cmd/protoc-gen-go-grpc
```

#### 3. Generate Go code
```bash
mkdir proto
cp /path/to/server/proto/rules_engine.proto proto/

# Generate Go code
protoc --go_out=. --go_opt=paths=source_relative \
    --go-grpc_out=. --go-grpc_opt=paths=source_relative \
    proto/rules_engine.proto
```

#### 4. Create client code (`main.go`)
```go
package main

import (
    "context"
    "log"
    "time"

    "google.golang.org/grpc"
    "google.golang.org/grpc/credentials/insecure"
    pb "bingo-client/proto"
)

func main() {
    // Connect to server
    conn, err := grpc.Dial("localhost:50051", grpc.WithTransportCredentials(insecure.NewCredentials()))
    if err != nil {
        log.Fatalf("Failed to connect: %v", err)
    }
    defer conn.Close()

    client := pb.NewRulesEngineServiceClient(conn)
    ctx := context.Background()

    // Test health check
    healthResp, err := client.HealthCheck(ctx, &pb.Empty{})
    if err != nil {
        log.Fatalf("Health check failed: %v", err)
    }
    log.Printf("Server health: %s", healthResp.Status)

    // Create test rule
    rule := &pb.Rule{
        Id:          "1",
        Name:        "Temperature Alert",
        Description: "Alert when temperature is too high",
        Conditions: []*pb.Condition{
            {
                ConditionType: &pb.Condition_Simple{
                    Simple: &pb.SimpleCondition{
                        Field:    "temperature",
                        Operator: pb.SimpleOperator_GREATER_THAN,
                        Value: &pb.Value{
                            Value: &pb.Value_NumberValue{NumberValue: 25.0},
                        },
                    },
                },
            },
        },
        Actions: []*pb.Action{
            {
                ActionType: &pb.Action_CreateFact{
                    CreateFact: &pb.CreateFactAction{
                        Fields: map[string]*pb.Value{
                            "alert": {
                                Value: &pb.Value_StringValue{StringValue: "high_temp"},
                            },
                        },
                    },
                },
            },
        },
        Priority:  100,
        Enabled:   true,
        Tags:      []string{"monitoring"},
        CreatedAt: time.Now().Unix(),
        UpdatedAt: time.Now().Unix(),
    }

    // Compile rules
    compileReq := &pb.CompileRulesRequest{
        Rules:     []*pb.Rule{rule},
        SessionId: "go_test_session",
    }

    compileResp, err := client.CompileRules(ctx, compileReq)
    if err != nil {
        log.Fatalf("Compilation failed: %v", err)
    }

    log.Printf("Compilation successful: %t", compileResp.Success)
    log.Printf("Session ID: %s", compileResp.SessionId)
    log.Printf("Rules compiled: %d", compileResp.RulesCompiled)
}
```

#### 5. Run the client
```bash
go run main.go
```

### Java Client

#### 1. Create Maven project (`pom.xml`)
```xml
<?xml version="1.0" encoding="UTF-8"?>
<project xmlns="http://maven.apache.org/POM/4.0.0"
         xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
         xsi:schemaLocation="http://maven.apache.org/POM/4.0.0 
         http://maven.apache.org/xsd/maven-4.0.0.xsd">
    <modelVersion>4.0.0</modelVersion>
    
    <groupId>com.example</groupId>
    <artifactId>bingo-java-client</artifactId>
    <version>1.0-SNAPSHOT</version>
    
    <properties>
        <maven.compiler.source>11</maven.compiler.source>
        <maven.compiler.target>11</maven.compiler.target>
        <grpc.version>1.53.0</grpc.version>
        <protobuf.version>3.21.12</protobuf.version>
    </properties>
    
    <dependencies>
        <dependency>
            <groupId>io.grpc</groupId>
            <artifactId>grpc-netty-shaded</artifactId>
            <version>${grpc.version}</version>
        </dependency>
        <dependency>
            <groupId>io.grpc</groupId>
            <artifactId>grpc-protobuf</artifactId>
            <version>${grpc.version}</version>
        </dependency>
        <dependency>
            <groupId>io.grpc</groupId>
            <artifactId>grpc-stub</artifactId>
            <version>${grpc.version}</version>
        </dependency>
        <dependency>
            <groupId>com.google.protobuf</groupId>
            <artifactId>protobuf-java</artifactId>
            <version>${protobuf.version}</version>
        </dependency>
    </dependencies>
    
    <build>
        <extensions>
            <extension>
                <groupId>kr.motd.maven</groupId>
                <artifactId>os-maven-plugin</artifactId>
                <version>1.6.2</version>
            </extension>
        </extensions>
        <plugins>
            <plugin>
                <groupId>org.xolstice.maven.plugins</groupId>
                <artifactId>protobuf-maven-plugin</artifactId>
                <version>0.6.1</version>
                <configuration>
                    <protocArtifact>com.google.protobuf:protoc:${protobuf.version}:exe:${os.detected.classifier}</protocArtifact>
                    <pluginId>grpc-java</pluginId>
                    <pluginArtifact>io.grpc:protoc-gen-grpc-java:${grpc.version}:exe:${os.detected.classifier}</pluginArtifact>
                </configuration>
                <executions>
                    <execution>
                        <goals>
                            <goal>compile</goal>
                            <goal>compile-custom</goal>
                        </goals>
                    </execution>
                </executions>
            </plugin>
        </plugins>
    </build>
</project>
```

#### 2. Add proto file
```bash
mkdir -p src/main/proto
cp /path/to/server/proto/rules_engine.proto src/main/proto/
```

#### 3. Create client code (`src/main/java/com/example/RulesEngineClient.java`)
```java
package com.example;

import io.grpc.ManagedChannel;
import io.grpc.ManagedChannelBuilder;
import rules_engine.v1.RulesEngineServiceGrpc;
import rules_engine.v1.RulesEngine.*;

import java.util.concurrent.TimeUnit;

public class RulesEngineClient {
    private final ManagedChannel channel;
    private final RulesEngineServiceGrpc.RulesEngineServiceBlockingStub blockingStub;

    public RulesEngineClient(String host, int port) {
        this(ManagedChannelBuilder.forAddress(host, port).usePlaintext());
    }

    public RulesEngineClient(ManagedChannelBuilder<?> channelBuilder) {
        channel = channelBuilder.build();
        blockingStub = RulesEngineServiceGrpc.newBlockingStub(channel);
    }

    public void shutdown() throws InterruptedException {
        channel.shutdown().awaitTermination(5, TimeUnit.SECONDS);
    }

    public void testConnection() {
        try {
            // Test health check
            HealthResponse healthResponse = blockingStub.healthCheck(
                com.google.protobuf.Empty.getDefaultInstance()
            );
            System.out.println("Server health: " + healthResponse.getStatus());

            // Create test rule
            Rule rule = Rule.newBuilder()
                .setId("1")
                .setName("Temperature Alert")
                .setDescription("Alert when temperature is too high")
                .addConditions(Condition.newBuilder()
                    .setSimple(SimpleCondition.newBuilder()
                        .setField("temperature")
                        .setOperator(SimpleOperator.GREATER_THAN)
                        .setValue(Value.newBuilder()
                            .setNumberValue(25.0)
                            .build())
                        .build())
                    .build())
                .addActions(Action.newBuilder()
                    .setCreateFact(CreateFactAction.newBuilder()
                        .putFields("alert", Value.newBuilder()
                            .setStringValue("high_temp")
                            .build())
                        .build())
                    .build())
                .setPriority(100)
                .setEnabled(true)
                .addTags("monitoring")
                .setCreatedAt(System.currentTimeMillis() / 1000)
                .setUpdatedAt(System.currentTimeMillis() / 1000)
                .build();

            // Compile rules
            CompileRulesRequest compileRequest = CompileRulesRequest.newBuilder()
                .addRules(rule)
                .setSessionId("java_test_session")
                .build();

            CompileRulesResponse compileResponse = blockingStub.compileRules(compileRequest);
            System.out.println("Compilation successful: " + compileResponse.getSuccess());
            System.out.println("Session ID: " + compileResponse.getSessionId());
            System.out.println("Rules compiled: " + compileResponse.getRulesCompiled());

        } catch (Exception e) {
            System.err.println("Error: " + e.getMessage());
        }
    }

    public static void main(String[] args) throws Exception {
        RulesEngineClient client = new RulesEngineClient("localhost", 50051);
        try {
            client.testConnection();
        } finally {
            client.shutdown();
        }
    }
}
```

#### 4. Build and run
```bash
mvn compile exec:java -Dexec.mainClass="com.example.RulesEngineClient"
```

## Common Connection Patterns

### Two-Phase Processing

This is the recommended approach for large datasets:

```python
# Phase 1: Compile rules once
compile_request = rules_engine_pb2.CompileRulesRequest(
    rules=rules,
    session_id="my_session"
)
compile_response = client.CompileRules(compile_request)
session_id = compile_response.session_id

# Phase 2: Stream facts multiple times
def stream_facts(facts):
    def generate_requests():
        yield rules_engine_pb2.ProcessFactsStreamRequest(session_id=session_id)
        for fact in facts:
            yield rules_engine_pb2.ProcessFactsStreamRequest(fact_batch=fact)
    
    for response in client.ProcessFactsStream(generate_requests()):
        process_result(response)

# Use the session multiple times
stream_facts(batch_1)
stream_facts(batch_2)
stream_facts(batch_3)
```

### Single-Call Processing

For smaller datasets or when convenience is preferred:

```rust
let request = ProcessWithRulesRequest {
    rules: rules_vec,
    facts: facts_vec,
    request_id: "single_call_test".to_string(),
    options: None,
    validate_rules_only: false,
};

let mut stream = client.process_with_rules_stream(request).await?.into_inner();

while let Some(response) = stream.next().await {
    match response?.response {
        Some(Response::RulesCompiled(compilation)) => {
            println!("Rules compiled: {}", compilation.rules_compiled);
        }
        Some(Response::ExecutionResult(result)) => {
            println!("Rule fired: {}", result.rule_name);
        }
        Some(Response::Completion(completion)) => {
            println!("Processing complete: {} facts", completion.total_facts_processed);
            break;
        }
        _ => {}
    }
}
```

## Testing Your Client

### Basic Connectivity Test

Create a simple test to verify your client can connect:

```bash
# Test 1: Check if server is running
telnet localhost 50051

# Test 2: Use grpc_cli if available
grpc_cli call localhost:50051 rules_engine.v1.RulesEngineService.HealthCheck ""

# Test 3: Check server logs for connection attempts
# (on server side)
tail -f /var/log/bingo-api.log
```

### Rule Compilation Test

Test rule compilation with a minimal rule:

```json
{
  "id": "test_rule",
  "name": "Test Rule",
  "conditions": [{
    "simple": {
      "field": "test_field",
      "operator": "EQUAL",
      "value": {"string_value": "test_value"}
    }
  }],
  "actions": [{
    "create_fact": {
      "fields": {
        "result": {"string_value": "test_passed"}
      }
    }
  }],
  "priority": 100,
  "enabled": true
}
```

## Common Issues and Solutions

### 1. Connection Refused
- **Cause**: Server not running or wrong port
- **Solution**: Verify server is running on correct port (default: 50051)

### 2. Proto File Mismatch
- **Cause**: Client using different proto file version than server
- **Solution**: Ensure proto files are synchronized and regenerate client code

### 3. Invalid Argument Errors
- **Cause**: Malformed rule structures or missing required fields
- **Solution**: Validate rule structure against proto schema

### 4. Timeout Errors
- **Cause**: Large rule sets taking too long to compile
- **Solution**: Increase client timeout or optimize rules

### 5. Stream Errors
- **Cause**: Network issues or server overload
- **Solution**: Implement retry logic and connection pooling

## Next Steps

1. **Review the full API documentation** in `grpc-api.md`
2. **Implement error handling** appropriate for your use case
3. **Add authentication** if required for your environment
4. **Optimize performance** based on your data volumes
5. **Set up monitoring** to track client health and performance

For more advanced usage patterns, troubleshooting, and performance optimization, refer to the comprehensive API documentation in `grpc-api.md`.