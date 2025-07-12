#!/bin/bash

# Bingo RETE Rules Engine - Kubernetes Deployment Script
# This script automates the deployment of Bingo gRPC service to Kubernetes

set -euo pipefail

# Configuration
NAMESPACE="${NAMESPACE:-bingo}"
CONTEXT="${KUBE_CONTEXT:-}"
DRY_RUN="${DRY_RUN:-false}"
SKIP_BUILD="${SKIP_BUILD:-false}"
IMAGE_TAG="${IMAGE_TAG:-latest}"
REGISTRY="${REGISTRY:-}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check prerequisites
check_prerequisites() {
    log_info "Checking prerequisites..."
    
    if ! command -v kubectl &> /dev/null; then
        log_error "kubectl is not installed or not in PATH"
        exit 1
    fi
    
    if ! command -v docker &> /dev/null && [[ "$SKIP_BUILD" != "true" ]]; then
        log_error "docker is not installed or not in PATH"
        exit 1
    fi
    
    if [[ -n "$CONTEXT" ]]; then
        if ! kubectl config get-contexts "$CONTEXT" &> /dev/null; then
            log_error "Kubernetes context '$CONTEXT' not found"
            exit 1
        fi
        kubectl config use-context "$CONTEXT"
        log_info "Using Kubernetes context: $CONTEXT"
    fi
    
    log_success "Prerequisites check passed"
}

# Build and push Docker image
build_and_push() {
    if [[ "$SKIP_BUILD" == "true" ]]; then
        log_info "Skipping Docker build (SKIP_BUILD=true)"
        return
    fi
    
    log_info "Building Docker image..."
    
    local image_name="bingo-grpc:$IMAGE_TAG"
    if [[ -n "$REGISTRY" ]]; then
        image_name="$REGISTRY/bingo-grpc:$IMAGE_TAG"
    fi
    
    docker build -t "$image_name" .
    
    if [[ -n "$REGISTRY" ]]; then
        log_info "Pushing image to registry..."
        docker push "$image_name"
        log_success "Image pushed: $image_name"
    else
        log_success "Image built: $image_name"
    fi
}

# Apply Kubernetes manifests
apply_manifests() {
    log_info "Applying Kubernetes manifests..."
    
    local dry_run_flag=""
    if [[ "$DRY_RUN" == "true" ]]; then
        dry_run_flag="--dry-run=client"
        log_warn "Running in dry-run mode"
    fi
    
    # Apply in order for dependencies
    local manifests=(
        "namespace.yaml"
        "rbac.yaml"
        "secrets.yaml"
        "configmap.yaml"
        "deployment.yaml"
        "service.yaml"
        "hpa.yaml"
        "ingress.yaml"
        "servicemonitor.yaml"
    )
    
    for manifest in "${manifests[@]}"; do
        if [[ -f "k8s/$manifest" ]]; then
            log_info "Applying $manifest..."
            kubectl apply -f "k8s/$manifest" $dry_run_flag
        else
            log_warn "Manifest $manifest not found, skipping"
        fi
    done
    
    log_success "All manifests applied"
}

# Wait for deployment to be ready
wait_for_deployment() {
    if [[ "$DRY_RUN" == "true" ]]; then
        log_info "Skipping deployment wait (dry-run mode)"
        return
    fi
    
    log_info "Waiting for deployment to be ready..."
    
    if ! kubectl wait --for=condition=available --timeout=300s deployment/bingo-grpc -n "$NAMESPACE"; then
        log_error "Deployment failed to become ready within 5 minutes"
        log_info "Recent events:"
        kubectl get events --sort-by=.metadata.creationTimestamp -n "$NAMESPACE" | tail -10
        exit 1
    fi
    
    log_success "Deployment is ready"
}

# Verify deployment
verify_deployment() {
    if [[ "$DRY_RUN" == "true" ]]; then
        log_info "Skipping deployment verification (dry-run mode)"
        return
    fi
    
    log_info "Verifying deployment..."
    
    # Check pod status
    local ready_pods
    ready_pods=$(kubectl get pods -l app=bingo-grpc -n "$NAMESPACE" -o jsonpath='{.items[?(@.status.phase=="Running")].metadata.name}' | wc -w)
    log_info "Ready pods: $ready_pods"
    
    # Check service endpoints
    if kubectl get endpoints bingo-grpc-service -n "$NAMESPACE" &> /dev/null; then
        local endpoints
        endpoints=$(kubectl get endpoints bingo-grpc-service -n "$NAMESPACE" -o jsonpath='{.subsets[0].addresses[*].ip}' | wc -w)
        log_info "Service endpoints: $endpoints"
    fi
    
    # Run health check if verify script exists
    if [[ -f "scripts/verify-deployment.sh" ]]; then
        log_info "Running health checks..."
        if bash scripts/verify-deployment.sh; then
            log_success "Health checks passed"
        else
            log_warn "Health checks failed - service may need time to start"
        fi
    fi
    
    log_success "Deployment verification completed"
}

# Display deployment information
show_deployment_info() {
    if [[ "$DRY_RUN" == "true" ]]; then
        return
    fi
    
    echo
    log_info "=== Deployment Information ==="
    
    echo
    echo "Pods:"
    kubectl get pods -l app=bingo-grpc -n "$NAMESPACE" -o wide
    
    echo
    echo "Services:"
    kubectl get services -l app=bingo-grpc -n "$NAMESPACE"
    
    echo
    echo "Ingress:"
    kubectl get ingress -l app=bingo-grpc -n "$NAMESPACE"
    
    echo
    echo "HPA:"
    kubectl get hpa -l app=bingo-grpc -n "$NAMESPACE"
    
    echo
    log_info "To monitor the deployment:"
    echo "  kubectl logs -f deployment/bingo-grpc -n $NAMESPACE"
    echo "  kubectl get pods -l app=bingo-grpc -n $NAMESPACE -w"
    
    echo
    log_info "To check service health:"
    echo "  kubectl port-forward svc/bingo-grpc-service 50051:50051 -n $NAMESPACE"
    echo "  grpc_health_probe -addr=localhost:50051"
}

# Cleanup function
cleanup() {
    log_info "Cleaning up..."
    
    local manifests=(
        "servicemonitor.yaml"
        "ingress.yaml"
        "hpa.yaml"
        "service.yaml"
        "deployment.yaml"
        "configmap.yaml"
        "secrets.yaml"
        "rbac.yaml"
        "namespace.yaml"
    )
    
    for manifest in "${manifests[@]}"; do
        if [[ -f "k8s/$manifest" ]]; then
            log_info "Deleting $manifest..."
            kubectl delete -f "k8s/$manifest" --ignore-not-found=true
        fi
    done
    
    log_success "Cleanup completed"
}

# Main function
main() {
    echo "Bingo RETE Rules Engine - Kubernetes Deployment"
    echo "=============================================="
    echo
    
    case "${1:-deploy}" in
        "deploy")
            check_prerequisites
            build_and_push
            apply_manifests
            wait_for_deployment
            verify_deployment
            show_deployment_info
            ;;
        "cleanup"|"clean")
            cleanup
            ;;
        "verify")
            verify_deployment
            ;;
        "info")
            show_deployment_info
            ;;
        *)
            echo "Usage: $0 [deploy|cleanup|verify|info]"
            echo
            echo "Commands:"
            echo "  deploy   - Deploy the application (default)"
            echo "  cleanup  - Remove the deployment"
            echo "  verify   - Verify existing deployment"
            echo "  info     - Show deployment information"
            echo
            echo "Environment variables:"
            echo "  NAMESPACE    - Kubernetes namespace (default: bingo)"
            echo "  KUBE_CONTEXT - Kubernetes context to use"
            echo "  DRY_RUN      - Run in dry-run mode (default: false)"
            echo "  SKIP_BUILD   - Skip Docker build step (default: false)"
            echo "  IMAGE_TAG    - Docker image tag (default: latest)"
            echo "  REGISTRY     - Docker registry to push to"
            exit 1
            ;;
    esac
}

# Run main function with all arguments
main "$@"