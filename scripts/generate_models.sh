#!/bin/bash
# Simple script to generate sea-orm entities with dto module preservation
#
# This script:
# 1. Removes existing entity files 
# 2. Generates new entities with sea-orm-cli
# 3. Fixes serde imports to use rocket::serde
# 4. Preserves the dto.rs module which contains custom DTOs/forms

set -e

MODELS_DIR=${1:-"./models/src"}

echo "🧹 Removing existing entity files..."
rm -f "$MODELS_DIR"/*.rs || true

echo "🚀 Generating new entities..."
sea-orm-cli generate entity -l --model-extra-attributes 'serde(crate="rocket::serde")' --with-serde both --with-copy-enums -o "$MODELS_DIR"

echo "🔧 Fixing serde imports..."
./scripts/fix_serde_imports.sh "$MODELS_DIR"

echo "📦 Restoring DTO module..."
git checkout HEAD -- "$MODELS_DIR/dto.rs" || echo "Warning: dto.rs not found in git, keeping current version"

echo "✅ Models generated successfully with preserved DTO module!"