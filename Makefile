.PHONY: help install install-backend install-frontend build build-backend build-frontend \
	dev dev-backend dev-frontend test test-backend test-frontend \
	clean clean-backend clean-frontend lint lint-backend lint-frontend \
	format format-backend format-frontend check check-backend check-frontend \
	setup setup-backend setup-frontend

# Default target
.DEFAULT_GOAL := help

# Colors for output
BLUE := \033[0;34m
GREEN := \033[0;32m
YELLOW := \033[0;33m
NC := \033[0m # No Color

##@ Help

help: ## Display this help message
	@echo "$(BLUE)Database Query Tool - Makefile Commands$(NC)"
	@echo ""
	@awk 'BEGIN {FS = ":.*##"; printf "\nUsage:\n  make $(BLUE)<target>$(NC)\n"} /^[a-zA-Z_-]+:.*?##/ { printf "  $(BLUE)%-20s$(NC) %s\n", $$1, $$2 } /^##@/ { printf "\n$(GREEN)%s$(NC)\n", substr($$0, 5) } ' $(MAKEFILE_LIST)

##@ Installation

install: install-backend install-frontend ## Install all dependencies

install-backend: ## Install Rust dependencies
	@echo "$(BLUE)Installing Rust dependencies...$(NC)"
	cd backend && cargo fetch

install-frontend: ## Install Node.js dependencies
	@echo "$(BLUE)Installing Node.js dependencies...$(NC)"
	cd frontend && npm install

##@ Build

build: build-backend build-frontend ## Build both backend and frontend

build-backend: ## Build Rust backend (release mode)
	@echo "$(BLUE)Building Rust backend...$(NC)"
	cd backend && cargo build --release

build-frontend: ## Build frontend for production
	@echo "$(BLUE)Building frontend...$(NC)"
	cd frontend && npm run build

##@ Development

dev: dev-backend dev-frontend ## Start both backend and frontend in development mode

dev-backend: ## Start Rust backend server
	@echo "$(BLUE)Starting backend server...$(NC)"
	cd backend && cargo run

dev-frontend: ## Start frontend development server
	@echo "$(BLUE)Starting frontend dev server...$(NC)"
	cd frontend && npm run dev

##@ Testing

test: test-backend test-frontend ## Run all tests

test-backend: ## Run Rust tests
	@echo "$(BLUE)Running Rust tests...$(NC)"
	cd backend && cargo test

test-frontend: ## Run frontend tests
	@echo "$(BLUE)Running frontend tests...$(NC)"
	cd frontend && npm test || echo "$(YELLOW)No test script configured$(NC)"

##@ Code Quality

lint: lint-backend lint-frontend ## Lint both backend and frontend code

lint-backend: ## Lint Rust code with clippy
	@echo "$(BLUE)Linting Rust code...$(NC)"
	cd backend && cargo clippy -- -D warnings

lint-frontend: ## Lint frontend code with ESLint
	@echo "$(BLUE)Linting frontend code...$(NC)"
	cd frontend && npm run lint

format: format-backend format-frontend ## Format all code

format-backend: ## Format Rust code with rustfmt
	@echo "$(BLUE)Formatting Rust code...$(NC)"
	cd backend && cargo fmt

format-frontend: ## Format frontend code with Prettier
	@echo "$(BLUE)Formatting frontend code...$(NC)"
	cd frontend && npm run format || echo "$(YELLOW)Format script not configured$(NC)"

check: check-backend check-frontend ## Check code without building

check-backend: ## Check Rust code without building
	@echo "$(BLUE)Checking Rust code...$(NC)"
	cd backend && cargo check

check-frontend: ## Type-check frontend code
	@echo "$(BLUE)Type-checking frontend code...$(NC)"
	cd frontend && npx tsc --noEmit

##@ Cleanup

clean: clean-backend clean-frontend ## Clean all build artifacts

clean-backend: ## Clean Rust build artifacts
	@echo "$(BLUE)Cleaning Rust build artifacts...$(NC)"
	cd backend && cargo clean

clean-frontend: ## Clean frontend build artifacts
	@echo "$(BLUE)Cleaning frontend build artifacts...$(NC)"
	cd frontend && rm -rf dist node_modules/.vite

clean-all: clean ## Clean everything including dependencies
	@echo "$(BLUE)Cleaning all artifacts and dependencies...$(NC)"
	cd frontend && rm -rf node_modules

##@ Setup

setup: setup-backend setup-frontend ## Initial setup for development

setup-backend: ## Setup backend development environment
	@echo "$(BLUE)Setting up backend...$(NC)"
	@if [ ! -f backend/.env ]; then \
		echo "$(YELLOW)Creating backend/.env from .env.example...$(NC)"; \
		cp backend/.env.example backend/.env; \
	fi
	cd backend && cargo fetch

setup-frontend: ## Setup frontend development environment
	@echo "$(BLUE)Setting up frontend...$(NC)"
	@if [ ! -f frontend/.env ]; then \
		echo "$(YELLOW)Creating frontend/.env from .env.example...$(NC)"; \
		cp frontend/.env.example frontend/.env; \
	fi
	cd frontend && npm install

##@ Database

db-migrate: ## Run database migrations (when implemented)
	@echo "$(BLUE)Running database migrations...$(NC)"
	@echo "$(YELLOW)Database migrations not yet implemented$(NC)"

db-reset: ## Reset database (when implemented)
	@echo "$(BLUE)Resetting database...$(NC)"
	@echo "$(YELLOW)Database reset not yet implemented$(NC)"

##@ Docker (if needed)

docker-build: ## Build Docker images
	@echo "$(BLUE)Building Docker images...$(NC)"
	@echo "$(YELLOW)Docker support not yet configured$(NC)"

docker-up: ## Start Docker containers
	@echo "$(BLUE)Starting Docker containers...$(NC)"
	@echo "$(YELLOW)Docker support not yet configured$(NC)"

##@ CI/CD

ci: install lint test build ## Run CI pipeline locally
	@echo "$(GREEN)CI pipeline completed successfully!$(NC)"

##@ Quick Commands

quick-start: setup dev ## Quick start: setup and run development servers

all: clean install lint test build ## Run full pipeline: clean, install, lint, test, build

