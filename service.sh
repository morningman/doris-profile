#!/bin/bash

# Doris Profile Visualizer Service Manager
# Usage: ./service.sh {start|stop|restart|status|build}

PROJECT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PID_FILE="$PROJECT_DIR/.service.pid"
LOG_FILE="$PROJECT_DIR/service.log"
DEV_PORT=5173
PROD_PORT=4173

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Helper functions
log() {
    echo -e "${BLUE}[$(date '+%Y-%m-%d %H:%M:%S')]${NC} $1"
}

success() {
    echo -e "${GREEN}✅ $1${NC}"
}

error() {
    echo -e "${RED}❌ $1${NC}"
}

warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

# Check if yarn is installed
check_yarn() {
    if ! command -v yarn &> /dev/null; then
        error "Yarn is not installed. Please install yarn first."
        exit 1
    fi
}

# Install dependencies
install_deps() {
    log "Installing dependencies..."
    cd "$PROJECT_DIR"
    yarn install
    if [ $? -eq 0 ]; then
        success "Dependencies installed successfully"
    else
        error "Failed to install dependencies"
        exit 1
    fi
}

# Build the project
build_project() {
    log "Building project..."
    cd "$PROJECT_DIR"
    yarn build
    if [ $? -eq 0 ]; then
        success "Project built successfully"
    else
        error "Failed to build project"
        exit 1
    fi
}

# Start development server
start_dev() {
    log "Starting development server on port $DEV_PORT..."
    cd "$PROJECT_DIR"
    
    # Check if port is already in use
    if lsof -Pi :$DEV_PORT -sTCP:LISTEN -t >/dev/null; then
        warning "Port $DEV_PORT is already in use"
        return 1
    fi
    
    # Start development server in background
    nohup yarn dev > "$LOG_FILE" 2>&1 &
    DEV_PID=$!
    echo $DEV_PID > "$PID_FILE"
    
    # Wait a moment and check if server started successfully
    sleep 3
    if kill -0 $DEV_PID 2>/dev/null; then
        success "Development server started successfully (PID: $DEV_PID)"
        success "Access the application at: http://localhost:$DEV_PORT"
    else
        error "Failed to start development server"
        rm -f "$PID_FILE"
        return 1
    fi
}

# Start production server
start_prod() {
    log "Starting production server on port $PROD_PORT..."
    cd "$PROJECT_DIR"
    
    # Check if dist directory exists
    if [ ! -d "$PROJECT_DIR/dist" ]; then
        warning "dist directory not found. Building project first..."
        build_project
    fi
    
    # Check if port is already in use
    if lsof -Pi :$PROD_PORT -sTCP:LISTEN -t >/dev/null; then
        warning "Port $PROD_PORT is already in use"
        return 1
    fi
    
    # Start production preview server in background
    nohup yarn preview --port $PROD_PORT > "$LOG_FILE" 2>&1 &
    PROD_PID=$!
    echo $PROD_PID > "$PID_FILE"
    
    # Wait a moment and check if server started successfully
    sleep 3
    if kill -0 $PROD_PID 2>/dev/null; then
        success "Production server started successfully (PID: $PROD_PID)"
        success "Access the application at: http://localhost:$PROD_PORT"
    else
        error "Failed to start production server"
        rm -f "$PID_FILE"
        return 1
    fi
}

# Stop the server
stop_server() {
    if [ -f "$PID_FILE" ]; then
        PID=$(cat "$PID_FILE")
        if kill -0 $PID 2>/dev/null; then
            log "Stopping server (PID: $PID)..."
            kill $PID
            sleep 2
            
            # Force kill if still running
            if kill -0 $PID 2>/dev/null; then
                warning "Force killing server..."
                kill -9 $PID
            fi
            
            rm -f "$PID_FILE"
            success "Server stopped successfully"
        else
            warning "Server is not running (stale PID file)"
            rm -f "$PID_FILE"
        fi
    else
        warning "No PID file found. Server may not be running."
    fi
    
    # Also kill any yarn dev/preview processes
    pkill -f "yarn dev" 2>/dev/null
    pkill -f "yarn preview" 2>/dev/null
}

# Check server status
check_status() {
    if [ -f "$PID_FILE" ]; then
        PID=$(cat "$PID_FILE")
        if kill -0 $PID 2>/dev/null; then
            success "Server is running (PID: $PID)"
            
            # Try to determine which port it's using
            if lsof -Pi :$DEV_PORT -sTCP:LISTEN -t >/dev/null; then
                echo "  🌐 Development server: http://localhost:$DEV_PORT"
            elif lsof -Pi :$PROD_PORT -sTCP:LISTEN -t >/dev/null; then
                echo "  🌐 Production server: http://localhost:$PROD_PORT"
            fi
            
            return 0
        else
            error "Server is not running (stale PID file)"
            rm -f "$PID_FILE"
            return 1
        fi
    else
        error "Server is not running"
        return 1
    fi
}

# Show help
show_help() {
    echo ""
    echo "🚀 Doris Profile Visualizer Service Manager"
    echo ""
    echo "Usage: $0 {start|start-dev|start-prod|stop|restart|status|build|install|logs|help}"
    echo ""
    echo "Commands:"
    echo "  start        Start development server (default)"
    echo "  start-dev    Start development server explicitly"
    echo "  start-prod   Start production server (builds first if needed)"
    echo "  stop         Stop the running server"
    echo "  restart      Restart the server"
    echo "  status       Check server status"
    echo "  build        Build the project for production"
    echo "  install      Install dependencies"
    echo "  logs         Show server logs"
    echo "  help         Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0 start              # Start development server"
    echo "  $0 start-prod         # Start production server"
    echo "  $0 restart            # Restart current server"
    echo ""
}

# Show logs
show_logs() {
    if [ -f "$LOG_FILE" ]; then
        log "Showing server logs (last 50 lines):"
        echo "----------------------------------------"
        tail -50 "$LOG_FILE"
        echo "----------------------------------------"
        echo "Use 'tail -f $LOG_FILE' to follow logs in real-time"
    else
        warning "No log file found"
    fi
}

# Main script logic
case "$1" in
    start|start-dev)
        check_yarn
        if ! check_status >/dev/null 2>&1; then
            install_deps
            start_dev
        else
            warning "Server is already running"
        fi
        ;;
    start-prod)
        check_yarn
        if ! check_status >/dev/null 2>&1; then
            install_deps
            start_prod
        else
            warning "Server is already running"
        fi
        ;;
    stop)
        stop_server
        ;;
    restart)
        check_yarn
        stop_server
        sleep 2
        install_deps
        start_dev
        ;;
    status)
        check_status
        ;;
    build)
        check_yarn
        install_deps
        build_project
        ;;
    install)
        check_yarn
        install_deps
        ;;
    logs)
        show_logs
        ;;
    help|--help|-h)
        show_help
        ;;
    *)
        error "Invalid command: $1"
        show_help
        exit 1
        ;;
esac