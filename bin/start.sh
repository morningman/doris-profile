#!/bin/bash

# Doris Profile Analyzer - Start Script
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
LOG_FILE="${PROJECT_ROOT}/logs/doris-profile-analyzer.log"

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

# Check if service is already running
check_running() {
    if [ -f "${PID_FILE}" ]; then
        PID=$(cat "${PID_FILE}")
        if ps -p "${PID}" > /dev/null 2>&1; then
            print_error "Service is already running (PID: ${PID})"
            print_info "Use './bin/stop.sh' to stop the service first"
            exit 1
        else
            print_info "Removing stale PID file"
            rm -f "${PID_FILE}"
        fi
    fi
}

# Create logs directory
setup_logs() {
    mkdir -p "${PROJECT_ROOT}/logs"
}

# Start the service
start_service() {
    local BINARY_PATH
    
    # Check if --debug flag is passed
    if [[ "$1" == "--debug" ]]; then
        BINARY_PATH="${PROJECT_ROOT}/target/debug/doris-profile-analyzer"
        print_info "Starting service in DEBUG mode..."
    else
        BINARY_PATH="${PROJECT_ROOT}/target/release/doris-profile-analyzer"
        print_info "Starting service in RELEASE mode..."
    fi
    
    # Check if binary exists
    if [ ! -f "${BINARY_PATH}" ]; then
        print_error "Binary not found: ${BINARY_PATH}"
        print_info "Please run './bin/build.sh' first"
        if [[ "$1" == "--debug" ]]; then
            print_info "Or run './bin/build.sh' (without --release) for debug build"
        else
            print_info "Or run './bin/build.sh --release' for release build"
        fi
        exit 1
    fi
    
    # Start the service in background
    nohup "${BINARY_PATH}" > "${LOG_FILE}" 2>&1 &
    PID=$!
    
    # Save PID to file
    echo "${PID}" > "${PID_FILE}"
    
    # Wait a moment and check if process is still running
    sleep 2
    
    if ps -p "${PID}" > /dev/null 2>&1; then
        print_status "Service started successfully"
        print_info "PID: ${PID}"
        print_info "PID file: ${PID_FILE}"
        print_info "Log file: ${LOG_FILE}"
        echo ""
        print_info "To check logs: tail -f ${LOG_FILE}"
        print_info "To stop service: ./bin/stop.sh"
        echo ""
        
        # Wait a bit more and show initial logs
        sleep 1
        print_info "Initial logs:"
        echo ""
        tail -10 "${LOG_FILE}"
    else
        print_error "Service failed to start"
        print_info "Check logs: ${LOG_FILE}"
        rm -f "${PID_FILE}"
        exit 1
    fi
}

# Main function
main() {
    echo -e "${BLUE}========================================${NC}"
    echo -e "${BLUE}Doris Profile Analyzer - Start Service${NC}"
    echo -e "${BLUE}========================================${NC}"
    echo ""
    
    check_running
    setup_logs
    start_service "$@"
}

# Run main function
main "$@"

