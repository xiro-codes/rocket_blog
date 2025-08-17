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

