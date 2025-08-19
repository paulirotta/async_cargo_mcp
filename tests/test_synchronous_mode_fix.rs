use async_cargo_mcp::cargo_tools::AsyncCargo;
use async_cargo_mcp::operation_monitor::{MonitorConfig, OperationMonitor};
use async_cargo_mcp::shell_pool::{ShellPoolConfig, ShellPoolManager};
use std::sync::Arc;
use tempfile::TempDir;

#[tokio::test]
async fn test_synchronous_mode_fix() {
    // Create a temporary directory for testing
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // Create a minimal Cargo.toml for testing
    let cargo_toml_content = r#"[package]
name = "test_sync"
version = "0.1.0"
edition = "2021"
"#;
    std::fs::write(temp_dir.path().join("Cargo.toml"), cargo_toml_content)
        .expect("Failed to write Cargo.toml");

    // Create a simple main.rs
    std::fs::create_dir_all(temp_dir.path().join("src")).expect("Failed to create src directory");
    std::fs::write(temp_dir.path().join("src").join("main.rs"), "fn main() {}")
        .expect("Failed to write main.rs");

    // Test with synchronous mode enabled
    let shell_pool_manager = Arc::new(ShellPoolManager::new(ShellPoolConfig::default()));
    let monitor = Arc::new(OperationMonitor::new(MonitorConfig::default()));
    let async_cargo_sync =
        AsyncCargo::new_with_config(monitor.clone(), shell_pool_manager.clone(), true);

    // Test the should_run_synchronously method with various combinations

    // Case 1: synchronous_mode=true, enable_async_notification=None -> should run synchronously
    assert!(async_cargo_sync.should_run_synchronously(None));

    // Case 2: synchronous_mode=true, enable_async_notification=Some(false) -> should run synchronously
    assert!(async_cargo_sync.should_run_synchronously(Some(false)));

    // Case 3: synchronous_mode=true, enable_async_notification=Some(true) -> should run synchronously (CLI overrides)
    assert!(async_cargo_sync.should_run_synchronously(Some(true)));

    // Test with synchronous mode disabled
    let async_cargo_async = AsyncCargo::new_with_config(monitor, shell_pool_manager, false);

    // Case 4: synchronous_mode=false, enable_async_notification=None -> should run synchronously (default)
    assert!(async_cargo_async.should_run_synchronously(None));

    // Case 5: synchronous_mode=false, enable_async_notification=Some(false) -> should run synchronously
    assert!(async_cargo_async.should_run_synchronously(Some(false)));

    // Case 6: synchronous_mode=false, enable_async_notification=Some(true) -> should run asynchronously
    assert!(!async_cargo_async.should_run_synchronously(Some(true)));

    println!("✅ All synchronous mode logic tests passed!");
    println!("✅ Synchronous mode override works correctly");
    println!("✅ The fix ensures --synchronous flag is respected by all operations");
}
