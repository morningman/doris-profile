#!/bin/bash

# Doris Profile Analyzer - Stop Script
# Compatible with macOS and Linux

set -e  # Exit on error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Get script directory and project root
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
PID_FILE="${SCRIPT_DIR}/doris-profile-analyzer.pid"

# Function to print status messages
print_status() {
    echo -e "${GREEN}[✓]${NC} $1"
}

print_error() {
    echo -e "${RED}[✗]${NC} $1"
}

print_info() {
    echo -e "${YELLOW}[i]${NC} $1"
}

# Stop the service
stop_service() {
    if [ ! -f "${PID_FILE}" ]; then
        print_error "PID file not found: ${PID_FILE}"
        print_info "Service may not be running"
        
        # Try to find and kill any running process
        print_info "Searching for running process..."
        PIDS=$(pgrep -f "doris-profile-analyzer" || true)
        
        if [ -n "${PIDS}" ]; then
            print_info "Found process(es): ${PIDS}"
            print_info "Killing process(es)..."
            pkill -f "doris-profile-analyzer"
            sleep 1
            print_status "Process(es) terminated"
        else
            print_info "No running process found"
        fi
        
        exit 0
    fi
    
    PID=$(cat "${PID_FILE}")
    
    # Check if process is running
    if ! ps -p "${PID}" > /dev/null 2>&1; then
        print_error "Process (PID: ${PID}) is not running"
        print_info "Removing stale PID file"
        rm -f "${PID_FILE}"
        exit 0
    fi
    
    print_info "Stopping service (PID: ${PID})..."
    
    # Try graceful shutdown first (SIGTERM)
    kill "${PID}" 2>/dev/null || true
    
    # Wait for process to terminate (max 10 seconds)
    WAIT_COUNT=0
    while ps -p "${PID}" > /dev/null 2>&1; do
        if [ ${WAIT_COUNT} -ge 10 ]; then
            print_info "Graceful shutdown timeout, forcing termination..."
            kill -9 "${PID}" 2>/dev/null || true
            sleep 1
            break
        fi
        sleep 1
        WAIT_COUNT=$((WAIT_COUNT + 1))
    done
    
    # Verify process is stopped
    if ps -p "${PID}" > /dev/null 2>&1; then
        print_error "Failed to stop process (PID: ${PID})"
        exit 1
    else
        print_status "Service stopped successfully"
        rm -f "${PID_FILE}"
    fi
}

# Main function
main() {
    echo -e "${BLUE}========================================${NC}"
    echo -e "${BLUE}Doris Profile Analyzer - Stop Service${NC}"
    echo -e "${BLUE}========================================${NC}"
    echo ""
    
    stop_service
    
    echo ""
    print_info "To start service again: ./bin/start.sh"
}

# Run main function
main "$@"

