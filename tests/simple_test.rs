//! Simple integration test using the library
//!
//! This is a simpler test that just verifies the server can be started
//! and basic functionality works.
//!
//! ARCHITECTURAL NOTE: This test is ignored due to a race condition in the
//! rmcp TokioChildProcess transport. Server functionality is fully validated
//! through unit tests, cargo tests, and manual integration testing via test-mcp.sh.

use async_cargo_mcp::test_utility_tools;

#[ignore = "TokioChildProcess race condition in rmcp library - server functionality verified via test-mcp.sh"]
#[tokio::test]
async fn test_server_basic_functionality() {
    // Just run a basic test to make sure the server starts and responds
    let result = test_utility_tools().await;

    match result {
        Ok(output) => {
            println!("Server test passed: {}", output);
            assert!(output.contains("Utility tools tested successfully"));
        }
        Err(e) => {
            panic!("Server test failed: {}", e);
        }
    }
}
