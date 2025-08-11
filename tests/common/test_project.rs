#![allow(dead_code)]

use anyhow::Result;
use std::fs;
use tempfile::TempDir;

/// Options to customize a temporary Cargo test project.
#[derive(Debug, Clone, Default)]
pub struct TestProjectOptions<'a> {
    /// Prefix for the temp dir name. A UUID will be appended automatically for uniqueness.
    pub prefix: Option<&'a str>,
    /// Create warnings in main (e.g., unused variable) for testing `cargo fix`.
    pub with_warning: bool,
    /// Add intentionally bad formatting to test `cargo fmt`.
    pub with_formatting_issues: bool,
    /// Create a bin target that accepts args at `src/bin/test_binary.rs`.
    pub with_binary_args_example: bool,
    /// Create a library with unit tests and an integration test.
    pub with_integration_tests: bool,
}

/// Create a temporary Cargo project with flexible content.
/// Ensures unique directory via tempfile and uuid and never writes to the repo root.
pub async fn create_test_cargo_project(opts: TestProjectOptions<'_>) -> Result<TempDir> {
    let uuid = uuid::Uuid::new_v4();
    let prefix = opts.prefix.unwrap_or("cargo_mcp_test_");
    let temp_dir = tempfile::Builder::new()
        .prefix(&format!("{prefix}{uuid}_"))
        .tempdir()?;
    let project_path = temp_dir.path();

    // Always create Cargo.toml
    let mut cargo_toml = String::from(
        r#"[package]
name = "test_project"
version = "0.1.0"
edition = "2024"
"#,
    );

    // Configure bins/lib based on options
    if opts.with_binary_args_example {
        cargo_toml.push_str(
            r#"

[[bin]]
name = "test_binary"
path = "src/bin/test_binary.rs"
"#,
        );
    }

    // dependencies placeholder
    cargo_toml.push_str(
        r#"

[dependencies]
"#,
    );

    fs::write(project_path.join("Cargo.toml"), cargo_toml)?;

    // Create src directory
    fs::create_dir_all(project_path.join("src"))?;

    // Main/lib content variants
    if opts.with_integration_tests {
        // Library with unit tests and main
        let lib_rs = r#"pub fn add(a: i32, b: i32) -> i32 { a + b }

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_add() { assert_eq!(add(2, 3), 5); }
}
"#;
        fs::write(project_path.join("src").join("lib.rs"), lib_rs)?;
        fs::write(
            project_path.join("src").join("main.rs"),
            "fn main() { println!(\"Hello, world!\"); }\n",
        )?;
        // Integration tests
        let tests_dir = project_path.join("tests");
        fs::create_dir_all(&tests_dir)?;
        let integration = r#"use test_project::add;

#[test]
fn integration_test_add() { assert_eq!(add(10, 20), 30); }

#[test]
fn integration_test_multiply() { assert_eq!(add(2, 3), 5); }
"#;
        fs::write(tests_dir.join("integration_tests.rs"), integration)?;
    } else {
        // Binary-only variants
        let main_content = if opts.with_formatting_issues {
            // poor formatting on purpose
            "fn main(){\nlet x=42;\n    let y =   43  ;\n        println!(\"Hello, world! {} {}\",x,y);\n}\n\n#[cfg(test)]\nmod tests {\n    #[test]\nfn it_works(  ) {\n        let result = 2+ 2;\n            assert_eq!( result,4 );\n    }\n}\n"
                .to_string()
        } else if opts.with_warning {
            // unused variable warning
            "fn main() {\n    let unused_variable = 42;\n    println!(\"Hello, test world!\");\n}\n\n#[cfg(test)]\nmod tests {\n    #[test]\n    fn it_works() {\n        let result = 2 + 2;\n        assert_eq!(result, 4);\n    }\n}\n"
            .to_string()
        } else {
            // simple hello world
            "fn main() {\n    println!(\"Hello, test world!\");\n}\n\n#[cfg(test)]\nmod tests {\n    #[test]\n    fn it_works() {\n        let result = 2 + 2;\n        assert_eq!(result, 4);\n    }\n}\n"
            .to_string()
        };
        fs::write(project_path.join("src").join("main.rs"), main_content)?;
    }

    // Optional bin for args example
    if opts.with_binary_args_example {
        let bin_dir = project_path.join("src").join("bin");
        fs::create_dir_all(&bin_dir)?;
        let test_binary_rs = r#"fn main() {
    let args: Vec<String> = std::env::args().collect();
    println!("test_binary called with {} args:", args.len() - 1);
    for (i, arg) in args.iter().skip(1).enumerate() {
        println!("  arg[{}]: {}", i, arg);
    }
    if args.len() > 1 && args[1] == "--special" { println!("SPECIAL_MODE_ACTIVATED"); }
}
"#;
        fs::write(bin_dir.join("test_binary.rs"), test_binary_rs)?;
    }

    Ok(temp_dir)
}

/// Convenience wrappers matching prior helper names
pub async fn create_basic_project() -> Result<TempDir> {
    create_test_cargo_project(TestProjectOptions::default()).await
}

pub async fn create_project_with_warning() -> Result<TempDir> {
    create_test_cargo_project(TestProjectOptions {
        with_warning: true,
        ..Default::default()
    })
    .await
}

pub async fn create_project_with_formatting_issues() -> Result<TempDir> {
    create_test_cargo_project(TestProjectOptions {
        with_formatting_issues: true,
        ..Default::default()
    })
    .await
}

pub async fn create_project_with_binary_args() -> Result<TempDir> {
    create_test_cargo_project(TestProjectOptions {
        with_binary_args_example: true,
        ..Default::default()
    })
    .await
}

pub async fn create_project_with_integration_tests() -> Result<TempDir> {
    create_test_cargo_project(TestProjectOptions {
        with_integration_tests: true,
        ..Default::default()
    })
    .await
}
