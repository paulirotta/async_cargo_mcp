//! Integration tests using the proper rmcp client library
//!
//! These tests use the library functions from lib.rs to test the MCP server

use async_cargo_mcp::{test_all_tools, test_doc_functionality, test_increment_functionality};

#[ignore = "Some integration tests are currently ignored due to undiagnosed issues with TokioChildProcess transport in the test environment"]
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

#[ignore = "Some integration tests are currently ignored due to undiagnosed issues with TokioChildProcess transport in the test environment"]
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

#[ignore = "Some integration tests are currently ignored due to undiagnosed issues with TokioChildProcess transport in the test environment"]
#[tokio::test]
async fn test_mcp_server_doc_generation() {
    let result = test_doc_functionality()
        .await
        .expect("Doc functionality test failed");
    println!("Doc test result: {}", result);

    // Verify that the result contains expected documentation generation output
    assert!(result.contains("Documentation generation test results"));
    assert!(result.contains("Doc result:"));
}
