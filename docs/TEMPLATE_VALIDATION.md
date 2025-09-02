# Template Validation and Maintenance

This document outlines the template validation process and tools for the Rocket Blog project.

## Template System Overview

The project uses Jinja2 templates (`.j2` files) with the `rocket_dyn_templates` crate and `minijinja` feature. There are 26 template files organized as follows:

```
templates/
├── base.html.j2              # Main base template
├── worktime_base.html.j2     # Worktime base template  
├── worktime_macros.html.j2   # Worktime macros
├── offline.html.j2           # Offline/PWA template
├── auth/                     # Authentication templates
├── blog/                     # Blog-related templates
├── error/                    # Error page templates
├── settings/                 # Settings templates
└── worktime/                 # Work time tracking templates
```

## Template Validation Process

### Recent Fixes Applied

1. **Fixed duplicate JavaScript variables** in `templates/blog/create.html.j2`
   - Removed 8 duplicate variable declarations and a duplicate event listener
   - Issue: Variables were declared twice in the same scope

2. **Fixed Jinja spacing issues** (40 fixes across 3 template files)
   - Corrected `{%extends` → `{% extends` in error templates
   - Fixed spacing in conditional tags: `{%if` → `{% if`, `{%endif` → `{% endif`
   - Standardized spacing throughout templates

3. **Validated template inheritance**
   - All `extends` statements are properly formatted
   - No duplicate block definitions found
   - Proper nesting of Jinja tags verified

## Validation Tools

### Automated Template Checker

A comprehensive validation script has been created to check templates for:

- **Jinja2 Syntax Issues**: Unclosed tags, invalid syntax, improper nesting
- **Template Inheritance**: Multiple extends, missing blocks, inheritance chain issues  
- **JavaScript Issues**: Duplicate variable declarations in the same scope
- **HTML Structure**: Basic validation of HTML attributes and structure

### Usage

```bash
# Run template validation
python3 scripts/validate_templates.py

# Apply automatic fixes for common issues
python3 scripts/fix_templates.py
```

### Manual Validation Checklist

When adding or modifying templates:

1. **Jinja Syntax**
   - [ ] All `{%` tags have space after opening: `{% tag`
   - [ ] All `%}` tags have space before closing: `tag %}`
   - [ ] Proper nesting of conditional and loop tags
   - [ ] No duplicate block names

2. **Template Inheritance**  
   - [ ] `extends` statement at the top (if used)
   - [ ] Only one `extends` per template
   - [ ] Proper block definitions and usage

3. **JavaScript/CSS**
   - [ ] No duplicate variable declarations in same scope
   - [ ] Proper string escaping in embedded scripts
   - [ ] Valid CSS syntax

4. **HTML Structure**
   - [ ] Properly closed tags
   - [ ] Valid attribute syntax
   - [ ] Accessible markup

## Common Issues and Solutions

### Jinja Spacing Issues
**Problem**: `{%if condition%}` (missing spaces)
**Solution**: `{% if condition %}`

### Duplicate Variables
**Problem**: Declaring the same JavaScript variable twice in the same scope
**Solution**: Remove duplicate declarations or use different scopes

### Template Inheritance
**Problem**: Multiple `extends` statements or `extends` not at top
**Solution**: Single `extends` at template beginning

### HTML Attributes  
**Problem**: Unclosed quotes in attributes
**Solution**: Ensure all attribute values are properly quoted

## Build Integration

Template validation is integrated into the build process:

```bash
# Check templates as part of build validation
cargo check  # Will fail if templates have critical syntax errors

# Full build with template validation
cargo build
```

## Error Categories

- **Critical Errors**: Syntax errors that prevent template compilation
- **Warnings**: Style issues that should be fixed but don't break functionality
- **Information**: Best practice suggestions

## Best Practices

1. **Consistent Formatting**: Use standard Jinja spacing throughout
2. **Template Organization**: Group related templates in directories
3. **Inheritance**: Use base templates to avoid duplication
4. **JavaScript**: Prefer template variables over inline scripts when possible
5. **Validation**: Run template validation before committing changes

## Future Improvements

- Add automated template validation to CI/CD pipeline
- Create template linting rules for development environment
- Implement template performance monitoring
- Add accessibility validation for templates

---

This validation process ensures template quality and maintainability across the Rocket Blog project.