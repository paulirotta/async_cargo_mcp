//! Test to verify that wait operations return complete cargo output instead of brief summaries
//!
//! This test specifically targets the issue where wait operations return messages like:
//! "Operation 'op_xxx' failed (no detailed error output available)"
//! instead of the full cargo build output with specific compile errors.

use async_cargo_mcp::operation_monitor::{MonitorConfig, OperationMonitor};
use std::sync::Arc;
use std::time::Duration;

#[tokio::test]
async fn test_wait_returns_full_stored_error_details() {
    // Set up operation monitor
    let monitor_config = MonitorConfig::default();
    let monitor = Arc::new(OperationMonitor::new(monitor_config));

    // Register an operation manually
    let operation_id = monitor
        .register_operation(
            "cargo build".to_string(),
            "Test build with expected failure".to_string(),
            Some(Duration::from_secs(300)),
            Some("/tmp/test".to_string()),
        )
        .await;

    // Start the operation
    monitor
        .start_operation(&operation_id)
        .await
        .expect("Failed to start operation");

    // Simulate a detailed build error (like what cargo build would produce)
    let detailed_error = r#"- Build failed in /tmp/test.
Error: error[E0425]: cannot find value `undefined_variable` in this scope
   --> src/main.rs:3:20
    |
3   |     println!("{}", undefined_variable);
    |                    ^^^^^^^^^^^^^^^^^^ not found in this scope

error: aborting due to previous error

For more information about this error, try `rustc --explain E0425`.
Output: Compiling test_project v0.1.0 (/tmp/test)"#;

    // Complete the operation with the detailed error
    monitor
        .complete_operation(&operation_id, Err(detailed_error.to_string()))
        .await
        .expect("Failed to complete operation");

    // Now test that wait returns the full detailed error
    let wait_result = monitor.wait_for_operation(&operation_id).await;

    assert!(wait_result.is_ok(), "wait_for_operation should never fail");

    let operations = wait_result.unwrap();
    assert_eq!(operations.len(), 1, "Should return exactly one operation");

    let operation = &operations[0];
    assert_eq!(operation.id, operation_id);

    // Check that the result contains the full detailed error
    match &operation.result {
        Some(Err(stored_error)) => {
            assert_eq!(
                stored_error, detailed_error,
                "Stored error should match the detailed error that was provided.\nExpected: {}\nActual: {}",
                detailed_error, stored_error
            );

            assert!(
                stored_error.contains("cannot find value `undefined_variable`"),
                "Should contain specific compilation error details"
            );

            assert!(
                stored_error.contains("error[E0425]"),
                "Should contain rustc error code"
            );

            assert!(
                stored_error.len() > 200,
                "Error should be detailed, not brief. Length: {}",
                stored_error.len()
            );
        }
        _ => panic!(
            "Operation result should be Some(Err(detailed_error)), got: {:?}",
            operation.result
        ),
    }

    println!("✓ Wait operation correctly returns full stored error details");
    println!(
        "📝 Stored error length: {} characters",
        operation
            .result
            .as_ref()
            .unwrap()
            .as_ref()
            .unwrap_err()
            .len()
    );
}
