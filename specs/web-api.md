# Web API Specification

## Overview

Stateless HTTP API for rule evaluation built on Axum with comprehensive error handling and observability. Features OpenAPI 3.0 specification with automatic documentation generation and Swagger UI integration.

**Key Design Principles:**
- **Stateless Architecture**: No shared state between requests for maximum concurrency
- **Per-Request Engines**: Fresh engine instance created for each evaluation
- **Hardcoded Calculators**: Built-in calculators compiled into the engine
- **Rules-with-Facts**: Rules provided alongside facts in each request

## API Architecture

```mermaid
graph TB
    subgraph "Client Layer"
        WEB[Web UI]
        API[API Clients] 
        CLI[CLI Tools]
    end
    
    subgraph "Application Layer"
        EVAL[/evaluate Endpoint]
        HEALTH[/health Endpoint]
        DOCS[/swagger-ui/ Endpoint]
        OPENAPI[/api-docs/openapi.json]
    end
    
    subgraph "Processing Layer (Per-Request)"
        ENGINE[Bingo Engine Instance]
        RETE[RETE Network]
        CALC[Built-in Calculators]
    end
    
    subgraph "Memory Layer (Per-Request)"
        MEM[ArenaFactStore]
    end
    
    WEB --> EVAL
    API --> EVAL
    CLI --> EVAL
    
    WEB --> HEALTH
    API --> HEALTH
    
    WEB --> DOCS
    API --> OPENAPI
    
    EVAL --> ENGINE
    ENGINE --> RETE
    RETE --> CALC
    
    ENGINE --> MEM
    
    note1[Engine instances created per-request]
    note2[No shared state between requests]
    note3[Perfect concurrency scaling]
```

## Core Endpoints

### POST /evaluate

**Purpose:** Evaluate rules against facts with comprehensive rule execution

#### Request Schema
```rust
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct EvaluateRequest {
    pub facts: Vec<ApiFact>,
    pub rules: Vec<ApiRule>,  // Rules are required with each request
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ApiFact {
    pub id: String,
    pub data: serde_json::Value,
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ApiRule {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub conditions: Vec<ApiCondition>,
    pub actions: Vec<ApiAction>,
    pub enabled: bool,
    pub priority: i32,
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

#### Response Schema
```rust
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct EvaluateResponse {
    pub request_id: String,
    pub results: Vec<ApiRuleExecutionResult>,
    pub rules_processed: usize,
    pub facts_processed: usize,
    pub rules_fired: usize,
    pub processing_time_ms: u64,
    pub stats: EngineStats,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ApiRuleExecutionResult {
    pub rule_id: String,
    pub fact_id: String,
    pub actions_executed: Vec<ActionResult>,
}
```

#### Example Request
```json
{
  "facts": [
    {
      "id": "fact_001",
      "data": {
        "employee_id": "emp_123",
        "hours_worked": 42.5,
        "is_student_visa": true,
        "weekly_limit": 20.0
      },
      "created_at": "2024-06-19T00:00:00Z"
    }
  ],
  "rules": [
    {
      "id": "student_visa_compliance",
      "name": "Student Visa Weekly Hours Compliance",
      "description": "Ensure student visa holders don't exceed 20 hours per week",
      "conditions": [
        {
          "type": "simple",
          "field": "is_student_visa",
          "operator": "equal",
          "value": true
        }
      ],
      "actions": [
        {
          "type": "call_calculator",
          "calculator_name": "threshold_checker",
          "input_mapping": {
            "value": "hours_worked",
            "threshold": "weekly_limit",
            "operator": "LessThanOrEqual"
          },
          "output_field": "compliance_status"
        }
      ],
      "enabled": true,
      "priority": 100,
      "tags": ["compliance", "student_visa"],
      "created_at": "2024-01-01T00:00:00Z",
      "updated_at": "2024-01-01T00:00:00Z"
    }
  ]
}
```

#### Example Response
```json
{
  "request_id": "req_12345",
  "results": [
    {
      "rule_id": "student_visa_compliance",
      "fact_id": "fact_001",
      "actions_executed": [
        {
          "type": "calculator_result",
          "calculator": "threshold_checker",
          "result": "{\\"passes\\": false, \\"value\\": 42.5, \\"threshold\\": 20.0, \\"operator\\": \\"LessThanOrEqual\\", \\"violation_amount\\": 22.5, \\"status\\": \\"non_compliant\\"}"
        }
      ]
    }
  ],
  "rules_processed": 1,
  "facts_processed": 1,
  "rules_fired": 1,
  "processing_time_ms": 2,
  "stats": {
    "rule_count": 1,
    "fact_count": 1,
    "node_count": 2,
    "memory_usage_bytes": 1024
  }
}
```

### GET /health

**Purpose:** Health check endpoint for monitoring and load balancers

#### Response Schema
```rust
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct HealthResponse {
    pub status: String,
    pub timestamp: DateTime<Utc>,
    pub version: String,
    pub uptime_seconds: u64,
}
```

#### Example Response
```json
{
  "status": "healthy",
  "timestamp": "2024-06-19T12:00:00Z",
  "version": "0.1.0",
  "uptime_seconds": 3600
}
```

### GET /swagger-ui/

**Purpose:** Interactive OpenAPI documentation interface

- **Implementation**: Swagger UI integration with auto-generated schemas
- **Features**: Interactive API exploration, request testing, schema validation
- **Auto-sync**: Documentation automatically reflects code changes

### GET /api-docs/openapi.json

**Purpose:** OpenAPI 3.0 specification in JSON format

- **Schema Generation**: Automatic from Rust type definitions using `utoipa`
- **Validation**: Request/response schemas with comprehensive validation rules
- **Documentation**: Rich descriptions and examples for all endpoints

## Rule Definition Schema

### Condition Types

#### Simple Condition
```rust
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SimpleCondition {
    #[serde(rename = "type")]
    pub condition_type: String, // "simple"
    pub field: String,
    pub operator: String,
    pub value: serde_json::Value,
}
```

**Supported Operators:**
- `equal` - Exact equality comparison
- `not_equal` - Inequality comparison
- `greater_than` - Numeric/date greater than
- `less_than` - Numeric/date less than
- `greater_than_or_equal` - Numeric/date greater than or equal
- `less_than_or_equal` - Numeric/date less than or equal
- `contains` - String/array contains check

### Action Types

#### Log Action
```rust
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct LogAction {
    #[serde(rename = "type")]
    pub action_type: String, // "log"
    pub message: String,
}
```

#### Set Field Action
```rust
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SetFieldAction {
    #[serde(rename = "type")]
    pub action_type: String, // "set_field"
    pub field: String,
    pub value: serde_json::Value,
}
```

#### Call Calculator Action
```rust
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CallCalculatorAction {
    #[serde(rename = "type")]
    pub action_type: String, // "call_calculator"
    pub calculator_name: String,
    pub input_mapping: HashMap<String, String>,
    pub output_field: String,
}
```

#### Create Fact Action
```rust
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateFactAction {
    #[serde(rename = "type")]
    pub action_type: String, // "create_fact"
    pub fact_id: String,
    pub fact_data: serde_json::Value,
}
```

#### Formula Action (Calculator DSL)
```rust
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct FormulaAction {
    #[serde(rename = "type")]
    pub action_type: String, // "formula"
    pub expression: String,
    pub output_field: String,
}
```

## Built-in Calculators

### threshold_checker
**Purpose:** Validate values against thresholds with compliance checking

**Input Parameters:**
- `value` (number): Value to check
- `threshold` (number): Threshold to compare against
- `operator` (string): Comparison operator (default: "LessThanOrEqual")

**Output:** JSON object with compliance status and violation details

### limit_validator
**Purpose:** Multi-tier validation with warning/critical/max levels

**Input Parameters:**
- `value` (number): Value to validate
- `warning_threshold` (number, optional): Warning level
- `critical_threshold` (number, optional): Critical level
- `max_threshold` (number, optional): Maximum allowed value

**Output:** JSON object with severity level and utilization details

### hours_between_datetime
**Purpose:** Calculate hours between two datetime values

**Input Parameters:**
- `start_datetime` (string): Start time in ISO 8601 format
- `end_datetime` (string): End time in ISO 8601 format

**Output:** Floating point number representing hours

## Error Handling

### Error Response Schema
```rust
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ApiError {
    pub error: ErrorDetails,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ErrorDetails {
    pub code: String,
    pub message: String,
    pub details: Option<serde_json::Value>,
    pub request_id: String,
}
```

### Error Types

#### Validation Error (400)
```json
{
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "Invalid rule condition",
    "details": {
      "field": "operator",
      "value": "invalid_op",
      "expected": ["equal", "not_equal", "contains"]
    },
    "request_id": "req_12345"
  }
}
```

#### Processing Error (500)
```json
{
  "error": {
    "code": "PROCESSING_ERROR",
    "message": "Rule evaluation failed",
    "details": {
      "rule_id": "rule_001",
      "error": "Calculator 'invalid_calc' not found"
    },
    "request_id": "req_12345"
  }
}
```

#### Rate Limit Error (429)
```json
{
  "error": {
    "code": "RATE_LIMIT_EXCEEDED",
    "message": "Too many requests",
    "details": {
      "limit": 100,
      "window": "60s",
      "retry_after": 30
    },
    "request_id": "req_12345"
  }
}
```

## HTTP Headers

### Request Headers
- `Content-Type: application/json` (required for POST requests)
- `Accept: application/json` (recommended)
- `User-Agent: <client-identifier>` (recommended for tracking)

### Response Headers
- `Content-Type: application/json`
- `X-Request-ID: <request-id>` (for request tracking)
- `X-Processing-Time: <milliseconds>` (processing duration)
- `X-Rules-Fired: <count>` (number of rules that executed)

## Configuration

### Environment Variables
```bash
BINGO_HOST=127.0.0.1        # Server bind address
BINGO_PORT=3000             # Server port
RUST_LOG=bingo=info         # Logging level
```

### Server Configuration
```rust
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub cors_enabled: bool,
    pub request_timeout: Duration,
    pub max_request_size: usize,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 3000,
            cors_enabled: true,
            request_timeout: Duration::from_secs(30),
            max_request_size: 10 * 1024 * 1024, // 10MB
        }
    }
}
```

## Performance Characteristics

### Request Processing
- **Throughput**: Processes 1M+ facts in <1 second
- **Latency**: Sub-second response for typical workloads
- **Memory**: Efficient memory usage with arena allocation
- **Concurrency**: Async request handling with single-threaded rule processing

### Scalability
- **Fact Capacity**: Tested up to 1M facts per request
- **Rule Capacity**: Supports thousands of rules with optimization
- **Memory Efficiency**: <3GB memory usage for enterprise workloads
- **Response Time**: Predictable performance characteristics

## Security Considerations

### Input Validation
- **Schema Validation**: Comprehensive JSON schema validation
- **Size Limits**: Configurable request size limits
- **Type Safety**: Strong typing prevents injection attacks
- **Sanitization**: Input sanitization for calculator parameters

### Error Information
- **Error Hiding**: Production mode hides internal error details
- **Request Tracking**: All requests logged with unique IDs
- **Rate Limiting**: Configurable rate limiting per client
- **CORS**: Configurable CORS policy for browser security

## Monitoring and Observability

### Structured Logging
```rust
#[instrument(skip(engine))]
async fn evaluate_facts(
    State(engine): State<Arc<Engine>>,
    Json(request): Json<EvaluateRequest>,
) -> Result<Json<EvaluateResponse>, ApiError> {
    info!(
        facts_count = request.facts.len(),
        rules_count = request.rules.as_ref().map(|r| r.len()).unwrap_or(0),
        "Processing evaluate request"
    );
    
    // Processing logic...
}
```

### Metrics Collection
- **Request Duration**: Processing time per request
- **Facts Processed**: Number of facts per request
- **Rules Fired**: Number of rules that executed
- **Memory Usage**: Peak memory usage per request
- **Error Rates**: Classification of error types and frequencies

### Health Monitoring
- **Endpoint Health**: `/health` endpoint for load balancer checks
- **Application Health**: Internal service health validation
- **Performance Metrics**: Real-time performance monitoring
- **Error Tracking**: Comprehensive error tracking and alerting