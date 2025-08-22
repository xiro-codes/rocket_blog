migrate: 
	sea-orm-cli migrate -d migrations 
force-migrate:
	sea-orm-cli migrate -d migrations fresh
new-migration NAME:
  sea-orm-cli migrate -d migrations generate {{NAME}}
migrate-status:
	sea-orm-cli migrate -d migrations status

# Generate models with custom code preserved in separate dto module
gen-models:
	# Remove only generated entity files, preserve dto.rs and other custom files
	rm ./models/src/account.rs ./models/src/comment.rs ./models/src/post.rs ./models/src/prelude.rs ./models/src/lib.rs || true
	sea-orm-cli generate entity -l --model-extra-attributes 'serde(crate="rocket::serde")' --with-serde both --with-copy-enums -o  ./models/src
	sh ./scripts/fix_serde_imports.sh ./models/src
	# Add dto module to lib.rs if not present
	echo "" >> ./models/src/lib.rs
	echo "// Custom DTOs and form structures" >> ./models/src/lib.rs
	echo "pub mod dto;" >> ./models/src/lib.rs
	echo "✅ Models generated with fixed serde imports and preserved DTO module"

# Local development commands
# Build the application in release mode
build:
	cargo build --release

# Build the application in debug mode (faster compilation)
build-dev:
	cargo build

# Run all tests
test:
	cargo test

# Run specific test by name
test-name NAME:
	cargo test {{NAME}}

# Run tests with output
test-verbose:
	cargo test -- --nocapture

# Check code without building
check:
	cargo check

# Format code
fmt:
	cargo fmt

# Check code formatting
fmt-check:
	cargo fmt --check

# Run clippy linter
clippy:
	cargo clippy --all-targets --all-features -- -D warnings

# Clean build artifacts
clean:
	cargo clean

# Run the application locally (development mode)
run:
	cargo run

# Alias for run command
dev: run

# Docker development and deployment commands
docker-dev:
	sh ./scripts/docker-deploy.sh dev

docker-dev-live:
	sh ./scripts/docker-deploy.sh dev-live

docker-prod:
	sh ./scripts/docker-deploy.sh prod

docker-setup-ssl:
	sh ./scripts/docker-deploy.sh setup-ssl

docker-renew-ssl:
	sh ./scripts/docker-deploy.sh renew-ssl

docker-status:
	sh ./scripts/docker-deploy.sh status

docker-logs SERVICE="":
	sh ./scripts/docker-deploy.sh logs {{SERVICE}}

docker-stop:
	sh ./scripts/docker-deploy.sh stop

docker-clean:
	sh ./scripts/docker-deploy.sh clean

docker-help:
	sh ./scripts/docker-deploy.sh help

# Backup Docker volumes to filesystem
docker-backup ENV="":
	sh ./scripts/docker-deploy.sh backup {{ENV}}

# Restore Docker volumes from backup
docker-restore ENV="":
	sh ./scripts/docker-deploy.sh restore {{ENV}}

# List available Docker volume backups
docker-backup-list:
	sh ./scripts/docker-deploy.sh backup-list

# Clean old Docker volume backups (default: 7 days)
docker-backup-clean DAYS="7":
	sh ./scripts/docker-deploy.sh backup-clean {{DAYS}}

# Run tests in Docker container
docker-test:
	@echo "🧪 Building and running tests in Docker container..."
	docker build -f scripts/docker/Dockerfile.test -t rocket-blog-test .
	docker run --rm rocket-blog-test

# Run tests in Docker container with verbose output
docker-test-verbose:
	@echo "🧪 Building and running tests in Docker container (verbose)..."
	docker build -f scripts/docker/Dockerfile.test -t rocket-blog-test .
	docker run --rm rocket-blog-test cargo test -- --nocapture

# Run code coverage with Tarpaulin in Docker container
docker-coverage:
	@echo "📊 Building and running code coverage with Tarpaulin in Docker container..."
	docker build -f scripts/docker/Dockerfile.coverage -t rocket-blog-coverage .
	docker run --rm rocket-blog-coverage

# Run code coverage with Tarpaulin in Docker container with output to file
docker-coverage-output:
	@echo "📊 Building and running code coverage with Tarpaulin in Docker container (saving output)..."
	docker build -f scripts/docker/Dockerfile.coverage -t rocket-blog-coverage .
	docker run --rm rocket-blog-coverage > coverage-report.txt
	@echo "✅ Coverage report saved to coverage-report.txt"

# Install and run Tarpaulin locally (requires cargo-tarpaulin to be installed)
coverage:
	@echo "📊 Running code coverage with Tarpaulin locally..."
	@if ! command -v cargo-tarpaulin >/dev/null 2>&1; then \
		echo "Installing cargo-tarpaulin..."; \
		cargo install cargo-tarpaulin; \
	fi
	cargo tarpaulin --verbose --timeout 60 --package app

# Install and run Tarpaulin locally with full workspace coverage
coverage-full:
	@echo "📊 Running full code coverage with Tarpaulin locally..."
	@if ! command -v cargo-tarpaulin >/dev/null 2>&1; then \
		echo "Installing cargo-tarpaulin..."; \
		cargo install cargo-tarpaulin; \
	fi
	cargo tarpaulin --verbose --all-features --timeout 120 --package app

# View application logs from the app_data volume
logs:
	@echo "📋 Reading output.log from app_data volume..."
	@if [ -f "./output.log" ]; then \
		echo "📄 Found local output.log:"; \
		tail -50 ./output.log; \
	else \
		echo "⚠️  No local output.log found. Use 'logs-docker' to read from container."; \
	fi

# View application logs from the Docker container
logs-docker:
	@echo "📋 Reading output.log from Docker container..."
	@container_id=$$(docker ps -q -f name=app); \
	if [ -n "$$container_id" ]; then \
		echo "Found container: $$container_id"; \
		docker exec $$container_id tail -50 /app/output.log 2>/dev/null || \
		docker exec $$container_id ls -la /app/ 2>/dev/null || \
		echo "❌ Could not access logs from container"; \
	else \
		echo "❌ No running app container found. Start with 'just docker-dev' first."; \
	fi

# View application logs with live tail (follow mode)
logs-follow:
	@echo "📋 Following output.log (press Ctrl+C to stop)..."
	@if [ -f "./output.log" ]; then \
		tail -f ./output.log; \
	else \
		echo "⚠️  No local output.log found. Use 'logs-docker-follow' for container logs."; \
	fi

# View application logs from Docker container with live tail
logs-docker-follow:
	@echo "📋 Following output.log from Docker container (press Ctrl+C to stop)..."
	@container_id=$$(docker ps -q -f name=app); \
	if [ -n "$$container_id" ]; then \
		docker exec $$container_id tail -f /app/output.log 2>/dev/null || \
		echo "❌ Could not access logs from container"; \
	else \
		echo "❌ No running app container found. Start with 'just docker-dev' first."; \
	fi

# Show log file locations and status
logs-info:
	@echo "📍 Log file locations:"
	@echo "Local:      ./output.log $(if [ -f "./output.log" ]; then echo "(exists, $$(wc -l < ./output.log) lines)"; else echo "(not found)"; fi)"
	@container_id=$$(docker ps -q -f name=app); \
	if [ -n "$$container_id" ]; then \
		echo "Container:  /app/output.log in container $$container_id"; \
		docker exec $$container_id ls -la /app/output.log 2>/dev/null || echo "           (not accessible)"; \
	else \
		echo "Container:  No running app container found"; \
	fi

# Show available commands
help:
	@echo "🚀 Rocket Blog Development Commands"
	@echo ""
	@echo "📊 Database:"
	@echo "  migrate              Run database migrations"
	@echo "  migrate-status       Check migration status"
	@echo "  force-migrate        Fresh migration (drops all data)"
	@echo "  new-migration NAME   Create new migration"
	@echo "  gen-models           Generate SeaORM models"
	@echo ""
	@echo "🦀 Local Development:"
	@echo "  build                Build application (release mode)"
	@echo "  build-dev            Build application (debug mode)"
	@echo "  run / dev            Run application locally"
	@echo "  test                 Run all tests"
	@echo "  test-name NAME       Run specific test by name"
	@echo "  test-verbose         Run tests with output"
	@echo "  coverage             Run code coverage with Tarpaulin locally"
	@echo "  coverage-full        Run full workspace coverage with Tarpaulin"
	@echo "  check                Check code without building"
	@echo "  fmt                  Format code"
	@echo "  fmt-check            Check code formatting"
	@echo "  clippy               Run clippy linter"
	@echo "  clean                Clean build artifacts"
	@echo ""
	@echo "🐳 Docker:"
	@echo "  docker-dev           Start development environment"
	@echo "  docker-dev-live      Start development with live reload"
	@echo "  docker-prod          Start production environment"
	@echo "  docker-test          Run tests in Docker container"
	@echo "  docker-test-verbose  Run tests in Docker container (verbose)"
	@echo "  docker-coverage      Run code coverage with Tarpaulin in Docker"
	@echo "  docker-coverage-output Run coverage with output saved to file"
	@echo "  docker-stop          Stop all containers"
	@echo "  docker-clean         Stop and remove all data"
	@echo "  docker-status        Show container status"
	@echo "  docker-logs [SERVICE] Show Docker logs"
	@echo ""
	@echo "📋 Application Logs:"
	@echo "  logs                 View recent application logs (local)"
	@echo "  logs-docker          View recent application logs (container)"
	@echo "  logs-follow          Follow application logs (local)"
	@echo "  logs-docker-follow   Follow application logs (container)"
	@echo "  logs-info            Show log file locations and status"

