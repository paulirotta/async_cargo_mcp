use crate::callback_system::{CallbackSender, ProgressUpdate, no_callback};
use crate::mcp_callback::mcp_callback;
use crate::operation_monitor::OperationMonitor;
use rmcp::{
    ErrorData, RoleServer, ServerHandler,
    handler::server::{router::tool::ToolRouter, tool::Parameters},
    model::*,
    schemars,
    service::RequestContext,
    tool, tool_handler, tool_router,
};
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

#[derive(Debug, Clone, serde::Deserialize, schemars::JsonSchema)]
pub struct DependencyRequest {
    pub name: String,
    pub version: Option<String>,
    pub features: Option<Vec<String>>,
    pub optional: Option<bool>,
    pub working_directory: String,
    /// Enable async callback notifications for operation progress
    pub enable_async_notifications: Option<bool>,
}

#[derive(Debug, Clone, serde::Deserialize, schemars::JsonSchema)]
pub struct RemoveDependencyRequest {
    pub name: String,
    pub working_directory: String,
    /// Enable async callback notifications for operation progress
    pub enable_async_notifications: Option<bool>,
}

#[derive(Debug, Clone, serde::Deserialize, schemars::JsonSchema)]
pub struct BuildRequest {
    pub working_directory: String,
    /// Optional binary name to build (--bin parameter)
    pub bin_name: Option<String>,
    /// Build all packages in the workspace
    pub workspace: Option<bool>,
    /// Exclude packages from the build
    pub exclude: Option<Vec<String>>,
    /// Build only this package's library
    pub lib: Option<bool>,
    /// Build all binaries
    pub bins: Option<bool>,
    /// Build all examples
    pub examples: Option<bool>,
    /// Build only the specified example
    pub example: Option<String>,
    /// Build all tests
    pub tests: Option<bool>,
    /// Build only the specified test target
    pub test: Option<String>,
    /// Build all targets
    pub all_targets: Option<bool>,
    /// Space or comma separated list of features to activate
    pub features: Option<Vec<String>>,
    /// Activate all available features
    pub all_features: Option<bool>,
    /// Do not activate the `default` feature
    pub no_default_features: Option<bool>,
    /// Build artifacts in release mode, with optimizations
    pub release: Option<bool>,
    /// Build artifacts with the specified profile
    pub profile: Option<String>,
    /// Number of parallel jobs, defaults to # of CPUs
    pub jobs: Option<u32>,
    /// Build for the target triple
    pub target: Option<String>,
    /// Directory for all generated artifacts
    pub target_dir: Option<String>,
    /// Path to Cargo.toml
    pub manifest_path: Option<String>,
    /// Additional arguments to pass to build
    pub args: Option<Vec<String>>,
    /// Enable async callback notifications for operation progress
    pub enable_async_notifications: Option<bool>,
}

#[derive(Debug, Clone, serde::Deserialize, schemars::JsonSchema)]
pub struct RunRequest {
    pub working_directory: String,
    /// Optional binary name to run (--bin parameter)
    pub bin_name: Option<String>,
    /// Arguments to pass to the binary being run (after -- separator)
    pub binary_args: Option<Vec<String>>,
    /// Space or comma separated list of features to activate
    pub features: Option<Vec<String>>,
    /// Activate all available features
    pub all_features: Option<bool>,
    /// Do not activate the `default` feature
    pub no_default_features: Option<bool>,
    /// Build artifacts in release mode, with optimizations
    pub release: Option<bool>,
    /// Build artifacts with the specified profile
    pub profile: Option<String>,
    /// Build for the target triple
    pub target: Option<String>,
    /// Number of parallel jobs, defaults to # of CPUs
    pub jobs: Option<u32>,
    /// Path to Cargo.toml
    pub manifest_path: Option<String>,
    /// Additional cargo arguments
    pub cargo_args: Option<Vec<String>>,
    /// Enable async callback notifications for operation progress
    pub enable_async_notifications: Option<bool>,
}

#[derive(Debug, Clone, serde::Deserialize, schemars::JsonSchema)]
pub struct TestRequest {
    pub working_directory: String,
    /// Test name filter - if specified, only run tests containing this string
    pub test_name: Option<String>,
    /// Arguments for the test binary (after -- separator)  
    pub test_args: Option<Vec<String>>,
    /// Additional arguments to pass to cargo test
    pub args: Option<Vec<String>>,
    /// Package to run tests for
    pub package: Option<String>,
    /// Test all packages in the workspace
    pub workspace: Option<bool>,
    /// Exclude packages from the test
    pub exclude: Option<Vec<String>>,
    /// Test only this package's library
    pub lib: Option<bool>,
    /// Test all binaries
    pub bins: Option<bool>,
    /// Test only the specified binary
    pub bin: Option<String>,
    /// Test all examples
    pub examples: Option<bool>,
    /// Test only the specified example
    pub example: Option<String>,
    /// Test all test targets
    pub tests: Option<bool>,
    /// Test only the specified test target
    pub test: Option<String>,
    /// Test all targets (does not include doctests)
    pub all_targets: Option<bool>,
    /// Test only this library's documentation
    pub doc: Option<bool>,
    /// Space or comma separated list of features to activate
    pub features: Option<Vec<String>>,
    /// Activate all available features
    pub all_features: Option<bool>,
    /// Do not activate the `default` feature
    pub no_default_features: Option<bool>,
    /// Build artifacts in release mode, with optimizations
    pub release: Option<bool>,
    /// Build artifacts with the specified profile
    pub profile: Option<String>,
    /// Number of parallel jobs, defaults to # of CPUs
    pub jobs: Option<u32>,
    /// Build for the target triple
    pub target: Option<String>,
    /// Compile, but don't run tests
    pub no_run: Option<bool>,
    /// Run all tests regardless of failure
    pub no_fail_fast: Option<bool>,
    /// Path to Cargo.toml
    pub manifest_path: Option<String>,
    /// Enable async callback notifications for operation progress
    pub enable_async_notifications: Option<bool>,
}

#[derive(Debug, Clone, serde::Deserialize, schemars::JsonSchema)]
pub struct CheckRequest {
    pub working_directory: String,
    /// Enable async callback notifications for operation progress
    pub enable_async_notifications: Option<bool>,
}

#[derive(Debug, Clone, serde::Deserialize, schemars::JsonSchema)]
pub struct UpdateRequest {
    pub working_directory: String,
    /// Enable async callback notifications for operation progress
    pub enable_async_notifications: Option<bool>,
}

#[derive(Debug, Clone, serde::Deserialize, schemars::JsonSchema)]
pub struct DocRequest {
    pub working_directory: String,
    /// Enable async callback notifications for operation progress
    pub enable_async_notifications: Option<bool>,
}
#[derive(Debug, Clone, serde::Deserialize, schemars::JsonSchema)]
pub struct ClippyRequest {
    pub working_directory: String,
    /// Additional arguments to pass to clippy (e.g., ["--fix", "--allow-dirty"])
    pub args: Option<Vec<String>>,
    /// Enable async callback notifications for operation progress
    pub enable_async_notifications: Option<bool>,
}

#[derive(Debug, Clone, serde::Deserialize, schemars::JsonSchema)]
pub struct NextestRequest {
    pub working_directory: String,
    /// Additional arguments to pass to nextest (e.g., ["--all-features"])
    pub args: Option<Vec<String>>,
    /// Enable async callback notifications for operation progress
    pub enable_async_notifications: Option<bool>,
}

#[derive(Debug, Clone, serde::Deserialize, schemars::JsonSchema)]
pub struct CleanRequest {
    pub working_directory: String,
    /// Enable async callback notifications for operation progress
    pub enable_async_notifications: Option<bool>,
}

#[derive(Debug, Clone, serde::Deserialize, schemars::JsonSchema)]
pub struct FixRequest {
    pub working_directory: String,
    /// Additional arguments to pass to fix (e.g., ["--allow-dirty"])
    pub args: Option<Vec<String>>,
    /// Enable async callback notifications for operation progress
    pub enable_async_notifications: Option<bool>,
}

#[derive(Debug, Clone, serde::Deserialize, schemars::JsonSchema)]
pub struct SearchRequest {
    pub query: String,
    /// Limit the number of results
    pub limit: Option<u32>,
    /// Enable async callback notifications for operation progress
    pub enable_async_notifications: Option<bool>,
}

#[derive(Debug, Clone, serde::Deserialize, schemars::JsonSchema)]
pub struct BenchRequest {
    pub working_directory: String,
    /// Additional arguments to pass to bench
    pub args: Option<Vec<String>>,
    /// Enable async callback notifications for operation progress
    pub enable_async_notifications: Option<bool>,
}

#[derive(Debug, Clone, serde::Deserialize, schemars::JsonSchema)]
pub struct InstallRequest {
    pub package: String,
    pub version: Option<String>,
    pub working_directory: String,
    /// Enable async callback notifications for operation progress
    pub enable_async_notifications: Option<bool>,
}

#[derive(Debug, Clone, serde::Deserialize, schemars::JsonSchema)]
pub struct UpgradeRequest {
    pub working_directory: String,
    /// Upgrade to latest incompatible version
    pub incompatible: Option<bool>,
    /// Upgrade pinned to latest incompatible version
    pub pinned: Option<bool>,
    /// Perform a dry run without making changes
    pub dry_run: Option<bool>,
    /// Specific packages to upgrade
    pub packages: Option<Vec<String>>,
    /// Packages to exclude from upgrade
    pub exclude: Option<Vec<String>>,
    /// Additional arguments to pass to upgrade
    pub args: Option<Vec<String>>,
    /// Enable async callback notifications for operation progress
    pub enable_async_notifications: Option<bool>,
}

#[derive(Debug, Clone, serde::Deserialize, schemars::JsonSchema)]
pub struct AuditRequest {
    pub working_directory: String,
    /// Output format (default, json, yaml)
    pub format: Option<String>,
    /// Show only vulnerable dependencies
    pub vulnerabilities_only: Option<bool>,
    /// Deny warnings as errors
    pub deny_warnings: Option<bool>,
    /// Additional arguments to pass to audit
    pub args: Option<Vec<String>>,
    /// Enable async callback notifications for operation progress
    pub enable_async_notifications: Option<bool>,
}

#[derive(Debug, Clone, serde::Deserialize, schemars::JsonSchema)]
pub struct FmtRequest {
    pub working_directory: String,
    /// Check formatting without making changes
    pub check: Option<bool>,
    /// Format all packages in the workspace
    pub all: Option<bool>,
    /// Additional arguments to pass to fmt
    pub args: Option<Vec<String>>,
    /// Enable async callback notifications for operation progress
    pub enable_async_notifications: Option<bool>,
}

#[derive(Debug, Clone, serde::Deserialize, schemars::JsonSchema)]
pub struct TreeRequest {
    pub working_directory: String,
    /// Maximum display depth of the dependency tree
    pub depth: Option<u32>,
    /// Space-separated list of features to activate
    pub features: Option<Vec<String>>,
    /// Activate all available features
    pub all_features: Option<bool>,
    /// Do not activate the `default` feature
    pub no_default_features: Option<bool>,
    /// Output format (normal, json)
    pub format: Option<String>,
    /// Additional arguments to pass to tree
    pub args: Option<Vec<String>>,
    /// Enable async callback notifications for operation progress
    pub enable_async_notifications: Option<bool>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct VersionRequest {
    /// Enable verbose output showing more version details
    pub verbose: Option<bool>,
    /// Enable async callback notifications for operation progress
    pub enable_async_notifications: Option<bool>,
}

#[derive(Debug, Clone, serde::Deserialize, schemars::JsonSchema)]
pub struct FetchRequest {
    pub working_directory: String,
    /// Fetch dependencies for the target triple
    pub target: Option<String>,
    /// Space-separated list of features to activate
    pub features: Option<Vec<String>>,
    /// Activate all available features
    pub all_features: Option<bool>,
    /// Do not activate the `default` feature
    pub no_default_features: Option<bool>,
    /// Additional arguments to pass to fetch
    pub args: Option<Vec<String>>,
    /// Enable async callback notifications for operation progress
    pub enable_async_notifications: Option<bool>,
}

#[derive(Debug, Clone, serde::Deserialize, schemars::JsonSchema)]
pub struct RustcRequest {
    pub working_directory: String,
    /// Additional arguments to pass to rustc
    pub rustc_args: Option<Vec<String>>,
    /// Additional arguments to pass to cargo rustc
    pub cargo_args: Option<Vec<String>>,
    /// Enable async callback notifications for operation progress
    pub enable_async_notifications: Option<bool>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct MetadataRequest {
    pub working_directory: String,
    /// Output format (json is default and recommended)
    pub format: Option<String>,
    /// Do not include dependencies in the output
    pub no_deps: Option<bool>,
    /// Space-separated list of features to activate
    pub features: Option<Vec<String>>,
    /// Activate all available features
    pub all_features: Option<bool>,
    /// Do not activate the `default` feature
    pub no_default_features: Option<bool>,
    /// Additional arguments to pass to metadata
    pub args: Option<Vec<String>>,
    /// Enable async callback notifications for operation progress
    pub enable_async_notifications: Option<bool>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct WaitRequest {
    /// Optional operation ID to wait for. If not provided, waits for all active operations
    pub operation_id: Option<String>,
    /// Optional list of operation IDs to wait for concurrently. If provided, waits for all listed operations.
    pub operation_ids: Option<Vec<String>>,
    /// Timeout in seconds (default: 300)
    pub timeout_secs: Option<u64>,
}

#[derive(Clone, Debug)]
pub struct AsyncCargo {
    tool_router: ToolRouter<AsyncCargo>,
    monitor: Arc<OperationMonitor>,
}

impl Default for AsyncCargo {
    fn default() -> Self {
        use crate::operation_monitor::MonitorConfig;
        let monitor_config = MonitorConfig::default();
        let monitor = Arc::new(OperationMonitor::new(monitor_config));
        Self::new(monitor)
    }
}

#[tool_router]
impl AsyncCargo {
    /// Register and start an async operation with the monitor using the external operation_id.
    async fn register_async_operation(
        &self,
        operation_id: &str,
        command: &str,
        description: &str,
        working_directory: Option<String>,
    ) {
        // Register with the external ID so `wait` can find it immediately
        let _ = self
            .monitor
            .register_operation_with_id(
                operation_id.to_string(),
                command.to_string(),
                description.to_string(),
                None,
                working_directory,
            )
            .await;
        // Mark as started
        let _ = self.monitor.start_operation(operation_id).await;
    }
    pub fn new(monitor: Arc<OperationMonitor>) -> Self {
        Self {
            tool_router: Self::tool_router(),
            monitor,
        }
    }

    /// Check availability of optional cargo components
    pub async fn check_component_availability() -> HashMap<String, bool> {
        use tokio::process::Command;
        let mut availability = HashMap::new();

        // Check clippy
        let clippy_available = Command::new("cargo")
            .args(["clippy", "--version"])
            .output()
            .await
            .map(|output| output.status.success())
            .unwrap_or(false);
        availability.insert("clippy".to_string(), clippy_available);

        // Check nextest
        let nextest_available = Command::new("cargo")
            .args(["nextest", "--version"])
            .output()
            .await
            .map(|output| output.status.success())
            .unwrap_or(false);
        availability.insert("nextest".to_string(), nextest_available);

        // Check cargo-edit (upgrade command)
        let cargo_edit_available = Command::new("cargo")
            .args(["upgrade", "--version"])
            .output()
            .await
            .map(|output| output.status.success())
            .unwrap_or(false);
        availability.insert("cargo-edit".to_string(), cargo_edit_available);

        // Check cargo-audit
        let cargo_audit_available = Command::new("cargo")
            .args(["audit", "--version"])
            .output()
            .await
            .map(|output| output.status.success())
            .unwrap_or(false);
        availability.insert("cargo-audit".to_string(), cargo_audit_available);

        // Check rustfmt (for cargo fmt)
        let rustfmt_available = Command::new("rustfmt")
            .arg("--version")
            .output()
            .await
            .map(|output| output.status.success())
            .unwrap_or(false);
        availability.insert("rustfmt".to_string(), rustfmt_available);

        // Check if cargo is available (should always be true if we got this far)
        let cargo_available = Command::new("cargo")
            .arg("--version")
            .output()
            .await
            .map(|output| output.status.success())
            .unwrap_or(false);
        availability.insert("cargo".to_string(), cargo_available);

        availability
    }

    /// Generate availability report for LLM
    pub async fn generate_availability_report() -> String {
        let availability = Self::check_component_availability().await;

        let mut report = String::from("Cargo MCP Server Availability Report:\n");
        report.push_str("=====================================\n\n");

        report.push_str("Core Commands (always available):\n");
        report.push_str("+ build, test, run, check, doc, add, remove, update, clean, fix, search, bench, install, tree, version, fetch, rustc, metadata\n\n");

        report.push_str("Optional Components:\n");

        if *availability.get("clippy").unwrap_or(&false) {
            report.push_str("+ clippy - Available (enhanced linting)\n");
        } else {
            report
                .push_str("- clippy - Not available (install with: rustup component add clippy)\n");
        }

        if *availability.get("nextest").unwrap_or(&false) {
            report.push_str("+ nextest - Available (faster test runner)\n");
        } else {
            report.push_str(
                "- nextest - Not available (install with: cargo install cargo-nextest)\n",
            );
        }

        if *availability.get("cargo-edit").unwrap_or(&false) {
            report.push_str("+ cargo-edit - Available (upgrade command for dependency updates)\n");
        } else {
            report.push_str(
                "- cargo-edit - Not available (install with: cargo install cargo-edit)\n",
            );
        }

        if *availability.get("cargo-audit").unwrap_or(&false) {
            report.push_str("+ cargo-audit - Available (security vulnerability scanning)\n");
        } else {
            report.push_str(
                "- cargo-audit - Not available (install with: cargo install cargo-audit)\n",
            );
        }

        if *availability.get("rustfmt").unwrap_or(&false) {
            report.push_str("+ rustfmt - Available (code formatting with cargo fmt)\n");
        } else {
            report.push_str(
                "- rustfmt - Not available (install with: rustup component add rustfmt)\n",
            );
        }

        report.push_str("\nRecommendations:\n");
        report
            .push_str("* Use 'nextest' instead of 'test' for faster test execution if available\n");
        report.push_str("* Use 'clippy' for enhanced code quality checks if available\n");
        report.push_str(
            "* Use 'upgrade' for intelligent dependency updates if cargo-edit is available\n",
        );
        report.push_str(
            "* Use 'audit' for security vulnerability scanning if cargo-audit is available\n",
        );
        report.push_str(
            "* Enable async notifications (enable_async_notifications=true) for long operations\n",
        );

        report
    }

    fn _create_resource_text(&self, uri: &str, name: &str) -> Resource {
        RawResource::new(uri, name.to_string()).no_annotation()
    }

    fn generate_operation_id(&self) -> String {
        use chrono::{Local, Timelike};
        let now = Local::now();
        let midnight = now
            .with_hour(0)
            .unwrap()
            .with_minute(0)
            .unwrap()
            .with_second(0)
            .unwrap()
            .with_nanosecond(0)
            .unwrap();
        format!(
            "op_{}",
            (now.timestamp_millis() - midnight.timestamp_millis()) as u64
        )
    }

    /// Generate a tool hint message for LLMs when async operations are running
    fn generate_tool_hint(&self, operation_id: &str, operation_type: &str) -> String {
        crate::tool_hints::preview(operation_id, operation_type)
    }

    /// Public helper to preview the standardized tool hint content.
    pub fn tool_hint_preview(operation_id: &str, operation_type: &str) -> String {
        crate::tool_hints::preview(operation_id, operation_type)
    }

    #[tool(
        description = "Wait for async cargo operations to complete. Provide operation_id for one, operation_ids for many, or leave empty to wait for all active operations. Always use async_cargo_mcp MCP tools; do not run cargo in a terminal. For operations >1s, set enable_async_notifications=true and call mcp_async_cargo_m_wait to collect results."
    )]
    async fn wait(
        &self,
        Parameters(req): Parameters<WaitRequest>,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, ErrorData> {
        use std::time::Duration;
        let timeout_duration = Duration::from_secs(req.timeout_secs.unwrap_or(300)); // Default 5-minute timeout

        // Determine wait mode: multiple IDs, single ID, or all
        let wait_result = if let Some(ids) = req.operation_ids.clone() {
            if ids.is_empty() {
                // Treat empty list same as waiting for all
                tokio::time::timeout(timeout_duration, self.monitor.wait_for_all_operations())
                    .await
                    .map_err(|_| {
                        ErrorData::internal_error("Wait timed out for all operations", None)
                    })?
            } else {
                // Wait for each ID concurrently and collect using join handles
                let monitor = self.monitor.clone();
                let handles: Vec<_> = ids
                    .into_iter()
                    .map(|id| {
                        let monitor = monitor.clone();
                        tokio::spawn(async move { monitor.wait_for_operation(&id).await })
                    })
                    .collect();

                let combined = tokio::time::timeout(timeout_duration, async move {
                    let mut merged = Vec::new();
                    for handle in handles {
                        match handle.await {
                            Ok(Ok(mut ops)) => merged.append(&mut ops),
                            Ok(Err(err)) => {
                                use crate::operation_monitor::{OperationInfo, OperationState};
                                let info = OperationInfo {
                                    id: "unknown".to_string(),
                                    command: "wait".to_string(),
                                    description: format!("Wait internal error: {err}"),
                                    state: OperationState::Failed,
                                    start_time: std::time::Instant::now(),
                                    end_time: Some(std::time::Instant::now()),
                                    timeout_duration: None,
                                    working_directory: None,
                                    result: Some(Err(err)),
                                    cancellation_token: tokio_util::sync::CancellationToken::new(),
                                };
                                merged.push(info);
                            }
                            Err(join_err) => {
                                use crate::operation_monitor::{OperationInfo, OperationState};
                                let msg = format!("Join error waiting for operation: {join_err}");
                                let info = OperationInfo {
                                    id: "unknown".to_string(),
                                    command: "wait".to_string(),
                                    description: msg.clone(),
                                    state: OperationState::Failed,
                                    start_time: std::time::Instant::now(),
                                    end_time: Some(std::time::Instant::now()),
                                    timeout_duration: None,
                                    working_directory: None,
                                    result: Some(Err(msg)),
                                    cancellation_token: tokio_util::sync::CancellationToken::new(),
                                };
                                merged.push(info);
                            }
                        }
                    }
                    merged
                })
                .await
                .map_err(|_| {
                    ErrorData::internal_error("Wait timed out for specified operations", None)
                })?;
                Ok(combined)
            }
        } else if let Some(op_id) = req.operation_id {
            // Wait for a specific operation
            tokio::time::timeout(timeout_duration, self.monitor.wait_for_operation(&op_id))
                .await
                .map_err(|_| ErrorData::internal_error("Wait timed out", None))?
        } else {
            // Wait for all active operations
            tokio::time::timeout(timeout_duration, self.monitor.wait_for_all_operations())
                .await
                .map_err(|_| ErrorData::internal_error("Wait timed out for all operations", None))?
        };

        match wait_result {
            Ok(results) => {
                let content: Vec<Content> = results
                    .into_iter()
                    .map(|op_info| {
                        let status = match &op_info.state {
                            crate::operation_monitor::OperationState::Completed => {
                                if let Some(Ok(output)) = &op_info.result {
                                    // Provide placeholder if an implementation returned an empty Output: section
                                    let mut normalized = output.clone();
                                    // Detect trailing 'Output:' with nothing meaningful after it
                                    if normalized.trim_end().ends_with("Output:")
                                        || normalized.matches("Output:").last().map(|_| {
                                            // crude check: last occurrence followed only by whitespace
                                            if let Some(pos) = normalized.rfind("Output:") {
                                                normalized[pos + 7..].trim().is_empty()
                                            } else { false }
                                        }).unwrap_or(false)
                                    {
                                        // Replace only the final empty Output: occurrence
                                        if let Some(pos) = normalized.rfind("Output:") {
                                            let (head, _) = normalized.split_at(pos + 7);
                                            normalized = format!("{head} (no command stdout captured – command produced no stdout)");
                                        }
                                    }
                                    format!(
                                        "OPERATION COMPLETED: '{}'\n\
                                        Command: {}\n\
                                        Description: {}\n\
                                        Working Directory: {}\n\
                                        \n\
                                        === FULL OUTPUT ===\n\
                                        {}",
                                        op_info.id,
                                        op_info.command,
                                        op_info.description,
                                        op_info.working_directory.as_deref().unwrap_or("Unknown"),
                                        normalized
                                    )
                                } else {
                                    format!("Operation '{}' completed successfully (no detailed output available)", op_info.id)
                                }
                            }
                            crate::operation_monitor::OperationState::Failed => {
                                // Debug logging to see what result we actually have
                                tracing::debug!("Failed operation '{}': result = {:?}", op_info.id, op_info.result);

                                if let Some(Err(error_output)) = &op_info.result {
                                    // Return full error output for LLM consumption - this is the key fix!
                                    format!(
                                        "OPERATION FAILED: '{}'\n\
                                        Command: {}\n\
                                        Description: {}\n\
                                        Working Directory: {}\n\
                                        \n\
                                        === FULL ERROR OUTPUT ===\n\
                                        {}",
                                        op_info.id,
                                        op_info.command,
                                        op_info.description,
                                        op_info.working_directory.as_deref().unwrap_or("Unknown"),
                                        error_output
                                    )
                                } else {
                                    format!("Operation '{}' failed (no detailed error output available)", op_info.id)
                                }
                            }
                            crate::operation_monitor::OperationState::Cancelled => {
                                format!(
                                    "🚫 OPERATION CANCELLED: '{}'\n\
                                    Command: {}\n\
                                    Description: {}",
                                    op_info.id, op_info.command, op_info.description
                                )
                            }
                            crate::operation_monitor::OperationState::TimedOut => {
                                format!(
                                    "⏰ OPERATION TIMED OUT: '{}'\n\
                                    Command: {}\n\
                                    Description: {}",
                                    op_info.id, op_info.command, op_info.description
                                )
                            }
                            _ => format!("🔄 Operation '{}' is still in progress", op_info.id),
                        };
                        Content::text(status)
                    })
                    .collect();

                Ok(CallToolResult::success(content))
            }
            Err(err) => Ok(CallToolResult::success(vec![Content::text(format!(
                "- Wait operation failed: {err}"
            ))])),
        }
    }

    #[tool(
        description = "CARGO BUILD: Safer than terminal cargo. Use enable_async_notifications=true for builds >1s to multitask. Structured output with isolation. Always use async_cargo_mcp MCP tools; do not run cargo in a terminal. For operations >1s, set enable_async_notifications=true and call mcp_async_cargo_m_wait to collect results."
    )]
    async fn build(
        &self,
        Parameters(req): Parameters<BuildRequest>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, ErrorData> {
        let build_id = self.generate_operation_id();

        // Check if async notifications are enabled
        if req.enable_async_notifications.unwrap_or(false) {
            // TRUE 2-STAGE ASYNC PATTERN:
            // 1. Send immediate response that operation has started
            // 2. Spawn background task to do actual work and send notifications

            let peer = context.peer.clone();
            let req_clone = req.clone();
            let build_id_clone = build_id.clone();
            let monitor = self.monitor.clone();

            // Register operation BEFORE spawning so wait() can find it immediately
            self.register_async_operation(
                &build_id,
                "cargo build",
                "Building project in background",
                Some(req.working_directory.clone()),
            )
            .await;

            // Spawn background task for actual build work
            tokio::spawn(async move {
                // Create MCP callback sender to notify the LLM client
                let callback = mcp_callback(peer.clone(), build_id_clone.clone());

                // Send started notification immediately
                let _ = callback
                    .send_progress(ProgressUpdate::Started {
                        operation_id: build_id_clone.clone(),
                        command: "cargo build".to_string(),
                        description: "Building project in background".to_string(),
                    })
                    .await;

                let started_at = Instant::now();
                // Do the actual build work
                let result = Self::build_implementation(&req_clone).await;

                // Store the result in the operation monitor for later retrieval by wait
                // This ensures the full output (stdout/stderr) is available to `wait`
                let _ = monitor
                    .complete_operation(&build_id_clone, result.clone())
                    .await;

                // Send completion notification with measured duration
                let duration_ms = started_at.elapsed().as_millis() as u64;
                let completion_update = match result {
                    Ok(msg) => ProgressUpdate::Completed {
                        operation_id: build_id_clone,
                        message: msg,
                        duration_ms,
                    },
                    Err(err) => ProgressUpdate::Failed {
                        operation_id: build_id_clone,
                        error: err,
                        duration_ms,
                    },
                };

                if let Err(e) = callback.send_progress(completion_update).await {
                    tracing::error!("Failed to send build completion progress update: {e:?}");
                }
            });

            // Return immediate response to LLM - this is the "first stage"
            let tool_hint = self.generate_tool_hint(&build_id, "build");
            Ok(CallToolResult::success(vec![Content::text(format!(
                "Build operation {build_id} started in background.{tool_hint}"
            ))]))
        } else {
            // Synchronous operation for when async notifications are disabled
            match Self::build_implementation(&req).await {
                Ok(result_msg) => Ok(CallToolResult::success(vec![Content::text(result_msg)])),
                Err(error_msg) => Ok(CallToolResult::success(vec![Content::text(error_msg)])),
            }
        }
    }

    /// Internal implementation of build logic
    async fn build_implementation(req: &BuildRequest) -> Result<String, String> {
        use tokio::process::Command;

        let mut cmd = Command::new("cargo");
        cmd.arg("build");

        // Add package selection
        if req.workspace.unwrap_or(false) {
            cmd.arg("--workspace");
        }

        if let Some(exclude) = &req.exclude {
            for package in exclude {
                cmd.arg("--exclude").arg(package);
            }
        }

        // Add target selection
        if req.lib.unwrap_or(false) {
            cmd.arg("--lib");
        }

        if req.bins.unwrap_or(false) {
            cmd.arg("--bins");
        }

        if let Some(bin_name) = &req.bin_name {
            cmd.arg("--bin").arg(bin_name);
        }

        if req.examples.unwrap_or(false) {
            cmd.arg("--examples");
        }

        if let Some(example) = &req.example {
            cmd.arg("--example").arg(example);
        }

        if req.tests.unwrap_or(false) {
            cmd.arg("--tests");
        }

        if let Some(test) = &req.test {
            cmd.arg("--test").arg(test);
        }

        if req.all_targets.unwrap_or(false) {
            cmd.arg("--all-targets");
        }

        // Add feature selection
        if let Some(features) = &req.features
            && !features.is_empty()
        {
            // Filter out literal "default" which causes error if not declared explicitly
            let filtered: Vec<String> = features
                .iter()
                .filter(|f| f.as_str() != "default")
                .cloned()
                .collect();
            if !filtered.is_empty() {
                cmd.arg("--features").arg(filtered.join(","));
            }
        }

        if req.all_features.unwrap_or(false) {
            cmd.arg("--all-features");
        }

        if req.no_default_features.unwrap_or(false) {
            cmd.arg("--no-default-features");
        }

        // Add compilation options
        if req.release.unwrap_or(false) {
            cmd.arg("--release");
        }

        if let Some(profile) = &req.profile {
            cmd.arg("--profile").arg(profile);
        }

        if let Some(jobs) = req.jobs {
            cmd.arg("--jobs").arg(jobs.to_string());
        }

        // Validate target if provided (avoid failing build for cross targets not installed)
        let mut target_note: Option<String> = None;
        if let Some(target) = &req.target {
            let target_installed = {
                use std::process::Command as StdCommand;
                // Use rustup to list targets; fallback to assuming installed if rustup not available
                if let Ok(output) = StdCommand::new("rustup")
                    .args(["target", "list", "--installed"])
                    .output()
                {
                    let list = String::from_utf8_lossy(&output.stdout);
                    list.lines().any(|l| l.trim() == target)
                } else {
                    true
                }
            };
            if target_installed {
                cmd.arg("--target").arg(target);
            } else {
                target_note = Some(format!(
                    "[info] Requested target '{target}' not installed; building with host target instead."
                ));
            }
        }

        if let Some(target_dir) = &req.target_dir {
            cmd.arg("--target-dir").arg(target_dir);
        }

        // Add manifest options
        if let Some(manifest_path) = &req.manifest_path {
            cmd.arg("--manifest-path").arg(manifest_path);
        }

        // Add additional arguments
        if let Some(args) = &req.args {
            for arg in args {
                cmd.arg(arg);
            }
        }

        // Set working directory
        cmd.current_dir(&req.working_directory);

        // Execute command and collect full output
        let output = cmd.output().await.map_err(|e| {
            format!(
                "- Build operation failed in {}.\nError: Failed to execute cargo build: {}",
                &req.working_directory, e
            )
        })?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        let working_dir_msg = format!(" in {}", &req.working_directory);
        let bin_msg = if let Some(bin_name) = &req.bin_name {
            format!(" (binary: {bin_name})")
        } else {
            String::new()
        };

        let stdout_display = if stdout.trim().is_empty() && stderr.trim().is_empty() {
            "(no compiler output – build likely up to date)".to_string()
        } else {
            stdout.to_string()
        };
        if output.status.success() {
            let mut msg = format!(
                "+ Build completed successfully{working_dir_msg}{bin_msg}.\nOutput: {stdout_display}"
            );
            if let Some(note) = target_note {
                msg.push_str("\n");
                msg.push_str(&note);
            }
            Ok(msg)
        } else {
            Err(format!(
                "- Build failed{working_dir_msg}{bin_msg}.\nError: {stderr}\nOutput: {stdout_display}"
            ))
        }
    }

    #[tool(
        description = "CARGO RUN: Safer than terminal cargo. Use enable_async_notifications=true for long-running apps to multitask. Structured output with isolation. Always use async_cargo_mcp MCP tools; do not run cargo in a terminal. For operations >1s, set enable_async_notifications=true and call mcp_async_cargo_m_wait to collect results."
    )]
    async fn run(
        &self,
        Parameters(req): Parameters<RunRequest>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, ErrorData> {
        let run_id = self.generate_operation_id();

        // Check if async notifications are enabled
        if req.enable_async_notifications.unwrap_or(false) {
            // TRUE 2-STAGE ASYNC PATTERN:
            // 1. Send immediate response that operation has started
            // 2. Spawn background task to do actual work and send notifications

            let peer = context.peer.clone();
            let req_clone = req.clone();
            let run_id_clone = run_id.clone();
            let monitor = self.monitor.clone();

            // Register operation before spawn so wait() can find it immediately
            self.register_async_operation(
                &run_id,
                "cargo run",
                "Running application in background",
                Some(req.working_directory.clone()),
            )
            .await;

            // Spawn background task for actual run work
            tokio::spawn(async move {
                // Create MCP callback sender to notify the LLM client
                let callback = mcp_callback(peer, run_id_clone.clone());

                // Send started notification immediately
                let _ = callback
                    .send_progress(ProgressUpdate::Started {
                        operation_id: run_id_clone.clone(),
                        command: "cargo run".to_string(),
                        description: "Running application in background".to_string(),
                    })
                    .await;

                // Do the actual run work
                let started_at = Instant::now();
                let result = Self::run_implementation(&req_clone).await;

                // Store result for wait()
                let _ = monitor
                    .complete_operation(&run_id_clone, result.clone())
                    .await;

                // Send completion notification
                let duration_ms = started_at.elapsed().as_millis() as u64;
                let completion_update = match result {
                    Ok(msg) => ProgressUpdate::Completed {
                        operation_id: run_id_clone,
                        message: msg,
                        duration_ms,
                    },
                    Err(err) => ProgressUpdate::Failed {
                        operation_id: run_id_clone,
                        error: err,
                        duration_ms,
                    },
                };

                let _ = callback.send_progress(completion_update).await;
            });

            // Return immediate response to LLM - this is the "first stage"
            let tool_hint = self.generate_tool_hint(&run_id, "run");
            Ok(CallToolResult::success(vec![Content::text(format!(
                "Run operation {run_id} started in background.{tool_hint}"
            ))]))
        } else {
            // Synchronous operation for when async notifications are disabled
            match Self::run_implementation(&req).await {
                Ok(result_msg) => Ok(CallToolResult::success(vec![Content::text(result_msg)])),
                Err(error_msg) => Ok(CallToolResult::success(vec![Content::text(error_msg)])),
            }
        }
    }

    /// Internal implementation of run logic
    async fn run_implementation(req: &RunRequest) -> Result<String, String> {
        use tokio::process::Command;

        let mut cmd = Command::new("cargo");
        cmd.arg("run");

        // Add feature selection
        if let Some(features) = &req.features
            && !features.is_empty()
        {
            cmd.arg("--features").arg(features.join(","));
        }

        if req.all_features.unwrap_or(false) {
            cmd.arg("--all-features");
        }

        if req.no_default_features.unwrap_or(false) {
            cmd.arg("--no-default-features");
        }

        // Add compilation options
        if req.release.unwrap_or(false) {
            cmd.arg("--release");
        }

        if let Some(profile) = &req.profile {
            cmd.arg("--profile").arg(profile);
        }

        if let Some(target) = &req.target {
            cmd.arg("--target").arg(target);
        }

        if let Some(jobs) = req.jobs {
            cmd.arg("--jobs").arg(jobs.to_string());
        }

        // Add manifest path
        if let Some(manifest_path) = &req.manifest_path {
            cmd.arg("--manifest-path").arg(manifest_path);
        }

        // Add --bin parameter if specified
        if let Some(bin_name) = &req.bin_name {
            cmd.arg("--bin").arg(bin_name);
        }

        // Add additional cargo arguments
        if let Some(cargo_args) = &req.cargo_args {
            for arg in cargo_args {
                cmd.arg(arg);
            }
        }

        // Add binary arguments after -- separator
        if let Some(binary_args) = &req.binary_args
            && !binary_args.is_empty()
        {
            eprintln!("DEBUG: Adding binary args: {binary_args:?}");
            cmd.arg("--");
            for arg in binary_args {
                cmd.arg(arg);
                eprintln!("DEBUG: Added binary arg: {arg}");
            }
        }

        // Set working directory
        cmd.current_dir(&req.working_directory);

        let output = cmd
            .output()
            .await
            .map_err(|e| format!("Run operation failed: Failed to execute cargo run: {e}"))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        let working_dir_msg = format!(" in {}", &req.working_directory);
        let bin_msg = if let Some(bin_name) = &req.bin_name {
            format!(" (binary: {bin_name})")
        } else {
            String::new()
        };

        let args_msg = if let Some(binary_args) = &req.binary_args {
            if !binary_args.is_empty() {
                format!(" with args: [{}]", binary_args.join(", "))
            } else {
                String::new()
            }
        } else {
            String::new()
        };

        if output.status.success() {
            Ok(format!(
                "+ Run operation completed successfully{working_dir_msg}{bin_msg}{args_msg}.\nOutput: {stdout}"
            ))
        } else {
            Err(format!(
                "- Run operation failed{working_dir_msg}{bin_msg}{args_msg}.\nErrors: {stderr}\nOutput: {stdout}"
            ))
        }
    }

    #[tool(
        description = "CARGO TEST: Safer than terminal cargo. ALWAYS use enable_async_notifications=true for test suites to multitask. Real-time progress with isolation. Always use async_cargo_mcp MCP tools; do not run cargo in a terminal. For operations >1s, set enable_async_notifications=true and call mcp_async_cargo_m_wait to collect results."
    )]
    async fn test(
        &self,
        Parameters(req): Parameters<TestRequest>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, ErrorData> {
        let test_id = self.generate_operation_id();

        // Check if async notifications are enabled
        if req.enable_async_notifications.unwrap_or(false) {
            // TRUE 2-STAGE ASYNC PATTERN:
            // 1. Send immediate response that operation has started
            // 2. Spawn background task to do actual work and send notifications

            let peer = context.peer.clone();
            let req_clone = req.clone();
            let test_id_clone = test_id.clone();
            let monitor = self.monitor.clone();

            // Register operation before spawn
            self.register_async_operation(
                &test_id,
                "cargo test",
                "Running test suite in background",
                Some(req.working_directory.clone()),
            )
            .await;

            // Spawn background task for actual test work
            tokio::spawn(async move {
                // Create MCP callback sender to notify the LLM client
                let callback = mcp_callback(peer, test_id_clone.clone());

                // Send started notification immediately
                let _ = callback
                    .send_progress(ProgressUpdate::Started {
                        operation_id: test_id_clone.clone(),
                        command: "cargo test".to_string(),
                        description: "Running test suite in background".to_string(),
                    })
                    .await;

                // Do the actual test work
                let started_at = Instant::now();
                let result = Self::test_implementation(&req_clone).await;
                // Store for wait
                let _ = monitor
                    .complete_operation(&test_id_clone, result.clone())
                    .await;

                // Send completion notification
                let duration_ms = started_at.elapsed().as_millis() as u64;
                let completion_update = match result {
                    Ok(msg) => ProgressUpdate::Completed {
                        operation_id: test_id_clone,
                        message: msg,
                        duration_ms,
                    },
                    Err(err) => ProgressUpdate::Failed {
                        operation_id: test_id_clone,
                        error: err,
                        duration_ms,
                    },
                };

                let _ = callback.send_progress(completion_update).await;
            });

            // Return immediate response to LLM - this is the "first stage"
            let tool_hint = self.generate_tool_hint(&test_id, "test");
            Ok(CallToolResult::success(vec![Content::text(format!(
                "Test operation {test_id} started in background.{tool_hint}"
            ))]))
        } else {
            // Synchronous operation for when async notifications are disabled
            match Self::test_implementation(&req).await {
                Ok(result_msg) => Ok(CallToolResult::success(vec![Content::text(result_msg)])),
                Err(error_msg) => Ok(CallToolResult::success(vec![Content::text(error_msg)])),
            }
        }
    }

    /// Internal implementation of test logic
    async fn test_implementation(req: &TestRequest) -> Result<String, String> {
        use tokio::process::Command;

        let test_id = uuid::Uuid::new_v4().to_string()[..8].to_string();

        let mut cmd = Command::new("cargo");
        cmd.arg("test");

        // Add package selection
        if let Some(package) = &req.package {
            cmd.arg("--package").arg(package);
        }

        if req.workspace.unwrap_or(false) {
            cmd.arg("--workspace");
        }

        if let Some(exclude) = &req.exclude {
            for pkg in exclude {
                cmd.arg("--exclude").arg(pkg);
            }
        }

        // Add target selection
        if req.lib.unwrap_or(false) {
            cmd.arg("--lib");
        }

        if req.bins.unwrap_or(false) {
            cmd.arg("--bins");
        }

        if let Some(bin) = &req.bin {
            cmd.arg("--bin").arg(bin);
        }

        if req.examples.unwrap_or(false) {
            cmd.arg("--examples");
        }

        if let Some(example) = &req.example {
            cmd.arg("--example").arg(example);
        }

        if req.tests.unwrap_or(false) {
            cmd.arg("--tests");
        }

        if let Some(test) = &req.test {
            cmd.arg("--test").arg(test);
        }

        if req.all_targets.unwrap_or(false) {
            cmd.arg("--all-targets");
        }

        if req.doc.unwrap_or(false) {
            cmd.arg("--doc");
        }

        // Add feature selection
        if let Some(features) = &req.features
            && !features.is_empty()
        {
            cmd.arg("--features").arg(features.join(","));
        }

        if req.all_features.unwrap_or(false) {
            cmd.arg("--all-features");
        }

        if req.no_default_features.unwrap_or(false) {
            cmd.arg("--no-default-features");
        }

        // Add compilation options
        if req.release.unwrap_or(false) {
            cmd.arg("--release");
        }

        if let Some(profile) = &req.profile {
            cmd.arg("--profile").arg(profile);
        }

        if let Some(jobs) = req.jobs {
            cmd.arg("--jobs").arg(jobs.to_string());
        }

        if let Some(target) = &req.target {
            cmd.arg("--target").arg(target);
        }

        // Add test options
        if req.no_run.unwrap_or(false) {
            cmd.arg("--no-run");
        }

        if req.no_fail_fast.unwrap_or(false) {
            cmd.arg("--no-fail-fast");
        }

        // Add manifest options
        if let Some(manifest_path) = &req.manifest_path {
            cmd.arg("--manifest-path").arg(manifest_path);
        }

        // Add additional cargo arguments
        if let Some(args) = &req.args {
            for arg in args {
                cmd.arg(arg);
            }
        }

        // Add test name filter as positional argument
        if let Some(test_name) = &req.test_name {
            cmd.arg(test_name);
        }

        // Add test arguments after -- separator
        if let Some(test_args) = &req.test_args
            && !test_args.is_empty()
        {
            cmd.arg("--");
            for arg in test_args {
                cmd.arg(arg);
            }
        }

        // Set working directory
        cmd.current_dir(&req.working_directory);

        let output = cmd
            .output()
            .await
            .map_err(|e| format!("Failed to execute cargo test: {e}"))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        let working_dir_msg = format!(" in {}", &req.working_directory);
        let test_filter_msg = if let Some(test_name) = &req.test_name {
            format!(" (filter: {test_name})")
        } else {
            String::new()
        };

        if output.status.success() {
            Ok(format!(
                "Test operation #{test_id} completed successfully{working_dir_msg}{test_filter_msg}.\nOutput: {stdout}"
            ))
        } else {
            Err(format!(
                "- Test operation #{test_id} failed{working_dir_msg}{test_filter_msg}.\nErrors: {stderr}\nOutput: {stdout}"
            ))
        }
    }

    #[tool(
        description = "CARGO CHECK: Safer than terminal cargo. Fast validation - async optional for large projects. Quick compile check. Always use async_cargo_mcp MCP tools; do not run cargo in a terminal. For operations >1s, set enable_async_notifications=true and call mcp_async_cargo_m_wait to collect results."
    )]
    async fn check(
        &self,
        Parameters(req): Parameters<CheckRequest>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, ErrorData> {
        let check_id = self.generate_operation_id();

        // Check if async notifications are enabled
        if req.enable_async_notifications.unwrap_or(false) {
            // TRUE 2-STAGE ASYNC PATTERN:
            // 1. Send immediate response that operation has started
            // 2. Spawn background task to do actual work and send notifications

            let peer = context.peer.clone();
            let req_clone = req.clone();
            let check_id_clone = check_id.clone();
            let monitor = self.monitor.clone();

            // Register operation before spawn
            self.register_async_operation(
                &check_id,
                "cargo check",
                "Checking project in background",
                Some(req.working_directory.clone()),
            )
            .await;

            // Spawn background task for actual check work
            tokio::spawn(async move {
                // Create MCP callback sender to notify the LLM client
                let callback = mcp_callback(peer, check_id_clone.clone());

                // Send started notification immediately
                let _ = callback
                    .send_progress(ProgressUpdate::Started {
                        operation_id: check_id_clone.clone(),
                        command: "cargo check".to_string(),
                        description: "Checking project in background".to_string(),
                    })
                    .await;

                // Do the actual check work
                let started_at = Instant::now();
                let result = Self::check_implementation(&req_clone).await;
                // Store for wait
                let _ = monitor
                    .complete_operation(&check_id_clone, result.clone())
                    .await;

                // Send completion notification
                let duration_ms = started_at.elapsed().as_millis() as u64;
                let completion_update = match result {
                    Ok(msg) => ProgressUpdate::Completed {
                        operation_id: check_id_clone,
                        message: msg,
                        duration_ms,
                    },
                    Err(err) => ProgressUpdate::Failed {
                        operation_id: check_id_clone,
                        error: err,
                        duration_ms,
                    },
                };

                let _ = callback.send_progress(completion_update).await;
            });

            // Return immediate response to LLM - this is the "first stage"
            let tool_hint = self.generate_tool_hint(&check_id, "check");
            Ok(CallToolResult::success(vec![Content::text(format!(
                "+ Check operation {check_id} started in background.{tool_hint}"
            ))]))
        } else {
            // Synchronous operation for when async notifications are disabled
            match Self::check_implementation(&req).await {
                Ok(result_msg) => Ok(CallToolResult::success(vec![Content::text(result_msg)])),
                Err(error_msg) => Ok(CallToolResult::success(vec![Content::text(error_msg)])),
            }
        }
    }

    /// Internal implementation of check logic
    async fn check_implementation(req: &CheckRequest) -> Result<String, String> {
        use tokio::process::Command;

        let mut cmd = Command::new("cargo");
        cmd.arg("check");

        // Set working directory
        cmd.current_dir(&req.working_directory);

        let output = cmd.output().await.map_err(|e| {
            format!(
                "- Check operation failed in {}.\nError: Failed to execute cargo check: {}",
                &req.working_directory, e
            )
        })?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        let working_dir_msg = format!(" in {}", &req.working_directory);

        if output.status.success() {
            Ok(format!(
                "+ Check operation completed successfully{working_dir_msg}.\nOutput: {stdout}"
            ))
        } else {
            Err(format!(
                "- Check operation failed{working_dir_msg}.\nErrors: {stderr}\nOutput: {stdout}"
            ))
        }
    }

    #[tool(
        description = "CARGO ADD: Safer than terminal cargo. Synchronous operation for Cargo.toml modifications. Handles version conflicts. Always use async_cargo_mcp MCP tools; do not run cargo in a terminal. For operations >1s, set enable_async_notifications=true and call mcp_async_cargo_m_wait to collect results."
    )]
    async fn add(
        &self,
        Parameters(req): Parameters<DependencyRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let add_id = self.generate_operation_id();

        // Always use synchronous execution for Cargo.toml modifications
        use tokio::process::Command;

        let mut cmd = Command::new("cargo");

        // Build the dependency specification
        let dep_spec = if let Some(version) = &req.version {
            format!("{}@{}", req.name, version)
        } else {
            req.name.clone()
        };

        cmd.arg("add").arg(&dep_spec);

        // Set working directory
        cmd.current_dir(&req.working_directory);

        // Add optional features
        if let Some(features) = &req.features
            && !features.is_empty()
        {
            cmd.arg("--features").arg(features.join(","));
        }

        // Add optional flag
        if req.optional.unwrap_or(false) {
            cmd.arg("--optional");
        }

        let output = cmd.output().await.map_err(|e| {
            ErrorData::internal_error(format!("Failed to execute cargo add: {e}"), None)
        })?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        let working_dir_msg = format!(" in {}", &req.working_directory);

        let result_msg = if output.status.success() {
            format!(
                "➕ Add operation #{add_id} completed successfully{working_dir_msg}.\nAdded dependency: {}\nOutput: {stdout}",
                req.name
            )
        } else {
            format!(
                "- Add operation #{add_id} failed{working_dir_msg}.\nDependency: {}\nError: {stderr}\nOutput: {stdout}",
                req.name
            )
        };

        Ok(CallToolResult::success(vec![Content::text(result_msg)]))
    }

    #[tool(
        description = "CARGO REMOVE: Safer than terminal cargo. Synchronous operation for Cargo.toml modifications. Prevents Cargo.toml corruption. Always use async_cargo_mcp MCP tools; do not run cargo in a terminal. For operations >1s, set enable_async_notifications=true and call mcp_async_cargo_m_wait to collect results."
    )]
    async fn remove(
        &self,
        Parameters(req): Parameters<RemoveDependencyRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let remove_id = self.generate_operation_id();

        // Always use synchronous execution for Cargo.toml modifications
        use tokio::process::Command;

        let mut cmd = Command::new("cargo");
        cmd.arg("remove").arg(&req.name);

        // Set working directory
        cmd.current_dir(&req.working_directory);

        let output = cmd.output().await.map_err(|e| {
            ErrorData::internal_error(format!("Failed to execute cargo remove: {e}"), None)
        })?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        let working_dir_msg = format!(" in {}", &req.working_directory);

        let result_msg = if output.status.success() {
            format!(
                "➖ Remove operation #{remove_id} completed successfully{working_dir_msg}.\nRemoved dependency: {}\nOutput: {stdout}",
                req.name
            )
        } else {
            format!(
                "- Remove operation #{remove_id} failed{working_dir_msg}.\nDependency: {}\nError: {stderr}\nOutput: {stdout}",
                req.name
            )
        };

        Ok(CallToolResult::success(vec![Content::text(result_msg)]))
    }

    #[tool(
        description = "CARGO UPDATE: Safer than terminal cargo. Use enable_async_notifications=true for large projects to multitask. Shows version changes. Always use async_cargo_mcp MCP tools; do not run cargo in a terminal. For operations >1s, set enable_async_notifications=true and call mcp_async_cargo_m_wait to collect results."
    )]
    async fn update(
        &self,
        Parameters(req): Parameters<UpdateRequest>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, ErrorData> {
        let update_id = self.generate_operation_id();

        // Check if async notifications are enabled
        if req.enable_async_notifications.unwrap_or(false) {
            // TRUE 2-STAGE ASYNC PATTERN:
            // 1. Send immediate response that operation has started
            // 2. Spawn background task to do actual work and send notifications

            let peer = context.peer.clone();
            let req_clone = req.clone();
            let update_id_clone = update_id.clone();
            let monitor = self.monitor.clone();

            // Register operation before spawn
            self.register_async_operation(
                &update_id,
                "cargo update",
                "Updating dependencies in background",
                Some(req.working_directory.clone()),
            )
            .await;

            // Spawn background task for actual update work
            tokio::spawn(async move {
                // Create MCP callback sender to notify the LLM client
                let callback = mcp_callback(peer, update_id_clone.clone());

                // Send started notification immediately
                let _ = callback
                    .send_progress(ProgressUpdate::Started {
                        operation_id: update_id_clone.clone(),
                        command: "cargo update".to_string(),
                        description: "Updating dependencies in background".to_string(),
                    })
                    .await;

                // Do the actual update work
                let started_at = Instant::now();
                let result = Self::update_implementation(&req_clone).await;
                // Store for wait
                let _ = monitor
                    .complete_operation(&update_id_clone, result.clone())
                    .await;

                // Send completion notification
                let duration_ms = started_at.elapsed().as_millis() as u64;
                let completion_update = match result {
                    Ok(msg) => ProgressUpdate::Completed {
                        operation_id: update_id_clone,
                        message: msg,
                        duration_ms,
                    },
                    Err(err) => ProgressUpdate::Failed {
                        operation_id: update_id_clone,
                        error: err,
                        duration_ms,
                    },
                };

                let _ = callback.send_progress(completion_update).await;
            });

            // Return immediate response to LLM - this is the "first stage"
            let tool_hint = self.generate_tool_hint(&update_id, "update");
            Ok(CallToolResult::success(vec![Content::text(format!(
                "Update operation {update_id} started in background.{tool_hint}"
            ))]))
        } else {
            // Synchronous operation for when async notifications are disabled
            match Self::update_implementation(&req).await {
                Ok(result_msg) => Ok(CallToolResult::success(vec![Content::text(result_msg)])),
                Err(error_msg) => Ok(CallToolResult::success(vec![Content::text(error_msg)])),
            }
        }
    }

    /// Internal implementation of update logic
    async fn update_implementation(req: &UpdateRequest) -> Result<String, String> {
        use tokio::process::Command;

        let mut cmd = Command::new("cargo");
        cmd.arg("update");

        // Set working directory
        cmd.current_dir(&req.working_directory);

        let output = cmd
            .output()
            .await
            .map_err(|e| format!("Update operation failed: Failed to execute cargo update: {e}"))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        let working_dir_msg = format!(" in {}", &req.working_directory);

        if output.status.success() {
            Ok(format!(
                "Update operation completed successfully{working_dir_msg}.\nOutput: {stdout}"
            ))
        } else {
            Err(format!(
                "- Update operation failed{working_dir_msg}.\nErrors: {stderr}\nOutput: {stdout}"
            ))
        }
    }

    #[tool(
        description = "CARGO DOC: Safer than terminal cargo. Use enable_async_notifications=true for large codebases to multitask. Creates LLM-friendly API reference. Always use async_cargo_mcp MCP tools; do not run cargo in a terminal. For operations >1s, set enable_async_notifications=true and call mcp_async_cargo_m_wait to collect results."
    )]
    async fn doc(
        &self,
        Parameters(req): Parameters<DocRequest>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, ErrorData> {
        let doc_id = self.generate_operation_id();

        // Check if async notifications are enabled
        if req.enable_async_notifications.unwrap_or(false) {
            // TRUE 2-STAGE ASYNC PATTERN:
            // 1. Send immediate response that operation has started
            // 2. Spawn background task to do actual work and send notifications

            let peer = context.peer.clone();
            let req_clone = req.clone();
            let doc_id_clone = doc_id.clone();
            let monitor = self.monitor.clone();

            // Register operation before spawn
            self.register_async_operation(
                &doc_id,
                "cargo doc",
                "Generating documentation in background",
                Some(req.working_directory.clone()),
            )
            .await;

            // Spawn background task for actual doc generation work
            tokio::spawn(async move {
                // Create MCP callback sender to notify the LLM client
                let callback = mcp_callback(peer, doc_id_clone.clone());

                // Send started notification immediately
                let _ = callback
                    .send_progress(ProgressUpdate::Started {
                        operation_id: doc_id_clone.clone(),
                        command: "cargo doc".to_string(),
                        description: "Generating documentation in background".to_string(),
                    })
                    .await;

                // Do the actual doc generation work
                let started_at = Instant::now();
                let result = Self::doc_implementation(&req_clone).await;
                // Store for wait
                let _ = monitor
                    .complete_operation(&doc_id_clone, result.clone())
                    .await;

                // Send completion notification
                let duration_ms = started_at.elapsed().as_millis() as u64;
                let completion_update = match result {
                    Ok(msg) => ProgressUpdate::Completed {
                        operation_id: doc_id_clone,
                        message: msg,
                        duration_ms,
                    },
                    Err(err) => ProgressUpdate::Failed {
                        operation_id: doc_id_clone,
                        error: err,
                        duration_ms,
                    },
                };

                let _ = callback.send_progress(completion_update).await;
            });

            // Return immediate response to LLM - this is the "first stage"
            let tool_hint = self.generate_tool_hint(&doc_id, "documentation generation");
            Ok(CallToolResult::success(vec![Content::text(format!(
                "📚 Documentation generation {doc_id} started in background.{tool_hint}"
            ))]))
        } else {
            // Synchronous operation for when async notifications are disabled
            match Self::doc_implementation(&req).await {
                Ok(result_msg) => Ok(CallToolResult::success(vec![Content::text(result_msg)])),
                Err(error_msg) => Ok(CallToolResult::success(vec![Content::text(error_msg)])),
            }
        }
    }

    // (HTML content parsing removed; doc tool reports path only)

    /// Internal implementation of doc generation logic
    async fn doc_implementation(req: &DocRequest) -> Result<String, String> {
        use tokio::process::Command;

        let mut cmd = Command::new("cargo");
        cmd.arg("doc").arg("--no-deps");

        // Set working directory
        cmd.current_dir(&req.working_directory);

        let output = cmd.output().await.map_err(|e| {
            format!("Documentation generation failed: Failed to execute cargo doc: {e}")
        })?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        let working_dir_msg = format!(" in {}", &req.working_directory);

        if output.status.success() {
            // Try to determine the crate name for the documentation path
            let crate_name = {
                // If working directory is specified, try to read Cargo.toml there
                let cargo_toml_path = format!("{}/Cargo.toml", &req.working_directory);
                std::fs::read_to_string(&cargo_toml_path)
                    .ok()
                    .and_then(|content| {
                        // Simple parsing to extract package name
                        content
                            .lines()
                            .find(|line| line.trim().starts_with("name"))
                            .and_then(|line| {
                                line.split('=')
                                    .nth(1)?
                                    .trim()
                                    .trim_matches('"')
                                    .split(' ')
                                    .next()
                                    .map(|s| s.replace('-', "_"))
                            })
                    })
                    .unwrap_or_else(|| "unknown_crate".to_string())
            };

            let doc_path = format!(
                "{}/target/doc/{}/index.html",
                &req.working_directory, crate_name
            );

            Ok(format!(
                "📚 Documentation generation completed successfully{working_dir_msg}.\nDocumentation generated at: {doc_path}\nThe generated documentation provides comprehensive API information that can be used by LLMs for more accurate and up-to-date project understanding.\n💡 Tip: Use this documentation to get the latest API details, examples, and implementation notes that complement the source code.\n\nOutput: {stdout}"
            ))
        } else {
            Err(format!(
                "- Documentation generation failed{working_dir_msg}.\nErrors: {stderr}\nOutput: {stdout}"
            ))
        }
    }
    #[tool(
        description = "CARGO CLIPPY: Safer than terminal cargo. Supports --fix via args=['--fix','--allow-dirty']. Fast operation - async optional. Always use async_cargo_mcp MCP tools; do not run cargo in a terminal. For operations >1s, set enable_async_notifications=true and call mcp_async_cargo_m_wait to collect results."
    )]
    async fn clippy(
        &self,
        Parameters(req): Parameters<ClippyRequest>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, ErrorData> {
        let clippy_id = self.generate_operation_id();

        // Check if async notifications are enabled
        if req.enable_async_notifications.unwrap_or(false) {
            // TRUE 2-STAGE ASYNC PATTERN:
            // 1. Send immediate response that operation has started
            // 2. Spawn background task to do actual work and send notifications

            let peer = context.peer.clone();
            let req_clone = req.clone();
            let clippy_id_clone = clippy_id.clone();
            let monitor = self.monitor.clone();

            // Register operation before spawn
            self.register_async_operation(
                &clippy_id,
                "cargo clippy",
                "Running linter in background",
                Some(req.working_directory.clone()),
            )
            .await;

            // Spawn background task for actual clippy work
            tokio::spawn(async move {
                // Create MCP callback sender to notify the LLM client
                let callback = mcp_callback(peer, clippy_id_clone.clone());

                // Send started notification immediately
                let _ = callback
                    .send_progress(ProgressUpdate::Started {
                        operation_id: clippy_id_clone.clone(),
                        command: "cargo clippy".to_string(),
                        description: "Running linter in background".to_string(),
                    })
                    .await;

                // Do the actual clippy work
                let started_at = Instant::now();
                let result = Self::clippy_implementation(&req_clone).await;
                // Store for wait
                let _ = monitor
                    .complete_operation(&clippy_id_clone, result.clone())
                    .await;

                // Send completion notification
                let duration_ms = started_at.elapsed().as_millis() as u64;
                let completion_update = match result {
                    Ok(msg) => ProgressUpdate::Completed {
                        operation_id: clippy_id_clone,
                        message: msg,
                        duration_ms,
                    },
                    Err(err) => ProgressUpdate::Failed {
                        operation_id: clippy_id_clone,
                        error: err,
                        duration_ms,
                    },
                };

                let _ = callback.send_progress(completion_update).await;
            });

            // Return immediate response to LLM - this is the "first stage"
            let tool_hint = self.generate_tool_hint(&clippy_id, "clippy linting");
            Ok(CallToolResult::success(vec![Content::text(format!(
                "Clippy operation {clippy_id} started in background.{tool_hint}"
            ))]))
        } else {
            // Synchronous operation for when async notifications are disabled
            match Self::clippy_implementation(&req).await {
                Ok(result_msg) => Ok(CallToolResult::success(vec![Content::text(result_msg)])),
                Err(error_msg) => Ok(CallToolResult::success(vec![Content::text(error_msg)])),
            }
        }
    }

    /// Internal implementation of clippy logic
    async fn clippy_implementation(req: &ClippyRequest) -> Result<String, String> {
        use tokio::process::Command;

        let mut cmd = Command::new("cargo");
        cmd.arg("clippy");

        // Add any additional arguments passed to clippy
        if let Some(args) = &req.args {
            cmd.args(args);
        }

        cmd.current_dir(&req.working_directory);

        let output = cmd
            .output()
            .await
            .map_err(|e| format!("Clippy operation failed: Failed to execute cargo clippy: {e}"))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        let working_dir_msg = format!(" in {}", &req.working_directory);

        if output.status.success() {
            Ok(format!(
                "Clippy operation passed with no warnings{working_dir_msg}.\nOutput: {stdout}",
            ))
        } else {
            Err(format!(
                "- Clippy operation failed{working_dir_msg}.\nErrors: {stderr}\nOutput: {stdout}",
            ))
        }
    }

    #[tool(
        description = "CARGO NEXTEST: Safer than terminal cargo. Faster test runner. ALWAYS use enable_async_notifications=true for test suites to multitask. Real-time progress with isolation. Always use async_cargo_mcp MCP tools; do not run cargo in a terminal. For operations >1s, set enable_async_notifications=true and call mcp_async_cargo_m_wait to collect results."
    )]
    async fn nextest(
        &self,
        Parameters(req): Parameters<NextestRequest>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, ErrorData> {
        let nextest_id = self.generate_operation_id();

        // First check if nextest is available
        let nextest_check = tokio::process::Command::new("cargo")
            .args(["nextest", "--version"])
            .output()
            .await;

        if nextest_check.is_err() || !nextest_check.unwrap().status.success() {
            return Ok(CallToolResult::success(vec![Content::text(format!(
                "- Nextest operation #{nextest_id} failed: cargo-nextest is not installed. 
📦 Install with: cargo install cargo-nextest
🔄 Falling back to regular cargo test is recommended."
            ))]));
        }

        // Check if async notifications are enabled
        if req.enable_async_notifications.unwrap_or(false) {
            // TRUE 2-STAGE ASYNC PATTERN:
            // 1. Send immediate response that operation has started
            // 2. Spawn background task to do actual work and send notifications

            let peer = context.peer.clone();
            let req_clone = req.clone();
            let nextest_id_clone = nextest_id.clone();
            let monitor = self.monitor.clone();

            // Register operation before spawn
            self.register_async_operation(
                &nextest_id,
                "cargo nextest run",
                "Running fast test suite in background",
                Some(req.working_directory.clone()),
            )
            .await;

            // Spawn background task for actual nextest work
            tokio::spawn(async move {
                // Create MCP callback sender to notify the LLM client
                let callback = mcp_callback(peer, nextest_id_clone.clone());

                // Send started notification immediately
                let _ = callback
                    .send_progress(ProgressUpdate::Started {
                        operation_id: nextest_id_clone.clone(),
                        command: "cargo nextest run".to_string(),
                        description: "Running fast test suite in background".to_string(),
                    })
                    .await;

                // Do the actual nextest work
                let started_at = Instant::now();
                let result = Self::nextest_implementation(&req_clone).await;
                // Store for wait
                let _ = monitor
                    .complete_operation(&nextest_id_clone, result.clone())
                    .await;

                // Send completion notification
                let duration_ms = started_at.elapsed().as_millis() as u64;
                let completion_update = match result {
                    Ok(msg) => ProgressUpdate::Completed {
                        operation_id: nextest_id_clone,
                        message: msg,
                        duration_ms,
                    },
                    Err(err) => ProgressUpdate::Failed {
                        operation_id: nextest_id_clone,
                        error: err,
                        duration_ms,
                    },
                };

                let _ = callback.send_progress(completion_update).await;
            });

            // Return immediate response to LLM - this is the "first stage"
            let tool_hint = self.generate_tool_hint(&nextest_id, "nextest");
            Ok(CallToolResult::success(vec![Content::text(format!(
                "Nextest operation {nextest_id} started in background.{tool_hint}"
            ))]))
        } else {
            // Synchronous operation for when async notifications are disabled
            match Self::nextest_implementation(&req).await {
                Ok(result_msg) => Ok(CallToolResult::success(vec![Content::text(result_msg)])),
                Err(error_msg) => Ok(CallToolResult::success(vec![Content::text(error_msg)])),
            }
        }
    }

    /// Internal implementation of nextest logic
    async fn nextest_implementation(req: &NextestRequest) -> Result<String, String> {
        use tokio::process::Command;

        let nextest_id = uuid::Uuid::new_v4().to_string()[..8].to_string();

        let mut cmd = Command::new("cargo");
        cmd.args(["nextest", "run"]);

        // Add any additional arguments
        if let Some(args) = &req.args {
            cmd.args(args);
        }

        cmd.current_dir(&req.working_directory);

        let output = cmd
            .output()
            .await
            .map_err(|e| format!("Failed to execute cargo nextest: {e}"))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        let working_dir_msg = format!(" in {}", &req.working_directory);

        if output.status.success() {
            Ok(format!(
                "Nextest operation #{nextest_id} completed successfully{working_dir_msg}.\nOutput: {stdout}"
            ))
        } else {
            Err(format!(
                "- Nextest operation #{nextest_id} failed{working_dir_msg}.\nErrors: {stderr}\nOutput: {stdout}"
            ))
        }
    }

    #[tool(
        description = "CARGO CLEAN: Safer than terminal cargo. Fast operation - async not needed. Frees disk space. Always use async_cargo_mcp MCP tools; do not run cargo in a terminal. For operations >1s, set enable_async_notifications=true and call mcp_async_cargo_m_wait to collect results."
    )]
    async fn clean(
        &self,
        Parameters(req): Parameters<CleanRequest>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, ErrorData> {
        let clean_id = self.generate_operation_id();

        // Check if async notifications are enabled
        if req.enable_async_notifications.unwrap_or(false) {
            // TRUE 2-STAGE ASYNC PATTERN:
            // 1. Send immediate response that operation has started
            // 2. Spawn background task to do actual work and send notifications

            let peer = context.peer.clone();
            let req_clone = req.clone();
            let clean_id_clone = clean_id.clone();
            let monitor = self.monitor.clone();

            // Register operation before spawn
            self.register_async_operation(
                &clean_id,
                "cargo clean",
                "Cleaning build artifacts in background",
                Some(req.working_directory.clone()),
            )
            .await;

            // Spawn background task for actual clean work
            tokio::spawn(async move {
                // Create MCP callback sender to notify the LLM client
                let callback = mcp_callback(peer, clean_id_clone.clone());

                // Send started notification immediately
                let _ = callback
                    .send_progress(ProgressUpdate::Started {
                        operation_id: clean_id_clone.clone(),
                        command: "cargo clean".to_string(),
                        description: "Cleaning build artifacts in background".to_string(),
                    })
                    .await;

                // Do the actual clean work
                let started_at = Instant::now();
                let result = Self::clean_implementation(&req_clone).await;
                // Store for wait
                let _ = monitor
                    .complete_operation(&clean_id_clone, result.clone())
                    .await;

                // Send completion notification
                let duration_ms = started_at.elapsed().as_millis() as u64;
                let completion_update = match result {
                    Ok(msg) => ProgressUpdate::Completed {
                        operation_id: clean_id_clone,
                        message: msg,
                        duration_ms,
                    },
                    Err(err) => ProgressUpdate::Failed {
                        operation_id: clean_id_clone,
                        error: err,
                        duration_ms,
                    },
                };

                let _ = callback.send_progress(completion_update).await;
            });

            // Return immediate response to LLM - this is the "first stage"
            let tool_hint = self.generate_tool_hint(&clean_id, "clean");
            Ok(CallToolResult::success(vec![Content::text(format!(
                "Clean operation {clean_id} started in background.{tool_hint}"
            ))]))
        } else {
            // Synchronous operation for when async notifications are disabled
            match Self::clean_implementation(&req).await {
                Ok(result_msg) => Ok(CallToolResult::success(vec![Content::text(result_msg)])),
                Err(error_msg) => Ok(CallToolResult::success(vec![Content::text(error_msg)])),
            }
        }
    }

    /// Internal implementation of clean logic
    async fn clean_implementation(req: &CleanRequest) -> Result<String, String> {
        use tokio::process::Command;

        let mut cmd = Command::new("cargo");
        cmd.arg("clean");

        cmd.current_dir(&req.working_directory);

        let output = cmd.output().await.map_err(|e| {
            format!(
                "- Clean operation failed in {}.\nError: Failed to execute cargo clean: {}",
                &req.working_directory, e
            )
        })?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        let working_dir_msg = format!(" in {}", &req.working_directory);

        if output.status.success() {
            Ok(format!(
                "Clean operation completed successfully{working_dir_msg}.\nOutput: {stdout}"
            ))
        } else {
            Err(format!(
                "- Clean operation failed{working_dir_msg}.\nErrors: {stderr}\nOutput: {stdout}"
            ))
        }
    }

    #[tool(
        description = "CARGO FIX: Safer than terminal cargo. Automatically fix compiler warnings. Supports --allow-dirty via args. Use async for large codebases. Always use async_cargo_mcp MCP tools; do not run cargo in a terminal. For operations >1s, set enable_async_notifications=true and call mcp_async_cargo_m_wait to collect results."
    )]
    async fn fix(
        &self,
        Parameters(req): Parameters<FixRequest>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, ErrorData> {
        let fix_id = self.generate_operation_id();

        // Check if async notifications are enabled
        if req.enable_async_notifications.unwrap_or(false) {
            // TRUE 2-STAGE ASYNC PATTERN:
            // 1. Send immediate response that operation has started
            // 2. Spawn background task to do actual work and send notifications

            let peer = context.peer.clone();
            let req_clone = req.clone();
            let fix_id_clone = fix_id.clone();
            let monitor = self.monitor.clone();

            // Register operation before spawn
            self.register_async_operation(
                &fix_id,
                "cargo fix",
                "Fixing compiler warnings in background",
                Some(req.working_directory.clone()),
            )
            .await;

            // Spawn background task for actual fix work
            tokio::spawn(async move {
                // Create MCP callback sender to notify the LLM client
                let callback = mcp_callback(peer, fix_id_clone.clone());

                // Send started notification immediately
                let _ = callback
                    .send_progress(ProgressUpdate::Started {
                        operation_id: fix_id_clone.clone(),
                        command: "cargo fix".to_string(),
                        description: "Fixing compiler warnings in background".to_string(),
                    })
                    .await;

                // Do the actual fix work
                let started_at = Instant::now();
                let result = Self::fix_implementation(&req_clone).await;
                // Store for wait
                let _ = monitor
                    .complete_operation(&fix_id_clone, result.clone())
                    .await;

                // Send completion notification
                let duration_ms = started_at.elapsed().as_millis() as u64;
                let completion_update = match result {
                    Ok(msg) => ProgressUpdate::Completed {
                        operation_id: fix_id_clone,
                        message: msg,
                        duration_ms,
                    },
                    Err(err) => ProgressUpdate::Failed {
                        operation_id: fix_id_clone,
                        error: err,
                        duration_ms,
                    },
                };

                let _ = callback.send_progress(completion_update).await;
            });

            // Return immediate response to LLM - this is the "first stage"
            let tool_hint = self.generate_tool_hint(&fix_id, "fix");
            Ok(CallToolResult::success(vec![Content::text(format!(
                "Fix operation {fix_id} started in background.{tool_hint}"
            ))]))
        } else {
            // Synchronous operation for when async notifications are disabled
            match Self::fix_implementation(&req).await {
                Ok(result_msg) => Ok(CallToolResult::success(vec![Content::text(result_msg)])),
                Err(error_msg) => Ok(CallToolResult::success(vec![Content::text(error_msg)])),
            }
        }
    }

    /// Internal implementation of fix logic
    async fn fix_implementation(req: &FixRequest) -> Result<String, String> {
        use tokio::process::Command;

        let mut cmd = Command::new("cargo");
        cmd.arg("fix");

        // Add any additional arguments
        if let Some(args) = &req.args {
            cmd.args(args);
        } else {
            // Default to --allow-dirty to avoid issues with uncommitted changes
            cmd.arg("--allow-dirty");
        }

        cmd.current_dir(&req.working_directory);

        let output = cmd.output().await.map_err(|e| {
            format!(
                "- Fix operation failed in {}.\nError: Failed to execute cargo fix: {}",
                &req.working_directory, e
            )
        })?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        let working_dir_msg = format!(" in {}", &req.working_directory);

        if output.status.success() {
            Ok(format!(
                "Fix operation completed successfully{working_dir_msg}.\nOutput: {stdout}"
            ))
        } else {
            Err(format!(
                "- Fix operation failed{working_dir_msg}.\nErrors: {stderr}\nOutput: {stdout}"
            ))
        }
    }

    #[tool(
        description = "CARGO SEARCH: Safer than terminal cargo. Search for crates on crates.io. Fast operation - async not needed unless searching many terms. Always use async_cargo_mcp MCP tools; do not run cargo in a terminal. For operations >1s, set enable_async_notifications=true and call mcp_async_cargo_m_wait to collect results."
    )]
    async fn search(
        &self,
        Parameters(req): Parameters<SearchRequest>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, ErrorData> {
        let search_id = self.generate_operation_id();

        // Check if async notifications are enabled
        if req.enable_async_notifications.unwrap_or(false) {
            // TRUE 2-STAGE ASYNC PATTERN:
            // 1. Send immediate response that operation has started
            // 2. Spawn background task to do actual work and send notifications

            let peer = context.peer.clone();
            let req_clone = req.clone();
            let search_id_clone = search_id.clone();
            let monitor = self.monitor.clone();

            // Register operation before spawn
            self.register_async_operation(
                &search_id,
                "cargo search",
                &format!("Searching crates.io for '{}' in background", req.query),
                None,
            )
            .await;

            // Spawn background task for actual search work
            tokio::spawn(async move {
                // Create MCP callback sender to notify the LLM client
                let callback = mcp_callback(peer, search_id_clone.clone());

                // Send started notification immediately
                let _ = callback
                    .send_progress(ProgressUpdate::Started {
                        operation_id: search_id_clone.clone(),
                        command: "cargo search".to_string(),
                        description: format!(
                            "Searching crates.io for '{}' in background",
                            req_clone.query
                        ),
                    })
                    .await;

                // Do the actual search work
                let started_at = Instant::now();
                let result = Self::search_implementation(&req_clone).await;
                // Store for wait
                let _ = monitor
                    .complete_operation(&search_id_clone, result.clone())
                    .await;

                // Send completion notification
                let duration_ms = started_at.elapsed().as_millis() as u64;
                let completion_update = match result {
                    Ok(msg) => ProgressUpdate::Completed {
                        operation_id: search_id_clone,
                        message: msg,
                        duration_ms,
                    },
                    Err(err) => ProgressUpdate::Failed {
                        operation_id: search_id_clone,
                        error: err,
                        duration_ms,
                    },
                };

                let _ = callback.send_progress(completion_update).await;
            });

            // Return immediate response to LLM - this is the "first stage"
            let tool_hint = self.generate_tool_hint(&search_id, "search");
            Ok(CallToolResult::success(vec![Content::text(format!(
                "Search operation {} started in background. Searching crates.io for '{}'.{}",
                search_id, req.query, tool_hint
            ))]))
        } else {
            // Synchronous operation for when async notifications are disabled
            match Self::search_implementation(&req).await {
                Ok(result_msg) => Ok(CallToolResult::success(vec![Content::text(result_msg)])),
                Err(error_msg) => Ok(CallToolResult::success(vec![Content::text(error_msg)])),
            }
        }
    }

    /// Internal implementation of search logic
    async fn search_implementation(req: &SearchRequest) -> Result<String, String> {
        use tokio::process::Command;

        let mut cmd = Command::new("cargo");
        cmd.arg("search").arg(&req.query);

        if let Some(limit) = req.limit {
            cmd.args(["--limit", &limit.to_string()]);
        }

        let output = cmd.output().await.map_err(|e| {
            format!(
                "- Search operation failed for query '{}'.\nError: Failed to execute cargo search: {}",
                req.query, e
            )
        })?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        if output.status.success() {
            Ok(format!(
                "Search operation completed successfully.\nQuery: {}\nResults:\n{stdout}",
                req.query
            ))
        } else {
            Err(format!(
                "- Search operation failed.\nQuery: {}\nErrors: {stderr}\nOutput: {stdout}",
                req.query
            ))
        }
    }

    #[tool(
        description = "CARGO BENCH: Safer than terminal cargo. ALWAYS use enable_async_notifications=true for benchmark suites to multitask. Performance testing. Always use async_cargo_mcp MCP tools; do not run cargo in a terminal. For operations >1s, set enable_async_notifications=true and call mcp_async_cargo_m_wait to collect results."
    )]
    async fn bench(
        &self,
        Parameters(req): Parameters<BenchRequest>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, ErrorData> {
        let bench_id = self.generate_operation_id();

        // Check if async notifications are enabled
        if req.enable_async_notifications.unwrap_or(false) {
            // TRUE 2-STAGE ASYNC PATTERN:
            // 1. Send immediate response that operation has started
            // 2. Spawn background task to do actual work and send notifications

            let peer = context.peer.clone();
            let req_clone = req.clone();
            let bench_id_clone = bench_id.clone();
            let monitor = self.monitor.clone();

            // Register operation before spawn
            self.register_async_operation(
                &bench_id,
                "cargo bench",
                "Running benchmarks in background",
                Some(req.working_directory.clone()),
            )
            .await;

            // Spawn background task for actual bench work
            tokio::spawn(async move {
                // Create MCP callback sender to notify the LLM client
                let callback = mcp_callback(peer, bench_id_clone.clone());

                // Send started notification immediately
                let _ = callback
                    .send_progress(ProgressUpdate::Started {
                        operation_id: bench_id_clone.clone(),
                        command: "cargo bench".to_string(),
                        description: "Running benchmarks in background".to_string(),
                    })
                    .await;

                // Do the actual bench work
                let started_at = Instant::now();
                let result = Self::bench_implementation(&req_clone).await;
                // Store for wait
                let _ = monitor
                    .complete_operation(&bench_id_clone, result.clone())
                    .await;

                // Send completion notification
                let duration_ms = started_at.elapsed().as_millis() as u64;
                let completion_update = match result {
                    Ok(msg) => ProgressUpdate::Completed {
                        operation_id: bench_id_clone,
                        message: msg,
                        duration_ms,
                    },
                    Err(err) => ProgressUpdate::Failed {
                        operation_id: bench_id_clone,
                        error: err,
                        duration_ms,
                    },
                };

                let _ = callback.send_progress(completion_update).await;
            });

            // Return immediate response to LLM - this is the "first stage"
            let tool_hint = self.generate_tool_hint(&bench_id, "benchmark");
            Ok(CallToolResult::success(vec![Content::text(format!(
                "Benchmark operation {bench_id} started in background.{tool_hint}"
            ))]))
        } else {
            // Synchronous operation for when async notifications are disabled
            match Self::bench_implementation(&req).await {
                Ok(result_msg) => Ok(CallToolResult::success(vec![Content::text(result_msg)])),
                Err(error_msg) => Ok(CallToolResult::success(vec![Content::text(error_msg)])),
            }
        }
    }

    /// Internal implementation of bench logic
    async fn bench_implementation(req: &BenchRequest) -> Result<String, String> {
        use tokio::process::Command;

        let mut cmd = Command::new("cargo");
        cmd.arg("bench");

        // Add any additional arguments
        if let Some(args) = &req.args {
            cmd.args(args);
        }

        cmd.current_dir(&req.working_directory);

        let output = cmd.output().await.map_err(|e| {
            format!(
                "- Benchmark operation failed in {}.\nError: Failed to execute cargo bench: {}",
                &req.working_directory, e
            )
        })?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        let working_dir_msg = format!(" in {}", &req.working_directory);

        if output.status.success() {
            Ok(format!(
                "🏃‍♂️ Benchmark operation completed successfully{working_dir_msg}.\nOutput: {stdout}"
            ))
        } else {
            Err(format!(
                "- Benchmark operation failed{working_dir_msg}.\nErrors: {stderr}\nOutput: {stdout}"
            ))
        }
    }

    #[tool(
        description = "CARGO INSTALL: Safer than terminal cargo. Use enable_async_notifications=true for large packages to multitask. Global tool installation. Always use async_cargo_mcp MCP tools; do not run cargo in a terminal. For operations >1s, set enable_async_notifications=true and call mcp_async_cargo_m_wait to collect results."
    )]
    async fn install(
        &self,
        Parameters(req): Parameters<InstallRequest>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, ErrorData> {
        let install_id = self.generate_operation_id();

        // Check if async notifications are enabled
        if req.enable_async_notifications.unwrap_or(false) {
            let peer = context.peer.clone();
            let req_clone = req.clone();
            let install_id_clone = install_id.clone();
            let monitor = self.monitor.clone();

            // Register operation before spawn
            self.register_async_operation(
                &install_id,
                "cargo install",
                &format!("Installing package '{}' in background", req.package),
                Some(req.working_directory.clone()),
            )
            .await;

            // Spawn background task for actual install work
            tokio::spawn(async move {
                let callback = mcp_callback(peer, install_id_clone.clone());

                let _ = callback
                    .send_progress(ProgressUpdate::Started {
                        operation_id: install_id_clone.clone(),
                        command: "cargo install".to_string(),
                        description: format!(
                            "Installing package '{}' in background",
                            req_clone.package
                        ),
                    })
                    .await;

                let started_at = Instant::now();
                let result = Self::install_implementation(&req_clone).await;

                let _ = monitor
                    .complete_operation(&install_id_clone, result.clone())
                    .await;

                let duration_ms = started_at.elapsed().as_millis() as u64;
                let completion_update = match result {
                    Ok(msg) => ProgressUpdate::Completed {
                        operation_id: install_id_clone,
                        message: msg,
                        duration_ms,
                    },
                    Err(err) => ProgressUpdate::Failed {
                        operation_id: install_id_clone,
                        error: err,
                        duration_ms,
                    },
                };

                let _ = callback.send_progress(completion_update).await;
            });

            let tool_hint = self.generate_tool_hint(&install_id, "install");
            Ok(CallToolResult::success(vec![Content::text(format!(
                "Install operation {} started in background. Installing package '{}'.{}",
                install_id, req.package, tool_hint
            ))]))
        } else {
            match Self::install_implementation(&req).await {
                Ok(result_msg) => Ok(CallToolResult::success(vec![Content::text(result_msg)])),
                Err(error_msg) => Ok(CallToolResult::success(vec![Content::text(error_msg)])),
            }
        }
    }

    /// Internal implementation of install logic
    async fn install_implementation(req: &InstallRequest) -> Result<String, String> {
        use tokio::process::Command;

        let mut cmd = Command::new("cargo");
        cmd.arg("install");

        let package_spec = if let Some(version) = &req.version {
            format!("{}@{}", req.package, version)
        } else {
            req.package.clone()
        };

        cmd.arg(&package_spec);
        cmd.current_dir(&req.working_directory);

        let output = cmd.output().await.map_err(|e| {
            format!(
                "- Install operation failed in {}.\nError: Failed to execute cargo install: {}",
                &req.working_directory, e
            )
        })?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let working_dir_msg = format!(" in {}", &req.working_directory);

        if output.status.success() {
            Ok(format!(
                "Install operation completed successfully{working_dir_msg}.\nInstalled package: {}\nOutput: {stdout}",
                req.package
            ))
        } else {
            Err(format!(
                "- Install operation failed{working_dir_msg}.\nPackage: {}\nErrors: {stderr}\nOutput: {stdout}",
                req.package
            ))
        }
    }

    #[tool(
        description = "CARGO UPGRADE: Safer than terminal cargo. Synchronous operation for Cargo.toml modifications. Updates dependencies to latest versions using cargo-edit. Always use async_cargo_mcp MCP tools; do not run cargo in a terminal. For operations >1s, set enable_async_notifications=true and call mcp_async_cargo_m_wait to collect results."
    )]
    async fn upgrade(
        &self,
        Parameters(req): Parameters<UpgradeRequest>,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, ErrorData> {
        let upgrade_id = self.generate_operation_id();

        // First check if cargo-edit (upgrade command) is available
        let upgrade_check = tokio::process::Command::new("cargo")
            .args(["upgrade", "--version"])
            .output()
            .await;

        if upgrade_check.is_err() || !upgrade_check.unwrap().status.success() {
            return Ok(CallToolResult::success(vec![Content::text(format!(
                "- Upgrade operation #{upgrade_id} failed: cargo-edit with upgrade command is not installed. 
📦 Install with: cargo install cargo-edit
🔄 Falling back to regular cargo update is recommended."
            ))]));
        }

        // Always use synchronous execution for Cargo.toml modifications
        match Self::upgrade_implementation(&req).await {
            Ok(result_msg) => Ok(CallToolResult::success(vec![Content::text(result_msg)])),
            Err(error_msg) => Ok(CallToolResult::success(vec![Content::text(error_msg)])),
        }
    }

    /// Internal implementation of upgrade logic
    async fn upgrade_implementation(req: &UpgradeRequest) -> Result<String, String> {
        use tokio::process::Command;

        let upgrade_id = uuid::Uuid::new_v4().to_string()[..8].to_string();

        let mut cmd = Command::new("cargo");
        cmd.arg("upgrade");

        // Add incompatible flag if requested
        if req.incompatible.unwrap_or(false) {
            cmd.arg("--incompatible");
        }

        // Add pinned flag if requested
        if req.pinned.unwrap_or(false) {
            cmd.arg("--pinned");
        }

        // Add dry run flag if requested
        if req.dry_run.unwrap_or(false) {
            cmd.arg("--dry-run");
        }

        // Add specific packages to upgrade
        if let Some(packages) = &req.packages {
            for package in packages {
                cmd.args(["--package", package]);
            }
        }

        // Add packages to exclude
        if let Some(exclude) = &req.exclude {
            for package in exclude {
                cmd.args(["--exclude", package]);
            }
        }

        // Add any additional arguments
        if let Some(args) = &req.args {
            cmd.args(args);
        }

        cmd.current_dir(&req.working_directory);

        let output = cmd
            .output()
            .await
            .map_err(|e| format!("Failed to execute cargo upgrade: {e}"))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        let working_dir_msg = format!(" in {}", &req.working_directory);

        if output.status.success() {
            let dry_run_msg = if req.dry_run.unwrap_or(false) {
                " (dry run - no changes made)"
            } else {
                ""
            };
            Ok(format!(
                "⬆️ Upgrade operation #{upgrade_id} completed successfully{working_dir_msg}{dry_run_msg}.\nOutput: {stdout}"
            ))
        } else {
            Err(format!(
                "- Upgrade operation #{upgrade_id} failed{working_dir_msg}.\nErrors: {stderr}\nOutput: {stdout}"
            ))
        }
    }

    #[tool(
        description = "CARGO AUDIT: Safer than terminal cargo. Security vulnerability scanning. Use enable_async_notifications=true for large projects to multitask. Identifies known security vulnerabilities. Always use async_cargo_mcp MCP tools; do not run cargo in a terminal. For operations >1s, set enable_async_notifications=true and call mcp_async_cargo_m_wait to collect results."
    )]
    async fn audit(
        &self,
        Parameters(req): Parameters<AuditRequest>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, ErrorData> {
        let audit_id = self.generate_operation_id();

        // First check if cargo-audit is available
        let audit_check = tokio::process::Command::new("cargo")
            .args(["audit", "--version"])
            .output()
            .await;

        if audit_check.is_err() || !audit_check.unwrap().status.success() {
            return Ok(CallToolResult::success(vec![Content::text(format!(
                "- Audit operation #{audit_id} failed: cargo-audit is not installed. 
📦 Install with: cargo install cargo-audit
🔒 This tool scans for known security vulnerabilities in dependencies."
            ))]));
        }

        // Check if async notifications are enabled
        if req.enable_async_notifications.unwrap_or(false) {
            // TRUE 2-STAGE ASYNC PATTERN:
            // 1. Send immediate response that operation has started
            // 2. Spawn background task to do actual work and send notifications

            let peer = context.peer.clone();
            let req_clone = req.clone();
            let audit_id_clone = audit_id.clone();
            let monitor = self.monitor.clone();

            // Register operation before spawn
            self.register_async_operation(
                &audit_id,
                "cargo audit",
                "Scanning for security vulnerabilities in background",
                Some(req.working_directory.clone()),
            )
            .await;

            // Spawn background task for actual audit work
            tokio::spawn(async move {
                // Create MCP callback sender to notify the LLM client
                let callback = mcp_callback(peer, audit_id_clone.clone());

                // Send started notification immediately
                let _ = callback
                    .send_progress(ProgressUpdate::Started {
                        operation_id: audit_id_clone.clone(),
                        command: "cargo audit".to_string(),
                        description: "Scanning for security vulnerabilities in background"
                            .to_string(),
                    })
                    .await;

                // Do the actual audit work
                let started_at = Instant::now();
                let result = Self::audit_implementation(&req_clone).await;
                // Store for wait
                let _ = monitor
                    .complete_operation(&audit_id_clone, result.clone())
                    .await;

                // Send completion notification
                let duration_ms = started_at.elapsed().as_millis() as u64;
                let completion_update = match result {
                    Ok(msg) => ProgressUpdate::Completed {
                        operation_id: audit_id_clone,
                        message: msg,
                        duration_ms,
                    },
                    Err(err) => ProgressUpdate::Failed {
                        operation_id: audit_id_clone,
                        error: err,
                        duration_ms,
                    },
                };

                let _ = callback.send_progress(completion_update).await;
            });

            // Return immediate response to LLM - this is the "first stage"
            let tool_hint = self.generate_tool_hint(&audit_id, "audit");
            Ok(CallToolResult::success(vec![Content::text(format!(
                "Audit operation {audit_id} started in background.{tool_hint}"
            ))]))
        } else {
            // Synchronous operation for when async notifications are disabled
            match Self::audit_implementation(&req).await {
                Ok(result_msg) => Ok(CallToolResult::success(vec![Content::text(result_msg)])),
                Err(error_msg) => Ok(CallToolResult::success(vec![Content::text(error_msg)])),
            }
        }
    }

    /// Internal implementation of audit logic
    async fn audit_implementation(req: &AuditRequest) -> Result<String, String> {
        use tokio::process::Command;

        let audit_id = uuid::Uuid::new_v4().to_string()[..8].to_string();

        let mut cmd = Command::new("cargo");
        cmd.arg("audit");

        // Add format flag if specified
        if let Some(format) = &req.format {
            cmd.args(["--format", format]);
        }

        // Add vulnerabilities-only flag if requested
        if req.vulnerabilities_only.unwrap_or(false) {
            cmd.arg("--vulnerabilities");
        }

        // Add deny warnings flag if requested
        if req.deny_warnings.unwrap_or(false) {
            cmd.arg("--deny-warnings");
        }

        // Add any additional arguments
        if let Some(args) = &req.args {
            cmd.args(args);
        }

        cmd.current_dir(&req.working_directory);

        let output = cmd
            .output()
            .await
            .map_err(|e| format!("Failed to execute cargo audit: {e}"))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        let working_dir_msg = format!(" in {}", &req.working_directory);

        if output.status.success() {
            Ok(format!(
                "Audit operation #{audit_id} completed successfully{working_dir_msg}.\nNo known vulnerabilities found.\nOutput: {stdout}"
            ))
        } else {
            // Check if it's a vulnerability warning (exit code 1) vs actual error
            let vulnerability_detected = output.status.code() == Some(1) && !stdout.is_empty();

            if vulnerability_detected {
                Err(format!(
                    "Audit operation #{audit_id} found security vulnerabilities{working_dir_msg}.\nVulnerabilities detected:\n{stdout}\nErrors: {stderr}"
                ))
            } else {
                Err(format!(
                    "- Audit operation #{audit_id} failed{working_dir_msg}.\nErrors: {stderr}\nOutput: {stdout}"
                ))
            }
        }
    }

    #[tool(
        description = "CARGO FMT: Safer than terminal cargo. Format Rust code using rustfmt. Use enable_async_notifications=true for large projects to multitask while code is being formatted. Always use async_cargo_mcp MCP tools; do not run cargo in a terminal. For operations >1s, set enable_async_notifications=true and call mcp_async_cargo_m_wait to collect results."
    )]
    async fn fmt(
        &self,
        Parameters(req): Parameters<FmtRequest>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, ErrorData> {
        let fmt_id = self.generate_operation_id();

        // Check if async notifications are enabled
        if req.enable_async_notifications.unwrap_or(false) {
            // TRUE 2-STAGE ASYNC PATTERN:
            // 1. Send immediate response that operation has started
            // 2. Spawn background task to do actual work and send notifications

            let peer = context.peer.clone();
            let req_clone = req.clone();
            let fmt_id_clone = fmt_id.clone();
            let monitor = self.monitor.clone();

            // Register operation before spawn
            self.register_async_operation(
                &fmt_id,
                "cargo fmt",
                "Formatting code in background",
                Some(req.working_directory.clone()),
            )
            .await;

            // Spawn background task for actual format work
            tokio::spawn(async move {
                // Create MCP callback sender to notify the LLM client
                let callback = mcp_callback(peer, fmt_id_clone.clone());

                // Send started notification immediately
                let _ = callback
                    .send_progress(ProgressUpdate::Started {
                        operation_id: fmt_id_clone.clone(),
                        command: "cargo fmt".to_string(),
                        description: "Formatting code in background".to_string(),
                    })
                    .await;

                // Do the actual format work
                let started_at = Instant::now();
                let result = Self::fmt_implementation(&req_clone).await;
                // Store for wait
                let _ = monitor
                    .complete_operation(&fmt_id_clone, result.clone())
                    .await;

                // Send completion notification
                let duration_ms = started_at.elapsed().as_millis() as u64;
                let completion_update = match result {
                    Ok(msg) => ProgressUpdate::Completed {
                        operation_id: fmt_id_clone,
                        message: msg,
                        duration_ms,
                    },
                    Err(err) => ProgressUpdate::Failed {
                        operation_id: fmt_id_clone,
                        error: err,
                        duration_ms,
                    },
                };

                let _ = callback.send_progress(completion_update).await;
            });

            // Return immediate response to LLM - this is the "first stage"
            let tool_hint = self.generate_tool_hint(&fmt_id, "format");
            Ok(CallToolResult::success(vec![Content::text(format!(
                "Format operation {fmt_id} started in background.{tool_hint}"
            ))]))
        } else {
            // Synchronous operation for when async notifications are disabled
            match Self::fmt_implementation(&req).await {
                Ok(result_msg) => Ok(CallToolResult::success(vec![Content::text(result_msg)])),
                Err(error_msg) => Ok(CallToolResult::success(vec![Content::text(error_msg)])),
            }
        }
    }

    /// Internal implementation of fmt logic
    async fn fmt_implementation(req: &FmtRequest) -> Result<String, String> {
        use tokio::process::Command;

        // First check if rustfmt is available
        let fmt_check = Command::new("rustfmt").arg("--version").output().await;

        if fmt_check.is_err() || !fmt_check.unwrap().status.success() {
            return Err(format!(
                "- Format operation failed: rustfmt is not installed in {}. 
📦 Install with: rustup component add rustfmt
✨ This tool formats Rust code according to style guidelines.",
                &req.working_directory
            ));
        }

        let mut cmd = Command::new("cargo");
        cmd.arg("fmt");

        // Add check flag if requested (don't make changes, just check)
        if req.check.unwrap_or(false) {
            cmd.arg("--check");
        }

        // Add all flag if requested (format all packages in workspace)
        if req.all.unwrap_or(false) {
            cmd.arg("--all");
        }

        // Add any additional arguments
        if let Some(args) = &req.args {
            cmd.args(args);
        }

        cmd.current_dir(&req.working_directory);

        let output = cmd.output().await.map_err(|e| {
            format!(
                "- Format operation failed in {}.\nError: Failed to execute cargo fmt: {}",
                &req.working_directory, e
            )
        })?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        let working_dir_msg = format!(" in {}", &req.working_directory);

        if output.status.success() {
            let check_msg = if req.check.unwrap_or(false) {
                " (check mode - no changes made)"
            } else {
                ""
            };
            Ok(format!(
                "Format operation completed successfully{working_dir_msg}{check_msg}.\nOutput: {stdout}"
            ))
        } else {
            // Check if it's a formatting issue (exit code 1) vs actual error
            let formatting_issues = output.status.code() == Some(1) && req.check.unwrap_or(false);

            if formatting_issues {
                Ok(format!(
                    "Format operation found formatting issues{working_dir_msg}.\nFiles need formatting:\n{stdout}\nErrors: {stderr}"
                ))
            } else {
                Err(format!(
                    "- Format operation failed{working_dir_msg}.\nErrors: {stderr}\nOutput: {stdout}"
                ))
            }
        }
    }

    #[tool(
        description = "CARGO TREE: Safer than terminal cargo. Display dependency tree. Use enable_async_notifications=true for large projects to multitask while dependency tree is being generated. Always use async_cargo_mcp MCP tools; do not run cargo in a terminal. For operations >1s, set enable_async_notifications=true and call mcp_async_cargo_m_wait to collect results."
    )]
    async fn tree(
        &self,
        Parameters(req): Parameters<TreeRequest>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, ErrorData> {
        let tree_id = self.generate_operation_id();

        // Check if async notifications are enabled
        if req.enable_async_notifications.unwrap_or(false) {
            // TRUE 2-STAGE ASYNC PATTERN:
            // 1. Send immediate response that operation has started
            // 2. Spawn background task to do actual work and send notifications

            let peer = context.peer.clone();
            let req_clone = req.clone();
            let tree_id_clone = tree_id.clone();
            let monitor = self.monitor.clone();

            // Register operation before spawn
            self.register_async_operation(
                &tree_id,
                "cargo tree",
                "Generating dependency tree in background",
                Some(req.working_directory.clone()),
            )
            .await;

            // Spawn background task for actual tree work
            tokio::spawn(async move {
                // Create MCP callback sender to notify the LLM client
                let callback = mcp_callback(peer, tree_id_clone.clone());

                // Send started notification immediately
                let _ = callback
                    .send_progress(ProgressUpdate::Started {
                        operation_id: tree_id_clone.clone(),
                        command: "cargo tree".to_string(),
                        description: "Generating dependency tree in background".to_string(),
                    })
                    .await;

                // Do the actual tree work
                let started_at = Instant::now();
                let result = Self::tree_implementation(&req_clone).await;
                // Store for wait
                let _ = monitor
                    .complete_operation(&tree_id_clone, result.clone())
                    .await;

                // Send completion notification
                let duration_ms = started_at.elapsed().as_millis() as u64;
                let completion_update = match result {
                    Ok(msg) => ProgressUpdate::Completed {
                        operation_id: tree_id_clone,
                        message: msg,
                        duration_ms,
                    },
                    Err(err) => ProgressUpdate::Failed {
                        operation_id: tree_id_clone,
                        error: err,
                        duration_ms,
                    },
                };

                let _ = callback.send_progress(completion_update).await;
            });

            // Return immediate response to LLM - this is the "first stage"
            let tool_hint = self.generate_tool_hint(&tree_id, "tree");
            Ok(CallToolResult::success(vec![Content::text(format!(
                "Tree operation {tree_id} started in background.{tool_hint}"
            ))]))
        } else {
            // Synchronous operation for when async notifications are disabled
            match Self::tree_implementation(&req).await {
                Ok(result_msg) => Ok(CallToolResult::success(vec![Content::text(result_msg)])),
                Err(error_msg) => Ok(CallToolResult::success(vec![Content::text(error_msg)])),
            }
        }
    }

    /// Internal implementation of tree logic
    async fn tree_implementation(req: &TreeRequest) -> Result<String, String> {
        use tokio::process::Command;

        let mut cmd = Command::new("cargo");
        cmd.arg("tree");

        // Add depth limit if specified
        if let Some(depth) = req.depth {
            cmd.args(["--depth", &depth.to_string()]);
        }

        // Add features if specified
        if let Some(features) = &req.features
            && !features.is_empty()
        {
            cmd.args(["--features", &features.join(",")]);
        }

        // Add all-features flag if requested
        if req.all_features.unwrap_or(false) {
            cmd.arg("--all-features");
        }

        // Add no-default-features flag if requested
        if req.no_default_features.unwrap_or(false) {
            cmd.arg("--no-default-features");
        }

        // Add format flag if specified
        if let Some(format) = &req.format {
            cmd.args(["--format", format]);
        }

        // Add any additional arguments
        if let Some(args) = &req.args {
            cmd.args(args);
        }

        cmd.current_dir(&req.working_directory);

        let output = cmd.output().await.map_err(|e| {
            format!(
                "- Tree operation failed in {}.\nError: Failed to execute cargo tree: {}",
                &req.working_directory, e
            )
        })?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        let working_dir_msg = format!(" in {}", &req.working_directory);

        if output.status.success() {
            Ok(format!(
                "Tree operation completed successfully{working_dir_msg}.\nDependency tree:\n{stdout}"
            ))
        } else {
            Err(format!(
                "- Tree operation failed{working_dir_msg}.\nErrors: {stderr}\nOutput: {stdout}"
            ))
        }
    }

    #[tool(
        description = "CARGO VERSION: Safer than terminal cargo. Show cargo version information. Fast operation that helps LLMs understand the available cargo capabilities. Always use async_cargo_mcp MCP tools; do not run cargo in a terminal. For operations >1s, set enable_async_notifications=true and call mcp_async_cargo_m_wait to collect results."
    )]
    async fn version(
        &self,
        Parameters(req): Parameters<VersionRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        use tokio::process::Command;

        let version_id = self.generate_operation_id();

        let mut cmd = Command::new("cargo");
        cmd.arg("version");

        // Add verbose flag if requested
        if req.verbose.unwrap_or(false) {
            cmd.arg("--verbose");
        }

        let output = cmd.output().await.map_err(|e| {
            ErrorData::internal_error(format!("Failed to execute cargo version: {e}"), None)
        })?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        let result_msg = if output.status.success() {
            format!(
                "📋 Version operation #{version_id} completed successfully.\nCargo version information:\n{stdout}"
            )
        } else {
            format!("- Version operation #{version_id} failed.\nErrors: {stderr}\nOutput: {stdout}")
        };

        Ok(CallToolResult::success(vec![Content::text(result_msg)]))
    }

    #[tool(
        description = "CARGO FETCH: Safer than terminal cargo. Fetch dependencies without building. Use enable_async_notifications=true for large dependency sets to multitask while downloading. Always use async_cargo_mcp MCP tools; do not run cargo in a terminal. For operations >1s, set enable_async_notifications=true and call mcp_async_cargo_m_wait to collect results."
    )]
    async fn fetch(
        &self,
        Parameters(req): Parameters<FetchRequest>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, ErrorData> {
        let fetch_id = self.generate_operation_id();

        // Check if async notifications are enabled
        if req.enable_async_notifications.unwrap_or(false) {
            // TRUE 2-STAGE ASYNC PATTERN:
            // 1. Send immediate response that operation has started
            // 2. Spawn background task to do actual work and send notifications

            let peer = context.peer.clone();
            let req_clone = req.clone();
            let fetch_id_clone = fetch_id.clone();
            let monitor = self.monitor.clone();

            // Register operation before spawn
            self.register_async_operation(
                &fetch_id,
                "cargo fetch",
                "Fetching dependencies in background",
                Some(req.working_directory.clone()),
            )
            .await;

            // Spawn background task for actual fetch work
            tokio::spawn(async move {
                // Create MCP callback sender to notify the LLM client
                let callback = mcp_callback(peer, fetch_id_clone.clone());

                // Send started notification immediately
                let _ = callback
                    .send_progress(ProgressUpdate::Started {
                        operation_id: fetch_id_clone.clone(),
                        command: "cargo fetch".to_string(),
                        description: "Fetching dependencies in background".to_string(),
                    })
                    .await;

                // Do the actual fetch work
                let started_at = Instant::now();
                let result = Self::fetch_implementation(&req_clone).await;
                // Store for wait
                let _ = monitor
                    .complete_operation(&fetch_id_clone, result.clone())
                    .await;

                // Send completion notification
                let duration_ms = started_at.elapsed().as_millis() as u64;
                let completion_update = match result {
                    Ok(msg) => ProgressUpdate::Completed {
                        operation_id: fetch_id_clone,
                        message: msg,
                        duration_ms,
                    },
                    Err(err) => ProgressUpdate::Failed {
                        operation_id: fetch_id_clone,
                        error: err,
                        duration_ms,
                    },
                };

                let _ = callback.send_progress(completion_update).await;
            });

            // Return immediate response to LLM - this is the "first stage"
            let tool_hint = self.generate_tool_hint(&fetch_id, "fetch");
            Ok(CallToolResult::success(vec![Content::text(format!(
                "Fetch operation {fetch_id} started in background.{tool_hint}"
            ))]))
        } else {
            // Synchronous operation for when async notifications are disabled
            match Self::fetch_implementation(&req).await {
                Ok(result_msg) => Ok(CallToolResult::success(vec![Content::text(result_msg)])),
                Err(error_msg) => Ok(CallToolResult::success(vec![Content::text(error_msg)])),
            }
        }
    }

    /// Internal implementation of fetch logic
    async fn fetch_implementation(req: &FetchRequest) -> Result<String, String> {
        use tokio::process::Command;

        let mut cmd = Command::new("cargo");
        cmd.arg("fetch");

        // Add target if specified
        if let Some(target) = &req.target {
            cmd.args(["--target", target]);
        }

        // Add features if specified
        if let Some(features) = &req.features
            && !features.is_empty()
        {
            cmd.args(["--features", &features.join(",")]);
        }

        // Add all-features flag if requested
        if req.all_features.unwrap_or(false) {
            cmd.arg("--all-features");
        }

        // Add no-default-features flag if requested
        if req.no_default_features.unwrap_or(false) {
            cmd.arg("--no-default-features");
        }

        // Add any additional arguments
        if let Some(args) = &req.args {
            cmd.args(args);
        }

        cmd.current_dir(&req.working_directory);

        let output = cmd.output().await.map_err(|e| {
            format!(
                "- Fetch operation failed in {}.\nError: Failed to execute cargo fetch: {}",
                &req.working_directory, e
            )
        })?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        let working_dir_msg = format!(" in {}", &req.working_directory);

        if output.status.success() {
            Ok(format!(
                "📦 Fetch operation completed successfully{working_dir_msg}.\nDependencies fetched:\n{stdout}"
            ))
        } else {
            Err(format!(
                "- Fetch operation failed{working_dir_msg}.\nErrors: {stderr}\nOutput: {stdout}"
            ))
        }
    }

    #[tool(
        description = "CARGO RUSTC: Safer than terminal cargo. Compile with custom rustc options. Use enable_async_notifications=true for complex builds to multitask while compiling. Always use async_cargo_mcp MCP tools; do not run cargo in a terminal. For operations >1s, set enable_async_notifications=true and call mcp_async_cargo_m_wait to collect results."
    )]
    async fn rustc(
        &self,
        Parameters(req): Parameters<RustcRequest>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, ErrorData> {
        let rustc_id = self.generate_operation_id();

        // Check if async notifications are enabled
        if req.enable_async_notifications.unwrap_or(false) {
            // TRUE 2-STAGE ASYNC PATTERN:
            // 1. Send immediate response that operation has started
            // 2. Spawn background task to do actual work and send notifications

            let peer = context.peer.clone();
            let req_clone = req.clone();
            let rustc_id_clone = rustc_id.clone();
            let monitor = self.monitor.clone();

            // Register operation before spawn
            self.register_async_operation(
                &rustc_id,
                "cargo rustc",
                "Compiling with custom rustc options in background",
                Some(req.working_directory.clone()),
            )
            .await;

            // Spawn background task for actual rustc work
            tokio::spawn(async move {
                // Create MCP callback sender to notify the LLM client
                let callback = mcp_callback(peer, rustc_id_clone.clone());

                // Send started notification immediately
                let _ = callback
                    .send_progress(ProgressUpdate::Started {
                        operation_id: rustc_id_clone.clone(),
                        command: "cargo rustc".to_string(),
                        description: "Compiling with custom rustc options in background"
                            .to_string(),
                    })
                    .await;

                // Do the actual rustc work
                let started_at = Instant::now();
                let result = Self::rustc_implementation(&req_clone).await;
                // Store for wait
                let _ = monitor
                    .complete_operation(&rustc_id_clone, result.clone())
                    .await;

                // Send completion notification
                let duration_ms = started_at.elapsed().as_millis() as u64;
                let completion_update = match result {
                    Ok(msg) => ProgressUpdate::Completed {
                        operation_id: rustc_id_clone,
                        message: msg,
                        duration_ms,
                    },
                    Err(err) => ProgressUpdate::Failed {
                        operation_id: rustc_id_clone,
                        error: err,
                        duration_ms,
                    },
                };

                let _ = callback.send_progress(completion_update).await;
            });

            // Return immediate response to LLM - this is the "first stage"
            let tool_hint = self.generate_tool_hint(&rustc_id, "rustc compilation");
            Ok(CallToolResult::success(vec![Content::text(format!(
                "Rustc operation {rustc_id} started in background.{tool_hint}"
            ))]))
        } else {
            // Synchronous operation for when async notifications are disabled
            match Self::rustc_implementation(&req).await {
                Ok(result_msg) => Ok(CallToolResult::success(vec![Content::text(result_msg)])),
                Err(error_msg) => Ok(CallToolResult::success(vec![Content::text(error_msg)])),
            }
        }
    }

    /// Internal implementation of rustc logic
    async fn rustc_implementation(req: &RustcRequest) -> Result<String, String> {
        use tokio::process::Command;

        let rustc_id = uuid::Uuid::new_v4().to_string()[..8].to_string();

        let mut cmd = Command::new("cargo");
        cmd.arg("rustc");

        // Add cargo-specific arguments first
        if let Some(cargo_args) = &req.cargo_args {
            cmd.args(cargo_args);
        }

        // Add rustc-specific arguments after --
        if let Some(rustc_args) = &req.rustc_args
            && !rustc_args.is_empty()
        {
            cmd.arg("--");
            cmd.args(rustc_args);
        }

        cmd.current_dir(&req.working_directory);

        let output = cmd
            .output()
            .await
            .map_err(|e| format!("Failed to execute cargo rustc: {e}"))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        let working_dir_msg = format!(" in {}", &req.working_directory);

        if output.status.success() {
            Ok(format!(
                "Rustc operation #{rustc_id} completed successfully{working_dir_msg}.\nOutput: {stdout}"
            ))
        } else {
            Err(format!(
                "- Rustc operation #{rustc_id} failed{working_dir_msg}.\nErrors: {stderr}\nOutput: {stdout}"
            ))
        }
    }

    #[tool(
        description = "CARGO METADATA: Safer than terminal cargo. Output JSON metadata about the project. Fast operation that provides LLMs with comprehensive project structure information. Always use async_cargo_mcp MCP tools; do not run cargo in a terminal. For operations >1s, set enable_async_notifications=true and call mcp_async_cargo_m_wait to collect results."
    )]
    async fn metadata(
        &self,
        Parameters(req): Parameters<MetadataRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        use tokio::process::Command;

        let metadata_id = self.generate_operation_id();

        let mut cmd = Command::new("cargo");
        cmd.arg("metadata");

        // Cargo metadata defaults to JSON format, but we can specify it explicitly
        if let Some(format) = &req.format {
            if format == "json" {
                cmd.arg("--format-version=1");
            }
        } else {
            // Default to JSON format
            cmd.arg("--format-version=1");
        }

        // Add no-deps flag if requested
        if req.no_deps.unwrap_or(false) {
            cmd.arg("--no-deps");
        }

        // Add features if specified
        if let Some(features) = &req.features
            && !features.is_empty()
        {
            cmd.args(["--features", &features.join(",")]);
        }

        // Add all-features flag if requested
        if req.all_features.unwrap_or(false) {
            cmd.arg("--all-features");
        }

        // Add no-default-features flag if requested
        if req.no_default_features.unwrap_or(false) {
            cmd.arg("--no-default-features");
        }

        // Add any additional arguments
        if let Some(args) = &req.args {
            cmd.args(args);
        }

        cmd.current_dir(&req.working_directory);

        let output = cmd.output().await.map_err(|e| {
            ErrorData::internal_error(format!("Failed to execute cargo metadata: {e}"), None)
        })?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        let working_dir_msg = format!(" in {}", &req.working_directory);

        let result_msg = if output.status.success() {
            // For JSON output, we might want to validate it's valid JSON
            let json_validation = if req.format.as_deref().unwrap_or("json") == "json" {
                match serde_json::from_str::<serde_json::Value>(&stdout) {
                    Ok(_) => " (JSON validated)",
                    Err(_) => " (JSON validation failed)",
                }
            } else {
                ""
            };

            format!(
                "📊 Metadata operation #{metadata_id} completed successfully{working_dir_msg}{json_validation}.\nProject metadata:\n{stdout}"
            )
        } else {
            format!(
                "- Metadata operation #{metadata_id} failed{working_dir_msg}.\nErrors: {stderr}\nOutput: {stdout}"
            )
        };

        Ok(CallToolResult::success(vec![Content::text(result_msg)]))
    }
}

#[tool_handler]
impl ServerHandler for AsyncCargo {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder()
                .enable_prompts()
                .enable_resources()
                .enable_tools()
                .build(),
            server_info: Implementation::from_build_env(),
            instructions: Some("Rust cargo operations with async support.\n\nGolden rules for LLM agents:\n1) Always use async_cargo_mcp MCP tools for ALL cargo operations. Do not run cargo in a terminal.\n2) For builds/tests >1s, set enable_async_notifications=true to multitask while work runs.\n3) After starting an async operation, you MUST call mcp_async_cargo_m_wait to retrieve results before making decisions.\n\nThese tools are safer than terminal commands and provide structured output, progress callbacks, and isolation.".to_string()),
        }
    }

    async fn list_resources(
        &self,
        _request: Option<PaginatedRequestParam>,
        _: RequestContext<RoleServer>,
    ) -> Result<ListResourcesResult, ErrorData> {
        Ok(ListResourcesResult {
            resources: vec![
                self._create_resource_text("str:////Users/to/some/path/", "cwd"),
                self._create_resource_text("memo://insights", "memo-name"),
            ],
            next_cursor: None,
        })
    }

    async fn read_resource(
        &self,
        ReadResourceRequestParam { uri }: ReadResourceRequestParam,
        _: RequestContext<RoleServer>,
    ) -> Result<ReadResourceResult, ErrorData> {
        match uri.as_str() {
            "str:////Users/to/some/path/" => {
                let cwd = "/Users/to/some/path/";
                Ok(ReadResourceResult {
                    contents: vec![ResourceContents::text(cwd, uri)],
                })
            }
            "memo://insights" => {
                let memo = "Business Intelligence Memo\n\nAnalysis has revealed 5 key insights ...";
                Ok(ReadResourceResult {
                    contents: vec![ResourceContents::text(memo, uri)],
                })
            }
            _ => Err(ErrorData::resource_not_found(
                "resource_not_found",
                Some(json!({
                    "uri": uri
                })),
            )),
        }
    }

    async fn list_prompts(
        &self,
        _request: Option<PaginatedRequestParam>,
        _: RequestContext<RoleServer>,
    ) -> Result<ListPromptsResult, ErrorData> {
        Ok(ListPromptsResult {
            next_cursor: None,
            prompts: vec![Prompt::new(
                "example_prompt",
                Some("This is an example prompt that takes one required argument, message"),
                Some(vec![PromptArgument {
                    name: "message".to_string(),
                    description: Some("A message to put in the prompt".to_string()),
                    required: Some(true),
                }]),
            )],
        })
    }

    async fn get_prompt(
        &self,
        GetPromptRequestParam { name, arguments }: GetPromptRequestParam,
        _: RequestContext<RoleServer>,
    ) -> Result<GetPromptResult, ErrorData> {
        match name.as_str() {
            "example_prompt" => {
                let message = arguments
                    .and_then(|json| json.get("message")?.as_str().map(|s| s.to_string()))
                    .ok_or_else(|| {
                        ErrorData::invalid_params("No message provided to example_prompt", None)
                    })?;

                let prompt =
                    format!("This is an example prompt with your message here: '{message}'");
                Ok(GetPromptResult {
                    description: None,
                    messages: vec![PromptMessage {
                        role: PromptMessageRole::User,
                        content: PromptMessageContent::text(prompt),
                    }],
                })
            }
            _ => Err(ErrorData::invalid_params("prompt not found", None)),
        }
    }

    async fn list_resource_templates(
        &self,
        _request: Option<PaginatedRequestParam>,
        _: RequestContext<RoleServer>,
    ) -> Result<ListResourceTemplatesResult, ErrorData> {
        Ok(ListResourceTemplatesResult {
            next_cursor: None,
            resource_templates: Vec::new(),
        })
    }

    async fn initialize(
        &self,
        request: InitializeRequestParam,
        context: RequestContext<RoleServer>,
    ) -> Result<InitializeResult, ErrorData> {
        tracing::debug!("=== INITIALIZE METHOD CALLED ===");
        tracing::debug!("Initialize request: {request:?}");
        tracing::debug!("Request context: {context:?}");

        if let Some(http_request_part) = context.extensions.get::<axum::http::request::Parts>() {
            let initialize_headers = &http_request_part.headers;
            let initialize_uri = &http_request_part.uri;
            tracing::debug!(?initialize_headers, %initialize_uri, "initialize from http server");
        } else {
            tracing::debug!("No HTTP request parts found - this is stdio transport");
        }

        // Generate and log availability report for LLM
        let availability_report = Self::generate_availability_report().await;
        tracing::info!("Cargo Component Availability:\n{}", availability_report);

        let mut result = self.get_info();

        // Add availability information to the instructions and GPT-5 (Preview) notice
        let enhanced_instructions = format!(
            "{}.\n\nAVAILABILITY REPORT:\n{}\n\nNOTE: GPT-5 (Preview) is enabled for all clients. You may select it in your client if supported.",
            result.instructions.unwrap_or_default(),
            availability_report
        );
        result.instructions = Some(enhanced_instructions);

        tracing::debug!("Initialize result: {result:?}");
        tracing::debug!("=== INITIALIZE METHOD RETURNING ===");
        Ok(result)
    }
}

/// Async cargo operations with callback support
impl AsyncCargo {
    // Small wrappers to reuse existing callback-based implementations while returning a Result<String,String>
    pub async fn build_add_result(
        &self,
        req: DependencyRequest,
        callback: Option<Box<dyn CallbackSender>>,
    ) -> Result<String, String> {
        self.add_with_callback(req, callback).await
    }

    pub async fn build_remove_result(
        &self,
        req: RemoveDependencyRequest,
        callback: Option<Box<dyn CallbackSender>>,
    ) -> Result<String, String> {
        self.remove_with_callback(req, callback).await
    }
    /// Add a dependency with optional async callback notifications
    pub async fn add_with_callback(
        &self,
        req: DependencyRequest,
        callback: Option<Box<dyn CallbackSender>>,
    ) -> Result<String, String> {
        use tokio::process::Command;

        let operation_id = self.generate_operation_id().to_string();
        let start_time = Instant::now();

        let callback = callback.unwrap_or_else(|| no_callback());

        // Send start notification
        let cmd_str = format!("cargo add {}", req.name);
        let _ = callback
            .send_progress(ProgressUpdate::Started {
                operation_id: operation_id.clone(),
                command: cmd_str.clone(),
                description: format!("Adding dependency: {}", req.name),
            })
            .await;

        let mut cmd = Command::new("cargo");

        // Build the dependency specification
        let dep_spec = if let Some(version) = &req.version {
            format!("{}@{}", req.name, version)
        } else {
            req.name.clone()
        };

        cmd.arg("add").arg(&dep_spec);

        // Set working directory
        cmd.current_dir(&req.working_directory);

        // Add optional features
        if let Some(features) = &req.features
            && !features.is_empty()
        {
            cmd.arg("--features").arg(features.join(","));
        }

        // Add optional flag
        if req.optional.unwrap_or(false) {
            cmd.arg("--optional");
        }

        // Execute command and collect full output
        let output = cmd
            .output()
            .await
            .map_err(|e| format!("Failed to execute cargo add: {e}"))?;

        let duration_ms = start_time.elapsed().as_millis() as u64;
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        let working_dir_msg = format!(" in {}", &req.working_directory);

        if output.status.success() {
            let success_msg = format!(
                "➕ Add operation completed successfully{working_dir_msg}.\nAdded dependency: {}\nOutput: {stdout}",
                req.name
            );

            // Send completion notification
            let _ = callback
                .send_progress(ProgressUpdate::Completed {
                    operation_id,
                    message: success_msg.clone(),
                    duration_ms,
                })
                .await;

            Ok(success_msg)
        } else {
            let error_msg = format!(
                "- Add operation failed{working_dir_msg}.\nDependency: {}\nError: {stderr}\nOutput: {stdout}",
                req.name
            );

            // Send failure notification
            let _ = callback
                .send_progress(ProgressUpdate::Failed {
                    operation_id,
                    error: error_msg.clone(),
                    duration_ms,
                })
                .await;

            Err(error_msg)
        }
    }

    /// Remove a dependency with optional async callback notifications
    pub async fn remove_with_callback(
        &self,
        req: RemoveDependencyRequest,
        callback: Option<Box<dyn CallbackSender>>,
    ) -> Result<String, String> {
        use tokio::process::Command;

        let operation_id = self.generate_operation_id().to_string();
        let start_time = Instant::now();

        let callback = callback.unwrap_or_else(|| no_callback());

        // Send start notification
        let cmd_str = format!("cargo remove {}", req.name);
        let _ = callback
            .send_progress(ProgressUpdate::Started {
                operation_id: operation_id.clone(),
                command: cmd_str.clone(),
                description: format!("Removing dependency: {}", req.name),
            })
            .await;

        let mut cmd = Command::new("cargo");
        cmd.arg("remove").arg(&req.name);

        // Set working directory
        cmd.current_dir(&req.working_directory);

        // Execute command and collect full output
        let output = cmd
            .output()
            .await
            .map_err(|e| format!("Failed to execute cargo remove: {e}"))?;

        let duration_ms = start_time.elapsed().as_millis() as u64;
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        let working_dir_msg = format!(" in {}", &req.working_directory);

        if output.status.success() {
            let success_msg = format!(
                "➖ Remove operation completed successfully{working_dir_msg}.\nRemoved dependency: {}\nOutput: {stdout}",
                req.name
            );

            // Send completion notification
            let _ = callback
                .send_progress(ProgressUpdate::Completed {
                    operation_id,
                    message: success_msg.clone(),
                    duration_ms,
                })
                .await;

            Ok(success_msg)
        } else {
            let error_msg = format!(
                "- Remove operation failed{working_dir_msg}.\nDependency: {}\nError: {stderr}\nOutput: {stdout}",
                req.name
            );

            // Send failure notification
            let _ = callback
                .send_progress(ProgressUpdate::Failed {
                    operation_id,
                    error: error_msg.clone(),
                    duration_ms,
                })
                .await;

            Err(error_msg)
        }
    }

    /// Build project with optional async callback notifications
    pub async fn build_with_callback(
        &self,
        req: BuildRequest,
        callback: Option<Box<dyn CallbackSender>>,
    ) -> Result<String, String> {
        use tokio::process::Command;
        let operation_id = self.generate_operation_id().to_string();
        let start_time = Instant::now();

        let callback = callback.unwrap_or_else(|| no_callback());

        // Send start notification
        let _ = callback
            .send_progress(ProgressUpdate::Started {
                operation_id: operation_id.clone(),
                command: "cargo build".to_string(),
                description: "Building project".to_string(),
            })
            .await;

        let mut cmd = Command::new("cargo");
        cmd.arg("build");

        // Set working directory
        cmd.current_dir(&req.working_directory);

        // Execute command and collect full output
        let output = cmd.output().await.map_err(|e| {
            format!(
                "- Build operation failed in {}.\nError: Failed to execute cargo build: {}",
                &req.working_directory, e
            )
        })?;

        let duration_ms = start_time.elapsed().as_millis() as u64;
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        let working_dir_msg = format!(" in {}", &req.working_directory);

        if output.status.success() {
            let success_msg =
                format!("+ Build completed successfully{working_dir_msg}.\nOutput: {stdout}");

            // Send completion notification
            let _ = callback
                .send_progress(ProgressUpdate::Completed {
                    operation_id,
                    message: success_msg.clone(),
                    duration_ms,
                })
                .await;

            Ok(success_msg)
        } else {
            let error_msg =
                format!("- Build failed{working_dir_msg}.\nError: {stderr}\nOutput: {stdout}");

            // Send failure notification
            let _ = callback
                .send_progress(ProgressUpdate::Failed {
                    operation_id,
                    error: error_msg.clone(),
                    duration_ms,
                })
                .await;

            Err(error_msg)
        }
    }

    /// Audit project for security vulnerabilities with optional async callback notifications
    pub async fn audit_with_callback(
        &self,
        req: AuditRequest,
        callback: Option<Box<dyn CallbackSender>>,
    ) -> Result<String, String> {
        use tokio::process::Command;

        let operation_id = self.generate_operation_id().to_string();
        let start_time = Instant::now();

        let callback = callback.unwrap_or_else(|| no_callback());

        // Send start notification
        let _ = callback
            .send_progress(ProgressUpdate::Started {
                operation_id: operation_id.clone(),
                command: "cargo audit".to_string(),
                description: "Scanning for security vulnerabilities".to_string(),
            })
            .await;

        // First check if cargo-audit is available
        let audit_check = Command::new("cargo")
            .args(["audit", "--version"])
            .output()
            .await;

        if audit_check.is_err() || !audit_check.unwrap().status.success() {
            let error_msg = "- Audit operation failed: cargo-audit is not installed. 
📦 Install with: cargo install cargo-audit
🔒 This tool scans for known security vulnerabilities in dependencies."
                .to_string();

            let _ = callback
                .send_progress(ProgressUpdate::Failed {
                    operation_id,
                    error: error_msg.clone(),
                    duration_ms: start_time.elapsed().as_millis() as u64,
                })
                .await;

            return Err(error_msg);
        }

        let mut cmd = Command::new("cargo");
        cmd.arg("audit");

        // Add format flag if specified
        if let Some(format) = &req.format {
            cmd.args(["--format", format]);
        }

        // Add vulnerabilities-only flag if requested
        if req.vulnerabilities_only.unwrap_or(false) {
            cmd.arg("--vulnerabilities");
        }

        // Add deny warnings flag if requested
        if req.deny_warnings.unwrap_or(false) {
            cmd.arg("--deny-warnings");
        }

        // Add any additional arguments
        if let Some(args) = &req.args {
            cmd.args(args);
        }

        // Set working directory
        cmd.current_dir(&req.working_directory);

        // Execute command and collect full output
        let output = cmd
            .output()
            .await
            .map_err(|e| format!("Failed to execute cargo audit: {e}"))?;

        let duration_ms = start_time.elapsed().as_millis() as u64;
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        let working_dir_msg = format!(" in {}", &req.working_directory);

        if output.status.success() {
            let success_msg = format!(
                "🔒 Audit completed successfully{working_dir_msg}.\nNo known vulnerabilities found.\nOutput: {stdout}"
            );

            // Send completion notification
            let _ = callback
                .send_progress(ProgressUpdate::Completed {
                    operation_id,
                    message: success_msg.clone(),
                    duration_ms,
                })
                .await;

            Ok(success_msg)
        } else {
            // Check if it's a vulnerability warning (exit code 1) vs actual error
            let vulnerability_detected = output.status.code() == Some(1) && !stdout.is_empty();

            let result_msg = if vulnerability_detected {
                format!(
                    "Audit found security vulnerabilities{working_dir_msg}.\nVulnerabilities detected:\n{stdout}\nErrors: {stderr}"
                )
            } else {
                format!("- Audit failed{working_dir_msg}.\nErrors: {stderr}\nOutput: {stdout}")
            };

            // For vulnerabilities, we treat it as a completion with warnings, not a failure
            if vulnerability_detected {
                let _ = callback
                    .send_progress(ProgressUpdate::Completed {
                        operation_id,
                        message: result_msg.clone(),
                        duration_ms,
                    })
                    .await;
                Ok(result_msg)
            } else {
                let _ = callback
                    .send_progress(ProgressUpdate::Failed {
                        operation_id,
                        error: result_msg.clone(),
                        duration_ms,
                    })
                    .await;
                Err(result_msg)
            }
        }
    }
}
