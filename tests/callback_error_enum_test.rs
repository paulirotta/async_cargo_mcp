use async_cargo_mcp::callback_system::CallbackError;

#[test]
fn test_callback_error_is_recoverable() {
    // Test if error indicates a potentially recoverable condition
    let send_failed = CallbackError::SendFailed("Network error".to_string());
    assert!(send_failed.is_recoverable());

    let timeout = CallbackError::Timeout("Request timed out".to_string());
    assert!(timeout.is_recoverable());

    let disconnected = CallbackError::Disconnected;
    assert!(!disconnected.is_recoverable()); // Connection lost - not recoverable

    let cancelled = CallbackError::Cancelled;
    assert!(!cancelled.is_recoverable()); // User cancelled - not recoverable
}

#[test]
fn test_callback_error_is_user_initiated() {
    // Test if error was caused by user action
    let cancelled = CallbackError::Cancelled;
    assert!(cancelled.is_user_initiated());

    let send_failed = CallbackError::SendFailed("Network error".to_string());
    assert!(!send_failed.is_user_initiated());

    let disconnected = CallbackError::Disconnected;
    assert!(!disconnected.is_user_initiated());

    let timeout = CallbackError::Timeout("Request timed out".to_string());
    assert!(!timeout.is_user_initiated());
}

#[test]
fn test_callback_error_error_code() {
    // Test getting error codes for programmatic handling
    let send_failed = CallbackError::SendFailed("Network error".to_string());
    assert_eq!(send_failed.error_code(), "SEND_FAILED");

    let disconnected = CallbackError::Disconnected;
    assert_eq!(disconnected.error_code(), "DISCONNECTED");

    let cancelled = CallbackError::Cancelled;
    assert_eq!(cancelled.error_code(), "CANCELLED");

    let timeout = CallbackError::Timeout("Request timed out".to_string());
    assert_eq!(timeout.error_code(), "TIMEOUT");
}

#[test]
fn test_callback_error_severity() {
    // Test error severity levels for logging
    let send_failed = CallbackError::SendFailed("Network error".to_string());
    assert_eq!(send_failed.severity(), "ERROR");

    let disconnected = CallbackError::Disconnected;
    assert_eq!(disconnected.severity(), "ERROR");

    let cancelled = CallbackError::Cancelled;
    assert_eq!(cancelled.severity(), "WARN"); // User action, less severe

    let timeout = CallbackError::Timeout("Request timed out".to_string());
    assert_eq!(timeout.severity(), "ERROR");
}

#[test]
fn test_callback_error_message_detail() {
    // Test extracting detailed message where available
    let send_failed = CallbackError::SendFailed("Network connection lost".to_string());
    assert_eq!(
        send_failed.message_detail(),
        Some("Network connection lost")
    );

    let timeout = CallbackError::Timeout("Operation exceeded 30s limit".to_string());
    assert_eq!(
        timeout.message_detail(),
        Some("Operation exceeded 30s limit")
    );

    let disconnected = CallbackError::Disconnected;
    assert_eq!(disconnected.message_detail(), None);

    let cancelled = CallbackError::Cancelled;
    assert_eq!(cancelled.message_detail(), None);
}
