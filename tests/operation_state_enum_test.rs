use async_cargo_mcp::operation_monitor::OperationState;

#[test]
fn test_operation_state_default() {
    // Test that OperationState has a sensible default
    let default_state = OperationState::default();
    assert_eq!(default_state, OperationState::Pending);
}

#[test]
fn test_is_active_states() {
    // Test active state detection
    assert!(OperationState::Pending.is_active());
    assert!(OperationState::Running.is_active());
    assert!(!OperationState::Completed.is_active());
    assert!(!OperationState::Failed.is_active());
    assert!(!OperationState::Cancelled.is_active());
    assert!(!OperationState::TimedOut.is_active());
}

#[test]
fn test_is_terminal_states() {
    // Test terminal state detection (states that cannot transition further)
    assert!(!OperationState::Pending.is_terminal());
    assert!(!OperationState::Running.is_terminal());
    assert!(OperationState::Completed.is_terminal());
    assert!(OperationState::Failed.is_terminal());
    assert!(OperationState::Cancelled.is_terminal());
    assert!(OperationState::TimedOut.is_terminal());
}

#[test]
fn test_is_success_states() {
    // Test success state detection
    assert!(!OperationState::Pending.is_success());
    assert!(!OperationState::Running.is_success());
    assert!(OperationState::Completed.is_success());
    assert!(!OperationState::Failed.is_success());
    assert!(!OperationState::Cancelled.is_success());
    assert!(!OperationState::TimedOut.is_success());
}

#[test]
fn test_is_failure_states() {
    // Test failure state detection
    assert!(!OperationState::Pending.is_failure());
    assert!(!OperationState::Running.is_failure());
    assert!(!OperationState::Completed.is_failure());
    assert!(OperationState::Failed.is_failure());
    assert!(OperationState::Cancelled.is_failure());
    assert!(OperationState::TimedOut.is_failure());
}

#[test]
fn test_as_status_string() {
    // Test string representation for status display
    assert_eq!(OperationState::Pending.as_status_string(), "PENDING");
    assert_eq!(OperationState::Running.as_status_string(), "RUNNING");
    assert_eq!(OperationState::Completed.as_status_string(), "COMPLETED");
    assert_eq!(OperationState::Failed.as_status_string(), "FAILED");
    assert_eq!(OperationState::Cancelled.as_status_string(), "CANCELLED");
    assert_eq!(OperationState::TimedOut.as_status_string(), "TIMED_OUT");
}

#[test]
fn test_as_lowercase_string() {
    // Test lowercase string representation for filtering
    assert_eq!(OperationState::Pending.as_lowercase_string(), "pending");
    assert_eq!(OperationState::Running.as_lowercase_string(), "running");
    assert_eq!(OperationState::Completed.as_lowercase_string(), "completed");
    assert_eq!(OperationState::Failed.as_lowercase_string(), "failed");
    assert_eq!(OperationState::Cancelled.as_lowercase_string(), "cancelled");
    assert_eq!(OperationState::TimedOut.as_lowercase_string(), "timedout");
}

#[test]
fn test_can_transition_to() {
    // Test valid state transitions
    let pending = OperationState::Pending;
    assert!(pending.can_transition_to(&OperationState::Running));
    assert!(pending.can_transition_to(&OperationState::Cancelled));
    assert!(!pending.can_transition_to(&OperationState::Completed)); // Must go through Running first
    assert!(!pending.can_transition_to(&OperationState::Failed)); // Must go through Running first

    let running = OperationState::Running;
    assert!(running.can_transition_to(&OperationState::Completed));
    assert!(running.can_transition_to(&OperationState::Failed));
    assert!(running.can_transition_to(&OperationState::Cancelled));
    assert!(running.can_transition_to(&OperationState::TimedOut));
    assert!(!running.can_transition_to(&OperationState::Pending)); // Cannot go backward

    // Terminal states cannot transition anywhere
    let completed = OperationState::Completed;
    assert!(!completed.can_transition_to(&OperationState::Running));
    assert!(!completed.can_transition_to(&OperationState::Failed));

    let failed = OperationState::Failed;
    assert!(!failed.can_transition_to(&OperationState::Running));
    assert!(!failed.can_transition_to(&OperationState::Completed));
}

#[test]
fn test_from_string() {
    // Test parsing state from string (useful for filters)
    assert_eq!(
        OperationState::from_filter_string("pending"),
        Some(OperationState::Pending)
    );
    assert_eq!(
        OperationState::from_filter_string("running"),
        Some(OperationState::Running)
    );
    assert_eq!(
        OperationState::from_filter_string("completed"),
        Some(OperationState::Completed)
    );
    assert_eq!(
        OperationState::from_filter_string("failed"),
        Some(OperationState::Failed)
    );
    assert_eq!(
        OperationState::from_filter_string("cancelled"),
        Some(OperationState::Cancelled)
    );
    assert_eq!(
        OperationState::from_filter_string("timedout"),
        Some(OperationState::TimedOut)
    );

    // Test case-insensitive parsing
    assert_eq!(
        OperationState::from_filter_string("PENDING"),
        Some(OperationState::Pending)
    );
    assert_eq!(
        OperationState::from_filter_string("Running"),
        Some(OperationState::Running)
    );

    // Test unknown strings
    assert_eq!(OperationState::from_filter_string("unknown"), None);
    assert_eq!(OperationState::from_filter_string(""), None);
}

#[test]
fn test_progress_category() {
    // Test categorizing states for progress reporting
    assert_eq!(OperationState::Pending.progress_category(), "waiting");
    assert_eq!(OperationState::Running.progress_category(), "active");
    assert_eq!(OperationState::Completed.progress_category(), "success");
    assert_eq!(OperationState::Failed.progress_category(), "error");
    assert_eq!(OperationState::Cancelled.progress_category(), "cancelled");
    assert_eq!(OperationState::TimedOut.progress_category(), "timeout");
}

#[test]
fn test_all_active_states() {
    // Test utility for getting all active states
    let active_states = OperationState::all_active_states();
    assert_eq!(active_states.len(), 2);
    assert!(active_states.contains(&OperationState::Pending));
    assert!(active_states.contains(&OperationState::Running));
    assert!(!active_states.contains(&OperationState::Completed));
}

#[test]
fn test_all_terminal_states() {
    // Test utility for getting all terminal states
    let terminal_states = OperationState::all_terminal_states();
    assert_eq!(terminal_states.len(), 4);
    assert!(terminal_states.contains(&OperationState::Completed));
    assert!(terminal_states.contains(&OperationState::Failed));
    assert!(terminal_states.contains(&OperationState::Cancelled));
    assert!(terminal_states.contains(&OperationState::TimedOut));
    assert!(!terminal_states.contains(&OperationState::Pending));
    assert!(!terminal_states.contains(&OperationState::Running));
}

#[test]
fn test_all_failure_states() {
    // Test utility for getting all failure states
    let failure_states = OperationState::all_failure_states();
    assert_eq!(failure_states.len(), 3);
    assert!(failure_states.contains(&OperationState::Failed));
    assert!(failure_states.contains(&OperationState::Cancelled));
    assert!(failure_states.contains(&OperationState::TimedOut));
    assert!(!failure_states.contains(&OperationState::Completed));
    assert!(!failure_states.contains(&OperationState::Pending));
    assert!(!failure_states.contains(&OperationState::Running));
}
