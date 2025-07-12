# Production Deployment Guide

This guide provides comprehensive instructions for deploying the Bingo RETE Rules Engine in production environments using Docker Compose and Kubernetes.

## Table of Contents

1. [Overview](#overview)
2. [Prerequisites](#prerequisites)
3. [Docker Compose Deployment](#docker-compose-deployment)
4. [Kubernetes Deployment](#kubernetes-deployment)
5. [Security Hardening](#security-hardening)
6. [Monitoring and Observability](#monitoring-and-observability)
7. [Performance Tuning](#performance-tuning)
8. [Disaster Recovery](#disaster-recovery)
9. [Troubleshooting](#troubleshooting)

## Overview

The Bingo RETE Rules Engine supports multiple deployment strategies:

- **Docker Compose**: For development, testing, and smaller production deployments
- **Kubernetes**: For scalable, enterprise production deployments
- **Bare Metal**: For high-performance, dedicated hardware deployments

## Prerequisites

### Common Requirements

- Docker 24.0+ with BuildKit support
- Docker Compose 2.20+
- At least 4GB RAM and 2 CPU cores
- Network access for image pulling and external dependencies

### Kubernetes Requirements

- Kubernetes 1.25+
- kubectl configured with cluster access
- Helm 3.10+ (optional, for package management)
- cert-manager (for TLS certificate automation)
- Prometheus Operator (for monitoring)

### Security Requirements

- TLS certificates for external communication
- Network policies configured
- RBAC properly configured
- Secrets management system (Vault, AWS Secrets Manager, etc.)

## Docker Compose Deployment

### Quick Start

1. **Clone and prepare the repository:**
   ```bash
   git clone <repository-url>
   cd bingo
   ```

2. **Configure environment:**
   ```bash
   cp .env.example .env
   # Edit .env with your configuration
   ```

3. **Deploy the stack:**
   ```bash
   docker-compose up -d
   ```

4. **Verify deployment:**
   ```bash
   docker-compose ps
   docker-compose logs bingo-grpc
   ```

### Configuration

The Docker Compose stack includes:

- **bingo-grpc**: Main gRPC service (3 replicas)
- **envoy-proxy**: Load balancer and service mesh
- **nginx**: HTTP proxy and static file serving
- **redis**: Caching layer
- **prometheus**: Metrics collection
- **grafana**: Metrics visualization
- **jaeger**: Distributed tracing

### Environment Variables

Key configuration options in `.env`:

```bash
# Service Configuration
GRPC_LISTEN_ADDRESS=0.0.0.0:50051
SERVICE_NAME=bingo-grpc-api
RUST_LOG=info

# Redis Configuration
REDIS_URL=redis://redis:6379
REDIS_PASSWORD=your-secure-password

# Monitoring
MONITORING_ENABLED=true
JAEGER_ENDPOINT=http://jaeger:14268/api/traces

# Security
TLS_ENABLED=true
TLS_CERT_PATH=/etc/ssl/certs/server.crt
TLS_KEY_PATH=/etc/ssl/private/server.key
```

## Kubernetes Deployment

### Quick Start

1. **Prepare the cluster:**
   ```bash
   # Install cert-manager
   kubectl apply -f https://github.com/cert-manager/cert-manager/releases/download/v1.13.0/cert-manager.yaml
   
   # Install Prometheus Operator
   helm repo add prometheus-community https://prometheus-community.github.io/helm-charts
   helm install prometheus prometheus-community/kube-prometheus-stack
   ```

2. **Configure deployment:**
   ```bash
   # Edit k8s/configmap.yaml with your settings
   # Edit k8s/secrets.yaml with your secrets (base64 encoded)
   # Edit k8s/ingress.yaml with your domain
   ```

3. **Deploy using the script:**
   ```bash
   ./k8s/deploy.sh deploy
   ```

4. **Verify deployment:**
   ```bash
   kubectl get pods -n bingo
   kubectl logs -f deployment/bingo-grpc -n bingo
   ```

### Manual Deployment

If you prefer manual control:

```bash
# Apply manifests in order
kubectl apply -f k8s/namespace.yaml
kubectl apply -f k8s/rbac.yaml
kubectl apply -f k8s/secrets.yaml
kubectl apply -f k8s/configmap.yaml
kubectl apply -f k8s/deployment.yaml
kubectl apply -f k8s/service.yaml
kubectl apply -f k8s/hpa.yaml
kubectl apply -f k8s/ingress.yaml
kubectl apply -f k8s/servicemonitor.yaml

# Wait for deployment
kubectl wait --for=condition=available --timeout=300s deployment/bingo-grpc -n bingo
```

### Kubernetes Components

#### Core Components

- **Namespace**: Isolated environment with resource quotas
- **Deployment**: 3+ replicas with rolling updates
- **Service**: ClusterIP, headless, and LoadBalancer services
- **ConfigMap**: Centralized configuration management
- **Secrets**: Encrypted storage for sensitive data

#### Scaling and Performance

- **HPA**: Auto-scaling based on CPU, memory, and custom metrics
- **VPA**: Automatic resource request/limit optimization
- **PodDisruptionBudget**: Ensure availability during updates
- **Resource Quotas**: Prevent resource exhaustion

#### Security

- **RBAC**: Minimal required permissions
- **ServiceAccount**: Dedicated service identity
- **NetworkPolicies**: Traffic segmentation
- **PodSecurityStandards**: Security baseline enforcement

#### Monitoring

- **ServiceMonitor**: Prometheus metrics collection
- **PrometheusRule**: Comprehensive alerting rules
- **Grafana Dashboards**: Performance visualization
- **Jaeger Integration**: Distributed tracing

## Security Hardening

### Network Security

1. **Enable TLS everywhere:**
   ```yaml
   # In configmap.yaml
   TLS_ENABLED: "true"
   TLS_MUTUAL_AUTH: "true"
   ```

2. **Configure network policies:**
   ```bash
   kubectl apply -f k8s/network-policies.yaml
   ```

3. **Use service mesh (optional):**
   ```bash
   # Install Istio for advanced traffic management
   istioctl install --set values.defaultRevision=default
   ```

### Pod Security

1. **Run as non-root user:**
   ```yaml
   securityContext:
     runAsNonRoot: true
     runAsUser: 1000
     runAsGroup: 1000
   ```

2. **Restrict capabilities:**
   ```yaml
   securityContext:
     allowPrivilegeEscalation: false
     capabilities:
       drop:
       - ALL
   ```

3. **Use read-only root filesystem:**
   ```yaml
   securityContext:
     readOnlyRootFilesystem: true
   ```

### Secrets Management

1. **Use external secrets (recommended):**
   ```bash
   # Install External Secrets Operator
   helm repo add external-secrets https://charts.external-secrets.io
   helm install external-secrets external-secrets/external-secrets -n external-secrets-system --create-namespace
   ```

2. **Configure secret store:**
   ```yaml
   apiVersion: external-secrets.io/v1beta1
   kind: SecretStore
   metadata:
     name: vault-backend
     namespace: bingo
   spec:
     provider:
       vault:
         server: "https://vault.example.com"
         path: "secret"
         version: "v2"
   ```

## Monitoring and Observability

### Metrics Collection

Key metrics to monitor:

- **Application Metrics**: Rules processed, violations, response times
- **gRPC Metrics**: Request rates, error rates, latency percentiles
- **Resource Metrics**: CPU, memory, network, disk usage
- **Business Metrics**: Rule execution patterns, compliance rates

### Alerting Rules

Critical alerts configured:

- Service availability (99.9% SLA)
- High error rates (>5%)
- High latency (>1s p95)
- Resource exhaustion (>90% utilization)
- Business rule violations (>10% rate)

### Dashboards

Pre-configured Grafana dashboards:

- Service overview and health
- Performance and latency metrics
- Resource utilization trends
- Business metrics and KPIs

### Log Management

1. **Structured logging:**
   ```bash
   # Configure JSON logging
   export RUST_LOG=bingo=info,warn
   export LOG_FORMAT=json
   ```

2. **Log aggregation:**
   ```bash
   # Deploy ELK stack or similar
   helm install elasticsearch elastic/elasticsearch
   helm install kibana elastic/kibana
   helm install filebeat elastic/filebeat
   ```

## Performance Tuning

### Resource Allocation

**Recommended starting resources:**

```yaml
resources:
  requests:
    cpu: 500m
    memory: 1Gi
  limits:
    cpu: 2
    memory: 4Gi
```

**Scale based on load:**

- Light load: 1 CPU, 2GB RAM
- Medium load: 2 CPU, 4GB RAM
- Heavy load: 4+ CPU, 8+ GB RAM

### Scaling Configuration

**Horizontal Pod Autoscaler:**

```yaml
minReplicas: 3
maxReplicas: 20
targetCPUUtilizationPercentage: 70
targetMemoryUtilizationPercentage: 80
```

**Vertical Pod Autoscaler:**

```yaml
updateMode: "Auto"
minAllowed:
  cpu: 250m
  memory: 512Mi
maxAllowed:
  cpu: 4
  memory: 8Gi
```

### Performance Optimization

1. **Enable caching:**
   ```yaml
   CACHE_ENABLED: "true"
   CACHE_TTL: "300"
   REDIS_URL: "redis://redis:6379"
   ```

2. **Optimize gRPC settings:**
   ```yaml
   GRPC_MAX_CONCURRENT_STREAMS: "1000"
   GRPC_KEEPALIVE_TIME: "30"
   GRPC_KEEPALIVE_TIMEOUT: "5"
   ```

3. **Tune garbage collection:**
   ```bash
   # For Rust applications
   export MALLOC_ARENA_MAX=2
   export MALLOC_MMAP_THRESHOLD=131072
   ```

## Disaster Recovery

### Backup Strategy

1. **Configuration backup:**
   ```bash
   # Backup ConfigMaps and Secrets
   kubectl get configmaps,secrets -n bingo -o yaml > backup-config.yaml
   ```

2. **Application state backup:**
   ```bash
   # If using persistent storage
   kubectl exec -n bingo deployment/bingo-grpc -- tar czf - /data | gzip > backup-data.tar.gz
   ```

### Recovery Procedures

1. **Service restoration:**
   ```bash
   # Restore from backup
   kubectl apply -f backup-config.yaml
   ./k8s/deploy.sh deploy
   ```

2. **Database recovery:**
   ```bash
   # Restore data if applicable
   kubectl exec -n bingo deployment/bingo-grpc -- tar xzf - /data < backup-data.tar.gz
   ```

### High Availability

1. **Multi-zone deployment:**
   ```yaml
   affinity:
     podAntiAffinity:
       preferredDuringSchedulingIgnoredDuringExecution:
       - weight: 100
         podAffinityTerm:
           labelSelector:
             matchExpressions:
             - key: app
               operator: In
               values:
               - bingo-grpc
           topologyKey: topology.kubernetes.io/zone
   ```

2. **Circuit breaker pattern:**
   ```yaml
   # Configure retries and timeouts
   CIRCUIT_BREAKER_ENABLED: "true"
   RETRY_ATTEMPTS: "3"
   TIMEOUT_SECONDS: "30"
   ```

## Troubleshooting

### Common Issues

1. **Service not starting:**
   ```bash
   # Check logs
   kubectl logs -f deployment/bingo-grpc -n bingo
   
   # Check events
   kubectl get events --sort-by=.metadata.creationTimestamp -n bingo
   
   # Check resource constraints
   kubectl describe pods -l app=bingo-grpc -n bingo
   ```

2. **High memory usage:**
   ```bash
   # Check memory metrics
   kubectl top pods -n bingo
   
   # Analyze memory patterns
   kubectl exec -n bingo deployment/bingo-grpc -- ps aux
   ```

3. **Connection issues:**
   ```bash
   # Test connectivity
   kubectl port-forward svc/bingo-grpc-service 50051:50051 -n bingo
   grpc_health_probe -addr=localhost:50051
   
   # Check service endpoints
   kubectl get endpoints -n bingo
   ```

### Performance Issues

1. **High latency:**
   ```bash
   # Check resource utilization
   kubectl top pods -n bingo
   
   # Analyze request patterns
   kubectl logs -f deployment/bingo-grpc -n bingo | grep "slow_request"
   ```

2. **Memory leaks:**
   ```bash
   # Monitor memory over time
   kubectl exec -n bingo deployment/bingo-grpc -- cat /proc/meminfo
   
   # Check for growing processes
   kubectl exec -n bingo deployment/bingo-grpc -- ps -eo pid,ppid,cmd,%mem,%cpu --sort=-%mem
   ```

### Monitoring and Alerts

1. **Check Prometheus targets:**
   ```bash
   kubectl port-forward svc/prometheus-operated 9090:9090
   # Browse to http://localhost:9090/targets
   ```

2. **View Grafana dashboards:**
   ```bash
   kubectl port-forward svc/grafana 3000:3000
   # Browse to http://localhost:3000
   ```

3. **Check alert status:**
   ```bash
   # View active alerts
   kubectl get prometheusrules -n bingo
   ```

For additional support, refer to:
- [gRPC Deployment Guide](./grpc-deployment-guide.md) for detailed gRPC configuration
- [Performance Testing Documentation](../tests/performance/README.md) for load testing
- [Architecture Documentation](../specs/architecture.md) for system design details