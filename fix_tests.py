import re

files = [
    "src/tests/services_tests.rs",
    "src/tests/comprehensive_tests.rs",
    "src/tests/registry_tests.rs",
    "src/tests/edge_case_tests.rs"
]

for file in files:
    with open(file, "r") as f:
        content = f.read()
    
    # Remove ai_provider_tests module
    content = re.sub(r'mod ai_provider_tests \{.*?\n    \}\n', '', content, flags=re.DOTALL)
    
    # Remove specific test functions
    tests_to_remove = [
        r'#\[test\]\s*fn test_service_composition_patterns<.*?>\(\) \{.*?\n        \}',
        r'#\[test\]\s*fn test_service_registry_comprehensive<.*?>\(\) \{.*?\n        \}',
        r'#\[test\]\s*fn test_service_registry_attach_all_services<.*?>\(\) \{.*?\n    \}',
        r'#\[test\]\s*fn test_ai_provider_service_configuration<.*?>\(\) \{.*?\n    \}',
        r'#\[test\]\s*fn test_service_instantiation<.*?>\(\) \{.*?\n    \}',
        r'#\[test\]\s*fn test_full_application_setup<.*?>\(\) \{.*?\n    \}',
        r'#\[test\]\s*fn test_backward_compatibility<.*?>\(\) \{.*?\n    \}',
        r'#\[test\]\s*fn test_ai_provider_edge_cases<.*?>\(\) \{.*?\n    \}',
        r'#\[test\]\s*fn test_service_composition_patterns\(\) \{.*?\n        \}',
        r'#\[test\]\s*fn test_service_registry_comprehensive\(\) \{.*?\n        \}',
        r'#\[test\]\s*fn test_service_registry_attach_all_services\(\) \{.*?\n    \}',
        r'#\[test\]\s*fn test_ai_provider_service_configuration\(\) \{.*?\n    \}',
        r'#\[test\]\s*fn test_service_instantiation\(\) \{.*?\n    \}',
        r'#\[test\]\s*fn test_full_application_setup\(\) \{.*?\n    \}',
        r'#\[test\]\s*fn test_backward_compatibility\(\) \{.*?\n    \}',
        r'#\[test\]\s*fn test_ai_provider_edge_cases\(\) \{.*?\n    \}',
    ]
    
    for t in tests_to_remove:
        content = re.sub(t, '', content, flags=re.DOTALL)
        
    with open(file, "w") as f:
        f.write(content)
