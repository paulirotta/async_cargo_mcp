use async_cargo_mcp::{
    cargo_tools::AsyncCargo,
    operation_monitor::{MonitorConfig, OperationMonitor},
    shell_pool::{ShellPoolConfig, ShellPoolManager},
    test_cargo_tools,
};
use std::sync::Arc;
use std::time::Instant;
use tempfile::TempDir;

/// Comprehensive performance benchmark for shell pool vs direct execution
#[tokio::test]
async fn benchmark_shell_pool_performance() {
    // Create a temporary directory for testing
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let working_dir = temp_dir.path().to_string_lossy().to_string();

    // Initialize a more realistic Rust project with dependencies
    std::fs::write(
        temp_dir.path().join("Cargo.toml"),
        r#"[package]
name = "benchmark-project"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = "1.0"
tokio = { version = "1.0", features = ["full"] }
"#,
    )
    .expect("Failed to write Cargo.toml");

    std::fs::create_dir_all(temp_dir.path().join("src")).expect("Failed to create src directory");
    std::fs::write(
        temp_dir.path().join("src").join("lib.rs"),
        r#"use serde::{Deserialize, Serialize};
use tokio;

#[derive(Debug, Serialize, Deserialize)]
pub struct TestStruct {
    pub name: String,
    pub value: i32,
}

impl TestStruct {
    pub fn new(name: String, value: i32) -> Self {
        Self { name, value }
    }
}

#[tokio::main]
async fn main() {
    let test = TestStruct::new("example".to_string(), 42);
    println!("Test struct: {:?}", test);
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_creation() {
        let test = TestStruct::new("test".to_string(), 100);
        assert_eq!(test.name, "test");
        assert_eq!(test.value, 100);
    }
    
    #[tokio::test]
    async fn test_async_functionality() {
        tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
        assert!(true);
    }
}
"#,
    )
    .expect("Failed to write lib.rs");

    // Create monitor configuration
    let monitor_config = OperationMonitorConfig::default();
    let monitor = Arc::new(OperationMonitor::new(monitor_config));

    // Test configurations
    let direct_config = ShellPoolConfig {
        enabled: false,
        ..Default::default()
    };
    let pool_config = ShellPoolConfig {
        enabled: true,
        shells_per_directory: 3, // Use multiple shells for better performance
        max_total_shells: 10,
        ..Default::default()
    };

    // Create services
    let direct_service = AsyncCargo::new(
        monitor.clone(),
        Arc::new(ShellPoolManager::new(direct_config)),
    );
    let pool_service = AsyncCargo::new(
        monitor.clone(),
        Arc::new(ShellPoolManager::new(pool_config)),
    );

    println!("🚀 Starting Shell Pool Performance Benchmark");
    println!("============================================");

    // Test different cargo operations
    let operations = vec![
        ("check", "cargo check"),
        ("build", "cargo build"),
        ("test", "cargo test"),
    ];

    const ITERATIONS: usize = 3;
    let mut results = Vec::new();

    for (op_name, _op_desc) in &operations {
        println!("\n📊 Benchmarking: {}", op_name);
        println!("------------------------");

        // Warm up the shell pools
        match *op_name {
            "check" => {
                let _ = pool_service
                    .check_implementation(
                        serde_json::json!({
                            "working_directory": working_dir
                        })
                        .as_object()
                        .unwrap()
                        .clone(),
                    )
                    .await;
            }
            "build" => {
                let _ = pool_service
                    .build_implementation(
                        serde_json::json!({
                            "working_directory": working_dir
                        })
                        .as_object()
                        .unwrap()
                        .clone(),
                    )
                    .await;
            }
            "test" => {
                let _ = pool_service
                    .test_implementation(
                        serde_json::json!({
                            "working_directory": working_dir
                        })
                        .as_object()
                        .unwrap()
                        .clone(),
                    )
                    .await;
            }
            _ => {}
        }

        // Benchmark direct execution
        let mut direct_times = Vec::new();
        for i in 0..ITERATIONS {
            let start = Instant::now();
            let result = match *op_name {
                "check" => {
                    direct_service
                        .check_implementation(
                            serde_json::json!({
                                "working_directory": working_dir
                            })
                            .as_object()
                            .unwrap()
                            .clone(),
                        )
                        .await
                }
                "build" => {
                    direct_service
                        .build_implementation(
                            serde_json::json!({
                                "working_directory": working_dir
                            })
                            .as_object()
                            .unwrap()
                            .clone(),
                        )
                        .await
                }
                "test" => {
                    direct_service
                        .test_implementation(
                            serde_json::json!({
                                "working_directory": working_dir
                            })
                            .as_object()
                            .unwrap()
                            .clone(),
                        )
                        .await
                }
                _ => Ok("".to_string()),
            };
            let duration = start.elapsed();
            direct_times.push(duration);

            if let Err(e) = result {
                println!(
                    "⚠️  Direct execution {} iteration {} failed: {}",
                    op_name,
                    i + 1,
                    e
                );
            }
        }

        // Benchmark shell pool execution
        let mut pool_times = Vec::new();
        for i in 0..ITERATIONS {
            let start = Instant::now();
            let result = match *op_name {
                "check" => {
                    pool_service
                        .check_implementation(
                            serde_json::json!({
                                "working_directory": working_dir
                            })
                            .as_object()
                            .unwrap()
                            .clone(),
                        )
                        .await
                }
                "build" => {
                    pool_service
                        .build_implementation(
                            serde_json::json!({
                                "working_directory": working_dir
                            })
                            .as_object()
                            .unwrap()
                            .clone(),
                        )
                        .await
                }
                "test" => {
                    pool_service
                        .test_implementation(
                            serde_json::json!({
                                "working_directory": working_dir
                            })
                            .as_object()
                            .unwrap()
                            .clone(),
                        )
                        .await
                }
                _ => Ok("".to_string()),
            };
            let duration = start.elapsed();
            pool_times.push(duration);

            if let Err(e) = result {
                println!(
                    "⚠️  Pool execution {} iteration {} failed: {}",
                    op_name,
                    i + 1,
                    e
                );
            }
        }

        // Calculate statistics
        let avg_direct = direct_times.iter().sum::<std::time::Duration>() / ITERATIONS as u32;
        let avg_pool = pool_times.iter().sum::<std::time::Duration>() / ITERATIONS as u32;
        let improvement = avg_direct.as_nanos() as f64 / avg_pool.as_nanos() as f64;

        let min_direct = direct_times
            .iter()
            .min()
            .unwrap_or(&std::time::Duration::ZERO);
        let min_pool = pool_times
            .iter()
            .min()
            .unwrap_or(&std::time::Duration::ZERO);
        let max_direct = direct_times
            .iter()
            .max()
            .unwrap_or(&std::time::Duration::ZERO);
        let max_pool = pool_times
            .iter()
            .max()
            .unwrap_or(&std::time::Duration::ZERO);

        println!(
            "Direct Execution:\n\
             │ Average: {:?}\n\
             │ Min: {:?}\n\
             │ Max: {:?}",
            avg_direct, min_direct, max_direct
        );

        println!(
            "Shell Pool Execution:\n\
             │ Average: {:?}\n\
             │ Min: {:?}\n\
             │ Max: {:?}",
            avg_pool, min_pool, max_pool
        );

        println!("🎯 Improvement Factor: {:.2}x", improvement);

        results.push((op_name, improvement, avg_direct, avg_pool));

        // Verify shell pools are faster
        if avg_pool >= avg_direct {
            println!("⚠️  Warning: Shell pools not faster for {}", op_name);
        }
    }

    // Summary report
    println!("\n📈 BENCHMARK SUMMARY");
    println!("====================");
    for (op, improvement, direct, pool) in &results {
        println!(
            "{:>6}: {:.2}x faster ({:?} → {:?})",
            op, improvement, direct, pool
        );
    }

    let avg_improvement =
        results.iter().map(|(_, imp, _, _)| imp).sum::<f64>() / results.len() as f64;
    println!("Average improvement: {:.2}x", avg_improvement);

    // Assert overall improvement
    assert!(
        avg_improvement > 1.0,
        "Shell pools should provide performance improvement on average"
    );

    println!("✅ Benchmark completed successfully!");
}
