#!/bin/bash

# Doris Profile Analyzer - Build Script
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

echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}Doris Profile Analyzer - Build Script${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""

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

# Check if required tools are installed
check_dependencies() {
    print_info "Checking dependencies..."
    
    if ! command -v node &> /dev/null; then
        print_error "Node.js is not installed"
        exit 1
    fi
    
    if ! command -v npm &> /dev/null; then
        print_error "npm is not installed"
        exit 1
    fi
    
    if ! command -v cargo &> /dev/null; then
        print_error "Rust/Cargo is not installed"
        exit 1
    fi
    
    print_status "All dependencies are installed"
}

# Build frontend
build_frontend() {
    print_info "Building frontend..."
    cd "${PROJECT_ROOT}/frontend"
    
    # Install dependencies if node_modules doesn't exist
    if [ ! -d "node_modules" ]; then
        print_info "Installing frontend dependencies..."
        npm install
    fi
    
    # Build frontend
    npm run build
    
    if [ $? -eq 0 ]; then
        print_status "Frontend build completed"
    else
        print_error "Frontend build failed"
        exit 1
    fi
}

# Build backend (dev)
build_backend_dev() {
    print_info "Building backend (debug mode)..."
    cd "${PROJECT_ROOT}"
    
    cargo build
    
    if [ $? -eq 0 ]; then
        print_status "Backend debug build completed"
    else
        print_error "Backend debug build failed"
        exit 1
    fi
}

# Build backend (release)
build_backend_release() {
    print_info "Building backend (release mode)..."
    cd "${PROJECT_ROOT}"
    
    cargo build --release
    
    if [ $? -eq 0 ]; then
        print_status "Backend release build completed"
    else
        print_error "Backend release build failed"
        exit 1
    fi
}

# Main build process
main() {
    check_dependencies
    
    echo ""
    print_info "Starting build process..."
    echo ""
    
    # Build frontend
    build_frontend
    
    echo ""
    
    # Check if --release flag is passed
    if [[ "$1" == "--release" ]]; then
        build_backend_release
        BINARY_PATH="${PROJECT_ROOT}/target/release/doris-profile-analyzer"
    else
        build_backend_dev
        BINARY_PATH="${PROJECT_ROOT}/target/debug/doris-profile-analyzer"
    fi
    
    echo ""
    echo -e "${GREEN}========================================${NC}"
    echo -e "${GREEN}Build completed successfully!${NC}"
    echo -e "${GREEN}========================================${NC}"
    echo ""
    print_info "Binary location: ${BINARY_PATH}"
    
    if [[ "$1" == "--release" ]]; then
        print_info "To start the service: ./bin/start.sh"
    else
        print_info "To start the service: ./bin/start.sh --debug"
    fi
}

# Run main function
main "$@"

