//! Tests for command line timeout parameter functionality

use async_cargo_mcp::operation_monitor::{MonitorConfig, OperationMonitor};
use clap::Parser;
use std::sync::Arc;
use std::time::Duration;

/// Mock Args structure to test CLI parsing
#[derive(Parser)]
#[command(
    name = "async_cargo_mcp_test",
    version = "test",
    about = "Test CLI arguments"
)]
struct TestArgs {
    /// Override default timeout in seconds (default: 300)
    #[arg(
        long,
        value_name = "SECONDS",
        help = "Set default timeout in seconds (default: 300)"
    )]
    timeout: Option<u64>,
}

#[test]
fn test_simple() {
    assert_eq!(2 + 2, 4);
}

#[tokio::test]
async fn test_cli_timeout_parameter_parsing() {
    // Test parsing various timeout values
    let test_cases = vec![
        (vec!["test"], None),                            // No timeout specified
        (vec!["test", "--timeout", "600"], Some(600)),   // 10 minutes
        (vec!["test", "--timeout", "120"], Some(120)),   // 2 minutes
        (vec!["test", "--timeout", "1800"], Some(1800)), // 30 minutes
    ];

    for (args, expected_timeout) in test_cases {
        let parsed = TestArgs::try_parse_from(args).unwrap();
        assert_eq!(parsed.timeout, expected_timeout);
    }
}

#[tokio::test]
async fn test_monitor_config_with_custom_timeout() {
    // Test that MonitorConfig can be created with custom timeout
    let custom_timeout = Duration::from_secs(600); // 10 minutes

    let config = MonitorConfig::with_timeout(custom_timeout);
    assert_eq!(config.default_timeout, custom_timeout);

    // Verify other defaults remain unchanged
    assert_eq!(config.cleanup_interval, Duration::from_secs(21600));
    assert_eq!(config.max_history_size, 1000);
    assert!(config.auto_cleanup);
}

#[tokio::test]
async fn test_default_timeout_unchanged_when_no_cli_arg() {
    // Ensure backward compatibility - default behavior should remain unchanged
    let config = MonitorConfig::default();
    assert_eq!(config.default_timeout, Duration::from_secs(300));

    // Test with no CLI timeout argument
    let args = TestArgs::try_parse_from(vec!["test"]).unwrap();
    assert_eq!(args.timeout, None);

    // When None is passed to MonitorConfig::with_timeout_option()
    let config_from_none = MonitorConfig::with_timeout_option(None);
    assert_eq!(config_from_none.default_timeout, Duration::from_secs(300));
}

#[tokio::test]
async fn test_monitor_respects_custom_timeout() {
    // Test that OperationMonitor uses the custom timeout
    let custom_timeout = Duration::from_secs(120); // 2 minutes
    let config = MonitorConfig::with_timeout(custom_timeout);
    let monitor = Arc::new(OperationMonitor::new(config));

    // Verify the monitor returns the correct timeout
    let retrieved_timeout = monitor.get_default_timeout().await;
    assert_eq!(retrieved_timeout, custom_timeout);
}
