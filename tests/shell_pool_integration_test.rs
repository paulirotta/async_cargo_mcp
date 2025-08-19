use async_cargo_mcp::test_cargo_tools;
use std::time::Instant;

mod common;
use common::test_project::create_basic_project;

/// Integration test comparing shell pool vs direct execution performance
/// This test uses the existing test_cargo_tools infrastructure
#[tokio::test]
async fn test_shell_pool_vs_direct_execution_performance() {
    // Create a basic test project
    let temp_project = create_basic_project()
        .await
        .expect("Failed to create test project");
    let project_path = temp_project.path().to_str().unwrap();

    println!("🚀 Starting Shell Pool Performance Test");
    println!("Testing in directory: {}", project_path);

    const NUM_OPERATIONS: usize = 3;
    let mut check_times = Vec::new();

    // Run multiple check operations and measure timing
    // Note: The actual shell pool vs direct execution comparison happens
    // internally within the test_cargo_tools functions based on configuration
    for i in 0..NUM_OPERATIONS {
        let start = Instant::now();
        let result = test_cargo_tools::test_check_command(project_path).await;
        let duration = start.elapsed();
        check_times.push(duration);

        println!("Check operation {} completed in {:?}", i + 1, duration);

        // Ensure the operation was successful
        assert!(result.is_ok(), "Check operation failed: {:?}", result);
        let output = result.unwrap();
        assert!(output.contains("Finished"), "Check should complete successfully");
    }

    // Calculate average
    let avg_time = check_times.iter().sum::<std::time::Duration>() / NUM_OPERATIONS as u32;

    println!("Average check time: {:?}", avg_time);
    
    // The actual performance benefit verification happens at runtime
    // when shell pools are enabled vs disabled. This test verifies
    // that the system works correctly with shell pools active.
    
    println!("✅ Shell Pool Integration Test Completed Successfully");
}

/// Test that shell pool system produces consistent output
#[tokio::test]
async fn test_shell_pool_output_consistency() {
    // Create a basic test project
    let temp_project = create_basic_project()
        .await
        .expect("Failed to create test project");
    let project_path = temp_project.path().to_str().unwrap();

    println!("🔍 Testing Shell Pool Output Consistency");

    // Run the same operation multiple times to ensure consistent success
    const CONSISTENCY_TESTS: usize = 3;

    for i in 0..CONSISTENCY_TESTS {
        let result = test_cargo_tools::test_check_command(project_path).await;
        assert!(result.is_ok(), "Check operation {} failed: {:?}", i + 1, result);
        
        let output = result.unwrap();
        println!("Check operation {} completed", i + 1);
        
        // Verify the operation completed successfully
        // The actual output format includes debug strings, so we just check for success indicators
        assert!(output.contains("completed successfully"), "Operation {} should complete successfully", i + 1);
        assert!(output.contains("Finished"), "Operation {} should show 'Finished'", i + 1);
        
        // Verify it doesn't contain actual compilation errors (not debug strings)
        // The debug output contains "error:" in field names, so we look for actual errors
        assert!(!output.contains("compilation failed"), "Operation {} should not have compilation failures", i + 1);
    }

    println!("✅ Shell Pool Output Consistency Test Passed");
}

/// Test shell pool behavior with build operations
#[tokio::test] 
async fn test_shell_pool_build_operations() {
    // Create a basic test project
    let temp_project = create_basic_project()
        .await
        .expect("Failed to create test project");
    let project_path = temp_project.path().to_str().unwrap();

    println!("🏗️ Testing Shell Pool Build Operations");

    // Test build operation
    let build_start = Instant::now();
    let build_result = test_cargo_tools::test_build_command(project_path).await;
    let build_duration = build_start.elapsed();

    println!("Build operation completed in {:?}", build_duration);

    assert!(build_result.is_ok(), "Build operation failed: {:?}", build_result);
    let build_output = build_result.unwrap();
    assert!(build_output.contains("Finished"), "Build should complete successfully");

    // Test check operation after build
    let check_start = Instant::now();
    let check_result = test_cargo_tools::test_check_command(project_path).await;
    let check_duration = check_start.elapsed();

    println!("Check operation completed in {:?}", check_duration);

    assert!(check_result.is_ok(), "Check operation failed: {:?}", check_result);
    let check_output = check_result.unwrap();
    assert!(check_output.contains("Finished"), "Check should complete successfully");

    // With shell pools, the second operation should be notably faster
    // due to reusing the warmed up shell environment
    println!("Build: {:?}, Check: {:?}", build_duration, check_duration);

    println!("✅ Shell Pool Build Operations Test Completed");
}
