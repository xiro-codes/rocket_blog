use app::database::{DatabaseConfig, DatabaseType, add_database_args};
use clap::Command;

/// Simple test to verify CLI argument parsing works correctly
#[tokio::main]
async fn main() {
    println!("Testing database CLI argument parsing...\n");

    // Test 1: Default configuration
    println!("1. Testing default configuration:");
    let args = vec!["test_app"];
    let cmd = Command::new("test_app");
    let cmd = add_database_args(cmd);
    let matches = cmd.try_get_matches_from(args).unwrap();
    let config = DatabaseConfig::from_args(&matches);
    
    println!("   Database type: {:?}", config.db_type);
    println!("   Auto-fallback: {}", config.auto_fallback);
    println!("   URL: {}", config.url);
    println!();

    // Test 2: SQLite database
    println!("2. Testing SQLite database selection:");
    let args = vec!["test_app", "--database", "sqlite"];
    let cmd = Command::new("test_app");
    let cmd = add_database_args(cmd);
    let matches = cmd.try_get_matches_from(args).unwrap();
    let config = DatabaseConfig::from_args(&matches);
    
    println!("   Database type: {:?}", config.db_type);
    println!("   Auto-fallback: {}", config.auto_fallback);
    println!("   URL: {}", config.url);
    println!();

    // Test 3: Memory database
    println!("3. Testing memory database selection:");
    let args = vec!["test_app", "--database", "memory"];
    let cmd = Command::new("test_app");
    let cmd = add_database_args(cmd);
    let matches = cmd.try_get_matches_from(args).unwrap();
    let config = DatabaseConfig::from_args(&matches);
    
    println!("   Database type: {:?}", config.db_type);
    println!("   Auto-fallback: {}", config.auto_fallback);
    println!("   URL: {}", config.url);
    println!("   Is memory database: {}", config.is_memory_database());
    println!();

    // Test 4: Auto-fallback enabled
    println!("4. Testing auto-fallback flag:");
    let args = vec!["test_app", "--auto-fallback"];
    let cmd = Command::new("test_app");
    let cmd = add_database_args(cmd);
    let matches = cmd.try_get_matches_from(args).unwrap();
    let config = DatabaseConfig::from_args(&matches);
    
    println!("   Database type: {:?}", config.db_type);
    println!("   Auto-fallback: {}", config.auto_fallback);
    println!("   URL: {}", config.url);
    println!();

    // Test 5: Database type conversion
    println!("5. Testing database type string conversion:");
    let test_cases = vec![
        "postgres", "postgresql", "pg", "sqlite", "memory", "sqlite-memory", "invalid"
    ];
    
    for case in test_cases {
        let db_type = DatabaseType::from_str(case);
        match db_type {
            Some(t) => println!("   '{}' -> {:?} ({})", case, t, t.display_name()),
            None => println!("   '{}' -> None (invalid)", case),
        }
    }
    println!();

    // Test 6: Default with fallback
    println!("6. Testing default configuration with fallback:");
    let config = DatabaseConfig::default_with_fallback();
    println!("   Database type: {:?}", config.db_type);
    println!("   Auto-fallback: {}", config.auto_fallback);
    println!("   URL: {}", config.url);
    println!();

    println!("✅ All CLI argument parsing tests completed successfully!");
}