//! Integration tests using the proper rmcp client library
//!
//! These tests use the library functions from lib.rs to test the MCP server

use async_cargo_mcp::{test_all_tools, test_increment_functionality};

#[tokio::test]
async fn test_mcp_server_all_tools() {
    let result = test_all_tools().await.expect("All tools test failed");
    println!("Test result: {}", result);

    // Verify that the result contains expected strings
    assert!(result.contains("All tools tested successfully"));
    assert!(result.contains("Increment:"));
    assert!(result.contains("Get Value:"));
    assert!(result.contains("Decrement:"));
    assert!(result.contains("Echo:"));
    assert!(result.contains("Sum:"));
}

#[tokio::test]
async fn test_mcp_server_increment_sequence() {
    let result = test_increment_functionality()
        .await
        .expect("Increment test failed");
    println!("Increment test result: {}", result);

    // Verify that the result contains expected progression
    assert!(result.contains("Increment test results"));
    assert!(result.contains("Initial:"));
    assert!(result.contains("After first increment:"));
    assert!(result.contains("After second increment:"));
    assert!(result.contains("Final value:"));
}
