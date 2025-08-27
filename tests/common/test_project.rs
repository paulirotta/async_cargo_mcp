#![allow(dead_code)]

use anyhow::Result;
use tempfile::TempDir;
use tokio::fs;

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

    // Prepare paths
    let cargo_toml_path = project_path.join("Cargo.toml");
    let src_dir = project_path.join("src");

    // Create src directory first (parents may not exist); after this we can write files under src concurrently
    fs::create_dir_all(&src_dir).await?;

    // Build write tasks we can run concurrently
    // 1) Write Cargo.toml
    let write_cargo_toml = async {
        fs::write(&cargo_toml_path, cargo_toml)
            .await
            .map_err(anyhow::Error::from)
    };

    // 2) Prepare and write src content (main or lib+main)
    let src_main_path = src_dir.join("main.rs");
    let src_lib_path = src_dir.join("lib.rs");

    let write_src_files = async {
        if opts.with_integration_tests {
            // Write lib.rs and main.rs concurrently
            let lib_rs = r#"pub fn add(a: i32, b: i32) -> i32 { a + b }

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_add() { assert_eq!(add(2, 3), 5); }
}
"#;
            let write_lib = fs::write(&src_lib_path, lib_rs);
            let write_main = fs::write(
                &src_main_path,
                "fn main() { println!(\"Hello, world!\"); }\n",
            );
            tokio::try_join!(
                async { write_lib.await.map_err(anyhow::Error::from) },
                async { write_main.await.map_err(anyhow::Error::from) },
            )?;
        } else {
            // Binary-only variants: prepare main.rs content
            let main_content = if opts.with_formatting_issues {
                // poor formatting on purpose
                "fn main(){\nlet x=42;\n    let y =   43  ;\n        println!(\"Hello, world! {} {}\",x,y);\n}\n\n#[cfg(test)]\nmod tests {\n    #[test]\nfn it_works(  ) {\n        let result = 2+ 2;\n            assert_eq!( result,4 );\n    }\n}\n".to_string()
            } else if opts.with_warning {
                // unused variable warning
                "fn main() {\n    let unused_variable = 42;\n    println!(\"Hello, test world!\");\n}\n\n#[cfg(test)]\nmod tests {\n    #[test]\n    fn it_works() {\n        let result = 2 + 2;\n        assert_eq!(result, 4);\n    }\n}\n".to_string()
            } else {
                // simple hello world
                "fn main() {\n    println!(\"Hello, test world!\");\n}\n\n#[cfg(test)]\nmod tests {\n    #[test]\n    fn it_works() {\n        let result = 2 + 2;\n        assert_eq!(result, 4);\n    }\n}\n".to_string()
            };
            fs::write(&src_main_path, main_content)
                .await
                .map_err(anyhow::Error::from)?;
        }
        Ok(())
    };

    // 3) Optional: tests directory and integration test file
    let write_tests = async {
        if opts.with_integration_tests {
            let tests_dir = project_path.join("tests");
            fs::create_dir_all(&tests_dir)
                .await
                .map_err(anyhow::Error::from)?;
            let integration = r#"use test_project::add;

#[test]
fn integration_test_add() { assert_eq!(add(10, 20), 30); }

#[test]
fn integration_test_multiply() { assert_eq!(add(2, 3), 5); }
"#;
            fs::write(tests_dir.join("integration_tests.rs"), integration)
                .await
                .map_err(anyhow::Error::from)?;
        }
        Ok(())
    };

    // 4) Optional: bin dir and example binary
    let write_bin = async {
        if opts.with_binary_args_example {
            let bin_dir = src_dir.join("bin");
            fs::create_dir_all(&bin_dir)
                .await
                .map_err(anyhow::Error::from)?;
            let test_binary_rs = r#"fn main() {
    let args: Vec<String> = std::env::args().collect();
    println!("test_binary called with {} args:", args.len() - 1);
    for (i, arg) in args.iter().skip(1).enumerate() {
        println!("  arg[{}]: {}", i, arg);
    }
    if args.len() > 1 && args[1] == "--special" { println!("SPECIAL_MODE_ACTIVATED"); }
}
"#;
            fs::write(bin_dir.join("test_binary.rs"), test_binary_rs)
                .await
                .map_err(anyhow::Error::from)?;
        }
        Ok(())
    };

    // Run independent tasks concurrently: write Cargo.toml, src files, tests (optional), bin (optional)
    tokio::try_join!(write_cargo_toml, write_src_files, write_tests, write_bin)?;

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
