# Generate and open Rust documentation
doc:
	@echo "📚 Generating Rust documentation..."
	cargo doc --lib --no-deps --document-private-items
	@echo "✅ Documentation generated successfully"

# Generate and open Rust documentation in browser
doc-open:
	@echo "📚 Generating and opening Rust documentation..."
	cargo doc --lib --no-deps --document-private-items --open
	@echo "✅ Documentation opened in browser"

# Generate documentation for all workspace members
doc-all:
	@echo "📚 Generating documentation for entire workspace..."
	cargo doc --workspace --no-deps --document-private-items
	@echo "✅ Workspace documentation generated successfully"

# Generate and open documentation for all workspace members
doc-all-open:
	@echo "📚 Generating and opening workspace documentation..."
	cargo doc --workspace --no-deps --document-private-items --open
	@echo "✅ Workspace documentation opened in browser"

# Clean documentation artifacts
doc-clean:
	@echo "🧹 Cleaning documentation artifacts..."
	cargo clean --doc
	@echo "✅ Documentation artifacts cleaned"

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

# NixOS Container commands
# Build and run the app in a NixOS container
container-run:
	nixos-container create rocket-blog --flake .#rocket-container
	nixos-container start rocket-blog
	@echo "🚀 Container 'rocket-blog' started. Access at http://$(nixos-container show-ip rocket-blog)"
	@echo "IP address: $(nixos-container show-ip rocket-blog)"

# Stop and destroy the container
container-clean:
	nixos-container destroy rocket-blog

# Update the container with latest changes
container-update:
	nixos-container update rocket-blog --flake .#rocket-container
	nixos-container restart rocket-blog

# View container logs
container-logs:
	nixos-container run rocket-blog -- journalctl -u rocket-blog -u rocket-worktime -f

