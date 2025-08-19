migrate: 
	sea-orm-cli migrate -d migrations 
force-migrate:
	sea-orm-cli migrate -d migrations fresh
new-migration NAME:
  sea-orm-cli migrate -d migrations generate {{NAME}}

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
	@echo "  force-migrate        Fresh migration (drops all data)"
	@echo "  new-migration NAME   Create new migration"
	@echo "  gen-models           Generate SeaORM models"
	@echo ""
	@echo "🐳 Docker:"
	@echo "  docker-dev           Start development environment"
	@echo "  docker-dev-live      Start development with live reload"
	@echo "  docker-prod          Start production environment"
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

