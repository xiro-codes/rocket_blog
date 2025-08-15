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

# Generate models with basic serde import fixing only (legacy)
gen-models-simple:
	# Remove only generated entity files, preserve dto.rs and other custom files  
	rm ./models/src/account.rs ./models/src/comment.rs ./models/src/post.rs ./models/src/prelude.rs ./models/src/lib.rs || true
	sea-orm-cli generate entity -l --model-extra-attributes 'serde(crate="rocket::serde")' --with-serde both --with-copy-enums -o  ./models/src
	sh ./scripts/fix_serde_imports.sh ./models/src
	# Add dto module to lib.rs if not present
	echo "" >> ./models/src/lib.rs  
	echo "// Custom DTOs and form structures" >> ./models/src/lib.rs
	echo "pub mod dto;" >> ./models/src/lib.rs
	echo "✅ Models generated with fixed serde imports and preserved DTO module"

# Test commands
test:
	cargo test --tests

test-unit:
	cargo test --tests base_service_tests auth_service_tests blog_service_tests tag_service_tests

test-integration: 
	cargo test --tests integration_tests route_tests

test-verbose:
	cargo test --tests --verbose

test-nocapture:
	cargo test --tests -- --nocapture

