//! Extreme edge case tests designed to expose specific race conditions in wait operations
//!
//! These tests use specific configurations, timing, and scenarios designed to trigger
//! the exact race conditions identified in the code analysis phase.

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

/// Test the most likely race condition: "The Vanishing Operation Race"
///
/// This test specifically targets the scenario where:
/// 1. Operation completes successfully
/// 2. Gets moved to completion_history
/// 3. Cleanup removes from both operations map AND completion_history due to size limits
/// 4. Wait called immediately after → finds nothing → should return "not found" quickly, not hang for 300s
///
/// This is the most critical race condition because it could cause LLMs to wait 5 minutes
/// for operations that are already complete.
#[tokio::test]
async fn test_critical_vanishing_operation_race_300s_timeout() -> Result<()> {
    let client = ()
        .serve(TokioChildProcess::new(Command::new("cargo").configure(
            |cmd| {
                cmd.arg("run").arg("--bin").arg("async_cargo_mcp").arg("--");
                // wait is available by default in async mode
            },
        ))?)
        .await?;

    println!("🔍 Testing critical 'vanishing operation' race condition");
    println!("This test verifies that completed operations don't cause 300s hangs");

    // Create a scenario designed to trigger cleanup
    // Start many very short operations to potentially trigger cleanup behavior
    let num_operations = 25;
    let mut all_op_ids = Vec::new();

    println!(
        "Starting {} very short operations to trigger potential cleanup",
        num_operations
    );

    for i in 0..num_operations {
        let _result = client
            .call_tool(CallToolRequestParam {
                name: "sleep".into(),
                arguments: Some(object!({
                    "duration_ms": 25, // Very short operations
                    "operation_id": format!("op_vanish_{}", i),
                    "enable_async_notification": true
                })),
            })
            .await?;

        all_op_ids.push(format!("op_vanish_{}", i));

        // Rapid succession to create many operations quickly
        if i % 5 == 0 {
            sleep(Duration::from_millis(10)).await;
        }
    }

    println!("✓ All operations started, waiting for completion");

    // Let all operations complete
    sleep(Duration::from_millis(500)).await;

    println!("📋 Now testing waits for potentially cleaned-up operations");

    // Test waiting for operations at different delays
    // Some should still be in completion_history, others might be cleaned up
    let test_cases = vec![
        (0, "immediate"),
        (1, "1 second later"),
        (3, "3 seconds later"),
        (5, "5 seconds later"),
    ];

    for (delay_seconds, description) in test_cases {
        if delay_seconds > 0 {
            println!(
                "⏳ Waiting {} seconds to vary cleanup timing...",
                delay_seconds
            );
            sleep(Duration::from_secs(delay_seconds)).await;
        }

        // Test a few operations from our set
        let test_op_ids = vec![
            &all_op_ids[0],  // First operation
            &all_op_ids[10], // Middle operation
            &all_op_ids[24], // Last operation
        ];

        for op_id in test_op_ids {
            println!("🔄 Testing wait for {} ({})", op_id, description);

            let start_time = Instant::now();

            let wait_result = client
                .call_tool(CallToolRequestParam {
                    name: "wait".into(),
                    arguments: Some(object!({ "operation_ids": [op_id.clone()] })),
                })
                .await?;

            let elapsed = start_time.elapsed();
            let wait_text = format!("{:?}", wait_result.content);

            println!("⏱️  Wait completed in {:?}", elapsed);

            // 🚨 CRITICAL ASSERTION: This is the core race condition test
            // If this fails, it means we have the "vanishing operation" race that causes 300s hangs
            assert!(
                elapsed.as_secs() < 30,
                "🚨 CRITICAL RACE CONDITION DETECTED: Wait for {} ({}) took {:?} - this indicates the 'vanishing operation' race where completed operations cause long hangs instead of quick 'not found' responses.\n\nWait result: {}",
                op_id,
                description,
                elapsed,
                wait_text
            );

            // Should get either completion or "not found" - both are acceptable
            let is_valid_response = wait_text.contains("OPERATION COMPLETED")
                || wait_text.contains("No operation found")
                || wait_text.contains("cleaned up");

            assert!(
                is_valid_response,
                "Wait for {} ({}) should return either completion or 'not found', got: {}",
                op_id, description, wait_text
            );

            if elapsed.as_secs() > 5 {
                println!(
                    "⚠️  WARNING: Wait took {:?} seconds - possible performance issue",
                    elapsed.as_secs()
                );
            } else if wait_text.contains("OPERATION COMPLETED") {
                println!("✅ Found completed operation in {:?}", elapsed);
            } else if wait_text.contains("No operation found") {
                println!(
                    "ℹ️  Operation not found (cleaned up) in {:?} - this is acceptable",
                    elapsed
                );
            }
        }
    }

    let _ = client.cancel().await;
    println!("🎯 Critical vanishing operation race test completed successfully");
    Ok(())
}

/// Test timeout boundary conditions where operation timeout and wait timeout could conflict
///
/// This tests the scenario where:
/// - Operation has a timeout of X seconds
/// - Wait timeout is also configured to X seconds  
/// - Both timeouts could trigger simultaneously causing undefined behavior
#[tokio::test]
async fn test_timeout_boundary_race_conditions() -> Result<()> {
    let client = ()
        .serve(TokioChildProcess::new(Command::new("cargo").configure(
            |cmd| {
                cmd.arg("run").arg("--bin").arg("async_cargo_mcp").arg("--");
                // wait is available by default in async mode
            },
        ))?)
        .await?;

    println!("🔍 Testing timeout boundary race conditions");

    // Test operations with durations that approach timeout boundaries
    let timeout_test_cases = vec![
        ("near_1s", 950),  // Just under 1 second
        ("at_1s", 1000),   // Exactly 1 second
        ("over_1s", 1050), // Just over 1 second
        ("near_2s", 1950), // Just under 2 seconds
        ("at_5s", 5000),   // 5 seconds - longer test
    ];

    for (test_name, duration_ms) in timeout_test_cases {
        println!(
            "⏱️ Testing timeout boundary: {} ({}ms)",
            test_name, duration_ms
        );

        let op_id = format!("op_timeout_{}", test_name);

        // Start operation
        let _result = client
            .call_tool(CallToolRequestParam {
                name: "sleep".into(),
                arguments: Some(object!({
                    "duration_ms": duration_ms,
                    "operation_id": op_id.clone(),
                    "enable_async_notification": true
                })),
            })
            .await?;

        // Wait for operation immediately (while it's running)
        let start_time = Instant::now();

        let wait_result = client
            .call_tool(CallToolRequestParam {
                name: "wait".into(),
                arguments: Some(object!({ "operation_ids": [op_id.clone()] })),
            })
            .await?;

        let elapsed = start_time.elapsed();
        let wait_text = format!("{:?}", wait_result.content);

        let expected_min = Duration::from_millis(duration_ms);
        let expected_max = expected_min + Duration::from_secs(5); // Allow 5s overhead

        println!(
            "⏱️ {} completed in {:?} (expected ~{:?})",
            test_name, elapsed, expected_min
        );

        // Wait time should be reasonable relative to operation time
        assert!(
            elapsed >= expected_min && elapsed <= expected_max,
            "Timeout boundary test {} took {:?}, expected between {:?} and {:?}: {}",
            test_name,
            elapsed,
            expected_min,
            expected_max,
            wait_text
        );

        // Should complete successfully (not timeout)
        assert!(
            wait_text.contains("OPERATION COMPLETED"),
            "Timeout boundary test {} should complete normally: {}",
            test_name,
            wait_text
        );

        // Brief pause between timeout tests
        sleep(Duration::from_millis(100)).await;
    }

    println!("✅ Timeout boundary race conditions test completed");
    Ok(())
}

/// Test the "Cleanup During Wait" race condition
///
/// This tests the scenario where:
/// - Wait starts polling for an operation
/// - Background cleanup runs mid-poll cycle
/// - Operation gets removed between poll iterations  
/// - Wait should handle this gracefully, not continue polling for remaining timeout
#[tokio::test]
async fn test_cleanup_during_wait_polling_race() -> Result<()> {
    let client = ()
        .serve(TokioChildProcess::new(Command::new("cargo").configure(
            |cmd| {
                cmd.arg("run").arg("--bin").arg("async_cargo_mcp").arg("--");
                // wait is available by default in async mode
            },
        ))?)
        .await?;

    println!("🔍 Testing 'cleanup during wait' polling race condition");

    // Create multiple operations that will complete and potentially be cleaned up
    let num_ops = 15;
    let mut operation_ids = Vec::new();

    println!("Creating {} operations for cleanup timing test", num_ops);

    for i in 0..num_ops {
        let _result = client
            .call_tool(CallToolRequestParam {
                name: "sleep".into(),
                arguments: Some(object!({
                    "duration_ms": 150,  // Medium duration
                    "operation_id": format!("op_cleanup_poll_{}", i),
                    "enable_async_notification": true
                })),
            })
            .await?;

        operation_ids.push(format!("op_cleanup_poll_{}", i));
    }

    // Wait for operations to complete
    sleep(Duration::from_millis(300)).await;

    println!("📋 Operations completed, now testing wait behavior with cleanup timing");

    // Test wait operations with different delays to catch cleanup cycles
    for (i, op_id) in operation_ids.iter().enumerate() {
        // Vary the delay to catch different phases of potential cleanup cycles
        let delay_ms = (i * 200) % 1000; // 0ms, 200ms, 400ms, 600ms, 800ms, 0ms, ...

        if delay_ms > 0 {
            sleep(Duration::from_millis(delay_ms as u64)).await;
        }

        println!(
            "🔄 Wait test {} for {} (after {}ms delay)",
            i + 1,
            op_id,
            delay_ms
        );

        let start_time = Instant::now();

        let wait_result = client
            .call_tool(CallToolRequestParam {
                name: "wait".into(),
                arguments: Some(object!({ "operation_ids": [op_id.clone()] })),
            })
            .await?;

        let elapsed = start_time.elapsed();
        let wait_text = format!("{:?}", wait_result.content);

        // Critical: Should not get stuck in polling loop if cleanup happens
        assert!(
            elapsed.as_secs() < 15,
            "Cleanup-during-wait test {} for {} took {:?} - possible polling race where wait continues after cleanup: {}",
            i + 1,
            op_id,
            elapsed,
            wait_text
        );

        // Should get definitive result
        assert!(
            wait_text.contains("OPERATION COMPLETED") || wait_text.contains("No operation found"),
            "Cleanup-during-wait test {} should get definitive result: {}",
            i + 1,
            wait_text
        );

        if elapsed.as_secs() > 3 {
            println!(
                "⚠️ Test {} took {:?} - monitoring for potential polling issues",
                i + 1,
                elapsed.as_secs()
            );
        }
    }

    println!("✅ Cleanup during wait polling race test completed");
    Ok(())
}

/// Test extreme concurrency scenario that stresses the wait system
///
/// This creates maximum stress on the wait implementation to expose any
/// resource contention or synchronization issues that could cause indefinite waits.
#[tokio::test]
async fn test_extreme_concurrency_wait_stress() -> Result<()> {
    let client = ()
        .serve(TokioChildProcess::new(Command::new("cargo").configure(
            |cmd| {
                cmd.arg("run").arg("--bin").arg("async_cargo_mcp").arg("--");
                // wait is available by default in async mode
            },
        ))?)
        .await?;

    println!("🔍 Testing extreme concurrency wait stress");

    // Create many operations
    let num_ops = 20;
    let mut operation_ids = Vec::new();

    println!(
        "Creating {} operations for concurrency stress test",
        num_ops
    );

    for i in 0..num_ops {
        let _result = client
            .call_tool(CallToolRequestParam {
                name: "sleep".into(),
                arguments: Some(object!({
                    "duration_ms": 300,
                    "operation_id": format!("op_stress_{}", i),
                    "enable_async_notification": true
                })),
            })
            .await?;

        operation_ids.push(format!("op_stress_{}", i));
    }

    println!("📋 Launching massive concurrent wait stress test");

    // Launch many concurrent waits for different operations
    let mut wait_handles = Vec::new();

    for (i, op_id) in operation_ids.iter().enumerate() {
        let op_id_clone = op_id.clone();
        let client_clone = client.clone();

        let handle = tokio::spawn(async move {
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

    // Collect all results
    let mut results = Vec::new();
    for handle in wait_handles {
        let result = handle
            .await
            .map_err(|e| anyhow::anyhow!("Join error: {}", e))?;
        results.push(result);
    }

    println!("📊 Analyzing {} concurrent wait results", results.len());

    let mut max_time = Duration::from_secs(0);
    let mut total_time = Duration::from_secs(0);
    let mut success_count = 0;

    for (i, op_id, elapsed, wait_result) in results {
        match wait_result {
            Ok(result) => {
                let wait_text = format!("{:?}", result.content);

                // Track timing statistics
                total_time += elapsed;
                if elapsed > max_time {
                    max_time = elapsed;
                }

                // Critical: No wait should hang for excessive time even under extreme load
                assert!(
                    elapsed.as_secs() < 25,
                    "Extreme concurrency test #{} for {} took {:?} - possible resource contention causing hang: {}",
                    i,
                    op_id,
                    elapsed,
                    wait_text
                );

                // Should get completion
                assert!(
                    wait_text.contains("OPERATION COMPLETED"),
                    "Extreme concurrency test #{} should complete: {}",
                    i,
                    wait_text
                );

                success_count += 1;
            }
            Err(e) => {
                panic!(
                    "Extreme concurrency test #{} for {} failed: {:?}",
                    i, op_id, e
                );
            }
        }
    }

    let avg_time = total_time / success_count as u32;

    println!("📈 Concurrency stress test results:");
    println!("  ✅ Successful waits: {}/{}", success_count, num_ops);
    println!("  ⏱️ Average wait time: {:?}", avg_time);
    println!("  ⏱️ Maximum wait time: {:?}", max_time);
    println!("  ⏱️ Total wait time: {:?}", total_time);

    // Overall performance should be reasonable even under extreme load
    assert!(
        avg_time.as_secs() < 10,
        "Average wait time {:?} too high under extreme load",
        avg_time
    );
    assert_eq!(
        success_count, num_ops,
        "Not all waits completed successfully"
    );

    let _ = client.cancel().await;
    println!("🎯 Extreme concurrency wait stress test completed successfully");
    Ok(())
}
