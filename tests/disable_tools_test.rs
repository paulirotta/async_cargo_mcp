use async_cargo_mcp::cargo_tools::AsyncCargo;
use async_cargo_mcp::operation_monitor::{MonitorConfig, OperationMonitor};
use async_cargo_mcp::shell_pool::{ShellPoolConfig, ShellPoolManager};
use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;

// Helper to create AsyncCargo with disabled tools once implemented
fn create_async_cargo_with_disabled(disabled: &[&str]) -> AsyncCargo {
    let monitor_config = MonitorConfig::with_timeout(Duration::from_secs(30));
    let monitor = Arc::new(OperationMonitor::new(monitor_config));
    let shell_pool_config = ShellPoolConfig::default();
    let shell_pool_manager = Arc::new(ShellPoolManager::new(shell_pool_config));
    let mut disabled_set = HashSet::new();
    for d in disabled {
        disabled_set.insert(d.to_string());
    }
    AsyncCargo::new_with_disabled(monitor, shell_pool_manager, false, disabled_set)
}

#[tokio::test]
async fn disabled_tool_rejected() {
    let async_cargo = create_async_cargo_with_disabled(&["build"]);
    assert!(async_cargo.is_tool_disabled_for_tests("build"));
    let err = async_cargo.ensure_enabled_for_tests("build").unwrap_err();
    let msg = format!("{err:?}");
    assert!(
        msg.contains("tool_disabled"),
        "Expected error message marker, got {msg}"
    );
}

#[tokio::test]
async fn enabled_tool_executes() {
    let async_cargo = create_async_cargo_with_disabled(&["build"]); // sleep still enabled
    assert!(!async_cargo.is_tool_disabled_for_tests("sleep"));
    // ensure_enabled should succeed
    assert!(async_cargo.ensure_enabled_for_tests("sleep").is_ok());
}
