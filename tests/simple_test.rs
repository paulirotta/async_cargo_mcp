//! Simple integration test using the library
//! 
//! This is a simpler test that just verifies the server can be started
//! and basic functionality works.

use async_cargo_mcp::test_all_tools;

#[tokio::test]
async fn test_server_basic_functionality() {
    // Just run a basic test to make sure the server starts and responds
    let result = test_all_tools().await;
    
    match result {
        Ok(output) => {
            println!("Server test passed: {}", output);
            assert!(output.contains("All tools tested successfully"));
        }
        Err(e) => {
            panic!("Server test failed: {}", e);
        }
    }
}
