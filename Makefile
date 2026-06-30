# ==========================================
# Dockerfiles - Makefile
# ==========================================

# Makefile - Development Commands
# ===========================

.PHONY: build up down logs tests

# ## Core Targets

# Build all Docker images
build:
	@echo "Building all Docker images..."
	$(MAKE) build-executor
	$(MAKE) build-gateway
	$(MAKE) build-quant

build-executor:
	docker build --build-arg RUST_VERSION=1.75 -t mev-system/executor-rust ./services/executor-rust

build-gateway:
	docker build --build-arg GO_VERSION=1.22 -t mev-system/gateway-go ./services/gateway-go

build-quant:
	docker build --build-arg PYTHON_VERSION=3.11 -t mev-system/quant-python ./services/quant-python

# Start full stack with Anvil fork
up:
	docker-compose up --build -d
	docker-compose logs -f anvil

# Stop all services
down:
	docker-compose down --remove-orphans

# Follow logs
logs:
	docker-compose logs -f

# ## Testing Targets

# Run tests in isolated containers
test-unit:
	@echo "Running unit tests..."
	docker run --rm --network mev_default mev-system/executor-rust cargo test --quiet
	docker run --rm --network mev_default mev-system/gateway-go go test ./...
	docker run --rm --network mev_default mev-system/quant-python poetry run pytest tests/ -x -v

# Run integration tests (requires Anvil fork)
test-integration: up
	sleep 30  # Wait for Anvil to be ready
	docker run --rm --network mev_default mev-system/executor-rust cargo test integration --quiet
	docker run --rm --network mev_default mev-system/gateway-go go test integration -v
	docker run --rm --network mev_default mev-system/quant-python poetry run pytest tests/robot/integration.robot -v

# Clean up including volumes
delete:
	docker-compose down -v
	docker volume rm mev_default_anvil_data 2>/dev/null || true

# Show system status
status:
	docker ps
	echo "--- Networks ---"
	docker network ls | grep mev_default
	echo "--- Volumes ---"
	docker volume ls | grep mev_default