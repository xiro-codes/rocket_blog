migrate: 
	sea-orm-cli migrate -d migrations 
force-migrate:
	sea-orm-cli migrate -d migrations fresh
new-migration NAME:
  sea-orm-cli migrate -d migrations generate {{NAME}}

# Generate models with automatic serde import fixing and custom code preservation
gen-models:
	python3 scripts/preserve_custom_code.py ./models/src
	rm ./models/src/*.rs || true
	sea-orm-cli generate entity -l --model-extra-attributes 'serde(crate="rocket::serde")' --with-serde both --with-copy-enums -o  ./models/src
	python3 scripts/preserve_custom_code.py ./models/src restore
	echo "✅ Models generated with preserved custom code and fixed serde imports"

# Generate models with basic serde import fixing only (fallback for systems without Python)
gen-models-simple:
	rm ./models/src/*.rs || true
	sea-orm-cli generate entity -l --model-extra-attributes 'serde(crate="rocket::serde")' --with-serde both --with-copy-enums -o  ./models/src
	./scripts/fix_serde_imports.sh ./models/src
	echo "✅ Models generated with fixed serde imports (manual code not preserved)"

