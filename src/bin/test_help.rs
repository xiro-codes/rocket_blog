use app::database::add_database_args;
use clap::Command;

/// Simple test to show the help message for the CLI arguments
fn main() {
    println!("=== Database CLI Arguments Help ===\n");

    let cmd = Command::new("rocket_blog")
        .version("0.1.0")
        .about("A modern blog application with dual database support")
        .long_about("Rocket Blog supports both PostgreSQL and SQLite databases with automatic fallback capabilities");

    let mut cmd = add_database_args(cmd);
    
    // Print the help message
    cmd.print_help().unwrap();
    println!("\n");
    
    println!("=== Usage Examples ===\n");
    println!("# Default (PostgreSQL with auto-fallback):");
    println!("cargo run --bin blog\n");
    
    println!("# Use SQLite database:");
    println!("cargo run --bin blog -- --database sqlite\n");
    
    println!("# Use in-memory database:");
    println!("cargo run --bin blog -- --database memory\n");
    
    println!("# Enable auto-fallback explicitly:");
    println!("cargo run --bin blog -- --auto-fallback\n");
    
    println!("# PostgreSQL with auto-fallback:");
    println!("cargo run --bin blog -- --database postgres --auto-fallback\n");
}