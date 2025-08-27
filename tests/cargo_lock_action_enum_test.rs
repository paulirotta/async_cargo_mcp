use async_cargo_mcp::cargo_tools::CargoLockAction;

#[test]
fn test_cargo_lock_action_default() {
    // Test that CargoLockAction has a sensible default
    let default_action = CargoLockAction::default();
    // Should default to C (do nothing) as safest option
    assert!(matches!(default_action, CargoLockAction::C));
}

#[test]
fn test_cargo_lock_action_is_methods() {
    // Test action type checking
    assert!(CargoLockAction::A.is_delete_and_clean());
    assert!(!CargoLockAction::A.is_delete_only());
    assert!(!CargoLockAction::A.is_no_op());

    assert!(!CargoLockAction::B.is_delete_and_clean());
    assert!(CargoLockAction::B.is_delete_only());
    assert!(!CargoLockAction::B.is_no_op());

    assert!(!CargoLockAction::C.is_delete_and_clean());
    assert!(!CargoLockAction::C.is_delete_only());
    assert!(CargoLockAction::C.is_no_op());
}

#[test]
fn test_cargo_lock_action_requires_deletion() {
    // Test if action requires file deletion
    assert!(CargoLockAction::A.requires_deletion());
    assert!(CargoLockAction::B.requires_deletion());
    assert!(!CargoLockAction::C.requires_deletion());
}

#[test]
fn test_cargo_lock_action_requires_clean() {
    // Test if action requires cargo clean
    assert!(CargoLockAction::A.requires_clean());
    assert!(!CargoLockAction::B.requires_clean());
    assert!(!CargoLockAction::C.requires_clean());
}

#[test]
fn test_cargo_lock_action_description() {
    // Test human-readable descriptions
    assert_eq!(
        CargoLockAction::A.description(),
        "Delete target/.cargo-lock then run cargo clean"
    );
    assert_eq!(
        CargoLockAction::B.description(),
        "Only delete .cargo-lock but do not clean"
    );
    assert_eq!(CargoLockAction::C.description(), "Do nothing");
}

#[test]
fn test_cargo_lock_action_as_letter() {
    // Test conversion to single letter representation
    assert_eq!(CargoLockAction::A.as_letter(), 'A');
    assert_eq!(CargoLockAction::B.as_letter(), 'B');
    assert_eq!(CargoLockAction::C.as_letter(), 'C');
}

#[test]
fn test_cargo_lock_action_from_letter() {
    // Test parsing from single letter
    assert_eq!(CargoLockAction::from_letter('A'), Some(CargoLockAction::A));
    assert_eq!(CargoLockAction::from_letter('B'), Some(CargoLockAction::B));
    assert_eq!(CargoLockAction::from_letter('C'), Some(CargoLockAction::C));

    // Test case insensitive
    assert_eq!(CargoLockAction::from_letter('a'), Some(CargoLockAction::A));
    assert_eq!(CargoLockAction::from_letter('b'), Some(CargoLockAction::B));
    assert_eq!(CargoLockAction::from_letter('c'), Some(CargoLockAction::C));

    // Test invalid
    assert_eq!(CargoLockAction::from_letter('D'), None);
    assert_eq!(CargoLockAction::from_letter('X'), None);
}

#[test]
fn test_cargo_lock_action_from_str() {
    // Test parsing from string using the new FromStr trait
    assert_eq!("A".parse::<CargoLockAction>(), Ok(CargoLockAction::A));
    assert_eq!(
        "delete-and-clean".parse::<CargoLockAction>(),
        Ok(CargoLockAction::A)
    );
    assert_eq!(
        "delete_and_clean".parse::<CargoLockAction>(),
        Ok(CargoLockAction::A)
    );

    assert_eq!("B".parse::<CargoLockAction>(), Ok(CargoLockAction::B));
    assert_eq!(
        "delete-only".parse::<CargoLockAction>(),
        Ok(CargoLockAction::B)
    );
    assert_eq!(
        "delete_only".parse::<CargoLockAction>(),
        Ok(CargoLockAction::B)
    );

    assert_eq!("C".parse::<CargoLockAction>(), Ok(CargoLockAction::C));
    assert_eq!("no-op".parse::<CargoLockAction>(), Ok(CargoLockAction::C));
    assert_eq!("noop".parse::<CargoLockAction>(), Ok(CargoLockAction::C));
    assert_eq!(
        "do-nothing".parse::<CargoLockAction>(),
        Ok(CargoLockAction::C)
    );

    // Test case insensitive
    assert_eq!("a".parse::<CargoLockAction>(), Ok(CargoLockAction::A));
    assert_eq!(
        "DELETE-AND-CLEAN".parse::<CargoLockAction>(),
        Ok(CargoLockAction::A)
    );

    // Test unknown cases
    assert!("unknown".parse::<CargoLockAction>().is_err());
    assert!("".parse::<CargoLockAction>().is_err());

    // Test direct .parse() usage (preferred over convenience method)
    assert_eq!(
        "A".parse::<CargoLockAction>().ok(),
        Some(CargoLockAction::A)
    );
    assert_eq!("unknown".parse::<CargoLockAction>().ok(), None);
}

#[test]
fn test_cargo_lock_action_display() {
    // Test display implementation for logging
    assert_eq!(CargoLockAction::A.to_string(), "Delete and clean (A)");
    assert_eq!(CargoLockAction::B.to_string(), "Delete only (B)");
    assert_eq!(CargoLockAction::C.to_string(), "Do nothing (C)");
}
