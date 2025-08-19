//! Edge case tests for wait operations with aggressive configurations
//!
//! These tests use specific configurations designed to expose potential race conditions
//! by forcing immediate cleanup, very short timeouts, and extreme concurrency scenarios.

use anyhow::Result;
use std::time::{Duration, Instant};
use tokio::time::sleep;

mod common;
use rmcp::{
    ServiceExt,
    model::CallToolRequestParam,
    object,
    transport::{ConfigureCommandExt, TokioChildProcess},
};
use tokio::process::Command;

#[allow(dead_code)]
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

/// Test edge case: Wait for operation immediately after it's likely to have been cleaned up
/// This test uses rapid fire operations to potentially trigger cleanup during wait
#[tokio::test]
async fn test_edge_immediate_cleanup_race() -> Result<()> {
    let client = ()
        .serve(TokioChildProcess::new(Command::new("cargo").configure(
            |cmd| {
                cmd.arg("run").arg("--bin").arg("async_cargo_mcp");
            },
        ))?)
        .await?;

    // Create many very short operations that will complete quickly and potentially be cleaned up
    let num_ops = 20;
    let mut operation_ids = Vec::new();

    // Start all operations rapidly
    for i in 0..num_ops {
        let _sleep_result = client
            .call_tool(CallToolRequestParam {
                name: "sleep".into(),
                arguments: Some(object!({
                    "duration_ms": 50, // Very short
                    "operation_id": format!("op_cleanup_race_{}", i),
                    "enable_async_notification": true
                })),
            })
            .await?;

        operation_ids.push(format!("op_cleanup_race_{}", i));
    }

    // Wait for all to complete
    sleep(Duration::from_millis(200)).await;

    // Now wait for operations in reverse order with different delays
    // This creates varied timing that might trigger cleanup races
    for (i, op_id) in operation_ids.iter().rev().enumerate() {
        // Add progressive delay to create different cleanup timing scenarios
        if i > 0 {
            sleep(Duration::from_millis((i as u64) * 50)).await;
        }

        let start_time = Instant::now();

        let wait_result = client
            .call_tool(CallToolRequestParam {
                name: "wait".into(),
                arguments: Some(object!({ "operation_ids": [op_id.clone()] })),
            })
            .await?;

        let elapsed = start_time.elapsed();
        let wait_text = format!("{:?}", wait_result.content);

        // Critical assertion: should NEVER hang for the full 300s timeout
        assert!(
            elapsed.as_secs() < 30,
            "Wait for {} took {:?} - possible cleanup race causing long wait: {}",
            op_id,
            elapsed,
            wait_text
        );

        // Should get meaningful response (not hang indefinitely)
        assert!(
            wait_text.contains("OPERATION COMPLETED")
                || wait_text.contains("No operation found")
                || wait_text.contains("cleaned up"),
            "Wait for {} should return meaningful response, not hang: {}",
            op_id,
            wait_text
        );

        if elapsed.as_secs() > 5 {
            println!(
                "WARNING: Wait for {} took {:?} seconds - potential race detected",
                op_id,
                elapsed.as_secs()
            );
        }
    }

    let _ = client.cancel().await;
    Ok(())
}

/// Test edge case: Concurrent waits for operations that complete at exactly the same time
/// This tests for race conditions in the completion_history management
#[tokio::test]
async fn test_edge_simultaneous_completion_concurrent_waits() -> Result<()> {
    let client = ()
        .serve(TokioChildProcess::new(Command::new("cargo").configure(
            |cmd| {
                cmd.arg("run").arg("--bin").arg("async_cargo_mcp");
            },
        ))?)
        .await?;

    // Start several operations with the SAME duration to complete simultaneously
    let duration_ms = 500; // All complete at roughly the same time
    let num_ops = 5;
    let mut operation_ids = Vec::new();

    println!(
        "Starting {} operations with {}ms duration",
        num_ops, duration_ms
    );

    for i in 0..num_ops {
        let _sleep_result = client
            .call_tool(CallToolRequestParam {
                name: "sleep".into(),
                arguments: Some(object!({
                    "duration_ms": duration_ms,
                    "operation_id": format!("op_simultaneous_{}", i),
                    "enable_async_notification": true
                })),
            })
            .await?;

        operation_ids.push(format!("op_simultaneous_{}", i));

        // Very brief delay to start them almost simultaneously
        sleep(Duration::from_millis(5)).await;
    }

    // Wait for just under completion time
    sleep(Duration::from_millis((duration_ms - 100) as u64)).await;

    // Now launch concurrent waits for ALL operations just before they complete
    // This should stress test the completion_history management
    println!("Launching concurrent waits for all operations");

    let mut wait_handles = Vec::new();
    for (i, op_id) in operation_ids.iter().enumerate() {
        let op_id_clone = op_id.clone();
        let client_clone = client.clone();

        let handle = tokio::spawn(async move {
            println!("Starting concurrent wait for: {}", op_id_clone);
            let start_time = Instant::now();

            let wait_result = client_clone
                .call_tool(CallToolRequestParam {
                    name: "wait".into(),
                    arguments: Some(object!({ "operation_ids": [op_id_clone.clone()] })),
                })
                .await;

            let elapsed = start_time.elapsed();
            (i, op_id_clone, elapsed, wait_result)
        });

        wait_handles.push(handle);
    }

    // Collect results from all concurrent waits
    let mut results = Vec::new();
    for handle in wait_handles {
        let result = handle
            .await
            .map_err(|e| anyhow::anyhow!("Join error: {}", e))?;
        results.push(result);
    }

    // Analyze all results
    for (i, op_id, elapsed, wait_result) in results {
        match wait_result {
            Ok(result) => {
                let wait_text = format!("{:?}", result.content);

                // Critical: Should not hang for excessive time
                assert!(
                    elapsed.as_secs() < 15,
                    "Concurrent wait {} for {} took {:?} - possible simultaneous completion race: {}",
                    i,
                    op_id,
                    elapsed,
                    wait_text
                );

                // Should get completion status
                assert!(
                    wait_text.contains("OPERATION COMPLETED"),
                    "Concurrent wait {} for {} should show completion: {}",
                    i,
                    op_id,
                    wait_text
                );

                println!(
                    "✓ Concurrent wait {} for {} completed in {:?}",
                    i, op_id, elapsed
                );
            }
            Err(e) => {
                panic!("Concurrent wait {} for {} failed: {:?}", i, op_id, e);
            }
        }
    }

    let _ = client.cancel().await;
    Ok(())
}

/// Test edge case: Wait operations with rapid succession and overlapping timeouts
/// This tests boundary conditions between operation timeouts and wait timeouts
#[tokio::test]
async fn test_edge_overlapping_timeout_scenarios() -> Result<()> {
    let client = ()
        .serve(TokioChildProcess::new(Command::new("cargo").configure(
            |cmd| {
                cmd.arg("run").arg("--bin").arg("async_cargo_mcp");
            },
        ))?)
        .await?;

    // Test scenario: operations with various durations near timeout boundaries
    let test_cases = vec![
        ("op_timeout_1s", 1000), // Well under timeout
        ("op_timeout_2s", 2000), // Still safe
        ("op_timeout_5s", 5000), // Longer but reasonable
    ];

    for (op_id, duration_ms) in test_cases {
        println!(
            "Testing timeout scenario: {} with {}ms duration",
            op_id, duration_ms
        );

        // Start operation
        let _sleep_result = client
            .call_tool(CallToolRequestParam {
                name: "sleep".into(),
                arguments: Some(object!({
                    "duration_ms": duration_ms,
                    "operation_id": op_id,
                    "enable_async_notification": true
                })),
            })
            .await?;

        // Wait immediately (while operation is still running)
        let start_time = Instant::now();

        let wait_result = client
            .call_tool(CallToolRequestParam {
                name: "wait".into(),
                arguments: Some(object!({ "operation_ids": [op_id] })),
            })
            .await?;

        let elapsed = start_time.elapsed();
        let wait_text = format!("{:?}", wait_result.content);

        // Wait time should be approximately the operation duration (plus overhead)
        let expected_duration = Duration::from_millis(duration_ms);
        let overhead_allowance = Duration::from_secs(2);

        assert!(
            elapsed < expected_duration + overhead_allowance,
            "Wait for {} took {:?}, expected ~{:?} + overhead: {}",
            op_id,
            elapsed,
            expected_duration,
            wait_text
        );

        // Should complete successfully
        assert!(
            wait_text.contains("OPERATION COMPLETED"),
            "Operation {} should complete: {}",
            op_id,
            wait_text
        );

        println!(
            "✓ {} completed in {:?} (expected ~{:?})",
            op_id, elapsed, expected_duration
        );

        // Brief pause between test cases
        sleep(Duration::from_millis(100)).await;
    }

    let _ = client.cancel().await;
    Ok(())
}

/// Test edge case: Extremely rapid wait calls for the same operation
/// This tests for potential resource leaks or contention in the wait implementation
#[tokio::test]
async fn test_edge_rapid_sequential_waits() -> Result<()> {
    let client = ()
        .serve(TokioChildProcess::new(Command::new("cargo").configure(
            |cmd| {
                cmd.arg("run").arg("--bin").arg("async_cargo_mcp");
            },
        ))?)
        .await?;

    // Start one short operation
    let _sleep_result = client
        .call_tool(CallToolRequestParam {
            name: "sleep".into(),
            arguments: Some(object!({
                "duration_ms": 300,
                "operation_id": "op_rapid_waits",
                "enable_async_notification": true
            })),
        })
        .await?;

    // Let it complete
    sleep(Duration::from_millis(500)).await;

    // Now perform many rapid sequential waits for the SAME completed operation
    let num_rapid_waits = 10;
    let mut wait_times = Vec::new();

    println!(
        "Performing {} rapid sequential waits for completed operation",
        num_rapid_waits
    );

    for i in 0..num_rapid_waits {
        let start_time = Instant::now();

        let wait_result = client
            .call_tool(CallToolRequestParam {
                name: "wait".into(),
                arguments: Some(object!({ "operation_ids": ["op_rapid_waits"] })),
            })
            .await?;

        let elapsed = start_time.elapsed();
        wait_times.push(elapsed);

        let wait_text = format!("{:?}", wait_result.content);

        // Each wait should be very fast since operation is already completed
        assert!(
            elapsed.as_millis() < 2000, // 2 seconds max
            "Rapid wait #{} took {:?} - should be fast for completed operation: {}",
            i + 1,
            elapsed,
            wait_text
        );

        // Should consistently show completion
        assert!(
            wait_text.contains("OPERATION COMPLETED") || wait_text.contains("No operation found"), // Might be cleaned up
            "Rapid wait #{} should show consistent result: {}",
            i + 1,
            wait_text
        );

        // Very brief delay between rapid waits
        sleep(Duration::from_millis(10)).await;
    }

    // Analyze timing patterns
    let avg_wait_time: Duration = wait_times.iter().sum::<Duration>() / wait_times.len() as u32;
    let max_wait_time = wait_times.iter().max().unwrap();
    let min_wait_time = wait_times.iter().min().unwrap();

    println!("Rapid wait timing analysis:");
    println!("  Average: {:?}", avg_wait_time);
    println!("  Min: {:?}", min_wait_time);
    println!("  Max: {:?}", max_wait_time);

    // All waits should be reasonably consistent and fast
    assert!(
        max_wait_time.as_millis() < 1000,
        "Maximum wait time {:?} is too high for completed operation",
        max_wait_time
    );

    let _ = client.cancel().await;
    Ok(())
}

/// Test edge case: Wait for operations during high system load
/// This simulates conditions that might trigger race conditions under resource pressure
#[tokio::test]
async fn test_edge_high_load_wait_behavior() -> Result<()> {
    let client = ()
        .serve(TokioChildProcess::new(Command::new("cargo").configure(
            |cmd| {
                cmd.arg("run").arg("--bin").arg("async_cargo_mcp");
            },
        ))?)
        .await?;

    // Create high load by starting many operations simultaneously
    let high_load_ops = 15;
    let mut operation_ids = Vec::new();

    println!(
        "Creating high load with {} simultaneous operations",
        high_load_ops
    );

    // Start all operations at once to create resource pressure
    for i in 0..high_load_ops {
        let _sleep_result = client
            .call_tool(CallToolRequestParam {
                name: "sleep".into(),
                arguments: Some(object!({
                    "duration_ms": 400, // Moderate duration
                    "operation_id": format!("op_load_{}", i),
                    "enable_async_notification": true
                })),
            })
            .await?;

        operation_ids.push(format!("op_load_{}", i));
    }

    // Immediately start waiting for some operations while others are still starting
    let test_operation_ids = vec![
        operation_ids[0].clone(),
        operation_ids[high_load_ops / 2].clone(),
        operation_ids[high_load_ops - 1].clone(),
    ];

    println!("Starting waits for operations under high load");

    for op_id in test_operation_ids {
        let start_time = Instant::now();

        let wait_result = client
            .call_tool(CallToolRequestParam {
                name: "wait".into(),
                arguments: Some(object!({ "operation_ids": [op_id.clone()] })),
            })
            .await?;

        let elapsed = start_time.elapsed();
        let wait_text = format!("{:?}", wait_result.content);

        // Even under high load, waits should not hang indefinitely
        assert!(
            elapsed.as_secs() < 20,
            "Wait for {} under high load took {:?} - possible resource contention race: {}",
            op_id,
            elapsed,
            wait_text
        );

        // Should complete successfully despite load
        assert!(
            wait_text.contains("OPERATION COMPLETED"),
            "Wait for {} under high load should complete: {}",
            op_id,
            wait_text
        );

        println!("✓ High load wait for {} completed in {:?}", op_id, elapsed);
    }

    let _ = client.cancel().await;
    Ok(())
}
