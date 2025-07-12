#!/bin/bash
set -euo pipefail

# Bingo RETE Rules Engine Production Startup Script
# This script handles production startup with proper configuration validation

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Default configuration
SERVICE_NAME="${SERVICE_NAME:-bingo-grpc}"
SERVICE_VERSION="${SERVICE_VERSION:-1.0.0}"
BINGO_ENVIRONMENT="${BINGO_ENVIRONMENT:-production}"
GRPC_LISTEN_ADDRESS="${GRPC_LISTEN_ADDRESS:-0.0.0.0:50051}"
RUST_LOG="${RUST_LOG:-info}"

# Production settings
TLS_ENABLED="${TLS_ENABLED:-true}"
AUTH_REQUIRED="${AUTH_REQUIRED:-true}"
METRICS_ENABLED="${METRICS_ENABLED:-true}"
RATE_LIMIT_RPM="${RATE_LIMIT_RPM:-10000}"

# Resource limits
MAX_CONNECTIONS="${MAX_CONNECTIONS:-1000}"
REQUEST_TIMEOUT_MS="${REQUEST_TIMEOUT_MS:-30000}"
MAX_MEMORY_MB="${MAX_MEMORY_MB:-4096}"

# Paths
BIN_PATH="${BIN_PATH:-$PROJECT_ROOT/target/release/bingo}"
CONFIG_PATH="${CONFIG_PATH:-$PROJECT_ROOT/config}"
LOG_PATH="${LOG_PATH:-/var/log/bingo}"
PID_PATH="${PID_PATH:-/var/run/bingo}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1" >&2
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1" >&2
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1" >&2
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1" >&2
}

# Check if running as root for production deployment
check_user() {
    if [[ $EUID -eq 0 ]] && [[ "${ALLOW_ROOT:-false}" != "true" ]]; then
        log_error "Do not run as root in production. Create a dedicated service user."
        log_info "Set ALLOW_ROOT=true to override this check (not recommended)"
        exit 1
    fi
}

# Validate required environment
validate_environment() {
    log_info "Validating production environment..."

    # Check binary exists
    if [[ ! -f "$BIN_PATH" ]]; then
        log_error "Binary not found at: $BIN_PATH"
        log_info "Build the project with: cargo build --release"
        exit 1
    fi

    # Check binary is recent (less than 24 hours old)
    if [[ -n "$(find "$BIN_PATH" -mtime +1 2>/dev/null)" ]]; then
        log_warn "Binary is more than 24 hours old. Consider rebuilding."
    fi

    # Create required directories
    mkdir -p "$LOG_PATH" "$PID_PATH" 2>/dev/null || true

    # Check disk space (require at least 1GB free)
    AVAILABLE_KB=$(df "$PWD" | awk 'NR==2 {print $4}')
    AVAILABLE_MB=$((AVAILABLE_KB / 1024))
    if [[ $AVAILABLE_MB -lt 1024 ]]; then
        log_error "Insufficient disk space: ${AVAILABLE_MB}MB available, 1GB required"
        exit 1
    fi

    # Check memory
    TOTAL_MEM_KB=$(grep MemTotal /proc/meminfo | awk '{print $2}')
    TOTAL_MEM_MB=$((TOTAL_MEM_KB / 1024))
    if [[ $TOTAL_MEM_MB -lt $MAX_MEMORY_MB ]]; then
        log_warn "System memory (${TOTAL_MEM_MB}MB) is less than configured limit (${MAX_MEMORY_MB}MB)"
    fi

    log_success "Environment validation completed"
}

# Run production readiness check
run_readiness_check() {
    log_info "Running production readiness validation..."

    # Export environment variables for validation
    export SERVICE_NAME SERVICE_VERSION BINGO_ENVIRONMENT
    export GRPC_LISTEN_ADDRESS RUST_LOG
    export TLS_ENABLED AUTH_REQUIRED METRICS_ENABLED RATE_LIMIT_RPM
    export MAX_CONNECTIONS REQUEST_TIMEOUT_MS

    # Run validation using the CLI tool (if available)
    if command -v "$PROJECT_ROOT/target/release/bingo-production" >/dev/null 2>&1; then
        "$PROJECT_ROOT/target/release/bingo-production" validate --strict --format text
        readiness_exit_code=$?
        
        if [[ $readiness_exit_code -ne 0 ]]; then
            log_error "Production readiness validation failed"
            exit $readiness_exit_code
        fi
    else
        log_warn "Production readiness CLI not found, skipping validation"
        log_info "Build with: cargo build --release --bin bingo-production"
    fi

    log_success "Production readiness validation passed"
}

# Check for already running instances
check_running_instances() {
    local pid_file="$PID_PATH/$SERVICE_NAME.pid"
    
    if [[ -f "$pid_file" ]]; then
        local existing_pid=$(cat "$pid_file" 2>/dev/null || echo "")
        if [[ -n "$existing_pid" ]] && kill -0 "$existing_pid" 2>/dev/null; then
            log_error "Service already running with PID: $existing_pid"
            log_info "Stop the service first with: kill $existing_pid"
            exit 1
        else
            log_warn "Stale PID file found, removing"
            rm -f "$pid_file"
        fi
    fi
}

# Setup signal handlers for graceful shutdown
setup_signal_handlers() {
    local pid_file="$PID_PATH/$SERVICE_NAME.pid"
    
    # Function to handle shutdown
    cleanup() {
        log_info "Received shutdown signal, cleaning up..."
        if [[ -f "$pid_file" ]]; then
            local service_pid=$(cat "$pid_file" 2>/dev/null || echo "")
            if [[ -n "$service_pid" ]] && kill -0 "$service_pid" 2>/dev/null; then
                log_info "Sending SIGTERM to service (PID: $service_pid)"
                kill -TERM "$service_pid"
                
                # Wait for graceful shutdown
                local wait_time=0
                while kill -0 "$service_pid" 2>/dev/null && [[ $wait_time -lt 30 ]]; do
                    sleep 1
                    ((wait_time++))
                done
                
                # Force kill if still running
                if kill -0 "$service_pid" 2>/dev/null; then
                    log_warn "Forcing shutdown with SIGKILL"
                    kill -KILL "$service_pid"
                fi
            fi
            rm -f "$pid_file"
        fi
        exit 0
    }
    
    trap cleanup SIGTERM SIGINT
}

# Start the service
start_service() {
    local pid_file="$PID_PATH/$SERVICE_NAME.pid"
    local log_file="$LOG_PATH/$SERVICE_NAME.log"
    
    log_info "Starting Bingo RETE Rules Engine..."
    log_info "Service: $SERVICE_NAME v$SERVICE_VERSION"
    log_info "Environment: $BINGO_ENVIRONMENT"
    log_info "Listen address: $GRPC_LISTEN_ADDRESS"
    log_info "Log file: $log_file"
    log_info "PID file: $pid_file"

    # Start the service in background
    "$BIN_PATH" > "$log_file" 2>&1 &
    local service_pid=$!
    
    # Write PID file
    echo "$service_pid" > "$pid_file"
    
    # Wait a moment to check if service started successfully
    sleep 2
    
    if kill -0 "$service_pid" 2>/dev/null; then
        log_success "Service started successfully (PID: $service_pid)"
        
        # If running in foreground mode, wait for service
        if [[ "${FOREGROUND:-false}" == "true" ]]; then
            log_info "Running in foreground mode. Press Ctrl+C to stop."
            wait "$service_pid"
        else
            log_info "Service running in background"
            log_info "Monitor logs with: tail -f $log_file"
            log_info "Stop service with: kill $service_pid"
        fi
    else
        log_error "Service failed to start"
        log_info "Check logs at: $log_file"
        rm -f "$pid_file"
        exit 1
    fi
}

# Health check after startup
post_startup_health_check() {
    if [[ "${SKIP_HEALTH_CHECK:-false}" == "true" ]]; then
        log_info "Skipping post-startup health check"
        return 0
    fi

    log_info "Running post-startup health check..."
    
    # Wait for service to be ready
    local max_attempts=30
    local attempt=0
    
    while [[ $attempt -lt $max_attempts ]]; do
        if command -v grpc_health_probe >/dev/null 2>&1; then
            if grpc_health_probe -addr="$GRPC_LISTEN_ADDRESS" >/dev/null 2>&1; then
                log_success "Health check passed"
                return 0
            fi
        else
            # Fallback to basic connection check
            if timeout 5 bash -c "</dev/tcp/${GRPC_LISTEN_ADDRESS%:*}/${GRPC_LISTEN_ADDRESS#*:}" 2>/dev/null; then
                log_success "Basic connectivity check passed"
                return 0
            fi
        fi
        
        ((attempt++))
        log_info "Health check attempt $attempt/$max_attempts failed, retrying..."
        sleep 2
    done
    
    log_error "Health check failed after $max_attempts attempts"
    return 1
}

# Display help
show_help() {
    cat << EOF
Bingo RETE Rules Engine Production Startup Script

USAGE:
    $0 [OPTIONS]

OPTIONS:
    --help              Show this help message
    --check-only        Run checks without starting service
    --foreground        Run in foreground mode
    --skip-health       Skip post-startup health check
    --validate-only     Run production readiness validation only

ENVIRONMENT VARIABLES:
    SERVICE_NAME        Service name (default: bingo-grpc)
    SERVICE_VERSION     Service version (default: 1.0.0)
    BINGO_ENVIRONMENT   Environment (default: production)
    GRPC_LISTEN_ADDRESS gRPC listen address (default: 0.0.0.0:50051)
    RUST_LOG           Log level (default: info)
    TLS_ENABLED        Enable TLS (default: true)
    AUTH_REQUIRED      Require authentication (default: true)
    MAX_CONNECTIONS    Max concurrent connections (default: 1000)
    BIN_PATH           Path to binary (default: target/release/bingo)
    ALLOW_ROOT         Allow running as root (default: false)

EXAMPLES:
    # Normal production startup
    $0

    # Run with custom settings
    SERVICE_NAME=bingo-prod MAX_CONNECTIONS=2000 $0

    # Check configuration only
    $0 --check-only

    # Run in foreground for debugging
    $0 --foreground

EOF
}

# Main execution
main() {
    case "${1:-}" in
        --help|-h)
            show_help
            exit 0
            ;;
        --check-only)
            check_user
            validate_environment
            run_readiness_check
            log_success "All checks passed - ready for production"
            exit 0
            ;;
        --validate-only)
            run_readiness_check
            exit 0
            ;;
        --foreground)
            export FOREGROUND=true
            ;;
        --skip-health)
            export SKIP_HEALTH_CHECK=true
            ;;
        "")
            # Normal startup
            ;;
        *)
            log_error "Unknown option: $1"
            show_help
            exit 1
            ;;
    esac

    # Pre-flight checks
    check_user
    validate_environment
    run_readiness_check
    check_running_instances
    
    # Setup and start
    setup_signal_handlers
    start_service
    
    # Post-startup validation
    if [[ "${FOREGROUND:-false}" != "true" ]]; then
        post_startup_health_check || log_warn "Service started but health check failed"
    fi
}

# Execute main function with all arguments
main "$@"