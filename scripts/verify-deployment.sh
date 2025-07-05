#!/bin/bash
set -euo pipefail

# Bingo gRPC Deployment Verification Script

GRPC_HOST="${GRPC_HOST:-localhost}"
GRPC_PORT="${GRPC_PORT:-50051}"
TIMEOUT="${TIMEOUT:-10}"

echo "🔍 Verifying Bingo gRPC deployment"
echo "   Host: $GRPC_HOST"
echo "   Port: $GRPC_PORT"
echo "   Timeout: ${TIMEOUT}s"
echo ""

# Function to check if grpcurl is available
check_grpcurl() {
    if ! command -v grpcurl &> /dev/null; then
        echo "❌ grpcurl is not installed. Please install it:"
        echo "   macOS: brew install grpcurl"
        echo "   Linux: Download from https://github.com/fullstorydev/grpcurl/releases"
        return 1
    fi
}

# Function to test gRPC connectivity
test_connectivity() {
    echo "🔌 Testing gRPC connectivity..."
    if timeout $TIMEOUT bash -c "</dev/tcp/$GRPC_HOST/$GRPC_PORT"; then
        echo "✅ TCP connection successful"
        return 0
    else
        echo "❌ Cannot connect to $GRPC_HOST:$GRPC_PORT"
        return 1
    fi
}

# Function to list gRPC services
list_services() {
    echo "📋 Listing available gRPC services..."
    if grpcurl -plaintext -connect-timeout ${TIMEOUT}s "$GRPC_HOST:$GRPC_PORT" list 2>/dev/null; then
        echo "✅ gRPC service discovery successful"
        return 0
    else
        echo "❌ Failed to list gRPC services"
        return 1
    fi
}

# Function to test health check
test_health_check() {
    echo "🏥 Testing health check..."
    local response
    if response=$(grpcurl -plaintext -connect-timeout ${TIMEOUT}s -d '{}' \
        "$GRPC_HOST:$GRPC_PORT" \
        rules_engine.v1.RulesEngineService/HealthCheck 2>/dev/null); then
        echo "✅ Health check successful"
        echo "   Response: $response"
        return 0
    else
        echo "❌ Health check failed"
        return 1
    fi
}

# Function to test rule compilation
test_rule_compilation() {
    echo "⚙️  Testing rule compilation..."
    local payload='{
        "session_id": "test-verification",
        "rules": [
            {
                "id": "1",
                "name": "Test Rule",
                "description": "Verification test rule",
                "conditions": [
                    {
                        "simple": {
                            "field": "status",
                            "operator": 0,
                            "value": {"string_value": "active"}
                        }
                    }
                ],
                "actions": [
                    {
                        "create_fact": {
                            "fields": {
                                "message": {"string_value": "Test rule fired!"}
                            }
                        }
                    }
                ]
            }
        ]
    }'
    
    local response
    if response=$(grpcurl -plaintext -connect-timeout ${TIMEOUT}s -d "$payload" \
        "$GRPC_HOST:$GRPC_PORT" \
        rules_engine.v1.RulesEngineService/CompileRules 2>/dev/null); then
        echo "✅ Rule compilation successful"
        
        # Extract key metrics from response
        local rules_compiled=$(echo "$response" | grep -o '"rules_compiled":[0-9]*' | cut -d':' -f2)
        local network_nodes=$(echo "$response" | grep -o '"network_nodes_created":[0-9]*' | cut -d':' -f2)
        local compilation_time=$(echo "$response" | grep -o '"compilation_time_ms":[0-9]*' | cut -d':' -f2)
        
        echo "   Rules compiled: ${rules_compiled:-N/A}"
        echo "   Network nodes: ${network_nodes:-N/A}"
        echo "   Compilation time: ${compilation_time:-N/A}ms"
        return 0
    else
        echo "❌ Rule compilation failed"
        return 1
    fi
}

# Function to run performance test
test_performance() {
    echo "⚡ Running basic performance test..."
    local start_time=$(date +%s%3N)
    local requests=10
    local successful=0
    
    for i in $(seq 1 $requests); do
        if grpcurl -plaintext -connect-timeout 5s -d '{}' \
            "$GRPC_HOST:$GRPC_PORT" \
            rules_engine.v1.RulesEngineService/HealthCheck >/dev/null 2>&1; then
            ((successful++))
        fi
    done
    
    local end_time=$(date +%s%3N)
    local total_time=$((end_time - start_time))
    local avg_time=$((total_time / requests))
    local success_rate=$((successful * 100 / requests))
    
    echo "   Requests: $requests"
    echo "   Successful: $successful"
    echo "   Success rate: ${success_rate}%"
    echo "   Average response time: ${avg_time}ms"
    
    if [ $success_rate -ge 90 ]; then
        echo "✅ Performance test passed"
        return 0
    else
        echo "❌ Performance test failed (success rate < 90%)"
        return 1
    fi
}

# Main verification flow
main() {
    local exit_code=0
    
    # Check prerequisites
    if ! check_grpcurl; then
        exit 1
    fi
    
    # Run tests
    if ! test_connectivity; then
        ((exit_code++))
    fi
    
    if ! list_services; then
        ((exit_code++))
    fi
    
    if ! test_health_check; then
        ((exit_code++))
    fi
    
    if ! test_rule_compilation; then
        ((exit_code++))
    fi
    
    if ! test_performance; then
        ((exit_code++))
    fi
    
    echo ""
    if [ $exit_code -eq 0 ]; then
        echo "🎉 All verification tests passed!"
        echo "   Bingo gRPC deployment is healthy and functional"
    else
        echo "💥 $exit_code test(s) failed"
        echo "   Please check the deployment and try again"
    fi
    
    return $exit_code
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -h|--host)
            GRPC_HOST="$2"
            shift 2
            ;;
        -p|--port)
            GRPC_PORT="$2"
            shift 2
            ;;
        -t|--timeout)
            TIMEOUT="$2"
            shift 2
            ;;
        --help)
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  -h, --host HOST     gRPC host (default: localhost)"
            echo "  -p, --port PORT     gRPC port (default: 50051)"
            echo "  -t, --timeout SEC   Connection timeout (default: 10)"
            echo "  --help              Show this help message"
            echo ""
            echo "Environment variables:"
            echo "  GRPC_HOST, GRPC_PORT, TIMEOUT"
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            echo "Use --help for usage information"
            exit 1
            ;;
    esac
done

main "$@"