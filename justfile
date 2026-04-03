default:
	@just --list

# Generate and open Rust documentation
[group('Docs')]
doc:
	@echo "📚 Generating Rust documentation..."
	cargo doc --lib --no-deps --document-private-items
	@echo "✅ Documentation generated successfully"

# Generate and open Rust documentation in browser
[group('Docs')]
doc-open:
	@echo "📚 Generating and opening Rust documentation..."
	cargo doc --lib --no-deps --document-private-items --open
	@echo "✅ Documentation opened in browser"

# Generate documentation for all workspace members
[group('Docs')]
doc-all:
	@echo "📚 Generating documentation for entire workspace..."
	cargo doc --workspace --no-deps --document-private-items
	@echo "✅ Workspace documentation generated successfully"

# Generate and open documentation for all workspace members
[group('Docs')]
doc-all-open:
	@echo "📚 Generating and opening workspace documentation..."
	cargo doc --workspace --no-deps --document-private-items --open
	@echo "✅ Workspace documentation opened in browser"

# Clean documentation artifacts
[group('Docs')]
doc-clean:
	@echo "🧹 Cleaning documentation artifacts..."
	cargo clean --doc
	@echo "✅ Documentation artifacts cleaned"

[group('Database')]
migrate: 
	sea-orm-cli migrate -d migrations 

[group('Database')]
force-migrate:
	sea-orm-cli migrate -d migrations fresh

[group('Database')]
new-migration NAME:
	sea-orm-cli migrate -d migrations generate {{NAME}}

[group('Database')]
migrate-status:
	sea-orm-cli migrate -d migrations status

# Generate models with custom code preserved in separate dto module
[group('Database')]
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

# Build the application in release mode
[group('Dev')]
build:
	cargo build --release

# Build the application in debug mode (faster compilation)
[group('Dev')]
build-dev:
	cargo build

# Run all tests
[group('Dev')]
test:
	cargo test

# Run specific test by name
[group('Dev')]
test-name NAME:
	cargo test {{NAME}}

# Run tests with output
[group('Dev')]
test-verbose:
	cargo test -- --nocapture

# Check code without building
[group('Dev')]
check:
	cargo check

# Format code
[group('Dev')]
fmt:
	cargo fmt

# Check code formatting
[group('Dev')]
fmt-check:
	cargo fmt --check

# Run clippy linter
[group('Dev')]
clippy:
	cargo clippy --all-targets --all-features -- -D warnings

# Clean build artifacts
[group('Dev')]
clean:
	cargo clean

# Run the application locally (development mode)
[group('Dev')]
run:
	cargo run

# Alias for run command
[group('Dev')]
dev: run

# Build and run the app in a NixOS container
[group('Container')]
container-run:
	sudo nixos-container create rocket-blog --flake .#rocket-container || true
	sudo nixos-container start rocket-blog
	@echo "🚀 Container 'rocket-blog' started. Access at http://$(sudo nixos-container show-ip rocket-blog)"
	@echo "IP address: $(sudo nixos-container show-ip rocket-blog)"

# Stop and destroy the container
[group('Container')]
container-clean:
	sudo nixos-container destroy rocket-blog

# Update the container with latest changes
[group('Container')]
container-update:
	sudo nixos-container update rocket-blog --flake .#rocket-container
	sudo nixos-container restart rocket-blog

# View container logs
[group('Container')]
container-logs:
	sudo nixos-container run rocket-blog -- journalctl -u rocket-blog -u rocket-worktime -f

# Build and run the dev container with local directory mounted
[group('Container')]
container-dev:
	sudo nixos-container create rocket-dev --flake .#rocket-dev-container || true
	sudo bash -c "echo 'EXTRA_NSPAWN_FLAGS=\"--bind=$(pwd):/host\"' >> /etc/nixos-containers/rocket-dev.conf"
	sudo nixos-container start rocket-dev
	@echo "🚀 Dev container 'rocket-dev' started. Access at http://$(sudo nixos-container show-ip rocket-dev)"
	@echo "Local directory mounted at /host inside the container."

# Stop and destroy the dev container
[group('Container')]
container-dev-clean:
	sudo nixos-container destroy rocket-dev

# Update the dev container with latest changes
[group('Container')]
container-dev-update:
	sudo nixos-container update rocket-dev --flake .#rocket-dev-container
	sudo nixos-container restart rocket-dev

# View dev container logs
[group('Container')]
container-dev-logs:
	sudo nixos-container run rocket-dev -- journalctl -u rocket-blog -u rocket-worktime -f
