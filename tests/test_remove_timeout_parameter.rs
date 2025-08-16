/// Test for removing timeout_secs parameter from WaitRequest
/// This test should initially fail, then pass once we remove the parameter
use async_cargo_mcp::cargo_tools::WaitRequest;

#[test]
fn test_wait_request_without_timeout_parameter() {
    // Test that WaitRequest can be deserialized without timeout_secs field
    let json_without_timeout = r#"{
        "operation_ids": ["op_123", "op_456"]
    }"#;

    let result: Result<WaitRequest, _> = serde_json::from_str(json_without_timeout);
    assert!(
        result.is_ok(),
        "WaitRequest should deserialize without timeout_secs field"
    );

    let wait_request = result.unwrap();
    assert_eq!(wait_request.operation_ids, vec!["op_123", "op_456"]);
}

#[test]
fn test_wait_request_now_rejects_timeout_parameter() {
    // Test that WaitRequest with timeout_secs field now fails to deserialize
    // because the field has been removed
    let json_with_timeout = r#"{
        "operation_ids": ["op_123"],
        "timeout_secs": 120
    }"#;

    let result: Result<WaitRequest, _> = serde_json::from_str(json_with_timeout);

    // Now this should fail because timeout_secs is no longer a valid field
    assert!(
        result.is_err(),
        "WaitRequest should reject unknown timeout_secs field"
    );
}

// TODO: Add schema test once timeout_secs is removed
