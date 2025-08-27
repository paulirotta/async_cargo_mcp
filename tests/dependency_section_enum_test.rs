use async_cargo_mcp::cargo_tools::DependencySection;

#[test]
fn test_dependency_section_to_args() {
    // Test conversion to command line arguments
    let dev = DependencySection::Dev;
    let dev_args = dev.to_args();
    assert_eq!(dev_args, vec!["--dev".to_string()]);

    let build = DependencySection::Build;
    let build_args = build.to_args();
    assert_eq!(build_args, vec!["--build".to_string()]);

    let target = DependencySection::Target("x86_64-pc-windows-gnu".to_string());
    let target_args = target.to_args();
    assert_eq!(
        target_args,
        vec!["--target".to_string(), "x86_64-pc-windows-gnu".to_string()]
    );
}

#[test]
fn test_dependency_section_apply_to_command() {
    // Test applying arguments to a tokio::process::Command
    use tokio::process::Command;

    let mut cmd = Command::new("cargo");
    DependencySection::Dev.apply_to_command(&mut cmd);
    // Note: We can't easily test Command internals, but we test the method exists

    let mut cmd2 = Command::new("cargo");
    DependencySection::Build.apply_to_command(&mut cmd2);

    let mut cmd3 = Command::new("cargo");
    DependencySection::Target("test".to_string()).apply_to_command(&mut cmd3);
}

#[test]
fn test_dependency_section_display() {
    // Test human-readable display
    assert_eq!(DependencySection::Dev.to_string(), "dev");
    assert_eq!(DependencySection::Build.to_string(), "build");
    assert_eq!(
        DependencySection::Target("test".to_string()).to_string(),
        "target (test)"
    );
}

#[test]
fn test_dependency_section_is_methods() {
    // Test section type checking
    assert!(DependencySection::Dev.is_dev());
    assert!(!DependencySection::Dev.is_build());
    assert!(!DependencySection::Dev.is_target());

    assert!(!DependencySection::Build.is_dev());
    assert!(DependencySection::Build.is_build());
    assert!(!DependencySection::Build.is_target());

    let target = DependencySection::Target("test".to_string());
    assert!(!target.is_dev());
    assert!(!target.is_build());
    assert!(target.is_target());
}

#[test]
fn test_dependency_section_target_name() {
    // Test getting target name
    assert_eq!(DependencySection::Dev.target_name(), None);
    assert_eq!(DependencySection::Build.target_name(), None);

    let target = DependencySection::Target("x86_64-unknown-linux-gnu".to_string());
    assert_eq!(target.target_name(), Some("x86_64-unknown-linux-gnu"));
}

#[test]
fn test_dependency_section_from_string() {
    // Test valid cases using the new FromStr trait
    assert_eq!(
        "dev".parse::<DependencySection>(),
        Ok(DependencySection::Dev)
    );
    assert_eq!(
        "build".parse::<DependencySection>(),
        Ok(DependencySection::Build)
    );
    assert_eq!(
        "target:test".parse::<DependencySection>(),
        Ok(DependencySection::Target("test".to_string()))
    );

    // Test case insensitivity
    assert_eq!(
        "DEV".parse::<DependencySection>(),
        Ok(DependencySection::Dev)
    );
    assert_eq!(
        "Build".parse::<DependencySection>(),
        Ok(DependencySection::Build)
    );

    // Test invalid cases
    assert!("unknown".parse::<DependencySection>().is_err());
    assert!("".parse::<DependencySection>().is_err());
    assert!("target:".parse::<DependencySection>().is_err()); // Empty target name

    // Test direct .parse() usage (preferred over convenience method)
    assert_eq!(
        "dev".parse::<DependencySection>().ok(),
        Some(DependencySection::Dev)
    );
    assert_eq!("unknown".parse::<DependencySection>().ok(), None);
}
