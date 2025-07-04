# Web API Specification

This document details the HTTP API for the Bingo Rules Engine.

## Endpoints

### Health & Monitoring

-   `GET /health`: Checks the health of the service.
-   `GET /engine/stats`: Provides statistics about the engine (stateless, so mostly informational).
-   `GET /cache/stats`: Returns detailed statistics for the ruleset and engine caches.
-   `GET /docs`: Serves the interactive OpenAPI documentation.

### Core Functionality

-   **`POST /rulesets`**: Registers and pre-compiles a set of rules. This is the recommended first step for production workloads. The rules are cached and can be referenced by a `ruleset_id`.
-   **`POST /evaluate`**: Evaluates a set of facts against rules. This endpoint supports two modes:
    1.  **Cached Mode (High Performance)**: Provide a `ruleset_id` and a list of `facts`. The engine uses the pre-compiled rules from the cache.
    2.  **Ad-hoc Mode**: Provide a list of `rules` and a list of `facts`. The rules are compiled on-the-fly (and cached for a short duration).

## Request/Response Formats

All endpoints use JSON for request and response bodies.

### Example: Cached Evaluation Workflow

**1. Register a Ruleset**

`POST /rulesets`
```json
{
  "ruleset_id": "payroll_v2",
  "rules": [ /* ... array of rule objects ... */ ]
}
```

**2. Evaluate Facts**

`POST /evaluate`
```json
{
  "ruleset_id": "payroll_v2",
  "facts": [ /* ... array of fact objects ... */ ]
}
```

## Error Handling

The API uses standard HTTP status codes to indicate success or failure. Error responses are returned as a JSON object with a consistent structure:

```json
{
  "error_code": "VALIDATION_ERROR",
  "message": "Invalid request payload: 'facts' field is required.",
  "request_id": "uuid-goes-here"
}
```