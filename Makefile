.PHONY: build clean help dev

# Default target
build:
	@echo "==> Step 1/3: Building frontend..."
	@cd frontend && npm install && npm run build
	@echo "==> Step 2/3: Building backend with embedded static files..."
	@cargo build --release
	@echo "==> Step 3/3: Copying binary to build directory..."
	@mkdir -p build
	@cp target/release/doris-profile-analyzer build/
	@echo ""
	@echo "✓ Build complete!"
	@echo "Binary location: build/doris-profile-analyzer"
	@echo ""
	@echo "Usage:"
	@echo "  ./build/doris-profile-analyzer --help"
	@echo "  ./build/doris-profile-analyzer --port 8080"
	@echo "  ./build/doris-profile-analyzer --port 3030 --host 127.0.0.1"

dev:
	@echo "Starting development mode..."
	@cd backend && cargo run

clean:
	@echo "Cleaning build artifacts..."
	@rm -rf frontend/dist
	@rm -rf target
	@rm -rf build
	@echo "✓ Clean complete!"

help:
	@echo "Available targets:"
	@echo "  make        - Build single executable with embedded frontend (default)"
	@echo "  make build  - Same as 'make'"
	@echo "  make dev    - Run backend in development mode"
	@echo "  make clean  - Clean all build artifacts"
	@echo "  make help   - Show this help message"
	@echo ""
	@echo "After build, run:"
	@echo "  ./build/doris-profile-analyzer --help"

