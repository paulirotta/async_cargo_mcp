//! Test the new server-side enhancements
//!
//! This test verifies:
//! 1. CLI flag for enable_wait works
//! 2. Status tool functions properly
//! 3. Automatic result push system works
//! 4. Wait tool is properly disabled by default

use async_cargo_mcp::{
    cargo_tools::AsyncCargo,
    operation_monitor::{MonitorConfig, OperationMonitor},
    shell_pool::{ShellPoolConfig, ShellPoolManager},
};
use std::collections::HashSet;
use std::sync::Arc;

#[tokio::test]
async fn test_enable_wait_flag_functionality() {
    let monitor_config = MonitorConfig::default();
    let monitor = Arc::new(OperationMonitor::new(monitor_config));
    let shell_pool_config = ShellPoolConfig::default();
    let shell_pool_manager = Arc::new(ShellPoolManager::new(shell_pool_config));

    // Test with wait disabled (default)
    let cargo_disabled = AsyncCargo::new_with_config_and_disabled(
        monitor.clone(),
        shell_pool_manager.clone(),
        false, // synchronous mode
        false, // enable_wait = false
        HashSet::new(),
    );

    // Test with wait enabled
    let cargo_enabled = AsyncCargo::new_with_config_and_disabled(
        monitor.clone(),
        shell_pool_manager.clone(),
        false, // synchronous mode
        true,  // enable_wait = true
        HashSet::new(),
    );

    // Both should be valid instances
    assert!(format!("{:?}", cargo_disabled).contains("enable_wait: false"));
    assert!(format!("{:?}", cargo_enabled).contains("enable_wait: true"));
}

#[tokio::test]
async fn test_status_tool_request_parsing() {
    use async_cargo_mcp::cargo_tools::StatusRequest;
    use serde_json::json;

    // Test parsing different status request formats
    let json_data = json!({
        "operation_id": "op_build_123",
        "working_directory": "/test/path",
        "state_filter": "active"
    });

    let parsed: Result<StatusRequest, _> = serde_json::from_value(json_data);
    assert!(parsed.is_ok());

    let request = parsed.unwrap();
    assert_eq!(request.operation_id, Some("op_build_123".to_string()));
    assert_eq!(request.working_directory, Some("/test/path".to_string()));
    assert_eq!(request.state_filter, Some("active".to_string()));

    // Test minimal request
    let minimal_json = json!({});
    let minimal_parsed: Result<StatusRequest, _> = serde_json::from_value(minimal_json);
    assert!(minimal_parsed.is_ok());

    let minimal_request = minimal_parsed.unwrap();
    assert_eq!(minimal_request.operation_id, None);
    assert_eq!(minimal_request.working_directory, None);
    assert_eq!(minimal_request.state_filter, None);
}

#[tokio::test]
async fn test_progress_update_final_result() {
    use async_cargo_mcp::callback_system::ProgressUpdate;

    let final_result = ProgressUpdate::FinalResult {
        operation_id: "op_test_456".to_string(),
        command: "cargo build".to_string(),
        description: "Test build operation".to_string(),
        working_directory: "/test/workspace".to_string(),
        success: true,
        duration_ms: 5000,
        full_output: "Finished dev [unoptimized + debuginfo] target(s) in 4.50s".to_string(),
    };

    // Test display formatting
    let display_str = format!("{}", final_result);
    assert!(display_str.contains("op_test_456"));
    assert!(display_str.contains("✅ COMPLETED"));
    assert!(display_str.contains("cargo build"));
    assert!(display_str.contains("Finished dev"));

    // Test failed result
    let failed_result = ProgressUpdate::FinalResult {
        operation_id: "op_test_789".to_string(),
        command: "cargo test".to_string(),
        description: "Test operation".to_string(),
        working_directory: "/test".to_string(),
        success: false,
        duration_ms: 2000,
        full_output: "test failed with errors".to_string(),
    };

    let failed_display = format!("{}", failed_result);
    assert!(failed_display.contains("❌ FAILED"));
    assert!(failed_display.contains("test failed with errors"));
}

#[tokio::test]
async fn test_comprehensive_final_result_creation() {
    use async_cargo_mcp::cargo_tools::AsyncCargo;

    // Test success case
    let success_result = Ok("Build completed successfully".to_string());
    let final_update = AsyncCargo::create_final_result_update(
        "op_build_999",
        "cargo build",
        "Building test project",
        "/test/workspace",
        &success_result,
        3000,
    );

    if let async_cargo_mcp::callback_system::ProgressUpdate::FinalResult {
        operation_id,
        success,
        full_output,
        duration_ms,
        ..
    } = final_update
    {
        assert_eq!(operation_id, "op_build_999");
        assert!(success);
        assert_eq!(full_output, "Build completed successfully");
        assert_eq!(duration_ms, 3000);
    } else {
        panic!("Expected FinalResult variant");
    }

    // Test failure case
    let error_result = Err("Build failed with compilation errors".to_string());
    let final_error_update = AsyncCargo::create_final_result_update(
        "op_build_998",
        "cargo build",
        "Building test project",
        "/test/workspace",
        &error_result,
        1500,
    );

    if let async_cargo_mcp::callback_system::ProgressUpdate::FinalResult {
        operation_id,
        success,
        full_output,
        duration_ms,
        ..
    } = final_error_update
    {
        assert_eq!(operation_id, "op_build_998");
        assert!(!success);
        assert_eq!(full_output, "Build failed with compilation errors");
        assert_eq!(duration_ms, 1500);
    } else {
        panic!("Expected FinalResult variant");
    }
}
