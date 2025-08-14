#!/usr/bin/env python3
"""
Script to preserve custom code when regenerating sea-orm entities.

This script:
1. Extracts manually written code from existing entity files
2. Allows regeneration of entities  
3. Restores manually written code back into generated files
4. Fixes serde imports to use rocket::serde instead of plain serde
"""

import os
import re
import sys
import json
from pathlib import Path

def extract_custom_code(file_path):
    """Extract custom code blocks (structs, impl blocks) that are not part of generated entity code."""
    if not file_path.exists():
        return []
    
    content = file_path.read_text()
    
    # Use regex to find custom structs that are not the main Model struct or Relation enum
    # Look for structs with names like FormDTO, TitleResult, etc.
    custom_blocks = []
    
    # Pattern to match custom struct definitions
    # Matches from #[derive...] through the end of struct definition  
    pattern = r'(#\[derive\([^}]+\)\]\s*(?:#\[[^\]]+\]\s*)*pub\s+struct\s+(?!Model\s|.*Relation)(\w*(?:DTO|Result|Form|Request|Response)\w*|[A-Z]\w+(?:DTO|Result|Form|Request|Response))\s*\{[^}]*\})'
    
    # Also try a more general approach - look for any struct that's not Model
    general_pattern = r'(#\[derive\([^}]*(?:FromForm|DerivePartialModel|FromQueryResult)[^}]*\)\]\s*(?:#\[[^\]]+\]\s*)*(?:#\[sea_orm\([^}]+\)\]\s*)*pub\s+struct\s+(?!Model\s)(\w+)\s*\{[^}]*\})'
    
    for pattern in [pattern, general_pattern]:
        matches = re.finditer(pattern, content, re.MULTILINE | re.DOTALL)
        for match in matches:
            block = match.group(1).strip()
            if block not in custom_blocks:
                custom_blocks.append(block)
    
    return custom_blocks

def fix_serde_imports(file_path):
    """Fix serde imports to use rocket::serde instead of plain serde."""
    if not file_path.exists():
        return False
    
    content = file_path.read_text()
    original_content = content
    
    # Replace plain serde imports with rocket::serde imports
    patterns = [
        (r'use serde::\{([^}]+)\};', r'use rocket::serde::{\1};'),
        (r'use serde::([^;]+);', r'use rocket::serde::\1;'),
    ]
    
    for old_pattern, new_pattern in patterns:
        content = re.sub(old_pattern, new_pattern, content)
    
    if content != original_content:
        file_path.write_text(content)
        return True
    return False

def restore_custom_code(file_path, custom_blocks):
    """Restore custom code blocks to the generated file."""
    if not file_path.exists() or not custom_blocks:
        return
    
    content = file_path.read_text()
    
    # Find the position to insert custom code (before impl ActiveModelBehavior)
    active_model_pattern = r'impl ActiveModelBehavior for ActiveModel \{\}'
    
    if re.search(active_model_pattern, content):
        custom_code = '\n'.join(custom_blocks)
        content = re.sub(active_model_pattern, custom_code + '\n\n' + r'impl ActiveModelBehavior for ActiveModel {}', content)
        file_path.write_text(content)

def backup_custom_code(models_dir):
    """Backup all custom code from existing entity files."""
    models_dir = Path(models_dir)
    backup = {}
    
    print(f"Scanning {models_dir} for custom code...")
    
    for rust_file in models_dir.glob('*.rs'):
        if rust_file.name in ['lib.rs', 'prelude.rs', 'mod.rs']:
            continue
        
        print(f"  Checking {rust_file.name}...")
        custom_blocks = extract_custom_code(rust_file)
        if custom_blocks:
            backup[rust_file.name] = custom_blocks
            print(f"    Found {len(custom_blocks)} custom block(s)")
    
    return backup

def restore_all_custom_code(models_dir, backup):
    """Restore custom code to all entity files and fix imports."""
    models_dir = Path(models_dir)
    
    for rust_file in models_dir.glob('*.rs'):
        if rust_file.name in ['lib.rs', 'prelude.rs', 'mod.rs']:
            continue
        
        print(f"Processing {rust_file.name}...")
        
        # Fix serde imports first
        fixed_imports = fix_serde_imports(rust_file)
        if fixed_imports:
            print(f"  Fixed serde imports")
        
        # Restore custom code if we have any backed up
        if rust_file.name in backup:
            restore_custom_code(rust_file, backup[rust_file.name])
            print(f"  Restored {len(backup[rust_file.name])} custom block(s)")

def save_backup(backup, backup_file):
    """Save backup to a JSON file."""
    with open(backup_file, 'w') as f:
        json.dump(backup, f, indent=2)

def load_backup(backup_file):
    """Load backup from a JSON file."""
    if not os.path.exists(backup_file):
        return {}
    with open(backup_file, 'r') as f:
        return json.load(f)

def main():
    if len(sys.argv) < 2:
        print("Usage:")
        print("  preserve_custom_code.py <models_src_dir>           - backup custom code")
        print("  preserve_custom_code.py <models_src_dir> restore   - restore custom code")
        sys.exit(1)
    
    models_dir = sys.argv[1]
    backup_file = os.path.join(os.path.dirname(models_dir), 'custom_code_backup.json')
    
    if len(sys.argv) > 2 and sys.argv[2] == 'restore':
        print("Restoring custom code and fixing imports...")
        backup = load_backup(backup_file)
        if not backup:
            print("No backup found! Run without 'restore' first.")
            sys.exit(1)
        
        restore_all_custom_code(models_dir, backup)
        print("Custom code restored and imports fixed!")
        
        # Clean up backup file
        if os.path.exists(backup_file):
            os.remove(backup_file)
    else:
        print("Backing up custom code...")
        backup = backup_custom_code(models_dir)
        save_backup(backup, backup_file)
        print(f"Backed up custom code from {len(backup)} files to {backup_file}")
        print("\nNow you can run: sea-orm-cli generate entity ...")
        print(f"Then run: python3 scripts/preserve_custom_code.py {models_dir} restore")

if __name__ == '__main__':
    main()