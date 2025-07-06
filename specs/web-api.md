# gRPC API Specification

This document details the gRPC streaming API for the Bingo Rules Engine.

## Service Definition

The Bingo Rules Engine provides a gRPC service (`RulesEngineService`) with multiple processing patterns optimized for different use cases.

### Core Methods

#### Two-Phase Processing
-   **`CompileRules`**: Validates and compiles rules, returning a session ID for subsequent fact processing.
-   **`ProcessFactsStream`**: Streams facts through pre-compiled rules with real-time processing control.

#### Single-Call Processing
-   **`ProcessWithRulesStream`**: Validates rules and processes facts in a single streaming call.
-   **`ProcessFactsBatch`**: Batch processing with periodic status updates.

#### Cached Ruleset Processing
-   **`RegisterRuleset`**: Registers and caches a compiled ruleset for high-performance repeated use.
-   **`EvaluateRulesetStream`**: Streams facts through a cached ruleset.

#### Health & Monitoring
-   **`HealthCheck`**: Returns service health status and version information.

## Processing Patterns

### Pattern 1: Two-Phase Processing (Recommended for High-Volume)

**Phase 1: Compile Rules**
```protobuf
CompileRulesRequest {
  repeated Rule rules = 1;
  string session_id = 2;
  ProcessingOptions options = 3;
}
```

**Phase 2: Stream Facts**
```protobuf
ProcessFactsStreamRequest {
  oneof request {
    string session_id = 1;
    Fact fact_batch = 2;
    ProcessingControl control = 3;
  }
}
```

### Pattern 2: Single-Call Streaming (Recommended for General Use)

```protobuf
ProcessWithRulesRequest {
  repeated Rule rules = 1;
  repeated Fact facts = 2;
  string request_id = 3;
  ProcessingOptions options = 4;
  bool validate_rules_only = 5;
}
```

### Pattern 3: Cached Ruleset Processing (Recommended for Production)

**Register Ruleset**
```protobuf
RegisterRulesetRequest {
  string ruleset_id = 1;
  repeated Rule rules = 2;
}
```

**Evaluate with Cached Ruleset**
```protobuf
EvaluateRulesetRequest {
  string ruleset_id = 1;
  repeated Fact facts = 2;
  string request_id = 3;
  ProcessingOptions options = 4;
}
```

## Data Types

### Core Types
-   **`Fact`**: Contains ID, data map, and timestamp
-   **`Rule`**: Contains conditions, actions, priority, and metadata
-   **`Value`**: Union type supporting string, number, boolean, and integer values
-   **`Condition`**: Simple or complex logical conditions
-   **`Action`**: Create fact, call calculator, or formula actions

### Response Types
-   **`RuleExecutionResult`**: Details of rule execution including matched facts and action results
-   **`ProcessingStatus`**: Progress updates during batch processing
-   **`ProcessingResponse`**: Union response type for streaming operations

## Streaming Features

### Real-Time Control
-   **Pause/Resume**: Control processing flow during streaming
-   **Stop**: Gracefully terminate processing
-   **Flush**: Force processing of buffered facts

### Progressive Responses
-   **Compilation Results**: Immediate feedback on rule validation
-   **Execution Results**: Real-time rule firing notifications
-   **Status Updates**: Progress tracking for large datasets
-   **Completion Summary**: Final processing statistics

## Error Handling

gRPC status codes are used to indicate errors:
-   `INVALID_ARGUMENT`: Malformed rules or facts
-   `FAILED_PRECONDITION`: Missing session or invalid state
-   `INTERNAL`: Engine compilation or execution errors
-   `UNIMPLEMENTED`: Features not yet available

Error details are provided in status messages with structured error information.