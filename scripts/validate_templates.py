#!/usr/bin/env python3
"""
Rocket Blog Template Validator

Validates all Jinja2 templates for syntax errors, best practices,
and common issues.

Usage:
    python3 scripts/validate_templates.py [--fix] [--verbose]

Options:
    --fix      Apply automatic fixes for common issues
    --verbose  Show detailed output
"""

import os
import re
import sys
import argparse
from pathlib import Path
from typing import List, Dict, Set, Tuple

class TemplateValidator:
    def __init__(self, templates_dir: str, fix_mode: bool = False, verbose: bool = False):
        self.templates_dir = Path(templates_dir)
        self.fix_mode = fix_mode
        self.verbose = verbose
        self.issues = []
        self.fixes_applied = 0
        
    def run(self):
        """Run the template validation process."""
        template_files = list(self.templates_dir.rglob("*.j2"))
        
        if self.verbose:
            print(f"{'Validating' if not self.fix_mode else 'Fixing'} {len(template_files)} template files...\n")
        
        for template_file in template_files:
            self.process_template(template_file)
            
        self.report_results()
        return len([i for i in self.issues if i['type'] == 'error']) == 0
    
    def process_template(self, file_path: Path):
        """Process a single template file."""
        try:
            with open(file_path, 'r', encoding='utf-8') as f:
                content = f.read()
                
            original_content = content
            rel_path = file_path.relative_to(self.templates_dir)
            
            if self.fix_mode:
                # Apply fixes
                content = self._fix_jinja_spacing(content)
                content = self._fix_common_issues(content)
                
                if content != original_content:
                    with open(file_path, 'w', encoding='utf-8') as f:
                        f.write(content)
                    if self.verbose:
                        print(f"✅ Fixed: {rel_path}")
            else:
                # Validate only
                self._validate_jinja_syntax(rel_path, content)
                self._validate_template_inheritance(rel_path, content)
                self._validate_javascript(rel_path, content)
                
        except Exception as e:
            self.issues.append({
                'file': str(rel_path),
                'line': 0,
                'type': 'error',
                'message': f"Failed to process file: {str(e)}"
            })
    
    def _fix_jinja_spacing(self, content: str) -> str:
        """Fix Jinja2 tag spacing issues."""
        fixed_content = content
        
        # Fix {%tag to {% tag
        def fix_opening_space(match):
            self.fixes_applied += 1
            return '{% ' + match.group(1)
        fixed_content = re.sub(r'{%([a-zA-Z])', fix_opening_space, fixed_content)
        
        # Fix tag%} to tag %}
        def fix_closing_space(match):
            self.fixes_applied += 1
            return match.group(1) + ' %}'
        fixed_content = re.sub(r'([a-zA-Z0-9])%}', fix_closing_space, fixed_content)
        
        return fixed_content
    
    def _fix_common_issues(self, content: str) -> str:
        """Fix other common template issues."""
        # Add more fixes as needed
        return content
    
    def _validate_jinja_syntax(self, file_path: Path, content: str):
        """Validate Jinja2 syntax."""
        lines = content.split('\n')
        tag_stack = []
        
        for i, line in enumerate(lines, 1):
            # Check for Jinja tags
            jinja_tags = re.findall(r'{%\s*(\w+).*?%}', line)
            for tag in jinja_tags:
                if tag in ['if', 'for', 'block', 'macro', 'filter', 'call', 'raw', 'with']:
                    tag_stack.append((tag, i))
                elif tag in ['endif', 'endfor', 'endblock', 'endmacro', 'endfilter', 'endcall', 'endraw', 'endwith']:
                    expected = tag[3:]  # Remove 'end' prefix
                    if tag_stack:
                        last_tag, last_line = tag_stack[-1]
                        if last_tag == expected:
                            tag_stack.pop()
                        else:
                            self.issues.append({
                                'file': str(file_path),
                                'line': i,
                                'type': 'error',
                                'message': f"Mismatched {tag} - expected end{last_tag} from line {last_line}"
                            })
                    else:
                        self.issues.append({
                            'file': str(file_path),
                            'line': i,
                            'type': 'error',
                            'message': f"Unexpected {tag} - no matching opening tag"
                        })
            
            # Check spacing issues
            if re.search(r'{%[a-zA-Z]', line) or re.search(r'[a-zA-Z0-9]%}', line):
                self.issues.append({
                    'file': str(file_path),
                    'line': i,
                    'type': 'style',
                    'message': "Jinja tag spacing issue (use spaces: {% tag %})"
                })
        
        # Check for unclosed tags
        for tag, line_num in tag_stack:
            self.issues.append({
                'file': str(file_path),
                'line': line_num,
                'type': 'error',
                'message': f"Unclosed {tag} tag"
            })
    
    def _validate_template_inheritance(self, file_path: Path, content: str):
        """Validate template inheritance patterns."""
        lines = content.split('\n')
        
        extends_count = 0
        block_names = set()
        
        for i, line in enumerate(lines, 1):
            # Check extends statements
            if re.search(r'{%\s*extends\s+', line):
                extends_count += 1
                if extends_count > 1:
                    self.issues.append({
                        'file': str(file_path),
                        'line': i,
                        'type': 'error',
                        'message': "Multiple extends statements found"
                    })
                    
                # Validate extends syntax
                if not re.search(r'{%\s*extends\s+["\'][^"\']+["\']\s*%}', line):
                    self.issues.append({
                        'file': str(file_path),
                        'line': i,
                        'type': 'error',
                        'message': "Invalid extends syntax"
                    })
            
            # Check block definitions
            block_match = re.search(r'{%\s*block\s+(\w+)\s*%}', line)
            if block_match:
                block_name = block_match.group(1)
                if block_name in block_names:
                    self.issues.append({
                        'file': str(file_path),
                        'line': i,
                        'type': 'error',
                        'message': f"Duplicate block definition: {block_name}"
                    })
                block_names.add(block_name)
    
    def _validate_javascript(self, file_path: Path, content: str):
        """Validate JavaScript in templates."""
        lines = content.split('\n')
        
        # Track JavaScript variable declarations at top level
        top_level_vars = {}
        in_script = False
        brace_depth = 0
        
        for i, line in enumerate(lines, 1):
            if '<script>' in line or '<script ' in line:
                in_script = True
                brace_depth = 0
                continue
            elif '</script>' in line:
                in_script = False
                continue
                
            if in_script:
                # Track brace depth to determine scope
                brace_depth += line.count('{') - line.count('}')
                
                # Only check for duplicates at top level (brace_depth == 0)
                if brace_depth == 0:
                    var_matches = re.findall(r'(const|let|var)\s+(\w+)\s*=', line)
                    for var_type, var_name in var_matches:
                        if var_name in top_level_vars:
                            self.issues.append({
                                'file': str(file_path),
                                'line': i,
                                'type': 'error',
                                'message': f"Duplicate JavaScript variable '{var_name}' (first declared at line {top_level_vars[var_name]})"
                            })
                        else:
                            top_level_vars[var_name] = i
    
    def report_results(self):
        """Generate a report of validation results."""
        if not self.issues and self.fix_mode:
            if self.fixes_applied > 0:
                print(f"✅ Applied {self.fixes_applied} fixes successfully!")
            else:
                print("✅ No fixes needed - templates are already valid!")
            return
        
        if not self.issues and not self.fix_mode:
            print("✅ All templates are valid!")
            return
            
        # Group by severity
        errors = [i for i in self.issues if i['type'] == 'error']
        warnings = [i for i in self.issues if i['type'] == 'warning']
        style_issues = [i for i in self.issues if i['type'] == 'style']
        
        if errors:
            print(f"❌ Found {len(errors)} critical errors:")
            self._print_issues(errors)
        
        if warnings:
            print(f"⚠️  Found {len(warnings)} warnings:")
            self._print_issues(warnings)
            
        if style_issues:
            print(f"💅 Found {len(style_issues)} style issues:")
            self._print_issues(style_issues)
        
        if errors:
            print("\n❌ Validation failed - fix critical errors before continuing")
        elif warnings or style_issues:
            print("\n⚠️  Validation passed with warnings - consider running with --fix")
        else:
            print("\n✅ Validation passed!")
    
    def _print_issues(self, issues):
        """Print a list of issues grouped by file."""
        by_file = {}
        for issue in issues:
            file_name = issue['file']
            if file_name not in by_file:
                by_file[file_name] = []
            by_file[file_name].append(issue)
        
        for file_name, file_issues in sorted(by_file.items()):
            print(f"\n  📄 {file_name}:")
            for issue in sorted(file_issues, key=lambda x: x['line']):
                print(f"    Line {issue['line']}: {issue['message']}")

def main():
    parser = argparse.ArgumentParser(description='Validate Rocket Blog templates')
    parser.add_argument('--fix', action='store_true', help='Apply automatic fixes')
    parser.add_argument('--verbose', action='store_true', help='Verbose output')
    args = parser.parse_args()
    
    # Determine templates directory
    script_dir = Path(__file__).parent
    if script_dir.name == 'scripts':
        templates_dir = script_dir.parent / 'templates'
    else:
        templates_dir = script_dir / 'templates'
    
    if not templates_dir.exists():
        print(f"❌ Templates directory not found: {templates_dir}")
        sys.exit(1)
    
    validator = TemplateValidator(str(templates_dir), args.fix, args.verbose)
    success = validator.run()
    
    sys.exit(0 if success else 1)

if __name__ == "__main__":
    main()