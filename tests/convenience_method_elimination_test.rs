use async_cargo_mcp::cargo_tools::{CargoLockAction, DependencySection};

/// Test suite for convenience method elimination
/// This test ensures that direct .parse() usage works as expected
/// before we eliminate the convenience methods

#[test]
fn test_dependency_section_direct_parsing() {
    // Test that direct .parse() calls work the same as convenience methods

    // Valid cases
    assert_eq!(
        "dev".parse::<DependencySection>().ok(),
        Some(DependencySection::Dev)
    );
    assert_eq!(
        "build".parse::<DependencySection>().ok(),
        Some(DependencySection::Build)
    );
    assert_eq!(
        "target:test".parse::<DependencySection>().ok(),
        Some(DependencySection::Target("test".to_string()))
    );

    // Case insensitive
    assert_eq!(
        "DEV".parse::<DependencySection>().ok(),
        Some(DependencySection::Dev)
    );
    assert_eq!(
        "Build".parse::<DependencySection>().ok(),
        Some(DependencySection::Build)
    );

    // Invalid cases
    assert_eq!("unknown".parse::<DependencySection>().ok(), None);
    assert_eq!("".parse::<DependencySection>().ok(), None);
    assert_eq!("target:".parse::<DependencySection>().ok(), None); // Empty target

    // Test that .parse().ok() pattern is now the standard way (convenience methods eliminated)
    let test_strings = vec!["dev", "build", "target:x86_64", "invalid", ""];
    for test_str in test_strings {
        let direct_parse = test_str.parse::<DependencySection>().ok();
        // Verify expected behavior: valid strings parse, invalid ones return None
        match test_str {
            "dev" => assert_eq!(direct_parse, Some(DependencySection::Dev)),
            "build" => assert_eq!(direct_parse, Some(DependencySection::Build)),
            "target:x86_64" => assert_eq!(
                direct_parse,
                Some(DependencySection::Target("x86_64".to_string()))
            ),
            "invalid" | "" => assert_eq!(direct_parse, None),
            _ => {} // Other valid patterns
        }
    }
}

#[test]
fn test_cargo_lock_action_direct_parsing() {
    // Test that direct .parse() calls work the same as convenience methods

    // Valid cases
    assert_eq!(
        "A".parse::<CargoLockAction>().ok(),
        Some(CargoLockAction::A)
    );
    assert_eq!(
        "B".parse::<CargoLockAction>().ok(),
        Some(CargoLockAction::B)
    );
    assert_eq!(
        "C".parse::<CargoLockAction>().ok(),
        Some(CargoLockAction::C)
    );

    // Alternative formats
    assert_eq!(
        "delete-and-clean".parse::<CargoLockAction>().ok(),
        Some(CargoLockAction::A)
    );
    assert_eq!(
        "delete-only".parse::<CargoLockAction>().ok(),
        Some(CargoLockAction::B)
    );
    assert_eq!(
        "no-op".parse::<CargoLockAction>().ok(),
        Some(CargoLockAction::C)
    );

    // Case insensitive
    assert_eq!(
        "a".parse::<CargoLockAction>().ok(),
        Some(CargoLockAction::A)
    );
    assert_eq!(
        "DELETE-AND-CLEAN".parse::<CargoLockAction>().ok(),
        Some(CargoLockAction::A)
    );

    // Invalid cases
    assert_eq!("unknown".parse::<CargoLockAction>().ok(), None);
    assert_eq!("".parse::<CargoLockAction>().ok(), None);

    // Test that .parse().ok() pattern is now the standard way (convenience methods eliminated)
    let test_strings = vec!["A", "B", "C", "a", "delete-and-clean", "invalid", ""];
    for test_str in test_strings {
        let direct_parse = test_str.parse::<CargoLockAction>().ok();
        // Verify expected behavior: valid strings parse, invalid ones return None
        match test_str {
            "A" | "a" => assert_eq!(direct_parse, Some(CargoLockAction::A)),
            "B" => assert_eq!(direct_parse, Some(CargoLockAction::B)),
            "C" => assert_eq!(direct_parse, Some(CargoLockAction::C)),
            "delete-and-clean" => assert_eq!(direct_parse, Some(CargoLockAction::A)),
            "invalid" | "" => assert_eq!(direct_parse, None),
            _ => {} // Other valid patterns
        }
    }
}

#[test]
fn test_preferred_error_handling_patterns() {
    // Test different error handling patterns with .parse()

    // Pattern 1: Using .ok() for Option<T> (matches current convenience method)
    let option_result: Option<DependencySection> = "dev".parse().ok();
    assert_eq!(option_result, Some(DependencySection::Dev));

    // Pattern 2: Using Result<T, E> for better error messages
    let result: Result<DependencySection, _> = "dev".parse();
    assert!(result.is_ok());

    let error_result: Result<DependencySection, _> = "invalid".parse();
    assert!(error_result.is_err());

    // Pattern 3: Using expect() for cases where we know the input is valid
    let expected: DependencySection = "dev".parse().expect("Should parse 'dev' successfully");
    assert_eq!(expected, DependencySection::Dev);
}

#[test]
fn test_convenience_method_elimination_safety() {
    // This test documents that eliminating from_string() methods is safe
    // because all functionality is preserved through the FromStr trait

    // Verify that FromStr provides all the functionality of the convenience methods
    assert!(
        std::str::FromStr::from_str("dev")
            .map(|_: DependencySection| ())
            .is_ok()
    );
    assert!(
        std::str::FromStr::from_str("A")
            .map(|_: CargoLockAction| ())
            .is_ok()
    );

    // Verify that error cases are handled properly
    assert!(
        std::str::FromStr::from_str("invalid")
            .map(|_: DependencySection| ())
            .is_err()
    );
    assert!(
        std::str::FromStr::from_str("invalid")
            .map(|_: CargoLockAction| ())
            .is_err()
    );
}

#[test]
fn test_tool_hint_preview_elimination() {
    use async_cargo_mcp::tool_hints;

    // Test that calling tool_hints::preview directly is equivalent
    // to the eliminated AsyncCargo::tool_hint_preview method
    let operation_id = "op_test_123";
    let operation_type = "build";

    let direct_call = tool_hints::preview(operation_id, operation_type);

    // Verify that the direct call provides all expected functionality
    assert!(
        direct_call.contains(operation_id),
        "Should contain operation ID"
    );
    assert!(
        direct_call.contains(operation_type),
        "Should contain operation type"
    );
    assert!(
        direct_call.contains("ASYNC CARGO OPERATION"),
        "Should have proper header"
    );
    assert!(
        direct_call.contains("status"),
        "Should mention status command"
    );
    assert!(direct_call.contains("wait"), "Should mention wait command");

    // This demonstrates that AsyncCargo::tool_hint_preview was unnecessary indirection
    // Direct calls to tool_hints::preview are cleaner and more explicit
}
