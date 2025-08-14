# Sea-ORM Entity Generation Automation

This directory contains scripts to automate the sea-orm entity generation process, solving two key problems:

1. **Serde Import Correction**: Automatically replace `serde::` imports with `rocket::serde::` imports
2. **Custom Code Preservation**: Preserve manually written code (like DTO structs) when regenerating entities

## Problem

When using `sea-orm-cli generate entity`, two issues occur:

1. The generated code uses `serde::{Deserialize, Serialize}` instead of `rocket::serde::{Deserialize, Serialize}` (required for Rocket integration)
2. Any manually written code (like FormDTO structs, custom impl blocks) gets overwritten

## Solution

### Option 1: Full Automation (Recommended)

Use `preserve_custom_code.py` for complete automation:

```bash
just gen-models
```

This will:
1. Backup any custom code from existing entity files
2. Regenerate all entity files with sea-orm-cli
3. Fix serde imports to use rocket::serde
4. Restore all custom code back into the files

### Option 2: Import Fixing Only

Use `fix_serde_imports.sh` for basic serde import fixing:

```bash
just gen-models-simple
```

This will:
1. Regenerate all entity files with sea-orm-cli  
2. Fix serde imports to use rocket::serde
3. ⚠️ **Does NOT preserve custom code** - you'll need to manually re-add it

## Scripts

### preserve_custom_code.py

**Requirements**: Python 3.6+

**Usage**:
```bash
# Backup custom code
python3 scripts/preserve_custom_code.py ./models/src

# After regenerating entities, restore custom code
python3 scripts/preserve_custom_code.py ./models/src restore
```

**What it preserves**:
- Custom structs (FormDTO, TitleResult, etc.)
- Custom derive attributes
- Custom serde attributes
- Any struct that contains `FromForm`, `DerivePartialModel`, or `FromQueryResult`

**What it fixes**:
- `use serde::{...}` → `use rocket::serde::{...}`
- `use serde::...` → `use rocket::serde::...`

### fix_serde_imports.sh

**Requirements**: bash, sed

**Usage**:
```bash
./scripts/fix_serde_imports.sh ./models/src
```

**What it fixes**:
- `use serde::{...}` → `use rocket::serde::{...}`
- `use serde::...` → `use rocket::serde::...`

## How It Works

1. **Custom Code Detection**: The Python script uses regex patterns to identify custom structs that aren't part of the standard sea-orm generated code (Model, Relation enum, etc.)

2. **Backup Storage**: Custom code is stored in a JSON file with the structure of each entity file preserved

3. **Import Fixing**: Simple regex replacement of serde imports with rocket::serde imports

4. **Code Restoration**: Custom code is inserted back into the generated files in the correct location (before `impl ActiveModelBehavior`)

## Example

Before regeneration, `account.rs` might contain:

```rust
use rocket::serde::{Deserialize, Serialize};

// ... generated Model struct ...

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, FromForm)]
#[serde(crate = "rocket::serde")]
pub struct FormDTO {
    pub username: String,
    pub password: String,
}

impl ActiveModelBehavior for ActiveModel {}
```

After running `sea-orm-cli generate entity`, it becomes:

```rust
use serde::{Deserialize, Serialize};

// ... generated Model struct ...

impl ActiveModelBehavior for ActiveModel {}
```

After running our automation, it's restored to:

```rust
use rocket::serde::{Deserialize, Serialize};

// ... generated Model struct ...

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, FromForm)]
#[serde(crate = "rocket::serde")]
pub struct FormDTO {
    pub username: String,
    pub password: String,
}

impl ActiveModelBehavior for ActiveModel {}
```

## Integration with justfile

The main project `justfile` has been updated with two commands:

- `just gen-models` - Full automation with custom code preservation
- `just gen-models-simple` - Basic serde fixing only (fallback)

This replaces the old manual process that required remembering to fix imports.