use async_cargo_mcp::callback_system::ProgressUpdate;

#[test]
fn test_progress_update_is_terminal() {
    // Test if update represents a terminal state
    let started = ProgressUpdate::Started {
        operation_id: "test_op".to_string(),
        command: "cargo build".to_string(),
        description: "Building project".to_string(),
    };
    assert!(!started.is_terminal());

    let progress = ProgressUpdate::Progress {
        operation_id: "test_op".to_string(),
        message: "Building...".to_string(),
        percentage: Some(50.0),
        current_step: Some("compiling".to_string()),
    };
    assert!(!progress.is_terminal());

    let output = ProgressUpdate::Output {
        operation_id: "test_op".to_string(),
        line: "   Compiling async_cargo_mcp".to_string(),
        is_stderr: false,
    };
    assert!(!output.is_terminal());

    let completed = ProgressUpdate::Completed {
        operation_id: "test_op".to_string(),
        message: "Build successful".to_string(),
        duration_ms: 1000,
    };
    assert!(completed.is_terminal());

    let failed = ProgressUpdate::Failed {
        operation_id: "test_op".to_string(),
        error: "Build failed".to_string(),
        duration_ms: 500,
    };
    assert!(failed.is_terminal());

    let cancelled = ProgressUpdate::Cancelled {
        operation_id: "test_op".to_string(),
        message: "Build cancelled".to_string(),
        duration_ms: 300,
    };
    assert!(cancelled.is_terminal());

    let final_result = ProgressUpdate::FinalResult {
        operation_id: "test_op".to_string(),
        command: "cargo build".to_string(),
        description: "Building project".to_string(),
        working_directory: "/path/to/project".to_string(),
        success: true,
        duration_ms: 2000,
        full_output: "Build completed".to_string(),
    };
    assert!(final_result.is_terminal());
}

#[test]
fn test_progress_update_is_success() {
    let completed = ProgressUpdate::Completed {
        operation_id: "test_op".to_string(),
        message: "Build successful".to_string(),
        duration_ms: 1000,
    };
    assert!(completed.is_success());

    let final_result_success = ProgressUpdate::FinalResult {
        operation_id: "test_op".to_string(),
        command: "cargo build".to_string(),
        description: "Building project".to_string(),
        working_directory: "/path/to/project".to_string(),
        success: true,
        duration_ms: 2000,
        full_output: "Build completed".to_string(),
    };
    assert!(final_result_success.is_success());

    let final_result_failed = ProgressUpdate::FinalResult {
        operation_id: "test_op".to_string(),
        command: "cargo build".to_string(),
        description: "Building project".to_string(),
        working_directory: "/path/to/project".to_string(),
        success: false,
        duration_ms: 500,
        full_output: "Build failed".to_string(),
    };
    assert!(!final_result_failed.is_success());

    let failed = ProgressUpdate::Failed {
        operation_id: "test_op".to_string(),
        error: "Build failed".to_string(),
        duration_ms: 500,
    };
    assert!(!failed.is_success());

    let started = ProgressUpdate::Started {
        operation_id: "test_op".to_string(),
        command: "cargo build".to_string(),
        description: "Building project".to_string(),
    };
    assert!(!started.is_success());
}

#[test]
fn test_progress_update_is_failure() {
    let failed = ProgressUpdate::Failed {
        operation_id: "test_op".to_string(),
        error: "Build failed".to_string(),
        duration_ms: 500,
    };
    assert!(failed.is_failure());

    let cancelled = ProgressUpdate::Cancelled {
        operation_id: "test_op".to_string(),
        message: "Build cancelled".to_string(),
        duration_ms: 300,
    };
    assert!(cancelled.is_failure());

    let final_result_failed = ProgressUpdate::FinalResult {
        operation_id: "test_op".to_string(),
        command: "cargo build".to_string(),
        description: "Building project".to_string(),
        working_directory: "/path/to/project".to_string(),
        success: false,
        duration_ms: 500,
        full_output: "Build failed".to_string(),
    };
    assert!(final_result_failed.is_failure());

    let completed = ProgressUpdate::Completed {
        operation_id: "test_op".to_string(),
        message: "Build successful".to_string(),
        duration_ms: 1000,
    };
    assert!(!completed.is_failure());
}

#[test]
fn test_progress_update_operation_id() {
    let test_id = "test_operation_123";
    
    let updates = vec![
        ProgressUpdate::Started {
            operation_id: test_id.to_string(),
            command: "cargo build".to_string(),
            description: "Building project".to_string(),
        },
        ProgressUpdate::Progress {
            operation_id: test_id.to_string(),
            message: "Building...".to_string(),
            percentage: Some(50.0),
            current_step: Some("compiling".to_string()),
        },
        ProgressUpdate::Output {
            operation_id: test_id.to_string(),
            line: "   Compiling async_cargo_mcp".to_string(),
            is_stderr: false,
        },
        ProgressUpdate::Completed {
            operation_id: test_id.to_string(),
            message: "Build successful".to_string(),
            duration_ms: 1000,
        },
        ProgressUpdate::Failed {
            operation_id: test_id.to_string(),
            error: "Build failed".to_string(),
            duration_ms: 500,
        },
        ProgressUpdate::Cancelled {
            operation_id: test_id.to_string(),
            message: "Build cancelled".to_string(),
            duration_ms: 300,
        },
        ProgressUpdate::FinalResult {
            operation_id: test_id.to_string(),
            command: "cargo build".to_string(),
            description: "Building project".to_string(),
            working_directory: "/path/to/project".to_string(),
            success: true,
            duration_ms: 2000,
            full_output: "Build completed".to_string(),
        },
    ];

    for update in &updates {
        assert_eq!(update.operation_id(), test_id);
    }
}

#[test]
fn test_progress_update_duration_ms() {
    let completed = ProgressUpdate::Completed {
        operation_id: "test_op".to_string(),
        message: "Build successful".to_string(),
        duration_ms: 1000,
    };
    assert_eq!(completed.duration_ms(), Some(1000));

    let failed = ProgressUpdate::Failed {
        operation_id: "test_op".to_string(),
        error: "Build failed".to_string(),
        duration_ms: 500,
    };
    assert_eq!(failed.duration_ms(), Some(500));

    let final_result = ProgressUpdate::FinalResult {
        operation_id: "test_op".to_string(),
        command: "cargo build".to_string(),
        description: "Building project".to_string(),
        working_directory: "/path/to/project".to_string(),
        success: true,
        duration_ms: 2000,
        full_output: "Build completed".to_string(),
    };
    assert_eq!(final_result.duration_ms(), Some(2000));

    let started = ProgressUpdate::Started {
        operation_id: "test_op".to_string(),
        command: "cargo build".to_string(),
        description: "Building project".to_string(),
    };
    assert_eq!(started.duration_ms(), None);
}

#[test]
fn test_progress_update_variant_name() {
    let updates = vec![
        (ProgressUpdate::Started {
            operation_id: "test".to_string(),
            command: "cargo build".to_string(),
            description: "Building".to_string(),
        }, "Started"),
        (ProgressUpdate::Progress {
            operation_id: "test".to_string(),
            message: "Building...".to_string(),
            percentage: Some(50.0),
            current_step: Some("compiling".to_string()),
        }, "Progress"),
        (ProgressUpdate::Output {
            operation_id: "test".to_string(),
            line: "   Compiling".to_string(),
            is_stderr: false,
        }, "Output"),
        (ProgressUpdate::Completed {
            operation_id: "test".to_string(),
            message: "Build successful".to_string(),
            duration_ms: 1000,
        }, "Completed"),
        (ProgressUpdate::Failed {
            operation_id: "test".to_string(),
            error: "Build failed".to_string(),
            duration_ms: 500,
        }, "Failed"),
        (ProgressUpdate::Cancelled {
            operation_id: "test".to_string(),
            message: "Build cancelled".to_string(),
            duration_ms: 300,
        }, "Cancelled"),
        (ProgressUpdate::FinalResult {
            operation_id: "test".to_string(),
            command: "cargo build".to_string(),
            description: "Building project".to_string(),
            working_directory: "/path/to/project".to_string(),
            success: true,
            duration_ms: 2000,
            full_output: "Build completed".to_string(),
        }, "FinalResult"),
    ];

    for (update, expected_name) in updates {
        assert_eq!(update.variant_name(), expected_name);
    }
}
