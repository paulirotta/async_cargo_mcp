//! Comprehensive test suite for wait operation race conditions and edge cases
//!
//! This test file specifically targets potential race conditions where LLMs could
//! wait indefinitely for operations that are already complete, focusing on:
//! - Cleanup timing races
//! - Timeout boundary conditions  
//! - Concurrent operation scenarios
//! - Configuration edge cases

use anyhow::Result;
use std::time::{Duration, Instant};
use tokio::time::sleep;

mod common;
use common::test_project::create_basic_project;
use rmcp::{
    ServiceExt,
    model::CallToolRequestParam,
    object,
    transport::{ConfigureCommandExt, TokioChildProcess},
};
use tokio::process::Command;

fn extract_operation_id(s: &str) -> Option<String> {
    if let Some(start) = s.find("op_") {
        let rest = &s[start..];
        let mut id = String::new();
        for ch in rest.chars() {
            if ch.is_alphanumeric() || ch == '_' || ch == '-' {
                id.push(ch);
            } else {
                break;
            }
        }
        if id.starts_with("op_") {
            return Some(id);
        }
    }
    None
}

/// Test Race Condition A1: Operation completes → cleanup removes from operations map → wait called
/// This tests the "vanishing operation" scenario where cleanup happens between completion and wait
#[tokio::test]
async fn test_race_a1_vanishing_operation_after_cleanup() -> Result<()> {
    let temp = create_basic_project().await?;
    let project_path = temp.path().to_str().unwrap().to_string();

    let client = ()
        .serve(TokioChildProcess::new(Command::new("cargo").configure(
            |cmd| {
                cmd.arg("run")
                    .arg("--bin")
                    .arg("async_cargo_mcp")
                    .arg("--")
                    .arg("--enable-wait");
            },
        ))?)
        .await?;

    // Start a very quick operation
    let check_result = client
        .call_tool(CallToolRequestParam {
            name: "check".into(),
            arguments: Some(object!({
                "working_directory": project_path,
                "enable_async_notification": true
            })),
        })
        .await?;

    let first_text = format!("{:?}", check_result.content);
    let op_id = extract_operation_id(&first_text).expect("operation id should be present");

    // Let the operation complete by waiting a reasonable time
    sleep(Duration::from_millis(2000)).await;

    // Force multiple rapid wait calls to try to trigger cleanup race
    let mut wait_results = Vec::new();
    for i in 0..5 {
        println!("Wait attempt {}: {}", i + 1, op_id);
        let start_time = Instant::now();

        let wait_result = client
            .call_tool(CallToolRequestParam {
                name: "wait".into(),
                arguments: Some(object!({ "operation_ids": [op_id.clone()] })),
            })
            .await;

        let elapsed = start_time.elapsed();

        match wait_result {
            Ok(result) => {
                let wait_text = format!("{:?}", result.content);
                wait_results.push((elapsed, wait_text.clone()));

                // Should never take close to the full 300s timeout for a completed operation
                assert!(
                    elapsed.as_secs() < 10,
                    "Wait #{} took too long ({:?}) for completed operation, possible race condition: {}",
                    i + 1,
                    elapsed,
                    wait_text
                );

                // Should contain completion status or helpful error message
                assert!(
                    wait_text.contains("OPERATION COMPLETED")
                        || wait_text.contains("OPERATION FAILED")
                        || wait_text.contains("No operation found"),
                    "Wait #{} result should contain completion status or helpful error: {}",
                    i + 1,
                    wait_text
                );
            }
            Err(e) => {
                panic!(
                    "Wait #{} failed with error (possible race condition): {:?}",
                    i + 1,
                    e
                );
            }
        }

        // Small delay between attempts to vary timing
        sleep(Duration::from_millis(100)).await;
    }

    // All wait attempts should be consistent - either all find the operation or all report it missing
    let first_result_type = if wait_results[0].1.contains("OPERATION COMPLETED")
        || wait_results[0].1.contains("OPERATION FAILED")
    {
        "found"
    } else {
        "missing"
    };

    for (i, (elapsed, result)) in wait_results.iter().enumerate() {
        let result_type =
            if result.contains("OPERATION COMPLETED") || result.contains("OPERATION FAILED") {
                "found"
            } else {
                "missing"
            };

        // All results should be consistent (no flip-flopping due to race conditions)
        assert_eq!(
            result_type,
            first_result_type,
            "Wait #{} result type '{}' differs from first wait '{}' - possible race condition. Result: {}",
            i + 1,
            result_type,
            first_result_type,
            result
        );

        // All waits should be reasonably fast
        assert!(
            elapsed.as_secs() < 5,
            "Wait #{} took {:?}, indicating possible race/hang condition",
            i + 1,
            elapsed
        );
    }

    let _ = client.cancel().await;
    Ok(())
}

/// Test Race Condition A2: Operation in completion_history → cleanup removes from completion_history → wait called  
/// This tests cleanup of completion history while wait operations are pending
#[tokio::test]
async fn test_race_a2_completion_history_cleanup_during_wait() -> Result<()> {
    let temp = create_basic_project().await?;
    let project_path = temp.path().to_str().unwrap().to_string();

    let client = ()
        .serve(TokioChildProcess::new(Command::new("cargo").configure(
            |cmd| {
                cmd.arg("run")
                    .arg("--bin")
                    .arg("async_cargo_mcp")
                    .arg("--")
                    .arg("--enable-wait");
            },
        ))?)
        .await?;

    // Start multiple quick operations to potentially fill completion history
    let mut operation_ids = Vec::new();

    for i in 0..3 {
        let check_result = client
            .call_tool(CallToolRequestParam {
                name: "check".into(),
                arguments: Some(object!({
                    "working_directory": project_path.clone(),
                    "enable_async_notification": true
                })),
            })
            .await?;

        let text = format!("{:?}", check_result.content);
        if let Some(op_id) = extract_operation_id(&text) {
            operation_ids.push(op_id);
            println!(
                "Started operation {}: {}",
                i + 1,
                operation_ids.last().unwrap()
            );
        }
    }

    assert_eq!(operation_ids.len(), 3, "Should have started 3 operations");

    // Let all operations complete
    sleep(Duration::from_millis(3000)).await;

    // Now test waiting for all operations with different timing patterns
    for (i, op_id) in operation_ids.iter().enumerate() {
        println!("Testing wait for operation {}: {}", i + 1, op_id);

        let start_time = Instant::now();

        let wait_result = client
            .call_tool(CallToolRequestParam {
                name: "wait".into(),
                arguments: Some(object!({ "operation_ids": [op_id.clone()] })),
            })
            .await?;

        let elapsed = start_time.elapsed();
        let wait_text = format!("{:?}", wait_result.content);

        // Should not hang for long periods
        assert!(
            elapsed.as_secs() < 15,
            "Wait for operation {} took {:?}, possible cleanup race condition: {}",
            op_id,
            elapsed,
            wait_text
        );

        // Should get meaningful result (either completion or helpful error)
        assert!(
            wait_text.contains("OPERATION COMPLETED")
                || wait_text.contains("OPERATION FAILED")
                || wait_text.contains("No operation found"),
            "Wait for operation {} should return meaningful result: {}",
            op_id,
            wait_text
        );

        // Add some delay to vary timing relative to any background cleanup
        sleep(Duration::from_millis(200)).await;
    }

    let _ = client.cancel().await;
    Ok(())
}

/// Test Race Condition B1: Wait timeout vs operation timeout boundary condition
/// This tests the scenario where both timeouts could trigger simultaneously
#[tokio::test]
async fn test_race_b1_double_timeout_boundary() -> Result<()> {
    let client = ()
        .serve(TokioChildProcess::new(Command::new("cargo").configure(
            |cmd| {
                cmd.arg("run")
                    .arg("--bin")
                    .arg("async_cargo_mcp")
                    .arg("--")
                    .arg("--enable-wait");
            },
        ))?)
        .await?;

    // Start a deterministic long-running operation that should complete before any timeout
    let sleep_result = client
        .call_tool(CallToolRequestParam {
            name: "sleep".into(),
            arguments: Some(object!({
                "duration_ms": 2000, // 2 seconds - well under any timeout
                "operation_id": "op_timeout_boundary_test",
                "enable_async_notification": true
            })),
        })
        .await?;

    let sleep_text = format!("{:?}", sleep_result.content);
    println!("Sleep operation started: {}", sleep_text);

    // Wait immediately - this should complete quickly since operation is only 2s
    let start_time = Instant::now();

    let wait_result = client
        .call_tool(CallToolRequestParam {
            name: "wait".into(),
            arguments: Some(object!({ "operation_ids": ["op_timeout_boundary_test"] })),
        })
        .await?;

    let elapsed = start_time.elapsed();
    let wait_text = format!("{:?}", wait_result.content);

    // Should complete in reasonable time (operation + overhead)
    assert!(
        elapsed.as_secs() < 10,
        "Wait should complete quickly for 2s operation, but took {:?}: {}",
        elapsed,
        wait_text
    );

    // Should show successful completion
    assert!(
        wait_text.contains("OPERATION COMPLETED"),
        "Operation should complete successfully: {}",
        wait_text
    );

    let _ = client.cancel().await;
    Ok(())
}

/// Test Race Condition B2: Very short-lived operations that complete and get cleaned up before wait timeout
#[tokio::test]
async fn test_race_b2_short_lived_operation_cleanup() -> Result<()> {
    let client = ()
        .serve(TokioChildProcess::new(Command::new("cargo").configure(
            |cmd| {
                cmd.arg("run")
                    .arg("--bin")
                    .arg("async_cargo_mcp")
                    .arg("--")
                    .arg("--enable-wait");
            },
        ))?)
        .await?;

    // Start several very short operations in rapid succession
    let mut operation_ids = Vec::new();

    for i in 0..5 {
        let _sleep_result = client
            .call_tool(CallToolRequestParam {
                name: "sleep".into(),
                arguments: Some(object!({
                    "duration_ms": 100, // Very short - 100ms
                    "operation_id": format!("op_short_{}", i),
                    "enable_async_notification": true
                })),
            })
            .await?;

        operation_ids.push(format!("op_short_{}", i));
        println!("Started short operation {}: op_short_{}", i, i);

        // Very brief delay between starts
        sleep(Duration::from_millis(10)).await;
    }

    // Let all operations complete
    sleep(Duration::from_millis(500)).await;

    // Now wait for them with various delays to test cleanup timing
    for (i, op_id) in operation_ids.iter().enumerate() {
        // Add increasing delay before each wait to vary cleanup timing
        sleep(Duration::from_millis(i as u64 * 100)).await;

        println!("Waiting for operation {}: {}", i, op_id);
        let start_time = Instant::now();

        let wait_result = client
            .call_tool(CallToolRequestParam {
                name: "wait".into(),
                arguments: Some(object!({ "operation_ids": [op_id.clone()] })),
            })
            .await?;

        let elapsed = start_time.elapsed();
        let wait_text = format!("{:?}", wait_result.content);

        // Should not hang waiting for already-completed short operations
        assert!(
            elapsed.as_secs() < 5,
            "Wait for short operation {} took {:?}, possible cleanup race: {}",
            op_id,
            elapsed,
            wait_text
        );

        // Should get either completion or helpful "not found" message
        assert!(
            wait_text.contains("OPERATION COMPLETED") || wait_text.contains("No operation found"),
            "Wait for short operation {} should return completion or not found: {}",
            op_id,
            wait_text
        );
    }

    let _ = client.cancel().await;
    Ok(())
}

/// Test Race Condition C1: Multiple concurrent waits for same operation ID
#[tokio::test]
async fn test_race_c1_concurrent_waits_same_operation() -> Result<()> {
    let temp = create_basic_project().await?;
    let project_path = temp.path().to_str().unwrap().to_string();

    let client = ()
        .serve(TokioChildProcess::new(Command::new("cargo").configure(
            |cmd| {
                cmd.arg("run")
                    .arg("--bin")
                    .arg("async_cargo_mcp")
                    .arg("--")
                    .arg("--enable-wait");
            },
        ))?)
        .await?;

    // Start one operation
    let build_result = client
        .call_tool(CallToolRequestParam {
            name: "build".into(),
            arguments: Some(object!({
                "working_directory": project_path,
                "enable_async_notification": true
            })),
        })
        .await?;

    let text = format!("{:?}", build_result.content);
    let op_id = extract_operation_id(&text).expect("operation id should be present");
    println!("Started build operation: {}", op_id);

    // Launch multiple concurrent wait operations for the SAME operation ID
    let num_concurrent_waits = 5;
    let mut wait_handles = Vec::new();

    for i in 0..num_concurrent_waits {
        let op_id_clone = op_id.clone();
        let client_clone = client.clone();

        let handle = tokio::spawn(async move {
            println!("Starting concurrent wait {}: {}", i + 1, op_id_clone);
            let start_time = Instant::now();

            let wait_result = client_clone
                .call_tool(CallToolRequestParam {
                    name: "wait".into(),
                    arguments: Some(object!({ "operation_ids": [op_id_clone.clone()] })),
                })
                .await;

            let elapsed = start_time.elapsed();
            (i + 1, elapsed, wait_result, op_id_clone)
        });

        wait_handles.push(handle);
    }

    // Collect all results
    let mut results = Vec::new();
    for handle in wait_handles {
        let result = handle
            .await
            .map_err(|e| anyhow::anyhow!("Join error: {}", e))?;
        results.push(result);
    }

    // Analyze results
    for (wait_num, elapsed, wait_result, op_id) in results {
        match wait_result {
            Ok(result) => {
                let wait_text = format!("{:?}", result.content);

                // Should not hang for extended periods
                assert!(
                    elapsed.as_secs() < 30,
                    "Concurrent wait {} for {} took {:?}, possible resource contention: {}",
                    wait_num,
                    op_id,
                    elapsed,
                    wait_text
                );

                // Should get completion status
                assert!(
                    wait_text.contains("OPERATION COMPLETED")
                        || wait_text.contains("OPERATION FAILED"),
                    "Concurrent wait {} should show completion: {}",
                    wait_num,
                    wait_text
                );

                println!("Concurrent wait {} completed in {:?}", wait_num, elapsed);
            }
            Err(e) => {
                panic!("Concurrent wait {} failed: {:?}", wait_num, e);
            }
        }
    }

    let _ = client.cancel().await;
    Ok(())
}

/// Test Race Condition C3: tokio::spawn join errors in wait operation
#[tokio::test]
async fn test_race_c3_wait_join_error_handling() -> Result<()> {
    let client = ()
        .serve(TokioChildProcess::new(Command::new("cargo").configure(
            |cmd| {
                cmd.arg("run")
                    .arg("--bin")
                    .arg("async_cargo_mcp")
                    .arg("--")
                    .arg("--enable-wait");
            },
        ))?)
        .await?;

    // Try to wait for multiple operations where some don't exist
    // This tests the join error handling path in the wait implementation
    let wait_result = client
        .call_tool(CallToolRequestParam {
            name: "wait".into(),
            arguments: Some(object!({
                "operation_ids": [
                    "op_nonexistent_1",
                    "op_nonexistent_2",
                    "op_nonexistent_3",
                    "op_nonexistent_4",
                    "op_nonexistent_5"
                ]
            })),
        })
        .await?;

    let wait_text = format!("{:?}", wait_result.content);

    // Should handle multiple non-existent operations gracefully
    assert!(
        wait_text.contains("No operation found"),
        "Should provide helpful messages for non-existent operations: {}",
        wait_text
    );

    // Should not contain join errors or panics
    assert!(
        !wait_text.contains("Join error") && !wait_text.contains("panic"),
        "Should not expose internal join errors to user: {}",
        wait_text
    );

    let _ = client.cancel().await;
    Ok(())
}

/// Test stress scenario: Many rapid operations with immediate waits
/// This tests the system under load to expose timing-sensitive race conditions
#[tokio::test]
async fn test_stress_rapid_operations_and_waits() -> Result<()> {
    let client = ()
        .serve(TokioChildProcess::new(Command::new("cargo").configure(
            |cmd| {
                cmd.arg("run")
                    .arg("--bin")
                    .arg("async_cargo_mcp")
                    .arg("--")
                    .arg("--enable-wait");
            },
        ))?)
        .await?;

    // Create many short operations rapidly
    let num_operations = 10;
    let mut operation_ids = Vec::new();

    println!("Starting {} rapid operations", num_operations);

    for i in 0..num_operations {
        let _sleep_result = client
            .call_tool(CallToolRequestParam {
                name: "sleep".into(),
                arguments: Some(object!({
                    "duration_ms": 200, // Short but not too short
                    "operation_id": format!("op_stress_{}", i),
                    "enable_async_notification": true
                })),
            })
            .await?;

        operation_ids.push(format!("op_stress_{}", i));

        // Very brief delay to create rapid succession
        sleep(Duration::from_millis(20)).await;
    }

    println!("All operations started, now testing waits");

    // Wait for all operations with minimal delays
    for (i, op_id) in operation_ids.iter().enumerate() {
        let start_time = Instant::now();

        let wait_result = client
            .call_tool(CallToolRequestParam {
                name: "wait".into(),
                arguments: Some(object!({ "operation_ids": [op_id.clone()] })),
            })
            .await?;

        let elapsed = start_time.elapsed();
        let wait_text = format!("{:?}", wait_result.content);

        // Should complete reasonably quickly even under stress
        assert!(
            elapsed.as_secs() < 10,
            "Stress test wait {} for {} took {:?}: {}",
            i + 1,
            op_id,
            elapsed,
            wait_text
        );

        // Should get meaningful result
        assert!(
            wait_text.contains("OPERATION COMPLETED") || wait_text.contains("No operation found"),
            "Stress test wait {} should get meaningful result: {}",
            i + 1,
            wait_text
        );

        println!("Stress wait {} completed in {:?}", i + 1, elapsed);
    }

    let _ = client.cancel().await;
    Ok(())
}
