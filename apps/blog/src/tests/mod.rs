// Test modules for all main crate components
pub mod config_tests;
pub mod pool_tests;
pub mod services_tests;
pub mod controllers_tests;
pub mod middleware_tests;
pub mod main_tests;
pub mod integration_tests;
pub mod comprehensive_tests;
pub mod redirect_tests;
pub mod ai_integration_tests;
pub mod seaorm_mock_tests;
pub mod rocket_mock_tests;
pub mod youtube_tests;

// Test modules for additional components
pub mod features_tests;
pub mod guards_tests;
pub mod responders_tests;
pub mod dto_tests;
pub mod types_tests;
pub mod registry_tests;
pub mod edge_case_tests;

// Test fairings and rocket testing utilities
pub mod fairings;

// Test utilities and mocks
pub mod utils;
pub mod mocks;