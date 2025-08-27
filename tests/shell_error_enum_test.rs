use async_cargo_mcp::shell_pool::ShellError;
use std::io;

fn create_serialization_error() -> ShellError {
    ShellError::SerializationError(
        serde_json::from_str::<serde_json::Value>("invalid json").unwrap_err(),
    )
}

#[test]
fn test_shell_error_is_recoverable() {
    // Test if error indicates a potentially recoverable condition
    let timeout = ShellError::Timeout;
    assert!(timeout.is_recoverable());

    let pool_full = ShellError::PoolFull;
    assert!(pool_full.is_recoverable());

    let spawn_error = ShellError::SpawnError(io::Error::new(
        io::ErrorKind::PermissionDenied,
        "permission denied",
    ));
    assert!(!spawn_error.is_recoverable()); // Permission issues typically not recoverable

    let process_died = ShellError::ProcessDied;
    assert!(process_died.is_recoverable()); // Can spawn a new process

    let serialization_error = create_serialization_error();
    assert!(!serialization_error.is_recoverable()); // Data structure issue

    let working_dir_error = ShellError::WorkingDirectoryError("path does not exist".to_string());
    assert!(!working_dir_error.is_recoverable()); // Path issue not automatically recoverable
}

#[test]
fn test_shell_error_is_resource_exhaustion() {
    // Test if error indicates resource exhaustion
    let pool_full = ShellError::PoolFull;
    assert!(pool_full.is_resource_exhaustion());

    let timeout = ShellError::Timeout;
    assert!(timeout.is_resource_exhaustion()); // Could be system overload

    let spawn_error = ShellError::SpawnError(io::Error::new(
        io::ErrorKind::PermissionDenied,
        "permission denied",
    ));
    assert!(!spawn_error.is_resource_exhaustion());

    let process_died = ShellError::ProcessDied;
    assert!(!process_died.is_resource_exhaustion());

    let serialization_error = create_serialization_error();
    assert!(!serialization_error.is_resource_exhaustion());

    let working_dir_error = ShellError::WorkingDirectoryError("path does not exist".to_string());
    assert!(!working_dir_error.is_resource_exhaustion());
}

#[test]
fn test_shell_error_is_io_error() {
    // Test if error is related to I/O operations
    let spawn_error = ShellError::SpawnError(io::Error::new(
        io::ErrorKind::PermissionDenied,
        "permission denied",
    ));
    assert!(spawn_error.is_io_error());

    let working_dir_error = ShellError::WorkingDirectoryError("path does not exist".to_string());
    assert!(working_dir_error.is_io_error());

    let timeout = ShellError::Timeout;
    assert!(!timeout.is_io_error());

    let pool_full = ShellError::PoolFull;
    assert!(!pool_full.is_io_error());

    let process_died = ShellError::ProcessDied;
    assert!(!process_died.is_io_error());

    let serialization_error = create_serialization_error();
    assert!(!serialization_error.is_io_error());
}

#[test]
fn test_shell_error_error_category() {
    // Test categorizing errors for handling
    let spawn_error = ShellError::SpawnError(io::Error::new(
        io::ErrorKind::PermissionDenied,
        "permission denied",
    ));
    assert_eq!(spawn_error.error_category(), "IO");

    let timeout = ShellError::Timeout;
    assert_eq!(timeout.error_category(), "TIMEOUT");

    let process_died = ShellError::ProcessDied;
    assert_eq!(process_died.error_category(), "PROCESS");

    let serialization_error = create_serialization_error();
    assert_eq!(serialization_error.error_category(), "SERIALIZATION");

    let pool_full = ShellError::PoolFull;
    assert_eq!(pool_full.error_category(), "RESOURCE");

    let working_dir_error = ShellError::WorkingDirectoryError("path does not exist".to_string());
    assert_eq!(working_dir_error.error_category(), "IO");
}

#[test]
fn test_shell_error_severity_level() {
    // Test error severity for logging
    let spawn_error = ShellError::SpawnError(io::Error::new(
        io::ErrorKind::PermissionDenied,
        "permission denied",
    ));
    assert_eq!(spawn_error.severity_level(), "ERROR");

    let timeout = ShellError::Timeout;
    assert_eq!(timeout.severity_level(), "WARN"); // Might be temporary

    let process_died = ShellError::ProcessDied;
    assert_eq!(process_died.severity_level(), "ERROR");

    let serialization_error = create_serialization_error();
    assert_eq!(serialization_error.severity_level(), "ERROR");

    let pool_full = ShellError::PoolFull;
    assert_eq!(pool_full.severity_level(), "WARN"); // Might be temporary

    let working_dir_error = ShellError::WorkingDirectoryError("path does not exist".to_string());
    assert_eq!(working_dir_error.severity_level(), "ERROR");
}
