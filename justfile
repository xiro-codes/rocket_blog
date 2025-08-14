migrate: 
	sea-orm-cli migrate -d migrations 
force-migrate:
	sea-orm-cli migrate -d migrations fresh
new-migration NAME:
  sea-orm-cli migrate -d migrations generate {{NAME}}
gen-models:
	rm ./models/src/*
	sea-orm-cli generate entity -l --model-extra-attributes 'serde(crate="rocket::serde")' --with-serde both --with-copy-enums -o  ./models/src
	echo "DONT FORGET to correct the serde import"

