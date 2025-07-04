# Bingo RETE Rules Engine - Production Deployment Guide

## Overview

This guide provides comprehensive instructions for deploying the Bingo RETE Rules Engine in production environments, including configuration, monitoring, scaling, and operational best practices.

## System Requirements

### Minimum Requirements

- **CPU**: 2 cores (4 recommended)
- **Memory**: 4GB RAM (8GB recommended)
- **Storage**: 10GB available space
- **Network**: 1Gbps network interface
- **OS**: Linux (Ubuntu 20.04+, RHEL 8+, CentOS 8+), macOS 10.15+, Windows Server 2019+

### Recommended Production Specifications

- **CPU**: 8+ cores for high-throughput scenarios
- **Memory**: 16-32GB RAM for large rulesets
- **Storage**: SSD with 100GB+ space for logs and data
- **Network**: 10Gbps for high-volume processing
- **Load Balancer**: For horizontal scaling scenarios

## Installation Methods

### 1. Docker Deployment (Recommended)

#### Build Container Image

```dockerfile
# Dockerfile
FROM rust:1.75-slim as builder

WORKDIR /app
COPY . .
RUN cargo build --release --workspace

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/target/release/bingo-api /usr/local/bin/
COPY --from=builder /app/config.toml /app/config.toml

EXPOSE 8080
CMD ["bingo-api", "--config", "/app/config.toml"]
```

#### Docker Compose Setup

```yaml
# docker-compose.yml
version: '3.8'

services:
  bingo-api:
    build: .
    ports:
      - "8080:8080"
    environment:
      - RUST_LOG=info
      - BINGO_CONFIG_PATH=/app/config.toml
    volumes:
      - ./config:/app/config
      - ./logs:/app/logs
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/health"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 60s

  nginx:
    image: nginx:alpine
    ports:
      - "80:80"
      - "443:443"
    volumes:
      - ./nginx.conf:/etc/nginx/nginx.conf
      - ./ssl:/etc/nginx/ssl
    depends_on:
      - bingo-api
    restart: unless-stopped

volumes:
  logs:
  config:
```

#### Deploy with Docker Compose

```bash
# Clone repository
git clone https://github.com/your-org/bingo-rules-engine.git
cd bingo-rules-engine

# Create production configuration
cp config.example.toml config.toml
# Edit config.toml with production settings

# Deploy
docker-compose up -d

# Verify deployment
curl http://localhost/health
```

### 2. Kubernetes Deployment

#### ConfigMap

```yaml
# k8s/configmap.yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: bingo-config
  namespace: bingo
data:
  config.toml: |
    [server]
    bind_address = "0.0.0.0"
    port = 8080
    workers = 8
    
    [engine]
    max_facts = 1000000
    rule_cache_ttl_seconds = 3600
    fact_cache_size = 100000
    
    [logging]
    level = "info"
    json_format = true
    
    [monitoring]
    metrics_enabled = true
    health_check_enabled = true
```

#### Deployment

```yaml
# k8s/deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: bingo-api
  namespace: bingo
spec:
  replicas: 3
  selector:
    matchLabels:
      app: bingo-api
  template:
    metadata:
      labels:
        app: bingo-api
    spec:
      containers:
      - name: bingo-api
        image: your-registry/bingo-api:latest
        ports:
        - containerPort: 8080
        env:
        - name: RUST_LOG
          value: "info"
        - name: BINGO_CONFIG_PATH
          value: "/app/config/config.toml"
        volumeMounts:
        - name: config
          mountPath: /app/config
        resources:
          requests:
            memory: "1Gi"
            cpu: "500m"
          limits:
            memory: "4Gi"
            cpu: "2"
        livenessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /ready
            port: 8080
          initialDelaySeconds: 5
          periodSeconds: 5
      volumes:
      - name: config
        configMap:
          name: bingo-config
```

#### Service and Ingress

```yaml
# k8s/service.yaml
apiVersion: v1
kind: Service
metadata:
  name: bingo-api-service
  namespace: bingo
spec:
  selector:
    app: bingo-api
  ports:
  - port: 80
    targetPort: 8080
  type: ClusterIP

---
# k8s/ingress.yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: bingo-api-ingress
  namespace: bingo
  annotations:
    kubernetes.io/ingress.class: nginx
    cert-manager.io/cluster-issuer: letsencrypt-prod
    nginx.ingress.kubernetes.io/rate-limit: "100"
spec:
  tls:
  - hosts:
    - api.yourdomain.com
    secretName: bingo-api-tls
  rules:
  - host: api.yourdomain.com
    http:
      paths:
      - path: /
        pathType: Prefix
        backend:
          service:
            name: bingo-api-service
            port:
              number: 80
```

#### Deploy to Kubernetes

```bash
# Create namespace
kubectl create namespace bingo

# Apply configurations
kubectl apply -f k8s/configmap.yaml
kubectl apply -f k8s/deployment.yaml
kubectl apply -f k8s/service.yaml
kubectl apply -f k8s/ingress.yaml

# Verify deployment
kubectl get pods -n bingo
kubectl logs -f deployment/bingo-api -n bingo
```

### 3. Bare Metal / VM Deployment

#### System Preparation

```bash
# Ubuntu/Debian
sudo apt update
sudo apt install -y build-essential curl git pkg-config libssl-dev

# RHEL/CentOS
sudo dnf groupinstall -y "Development Tools"
sudo dnf install -y curl git openssl-devel

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
```

#### Build and Install

```bash
# Clone and build
git clone https://github.com/your-org/bingo-rules-engine.git
cd bingo-rules-engine
cargo build --release --workspace

# Install binary
sudo cp target/release/bingo-api /usr/local/bin/
sudo chmod +x /usr/local/bin/bingo-api

# Create service user
sudo useradd -r -s /bin/false bingo
sudo mkdir -p /etc/bingo /var/log/bingo /var/lib/bingo
sudo chown bingo:bingo /var/log/bingo /var/lib/bingo
```

#### Systemd Service

```ini
# /etc/systemd/system/bingo-api.service
[Unit]
Description=Bingo RETE Rules Engine API
After=network.target
Wants=network.target

[Service]
Type=exec
User=bingo
Group=bingo
ExecStart=/usr/local/bin/bingo-api --config /etc/bingo/config.toml
Restart=always
RestartSec=5
StandardOutput=journal
StandardError=journal
SyslogIdentifier=bingo-api
KillMode=mixed
TimeoutStopSec=30

# Security settings
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/var/log/bingo /var/lib/bingo
CapabilityBoundingSet=CAP_NET_BIND_SERVICE
AmbientCapabilities=CAP_NET_BIND_SERVICE

[Install]
WantedBy=multi-user.target
```

#### Start Service

```bash
# Create configuration
sudo cp config.example.toml /etc/bingo/config.toml
sudo chown bingo:bingo /etc/bingo/config.toml

# Enable and start service
sudo systemctl daemon-reload
sudo systemctl enable bingo-api
sudo systemctl start bingo-api

# Check status
sudo systemctl status bingo-api
sudo journalctl -u bingo-api -f
```

## Configuration Reference

### Core Configuration File

```toml
# /etc/bingo/config.toml

[server]
bind_address = "0.0.0.0"
port = 8080
workers = 8
max_connections = 1000
keep_alive_timeout = 75
request_timeout = 30

[engine]
max_facts = 1000000
max_rules = 10000
rule_cache_ttl_seconds = 3600
fact_cache_size = 100000
calculator_timeout_seconds = 30
max_actions_per_rule = 50

[performance]
enable_fast_lookup = true
enable_field_indexing = true
indexed_fields = ["entity_id", "status", "category", "user_id"]
batch_size = 1000
parallel_processing = true

[caching]
ruleset_cache_enabled = true
ruleset_cache_ttl_seconds = 3600
fact_lookup_cache_size = 50000
calculator_result_cache_size = 10000

[security]
enable_rate_limiting = true
rate_limit_requests_per_minute = 1000
enable_input_validation = true
max_fact_size_bytes = 1048576  # 1MB
max_request_size_bytes = 10485760  # 10MB

[logging]
level = "info"
json_format = true
log_file = "/var/log/bingo/bingo-api.log"
max_file_size = "100MB"
max_files = 10

[monitoring]
metrics_enabled = true
metrics_endpoint = "/metrics"
health_check_enabled = true
health_check_endpoint = "/health"
telemetry_enabled = true

[database]
# Optional: External fact storage
# type = "postgresql"
# connection_string = "postgresql://user:pass@localhost/bingo"
# connection_pool_size = 10
```

### Environment Variables

Key environment variables for configuration override:

```bash
# Server settings
export BINGO_BIND_ADDRESS="0.0.0.0"
export BINGO_PORT="8080"
export BINGO_WORKERS="8"

# Engine settings
export BINGO_MAX_FACTS="1000000"
export BINGO_RULE_CACHE_TTL="3600"
export BINGO_FACT_CACHE_SIZE="100000"

# Logging
export RUST_LOG="info"
export BINGO_LOG_LEVEL="info"
export BINGO_LOG_FORMAT="json"

# Security
export BINGO_RATE_LIMIT="1000"
export BINGO_MAX_FACT_SIZE="1048576"

# Monitoring
export BINGO_METRICS_ENABLED="true"
export BINGO_TELEMETRY_ENABLED="true"
```

## Load Balancer Configuration

### Nginx Configuration

```nginx
# /etc/nginx/sites-available/bingo-api
upstream bingo_backend {
    least_conn;
    server 10.0.1.10:8080 max_fails=3 fail_timeout=30s;
    server 10.0.1.11:8080 max_fails=3 fail_timeout=30s;
    server 10.0.1.12:8080 max_fails=3 fail_timeout=30s;
}

server {
    listen 80;
    listen 443 ssl http2;
    server_name api.yourdomain.com;

    # SSL configuration
    ssl_certificate /etc/ssl/certs/api.yourdomain.com.pem;
    ssl_certificate_key /etc/ssl/private/api.yourdomain.com.key;
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers ECDHE-RSA-AES256-GCM-SHA512:DHE-RSA-AES256-GCM-SHA512;

    # Rate limiting
    limit_req_zone $binary_remote_addr zone=api:10m rate=10r/s;
    limit_req zone=api burst=20 nodelay;

    # Compression
    gzip on;
    gzip_types application/json application/javascript text/css;

    location / {
        proxy_pass http://bingo_backend;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        
        # Timeouts
        proxy_connect_timeout 5s;
        proxy_send_timeout 30s;
        proxy_read_timeout 30s;
        
        # Buffering
        proxy_buffering on;
        proxy_buffer_size 4k;
        proxy_buffers 8 4k;
        
        # Health check
        proxy_next_upstream error timeout invalid_header http_500 http_502 http_503;
    }

    location /health {
        proxy_pass http://bingo_backend;
        access_log off;
    }

    location /metrics {
        proxy_pass http://bingo_backend;
        # Restrict to monitoring systems
        allow 10.0.0.0/8;
        deny all;
    }
}
```

## Monitoring and Observability

### Prometheus Metrics

The engine exposes comprehensive metrics:

```yaml
# prometheus.yml
global:
  scrape_interval: 15s

scrape_configs:
  - job_name: 'bingo-api'
    static_configs:
      - targets: ['api.yourdomain.com:8080']
    metrics_path: '/metrics'
    scrape_interval: 10s
    scheme: https
```

**Key Metrics:**
- `bingo_facts_processed_total`: Total facts processed
- `bingo_rules_executed_total`: Total rule executions
- `bingo_cache_hit_rate`: Cache performance
- `bingo_request_duration_seconds`: Request latencies
- `bingo_memory_usage_bytes`: Memory consumption
- `bingo_active_connections`: Current connections

### Security Configuration

#### TLS/SSL Setup

```bash
# Let's Encrypt (production)
certbot certonly --nginx -d api.yourdomain.com
```

#### API Authentication

```toml
[security]
authentication_required = true
jwt_secret = "your-jwt-secret-key"
jwt_expiration_seconds = 3600
api_key_header = "X-API-Key"
```

### Performance Optimization

#### System-Level Optimization

```bash
# /etc/sysctl.conf
net.core.somaxconn = 65535
net.core.netdev_max_backlog = 5000
net.ipv4.tcp_max_syn_backlog = 4096
net.ipv4.tcp_keepalive_time = 600

# Apply settings
sudo sysctl -p
```

## Backup and Recovery

### Configuration Backup

```bash
#!/bin/bash
# backup-config.sh

BACKUP_DIR="/backup/bingo/$(date +%Y%m%d)"
mkdir -p "$BACKUP_DIR"

# Backup configuration
cp /etc/bingo/config.toml "$BACKUP_DIR/"

# Backup rules
cp -r /var/lib/bingo/rules "$BACKUP_DIR/"

echo "Backup completed: $BACKUP_DIR"
```

## Scaling Strategies

### Horizontal Scaling

#### Auto-Scaling with Kubernetes

```yaml
# k8s/hpa.yaml
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: bingo-api-hpa
  namespace: bingo
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: bingo-api
  minReplicas: 3
  maxReplicas: 20
  metrics:
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: 70
```

## Troubleshooting

### Common Issues

#### High Memory Usage

```bash
# Check memory statistics
curl http://localhost:8080/debug/memory

# Reduce cache sizes in config.toml:
[caching]
fact_cache_size = 10000
ruleset_cache_size = 1000
```

#### Slow Rule Execution

```bash
# Check rule performance
curl http://localhost:8080/debug/rules/performance

# Enable debug logging
export RUST_LOG=debug
sudo systemctl restart bingo-api
```

### Log Analysis

```bash
# Search for errors
grep ERROR /var/log/bingo/bingo-api.log

# Monitor real-time logs
tail -f /var/log/bingo/bingo-api.log | grep -E "(ERROR|WARN)"
```

## Maintenance

### Regular Maintenance Tasks

```bash
#!/bin/bash
# daily-maintenance.sh

# Rotate logs
sudo logrotate /etc/logrotate.d/bingo-api

# Check disk space
df -h /var/lib/bingo /var/log/bingo

# Health check
curl -f http://localhost:8080/health || \
  echo "Health check failed" | mail -s "Bingo API Alert" admin@yourdomain.com
```

### Updates and Upgrades

```bash
#!/bin/bash
# upgrade.sh

VERSION=$1

# Backup current installation
./backup-config.sh

# Stop service
sudo systemctl stop bingo-api

# Install new binary
sudo cp "bingo-api-$VERSION/bingo-api" /usr/local/bin/
sudo chmod +x /usr/local/bin/bingo-api

# Start service
sudo systemctl start bingo-api

# Verify upgrade
curl http://localhost:8080/version
```

This deployment guide provides comprehensive coverage of production deployment scenarios, from containerized environments to bare metal installations, with detailed configuration, monitoring, and operational procedures.