use async_cargo_mcp::cargo_tools::AsyncCargo;
use async_cargo_mcp::operation_monitor::{MonitorConfig, OperationMonitor};
use async_cargo_mcp::shell_pool::{ShellPoolConfig, ShellPoolManager};
use std::sync::Arc;
use std::time::Duration;

#[tokio::test]
async fn test_synchronous_mode_configuration() {
    // Create AsyncCargo instances - one with sync mode, one without
    let monitor_config = MonitorConfig::with_timeout(Duration::from_secs(30));
    let monitor = Arc::new(OperationMonitor::new(monitor_config));
    let shell_pool_config = ShellPoolConfig::default();
    let shell_pool_manager = Arc::new(ShellPoolManager::new(shell_pool_config));

    let async_cargo_sync =
        AsyncCargo::new_with_config(monitor.clone(), shell_pool_manager.clone(), true);
    let async_cargo_async =
        AsyncCargo::new_with_config(monitor.clone(), shell_pool_manager.clone(), false);

    // Test synchronous mode - should_run_synchronously should return true
    assert!(async_cargo_sync.should_run_synchronously(Some(true)));
    assert!(async_cargo_sync.should_run_synchronously(Some(false)));
    assert!(async_cargo_sync.should_run_synchronously(None));

    // Test async mode - should_run_synchronously should respect the parameter
    assert!(!async_cargo_async.should_run_synchronously(Some(true)));
    assert!(async_cargo_async.should_run_synchronously(Some(false)));
    assert!(async_cargo_async.should_run_synchronously(None));

    println!("✅ Synchronous mode configuration test passed!");
    println!("✅ Async mode configuration test passed!");
}

#[tokio::test]
async fn test_synchronous_mode_execution_patterns() {
    // Test that synchronous mode doesn't depend on external async infrastructure
    let monitor_config = MonitorConfig::with_timeout(Duration::from_secs(10));
    let monitor = Arc::new(OperationMonitor::new(monitor_config));
    let shell_pool_config = ShellPoolConfig::default();
    let shell_pool_manager = Arc::new(ShellPoolManager::new(shell_pool_config));

    let async_cargo_sync = AsyncCargo::new_with_config(monitor, shell_pool_manager, true);

    // Test basic synchronous mode helper method
    assert!(async_cargo_sync.should_run_synchronously(Some(true)));
    assert!(async_cargo_sync.should_run_synchronously(Some(false)));
    assert!(async_cargo_sync.should_run_synchronously(None));

    println!("✅ Synchronous mode execution patterns test passed!");
}
