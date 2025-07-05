# Bingo RETE Rules Engine - gRPC Production Deployment Guide

## Overview

This guide provides comprehensive instructions for deploying the Bingo RETE Rules Engine's **gRPC streaming API** in production environments. The engine has been completely migrated from HTTP/REST to gRPC for superior performance, streaming capabilities, and lower overhead.

## Key gRPC Benefits

- **Streaming Interface**: Real-time fact processing with O(1) memory usage
- **Two-Phase Processing**: Compile rules once, stream facts efficiently
- **High Performance**: Protocol buffer serialization, HTTP/2 multiplexing
- **Type Safety**: Strong typing with automatic code generation
- **Backpressure Control**: Client can control processing flow

## System Requirements

### Minimum Requirements

- **CPU**: 2 cores (4 recommended)
- **Memory**: 4GB RAM (8GB recommended)
- **Storage**: 10GB available space
- **Network**: 1Gbps network interface with HTTP/2 support
- **OS**: Linux (Ubuntu 20.04+, RHEL 8+), macOS 10.15+, Windows Server 2019+

### gRPC-Specific Requirements

- **Protocol Buffer Compiler**: For building from source
- **gRPC Health Probe**: For Kubernetes health checks
- **HTTP/2 Load Balancer**: For proper gRPC load balancing
- **TLS Certificates**: Required for production gRPC deployments

## Installation Methods

### 1. Docker Deployment (Recommended)

#### Build Container Image

The provided Dockerfile includes protobuf compiler and gRPC optimizations:

```bash
# Build the image
docker build -t bingo-grpc:latest .

# Run the container
docker run -d \
  --name bingo-grpc \
  -p 50051:50051 \
  -e GRPC_LISTEN_ADDRESS=0.0.0.0:50051 \
  -e RUST_LOG=info \
  bingo-grpc:latest
```

#### Docker Compose Setup

Use the provided `docker-compose.yml` for a complete setup:

```bash
# Start the full stack
docker-compose up -d

# View logs
docker-compose logs -f bingo-grpc

# Test gRPC connection (requires grpcurl)
grpcurl -plaintext localhost:50051 list
```

**Included Services:**
- **bingo-grpc**: Main gRPC service on port 50051
- **envoy**: HTTP-to-gRPC proxy on port 8080 (for legacy clients)
- **nginx-grpc-lb**: gRPC load balancer on port 50052

### 2. Kubernetes Deployment

#### Quick Deploy

```bash
# Set your image registry
export REGISTRY=your-registry.com
export IMAGE_TAG=v1.0.0

# Deploy to Kubernetes
cd k8s
./deploy.sh

# Verify deployment
kubectl get pods -n bingo
kubectl logs -f deployment/bingo-grpc -n bingo
```

#### Manual Deployment

```bash
# Create namespace and apply configurations
kubectl apply -f k8s/namespace.yaml
kubectl apply -f k8s/configmap.yaml
kubectl apply -f k8s/deployment.yaml
kubectl apply -f k8s/service.yaml
kubectl apply -f k8s/hpa.yaml

# Optional: Ingress and monitoring
kubectl apply -f k8s/ingress.yaml
kubectl apply -f k8s/servicemonitor.yaml
```

#### gRPC Client Connection

```bash
# Port forward for testing
kubectl port-forward svc/bingo-grpc-service 50051:50051 -n bingo

# Test with grpcurl
grpcurl -plaintext localhost:50051 rules_engine.v1.RulesEngineService/HealthCheck
```

### 3. Bare Metal / VM Deployment

#### System Preparation

```bash
# Ubuntu/Debian - Install protobuf compiler
sudo apt update
sudo apt install -y build-essential curl git pkg-config libssl-dev protobuf-compiler

# RHEL/CentOS
sudo dnf groupinstall -y "Development Tools"
sudo dnf install -y curl git openssl-devel protobuf-compiler

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
```

#### Build and Install

```bash
# Clone and build
git clone https://github.com/your-org/bingo-rules-engine.git
cd bingo-rules-engine
cargo build --release --bin bingo

# Install binary
sudo cp target/release/bingo /usr/local/bin/
sudo chmod +x /usr/local/bin/bingo
```

#### Systemd Service

```ini
# /etc/systemd/system/bingo-grpc.service
[Unit]
Description=Bingo RETE Rules Engine gRPC API
After=network.target
Wants=network.target

[Service]
Type=exec
User=bingo
Group=bingo
ExecStart=/usr/local/bin/bingo
Environment=GRPC_LISTEN_ADDRESS=0.0.0.0:50051
Environment=RUST_LOG=info
Environment=SERVICE_NAME=bingo-grpc-api
Restart=always
RestartSec=5
StandardOutput=journal
StandardError=journal
SyslogIdentifier=bingo-grpc

# Security settings
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true

[Install]
WantedBy=multi-user.target
```

## gRPC Configuration

### Environment Variables

```bash
# gRPC server settings
export GRPC_LISTEN_ADDRESS="0.0.0.0:50051"
export SERVICE_NAME="bingo-grpc-api"
export SERVICE_VERSION="0.1.0"
export BINGO_ENVIRONMENT="production"

# Logging
export RUST_LOG="info"

# Optional: OTEL tracing
export OTEL_SERVICE_NAME="bingo-grpc"
export OTEL_SERVICE_VERSION="0.1.0"
```

### gRPC Service Methods

The service implements the following streaming methods:

1. **CompileRules**: Validate and compile rules (unary)
2. **ProcessFactsStream**: Stream facts through pre-compiled rules
3. **ProcessWithRulesStream**: Single-call rule compilation + fact streaming
4. **HealthCheck**: Service health verification

## Load Balancer Configuration

### Nginx gRPC Load Balancing

```nginx
# /etc/nginx/sites-available/bingo-grpc
upstream bingo_grpc_backend {
    least_conn;
    server 10.0.1.10:50051 max_fails=3 fail_timeout=30s;
    server 10.0.1.11:50051 max_fails=3 fail_timeout=30s;
    server 10.0.1.12:50051 max_fails=3 fail_timeout=30s;
}

server {
    listen 50051 http2;
    server_name grpc.yourdomain.com;

    # SSL configuration for gRPC
    ssl_certificate /etc/ssl/certs/grpc.yourdomain.com.pem;
    ssl_certificate_key /etc/ssl/private/grpc.yourdomain.com.key;
    ssl_protocols TLSv1.2 TLSv1.3;

    # gRPC-specific settings
    grpc_connect_timeout 5s;
    grpc_send_timeout 30s;
    grpc_read_timeout 30s;

    location / {
        grpc_pass grpc://bingo_grpc_backend;
        grpc_set_header Host $host;
        grpc_set_header X-Real-IP $remote_addr;
        grpc_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
    }
}
```

### HAProxy gRPC Configuration

```
# /etc/haproxy/haproxy.cfg
global
    daemon

defaults
    mode http
    timeout connect 5000ms
    timeout client 50000ms
    timeout server 50000ms

frontend grpc_frontend
    bind *:50051 proto h2
    default_backend grpc_backend

backend grpc_backend
    balance roundrobin
    option httpchk GET /grpc.health.v1.Health/Check
    server grpc1 10.0.1.10:50051 check proto h2
    server grpc2 10.0.1.11:50051 check proto h2
    server grpc3 10.0.1.12:50051 check proto h2
```

## Client Connection Examples

### grpcurl (CLI Testing)

```bash
# List available services
grpcurl -plaintext localhost:50051 list

# Health check
grpcurl -plaintext localhost:50051 \
  rules_engine.v1.RulesEngineService/HealthCheck

# Compile rules
grpcurl -plaintext -d '{
  "session_id": "test-session",
  "rules": [
    {
      "id": "1",
      "name": "Test Rule",
      "conditions": [
        {
          "simple": {
            "field": "status",
            "operator": "SIMPLE_OPERATOR_EQUAL",
            "value": {"string_value": "active"}
          }
        }
      ],
      "actions": [
        {
          "create_fact": {
            "fields": {
              "message": {"string_value": "Rule fired!"}
            }
          }
        }
      ]
    }
  ]
}' localhost:50051 \
  rules_engine.v1.RulesEngineService/CompileRules
```

### Python Client

```python
import grpc
import rules_engine_pb2
import rules_engine_pb2_grpc

# Connect to gRPC service
channel = grpc.insecure_channel('localhost:50051')
stub = rules_engine_pb2_grpc.RulesEngineServiceStub(channel)

# Health check
health_response = stub.HealthCheck(rules_engine_pb2.Empty())
print(f"Status: {health_response.status}")

# Compile rules
request = rules_engine_pb2.CompileRulesRequest(
    session_id="python-session",
    rules=[
        rules_engine_pb2.Rule(
            id="1",
            name="Python Test Rule",
            # ... rule definition
        )
    ]
)
response = stub.CompileRules(request)
print(f"Compiled {response.rules_compiled} rules")
```

### Node.js Client

```javascript
const grpc = require('@grpc/grpc-js');
const protoLoader = require('@grpc/proto-loader');

// Load protobuf
const packageDefinition = protoLoader.loadSync('rules_engine.proto');
const rulesEngine = grpc.loadPackageDefinition(packageDefinition).rules_engine.v1;

// Create client
const client = new rulesEngine.RulesEngineService(
  'localhost:50051',
  grpc.credentials.createInsecure()
);

// Health check
client.HealthCheck({}, (error, response) => {
  if (error) {
    console.error('Error:', error);
  } else {
    console.log('Status:', response.status);
  }
});
```

## Monitoring and Observability

### Health Checks

The service provides gRPC health checks:

```bash
# Using grpc_health_probe
grpc_health_probe -addr=localhost:50051

# Using grpcurl
grpcurl -plaintext localhost:50051 grpc.health.v1.Health/Check
```

### Metrics Collection

For Prometheus monitoring:

```yaml
# prometheus.yml
scrape_configs:
  - job_name: 'bingo-grpc'
    kubernetes_sd_configs:
      - role: endpoints
        namespaces:
          names: ['bingo']
    relabel_configs:
      - source_labels: [__meta_kubernetes_service_name]
        action: keep
        regex: bingo-grpc-service
```

### Key gRPC Metrics

- `grpc_server_started_total`: Total gRPC requests started
- `grpc_server_handled_total`: Total gRPC requests completed
- `grpc_server_handling_seconds`: Request duration histogram
- `grpc_server_msg_received_total`: Messages received
- `grpc_server_msg_sent_total`: Messages sent

## Security Configuration

### TLS/SSL Setup

```bash
# Generate self-signed certificate for development
openssl req -x509 -newkey rsa:4096 -keyout key.pem -out cert.pem -days 365 -nodes

# Production: Use Let's Encrypt or corporate CA
certbot certonly --standalone -d grpc.yourdomain.com
```

### mTLS (Mutual TLS)

For enhanced security, configure mutual TLS:

```rust
// Server-side mTLS configuration (if implementing)
let tls = ServerTlsConfig::new()
    .identity(Identity::from_pem(cert, key))
    .client_ca_root(Certificate::from_pem(ca_cert));
```

## Performance Optimization

### gRPC-Specific Tuning

```bash
# System-level optimization for gRPC
echo 'net.core.rmem_default = 262144' >> /etc/sysctl.conf
echo 'net.core.rmem_max = 16777216' >> /etc/sysctl.conf
echo 'net.core.wmem_default = 262144' >> /etc/sysctl.conf
echo 'net.core.wmem_max = 16777216' >> /etc/sysctl.conf
sysctl -p
```

### Connection Pooling

For high-throughput scenarios:
- Use connection pooling in clients
- Configure appropriate `GRPC_KEEPALIVE_*` settings
- Implement circuit breakers for fault tolerance

## Troubleshooting

### Common gRPC Issues

#### Connection Refused

```bash
# Check if service is listening
netstat -tlnp | grep 50051
ss -tlnp | grep 50051

# Test basic connectivity
telnet localhost 50051
```

#### HTTP/2 Issues

```bash
# Verify HTTP/2 support
curl -v --http2-prior-knowledge http://localhost:50051

# Check with grpcurl
grpcurl -vv -plaintext localhost:50051 list
```

#### TLS Certificate Problems

```bash
# Verify certificate
openssl s_client -connect grpc.yourdomain.com:50051 -servername grpc.yourdomain.com

# Test with insecure connection
grpcurl -plaintext -insecure localhost:50051 list
```

### Performance Debugging

```bash
# Check gRPC server performance
grpcurl -plaintext -d '{}' localhost:50051 \
  rules_engine.v1.RulesEngineService/HealthCheck

# Monitor with continuous requests
while true; do
  grpcurl -plaintext localhost:50051 \
    rules_engine.v1.RulesEngineService/HealthCheck
  sleep 1
done
```

## Migration from HTTP/REST

### Client Migration

1. **Protocol**: Change from HTTP/1.1 to gRPC/HTTP/2
2. **Serialization**: JSON → Protocol Buffers
3. **Streaming**: Leverage bidirectional streaming for better performance
4. **Error Handling**: gRPC status codes instead of HTTP status codes

### Gradual Migration

Use Envoy proxy for gradual migration:
- HTTP clients → Envoy → gRPC service
- Provides time to migrate clients incrementally

## Production Checklist

- [ ] TLS certificates configured and valid
- [ ] Load balancer supports HTTP/2 and gRPC
- [ ] Health checks implemented and configured
- [ ] Monitoring and alerting set up
- [ ] Resource limits configured (CPU, memory)
- [ ] Horizontal Pod Autoscaler configured
- [ ] Security policies applied (NetworkPolicy, PodSecurityPolicy)
- [ ] Backup and disaster recovery procedures documented
- [ ] Client connection pooling configured
- [ ] Circuit breakers implemented in clients

This deployment guide ensures a robust, scalable, and secure gRPC deployment of the Bingo RETE Rules Engine.