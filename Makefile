.PHONY: help install train build-backend build-frontend run-backend run-frontend clean

help:
	@echo "Traffic Classifier - Available Commands"
	@echo ""
	@echo "  make install     - Install all system dependencies (Rust, libpcap)"
	@echo "  make train       - Train ML model"
	@echo "  make build-backend  - Build Rust backend"
	@echo "  make build-frontend  - Build React frontend"
	@echo "  make run-backend     - Run backend server"
	@echo "  make run-frontend    - Run frontend dev server"
	@echo "  make all          - Build everything"
	@echo "  make clean        - Clean build artifacts"

install:
	@echo "Installing dependencies..."
	@if ! command -v rustc &> /dev/null; then \
		echo "Installing Rust..."; \
		curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y; \
	fi
	@echo "Rust installed"

train:
	@echo "Training ML model..."
	@cd classifier && python3 -m venv venv 2>/dev/null || true
	@cd classifier && source venv/bin/activate && pip install torch numpy scikit-learn onnx --quiet
	@cd classifier && source venv/bin/activate && python train.py

build-backend:
	@echo "Building backend..."
	@cd backend && cargo build --release

build-frontend:
	@echo "Building frontend..."
	@cd frontend && npm install && npm run build

run-backend:
	@echo "Starting backend..."
	@cd backend && cargo run --release

run-frontend:
	@echo "Starting frontend..."
	@cd frontend && npm run dev

all: build-backend build-frontend

clean:
	@cd backend && cargo clean
	@cd frontend && rm -rf node_modules dist
	@cd classifier && rm -rf venv *.onnx *.pth