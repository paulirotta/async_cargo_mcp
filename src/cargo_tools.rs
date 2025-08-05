use crate::callback_system::{CallbackSender, LoggingCallbackSender, ProgressUpdate, no_callback};
use crate::mcp_callback::mcp_callback;
use rmcp::{
    ErrorData as McpError, RoleServer, ServerHandler,
    handler::server::{router::tool::ToolRouter, tool::Parameters},
    model::*,
    schemars,
    service::RequestContext,
    tool, tool_handler, tool_router,
};
use serde_json::json;
use std::collections::HashMap;
use std::time::Instant;

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct DependencyRequest {
    pub name: String,
    pub version: Option<String>,
    pub features: Option<Vec<String>>,
    pub optional: Option<bool>,
    pub working_directory: String,
    /// Enable async callback notifications for operation progress
    pub enable_async_notifications: Option<bool>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct RemoveDependencyRequest {
    pub name: String,
    pub working_directory: String,
    /// Enable async callback notifications for operation progress
    pub enable_async_notifications: Option<bool>,
}

#[derive(Debug, Clone, serde::Deserialize, schemars::JsonSchema)]
pub struct BuildRequest {
    pub working_directory: String,
    /// Enable async callback notifications for operation progress
    pub enable_async_notifications: Option<bool>,
}

#[derive(Debug, Clone, serde::Deserialize, schemars::JsonSchema)]
pub struct RunRequest {
    pub working_directory: String,
    /// Enable async callback notifications for operation progress
    pub enable_async_notifications: Option<bool>,
}

#[derive(Debug, Clone, serde::Deserialize, schemars::JsonSchema)]
pub struct TestRequest {
    pub working_directory: String,
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

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct InstallRequest {
    pub package: String,
    pub version: Option<String>,
    pub working_directory: String,
    /// Enable async callback notifications for operation progress
    pub enable_async_notifications: Option<bool>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
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

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
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

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
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

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
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

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
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

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
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

#[derive(Clone, Debug)]
pub struct AsyncCargo {
    tool_router: ToolRouter<AsyncCargo>,
}

#[tool_router]
impl AsyncCargo {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
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
        report.push_str("✅ • build, test, run, check, doc, add, remove, update, clean, fix, search, bench, install, tree, version, fetch, rustc, metadata\n\n");

        report.push_str("Optional Components:\n");

        if *availability.get("clippy").unwrap_or(&false) {
            report.push_str("✅ clippy - Available (enhanced linting)\n");
        } else {
            report.push_str(
                "❌ clippy - Not available (install with: rustup component add clippy)\n",
            );
        }

        if *availability.get("nextest").unwrap_or(&false) {
            report.push_str("✅ nextest - Available (faster test runner)\n");
        } else {
            report.push_str(
                "❌ nextest - Not available (install with: cargo install cargo-nextest)\n",
            );
        }

        if *availability.get("cargo-edit").unwrap_or(&false) {
            report.push_str("✅ cargo-edit - Available (upgrade command for dependency updates)\n");
        } else {
            report.push_str(
                "❌ cargo-edit - Not available (install with: cargo install cargo-edit)\n",
            );
        }

        if *availability.get("cargo-audit").unwrap_or(&false) {
            report.push_str("✅ cargo-audit - Available (security vulnerability scanning)\n");
        } else {
            report.push_str(
                "❌ cargo-audit - Not available (install with: cargo install cargo-audit)\n",
            );
        }

        if *availability.get("rustfmt").unwrap_or(&false) {
            report.push_str("✅ rustfmt - Available (code formatting with cargo fmt)\n");
        } else {
            report.push_str(
                "❌ rustfmt - Not available (install with: rustup component add rustfmt)\n",
            );
        }

        report.push_str("\n💡 Recommendations:\n");
        report.push_str(
            "⚡ • Use 'nextest' instead of 'test' for faster test execution if available\n",
        );
        report.push_str("🔍 • Use 'clippy' for enhanced code quality checks if available\n");
        report.push_str(
            "⬆️ • Use 'upgrade' for intelligent dependency updates if cargo-edit is available\n",
        );
        report.push_str(
            "🔒 • Use 'audit' for security vulnerability scanning if cargo-audit is available\n",
        );
        report.push_str(
            "🔄 • Enable async notifications (enable_async_notifications=true) for long operations\n",
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

    #[tool(
        description = "BUILD: Safer than terminal cargo. Use enable_async_notifications=true for builds >1s to multitask. Structured output + isolation."
    )]
    async fn build(
        &self,
        Parameters(req): Parameters<BuildRequest>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let build_id = self.generate_operation_id();

        // Check if async notifications are enabled
        if req.enable_async_notifications.unwrap_or(false) {
            // TRUE 2-STAGE ASYNC PATTERN:
            // 1. Send immediate response that operation has started
            // 2. Spawn background task to do actual work and send notifications

            let peer = context.peer.clone();
            let req_clone = req.clone();
            let build_id_clone = build_id.clone();

            // Spawn background task for actual build work
            tokio::spawn(async move {
                // Create MCP callback sender to notify the LLM client
                let callback = mcp_callback(peer, build_id_clone.clone());

                // Send started notification immediately
                let _ = callback
                    .send_progress(ProgressUpdate::Started {
                        operation_id: build_id_clone.clone(),
                        command: "cargo build".to_string(),
                        description: "Building project in background".to_string(),
                    })
                    .await;

                // Do the actual build work
                let result = Self::build_implementation(&req_clone).await;

                // Send completion notification
                let completion_update = match result {
                    Ok(msg) => ProgressUpdate::Completed {
                        operation_id: build_id_clone,
                        message: msg,
                        duration_ms: 0, // TODO: Add actual timing
                    },
                    Err(err) => ProgressUpdate::Failed {
                        operation_id: build_id_clone,
                        error: err,
                        duration_ms: 0,
                    },
                };

                let _ = callback.send_progress(completion_update).await;
            });

            // Return immediate response to LLM - this is the "first stage"
            Ok(CallToolResult::success(vec![Content::text(format!(
                "✅ Build operation {} started in background. You will receive progress notifications as the build proceeds.",
                build_id
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

        // Set working directory
        cmd.current_dir(&req.working_directory);

        // Execute command and collect full output
        let output = cmd.output().await.map_err(|e| {
            format!(
                "❌ Build operation failed in {}.\nError: Failed to execute cargo build: {}",
                &req.working_directory, e
            )
        })?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        let working_dir_msg = format!(" in {}", &req.working_directory);

        if output.status.success() {
            Ok(format!(
                "✅ Build completed successfully{working_dir_msg}.\nOutput: {stdout}"
            ))
        } else {
            Err(format!(
                "❌ Build failed{working_dir_msg}.\nError: {stderr}\nOutput: {stdout}"
            ))
        }
    }

    #[tool(
        description = "RUN: Safer than terminal cargo. Use enable_async_notifications=true for long apps to multitask. Structured output + isolation."
    )]
    async fn run(
        &self,
        Parameters(req): Parameters<RunRequest>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let run_id = self.generate_operation_id();

        // Check if async notifications are enabled
        if req.enable_async_notifications.unwrap_or(false) {
            // TRUE 2-STAGE ASYNC PATTERN:
            // 1. Send immediate response that operation has started
            // 2. Spawn background task to do actual work and send notifications

            let peer = context.peer.clone();
            let req_clone = req.clone();
            let run_id_clone = run_id.clone();

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
                let result = Self::run_implementation(&req_clone).await;

                // Send completion notification
                let completion_update = match result {
                    Ok(msg) => ProgressUpdate::Completed {
                        operation_id: run_id_clone,
                        message: msg,
                        duration_ms: 0, // TODO: Add actual timing
                    },
                    Err(err) => ProgressUpdate::Failed {
                        operation_id: run_id_clone,
                        error: err,
                        duration_ms: 0,
                    },
                };

                let _ = callback.send_progress(completion_update).await;
            });

            // Return immediate response to LLM - this is the "first stage"
            Ok(CallToolResult::success(vec![Content::text(format!(
                "🚀 Run operation {} started in background. You will receive progress notifications as the application runs.",
                run_id
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

        // Set working directory
        cmd.current_dir(&req.working_directory);

        let output = cmd
            .output()
            .await
            .map_err(|e| format!("Run operation failed: Failed to execute cargo run: {}", e))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        let working_dir_msg = format!(" in {}", &req.working_directory);

        if output.status.success() {
            Ok(format!(
                "✅ Run operation completed successfully{working_dir_msg}.\nOutput: {stdout}"
            ))
        } else {
            Err(format!(
                "❌ Run operation failed{working_dir_msg}.\nErrors: {stderr}\nOutput: {stdout}"
            ))
        }
    }

    #[tool(
        description = "TEST: Safer than terminal cargo. ALWAYS use enable_async_notifications=true for test suites to multitask. Real-time progress + isolation."
    )]
    async fn test(
        &self,
        Parameters(req): Parameters<TestRequest>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let test_id = self.generate_operation_id();

        // Check if async notifications are enabled
        if req.enable_async_notifications.unwrap_or(false) {
            // TRUE 2-STAGE ASYNC PATTERN:
            // 1. Send immediate response that operation has started
            // 2. Spawn background task to do actual work and send notifications

            let peer = context.peer.clone();
            let req_clone = req.clone();
            let test_id_clone = test_id.clone();

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
                let result = Self::test_implementation(&req_clone).await;

                // Send completion notification
                let completion_update = match result {
                    Ok(msg) => ProgressUpdate::Completed {
                        operation_id: test_id_clone,
                        message: msg,
                        duration_ms: 0, // TODO: Add actual timing
                    },
                    Err(err) => ProgressUpdate::Failed {
                        operation_id: test_id_clone,
                        error: err,
                        duration_ms: 0,
                    },
                };

                let _ = callback.send_progress(completion_update).await;
            });

            // Return immediate response to LLM - this is the "first stage"
            Ok(CallToolResult::success(vec![Content::text(format!(
                "🧪 Test operation {} started in background. You will receive progress notifications as the tests run.",
                test_id
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

        // Set working directory
        cmd.current_dir(&req.working_directory);

        let output = cmd
            .output()
            .await
            .map_err(|e| format!("Failed to execute cargo test: {}", e))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        let working_dir_msg = format!(" in {}", &req.working_directory);

        if output.status.success() {
            Ok(format!(
                "🧪 Test operation #{test_id} completed successfully{working_dir_msg}.\nOutput: {stdout}"
            ))
        } else {
            Err(format!(
                "❌ Test operation #{test_id} failed{working_dir_msg}.\nErrors: {stderr}\nOutput: {stdout}"
            ))
        }
    }

    #[tool(
        description = "CHECK: Safer than terminal cargo. Fast validation - async optional for large projects. Quick compile check."
    )]
    async fn check(
        &self,
        Parameters(req): Parameters<CheckRequest>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let check_id = self.generate_operation_id();

        // Check if async notifications are enabled
        if req.enable_async_notifications.unwrap_or(false) {
            // TRUE 2-STAGE ASYNC PATTERN:
            // 1. Send immediate response that operation has started
            // 2. Spawn background task to do actual work and send notifications

            let peer = context.peer.clone();
            let req_clone = req.clone();
            let check_id_clone = check_id.clone();

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
                let result = Self::check_implementation(&req_clone).await;

                // Send completion notification
                let completion_update = match result {
                    Ok(msg) => ProgressUpdate::Completed {
                        operation_id: check_id_clone,
                        message: msg,
                        duration_ms: 0, // TODO: Add actual timing
                    },
                    Err(err) => ProgressUpdate::Failed {
                        operation_id: check_id_clone,
                        error: err,
                        duration_ms: 0,
                    },
                };

                let _ = callback.send_progress(completion_update).await;
            });

            // Return immediate response to LLM - this is the "first stage"
            Ok(CallToolResult::success(vec![Content::text(format!(
                "✅ Check operation {} started in background. You will receive progress notifications as the check proceeds.",
                check_id
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
                "❌ Check operation failed in {}.\nError: Failed to execute cargo check: {}",
                &req.working_directory, e
            )
        })?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        let working_dir_msg = format!(" in {}", &req.working_directory);

        if output.status.success() {
            Ok(format!(
                "✅ Check operation completed successfully{working_dir_msg}.\nOutput: {stdout}"
            ))
        } else {
            Err(format!(
                "❌ Check operation failed{working_dir_msg}.\nErrors: {stderr}\nOutput: {stdout}"
            ))
        }
    }

    #[tool(
        description = "ADD: Safer than terminal cargo. Use enable_async_notifications=true for complex deps to multitask. Handles version conflicts."
    )]
    async fn add(
        &self,
        Parameters(req): Parameters<DependencyRequest>,
    ) -> Result<CallToolResult, McpError> {
        let add_id = self.generate_operation_id();

        // Check if async notifications are enabled
        if req.enable_async_notifications.unwrap_or(false) {
            // Use the callback-enabled version for async notifications
            let callback: Box<dyn CallbackSender> =
                Box::new(LoggingCallbackSender::new(format!("cargo_add_{}", add_id)));

            match self.add_with_callback(req, Some(callback)).await {
                Ok(result_msg) => Ok(CallToolResult::success(vec![Content::text(result_msg)])),
                Err(error_msg) => Ok(CallToolResult::success(vec![Content::text(error_msg)])),
            }
        } else {
            // Use direct execution for synchronous operation
            use tokio::process::Command;

            // TODO: Add asynchronous callback mechanism here for real-time progress updates
            // Implementation plan:
            // 1. Stream dependency resolution and download progress to the LLM
            // 2. Show real-time progress for fetching crates and building dependencies
            // 3. Provide detailed error messages if dependency resolution fails
            // 4. Allow cancellation of long-running dependency installations
            // 5. Show version conflict warnings and resolution suggestions
            // This would allow streaming command output back to the LLM during long operations

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
            if let Some(features) = &req.features {
                if !features.is_empty() {
                    cmd.arg("--features").arg(features.join(","));
                }
            }

            // Add optional flag
            if req.optional.unwrap_or(false) {
                cmd.arg("--optional");
            }

            let output = cmd.output().await.map_err(|e| {
                McpError::internal_error(format!("Failed to execute cargo add: {}", e), None)
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
                    "❌ Add operation #{add_id} failed{working_dir_msg}.\nDependency: {}\nError: {stderr}\nOutput: {stdout}",
                    req.name
                )
            };

            Ok(CallToolResult::success(vec![Content::text(result_msg)]))
        }
    }

    #[tool(
        description = "REMOVE: Safer than terminal cargo. Fast operation - async not needed. Prevents Cargo.toml corruption."
    )]
    async fn remove(
        &self,
        Parameters(req): Parameters<RemoveDependencyRequest>,
    ) -> Result<CallToolResult, McpError> {
        let remove_id = self.generate_operation_id();

        // Check if async notifications are enabled
        if req.enable_async_notifications.unwrap_or(false) {
            // Use the callback-enabled version for async notifications
            let callback: Box<dyn CallbackSender> = Box::new(LoggingCallbackSender::new(format!(
                "cargo_remove_{}",
                remove_id
            )));

            match self.remove_with_callback(req, Some(callback)).await {
                Ok(result_msg) => Ok(CallToolResult::success(vec![Content::text(result_msg)])),
                Err(error_msg) => Ok(CallToolResult::success(vec![Content::text(error_msg)])),
            }
        } else {
            // Use direct execution for synchronous operation
            use tokio::process::Command;

            // TODO: Add asynchronous callback mechanism here for progress updates
            // Implementation plan:
            // 1. Provide real-time feedback on dependency removal process
            // 2. Show which files are being updated during removal
            // 3. Alert about any conflicts or issues during removal
            // 4. Allow early termination if removal encounters problems
            // Useful for informing the LLM about dependency removal progress

            let mut cmd = Command::new("cargo");
            cmd.arg("remove").arg(&req.name);

            // Set working directory
            cmd.current_dir(&req.working_directory);

            let output = cmd.output().await.map_err(|e| {
                McpError::internal_error(format!("Failed to execute cargo remove: {}", e), None)
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
                    "❌ Remove operation #{remove_id} failed{working_dir_msg}.\nDependency: {}\nError: {stderr}\nOutput: {stdout}",
                    req.name
                )
            };

            Ok(CallToolResult::success(vec![Content::text(result_msg)]))
        }
    }

    #[tool(
        description = "UPDATE: Safer than terminal cargo. Use enable_async_notifications=true for large projects to multitask. Shows version changes."
    )]
    async fn update(
        &self,
        Parameters(req): Parameters<UpdateRequest>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let update_id = self.generate_operation_id();

        // Check if async notifications are enabled
        if req.enable_async_notifications.unwrap_or(false) {
            // TRUE 2-STAGE ASYNC PATTERN:
            // 1. Send immediate response that operation has started
            // 2. Spawn background task to do actual work and send notifications

            let peer = context.peer.clone();
            let req_clone = req.clone();
            let update_id_clone = update_id.clone();

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
                let result = Self::update_implementation(&req_clone).await;

                // Send completion notification
                let completion_update = match result {
                    Ok(msg) => ProgressUpdate::Completed {
                        operation_id: update_id_clone,
                        message: msg,
                        duration_ms: 0, // TODO: Add actual timing
                    },
                    Err(err) => ProgressUpdate::Failed {
                        operation_id: update_id_clone,
                        error: err,
                        duration_ms: 0,
                    },
                };

                let _ = callback.send_progress(completion_update).await;
            });

            // Return immediate response to LLM - this is the "first stage"
            Ok(CallToolResult::success(vec![Content::text(format!(
                "⬆️ Update operation {} started in background. You will receive progress notifications as dependencies are updated.",
                update_id
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

        let output = cmd.output().await.map_err(|e| {
            format!(
                "Update operation failed: Failed to execute cargo update: {}",
                e
            )
        })?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        let working_dir_msg = format!(" in {}", &req.working_directory);

        if output.status.success() {
            Ok(format!(
                "⬆️ Update operation completed successfully{working_dir_msg}.\nOutput: {stdout}"
            ))
        } else {
            Err(format!(
                "❌ Update operation failed{working_dir_msg}.\nErrors: {stderr}\nOutput: {stdout}"
            ))
        }
    }

    #[tool(
        description = "DOC: Safer than terminal cargo. Use enable_async_notifications=true for large codebases to multitask. Creates LLM-friendly API reference."
    )]
    async fn doc(
        &self,
        Parameters(req): Parameters<DocRequest>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let doc_id = self.generate_operation_id();

        // Check if async notifications are enabled
        if req.enable_async_notifications.unwrap_or(false) {
            // TRUE 2-STAGE ASYNC PATTERN:
            // 1. Send immediate response that operation has started
            // 2. Spawn background task to do actual work and send notifications

            let peer = context.peer.clone();
            let req_clone = req.clone();
            let doc_id_clone = doc_id.clone();

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
                let result = Self::doc_implementation(&req_clone).await;

                // Send completion notification
                let completion_update = match result {
                    Ok(msg) => ProgressUpdate::Completed {
                        operation_id: doc_id_clone,
                        message: msg,
                        duration_ms: 0, // TODO: Add actual timing
                    },
                    Err(err) => ProgressUpdate::Failed {
                        operation_id: doc_id_clone,
                        error: err,
                        duration_ms: 0,
                    },
                };

                let _ = callback.send_progress(completion_update).await;
            });

            // Return immediate response to LLM - this is the "first stage"
            Ok(CallToolResult::success(vec![Content::text(format!(
                "📚 Documentation generation {} started in background. You will receive progress notifications as documentation is generated.",
                doc_id
            ))]))
        } else {
            // Synchronous operation for when async notifications are disabled
            match Self::doc_implementation(&req).await {
                Ok(result_msg) => Ok(CallToolResult::success(vec![Content::text(result_msg)])),
                Err(error_msg) => Ok(CallToolResult::success(vec![Content::text(error_msg)])),
            }
        }
    }

    /// Internal implementation of doc generation logic
    async fn doc_implementation(req: &DocRequest) -> Result<String, String> {
        use tokio::process::Command;

        let mut cmd = Command::new("cargo");
        cmd.arg("doc").arg("--no-deps");

        // Set working directory
        cmd.current_dir(&req.working_directory);

        let output = cmd.output().await.map_err(|e| {
            format!(
                "Documentation generation failed: Failed to execute cargo doc: {}",
                e
            )
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
                "📚 Documentation generation completed successfully{working_dir_msg}.
Documentation generated at: {}
The generated documentation provides comprehensive API information that can be used by LLMs for more accurate and up-to-date project understanding.
💡 Tip: Use this documentation to get the latest API details, examples, and implementation notes that complement the source code.

Output: {stdout}",
                doc_path
            ))
        } else {
            Err(format!(
                "❌ Documentation generation failed{working_dir_msg}.\nErrors: {stderr}\nOutput: {stdout}"
            ))
        }
    }
    #[tool(
        description = "CLIPPY: Safer than terminal cargo. Supports --fix via args=['--fix','--allow-dirty']. Fast operation - async optional."
    )]
    async fn clippy(
        &self,
        Parameters(req): Parameters<ClippyRequest>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let clippy_id = self.generate_operation_id();

        // Check if async notifications are enabled
        if req.enable_async_notifications.unwrap_or(false) {
            // TRUE 2-STAGE ASYNC PATTERN:
            // 1. Send immediate response that operation has started
            // 2. Spawn background task to do actual work and send notifications

            let peer = context.peer.clone();
            let req_clone = req.clone();
            let clippy_id_clone = clippy_id.clone();

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
                let result = Self::clippy_implementation(&req_clone).await;

                // Send completion notification
                let completion_update = match result {
                    Ok(msg) => ProgressUpdate::Completed {
                        operation_id: clippy_id_clone,
                        message: msg,
                        duration_ms: 0, // TODO: Add actual timing
                    },
                    Err(err) => ProgressUpdate::Failed {
                        operation_id: clippy_id_clone,
                        error: err,
                        duration_ms: 0,
                    },
                };

                let _ = callback.send_progress(completion_update).await;
            });

            // Return immediate response to LLM - this is the "first stage"
            Ok(CallToolResult::success(vec![Content::text(format!(
                "🔍 Clippy operation {} started in background. You will receive progress notifications as linting proceeds.",
                clippy_id
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

        let output = cmd.output().await.map_err(|e| {
            format!(
                "Clippy operation failed: Failed to execute cargo clippy: {}",
                e
            )
        })?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        let working_dir_msg = format!(" in {}", &req.working_directory);

        if output.status.success() {
            Ok(format!(
                "🔍 Clippy operation passed with no warnings{working_dir_msg}.\nOutput: {stdout}",
            ))
        } else {
            Err(format!(
                "❌ Clippy operation failed{working_dir_msg}.\nErrors: {stderr}\nOutput: {stdout}",
            ))
        }
    }

    #[tool(
        description = "NEXTEST: Safer than terminal cargo. Faster test runner. ALWAYS use enable_async_notifications=true for test suites to multitask. Real-time progress + isolation."
    )]
    async fn nextest(
        &self,
        Parameters(req): Parameters<NextestRequest>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let nextest_id = self.generate_operation_id();

        // First check if nextest is available
        let nextest_check = tokio::process::Command::new("cargo")
            .args(["nextest", "--version"])
            .output()
            .await;

        if nextest_check.is_err() || !nextest_check.unwrap().status.success() {
            return Ok(CallToolResult::success(vec![Content::text(format!(
                "❌ Nextest operation #{nextest_id} failed: cargo-nextest is not installed. 
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
                let result = Self::nextest_implementation(&req_clone).await;

                // Send completion notification
                let completion_update = match result {
                    Ok(msg) => ProgressUpdate::Completed {
                        operation_id: nextest_id_clone,
                        message: msg,
                        duration_ms: 0, // TODO: Add actual timing
                    },
                    Err(err) => ProgressUpdate::Failed {
                        operation_id: nextest_id_clone,
                        error: err,
                        duration_ms: 0,
                    },
                };

                let _ = callback.send_progress(completion_update).await;
            });

            // Return immediate response to LLM - this is the "first stage"
            Ok(CallToolResult::success(vec![Content::text(format!(
                "⚡ Nextest operation {} started in background. You will receive progress notifications as the fast tests run.",
                nextest_id
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
            .map_err(|e| format!("Failed to execute cargo nextest: {}", e))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        let working_dir_msg = format!(" in {}", &req.working_directory);

        if output.status.success() {
            Ok(format!(
                "⚡ Nextest operation #{nextest_id} completed successfully{working_dir_msg}.\nOutput: {stdout}"
            ))
        } else {
            Err(format!(
                "❌ Nextest operation #{nextest_id} failed{working_dir_msg}.\nErrors: {stderr}\nOutput: {stdout}"
            ))
        }
    }

    #[tool(
        description = "CLEAN: Safer than terminal cargo. Fast operation - async not needed. Frees disk space."
    )]
    async fn clean(
        &self,
        Parameters(req): Parameters<CleanRequest>,
    ) -> Result<CallToolResult, McpError> {
        use tokio::process::Command;

        let clean_id = self.generate_operation_id();

        let mut cmd = Command::new("cargo");
        cmd.arg("clean");

        cmd.current_dir(&req.working_directory);

        let output = cmd.output().await.map_err(|e| {
            McpError::internal_error(format!("Failed to execute cargo clean: {}", e), None)
        })?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        let working_dir_msg = format!(" in {}", &req.working_directory);

        let result_msg = if output.status.success() {
            format!(
                "🧹 Clean operation #{clean_id} completed successfully{working_dir_msg}.\nOutput: {stdout}"
            )
        } else {
            format!(
                "❌ Clean operation #{clean_id} failed{working_dir_msg}.\nErrors: {stderr}\nOutput: {stdout}"
            )
        };

        Ok(CallToolResult::success(vec![Content::text(result_msg)]))
    }

    #[tool(
        description = "FIX: Safer than terminal cargo. Automatically fix compiler warnings. Supports --allow-dirty via args. Use async for large codebases."
    )]
    async fn fix(
        &self,
        Parameters(req): Parameters<FixRequest>,
    ) -> Result<CallToolResult, McpError> {
        use tokio::process::Command;

        let fix_id = self.generate_operation_id();

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
            McpError::internal_error(format!("Failed to execute cargo fix: {}", e), None)
        })?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        let working_dir_msg = format!(" in {}", &req.working_directory);

        let result_msg = if output.status.success() {
            format!(
                "🔧 Fix operation #{fix_id} completed successfully{working_dir_msg}.\nOutput: {stdout}"
            )
        } else {
            format!(
                "❌ Fix operation #{fix_id} failed{working_dir_msg}.\nErrors: {stderr}\nOutput: {stdout}"
            )
        };

        Ok(CallToolResult::success(vec![Content::text(result_msg)]))
    }

    #[tool(
        description = "SEARCH: Safer than terminal cargo. Search for crates on crates.io. Fast operation - async not needed unless searching many terms."
    )]
    async fn search(
        &self,
        Parameters(req): Parameters<SearchRequest>,
    ) -> Result<CallToolResult, McpError> {
        use tokio::process::Command;

        let search_id = self.generate_operation_id();

        let mut cmd = Command::new("cargo");
        cmd.arg("search").arg(&req.query);

        if let Some(limit) = req.limit {
            cmd.args(["--limit", &limit.to_string()]);
        }

        let output = cmd.output().await.map_err(|e| {
            McpError::internal_error(format!("Failed to execute cargo search: {}", e), None)
        })?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        let result_msg = if output.status.success() {
            format!(
                "🔍 Search operation #{search_id} completed successfully.\nQuery: {}\nResults:\n{stdout}",
                req.query
            )
        } else {
            format!(
                "❌ Search operation #{search_id} failed.\nQuery: {}\nErrors: {stderr}\nOutput: {stdout}",
                req.query
            )
        };

        Ok(CallToolResult::success(vec![Content::text(result_msg)]))
    }

    #[tool(
        description = "BENCH: Safer than terminal cargo. ALWAYS use enable_async_notifications=true for benchmark suites to multitask. Performance testing."
    )]
    async fn bench(
        &self,
        Parameters(req): Parameters<BenchRequest>,
    ) -> Result<CallToolResult, McpError> {
        use tokio::process::Command;

        let bench_id = self.generate_operation_id();

        let mut cmd = Command::new("cargo");
        cmd.arg("bench");

        // Add any additional arguments
        if let Some(args) = &req.args {
            cmd.args(args);
        }

        cmd.current_dir(&req.working_directory);

        let output = cmd.output().await.map_err(|e| {
            McpError::internal_error(format!("Failed to execute cargo bench: {}", e), None)
        })?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        let working_dir_msg = format!(" in {}", &req.working_directory);

        let result_msg = if output.status.success() {
            format!(
                "🏃‍♂️ Benchmark operation #{bench_id} completed successfully{working_dir_msg}.\nOutput: {stdout}"
            )
        } else {
            format!(
                "❌ Benchmark operation #{bench_id} failed{working_dir_msg}.\nErrors: {stderr}\nOutput: {stdout}"
            )
        };

        Ok(CallToolResult::success(vec![Content::text(result_msg)]))
    }

    #[tool(
        description = "INSTALL: Safer than terminal cargo. Use enable_async_notifications=true for large packages to multitask. Global tool installation."
    )]
    async fn install(
        &self,
        Parameters(req): Parameters<InstallRequest>,
    ) -> Result<CallToolResult, McpError> {
        use tokio::process::Command;

        let install_id = self.generate_operation_id();

        let mut cmd = Command::new("cargo");
        cmd.arg("install");

        // Build the package specification
        let package_spec = if let Some(version) = &req.version {
            format!("{}@{}", req.package, version)
        } else {
            req.package.clone()
        };

        cmd.arg(&package_spec);

        cmd.current_dir(&req.working_directory);

        let output = cmd.output().await.map_err(|e| {
            McpError::internal_error(format!("Failed to execute cargo install: {}", e), None)
        })?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        let working_dir_msg = format!(" in {}", &req.working_directory);

        let result_msg = if output.status.success() {
            format!(
                "📦 Install operation #{install_id} completed successfully{working_dir_msg}.\nInstalled package: {}\nOutput: {stdout}",
                req.package
            )
        } else {
            format!(
                "❌ Install operation #{install_id} failed{working_dir_msg}.\nPackage: {}\nErrors: {stderr}\nOutput: {stdout}",
                req.package
            )
        };

        Ok(CallToolResult::success(vec![Content::text(result_msg)]))
    }

    #[tool(
        description = "UPGRADE: Safer than terminal cargo. Use enable_async_notifications=true for large projects to multitask. Updates dependencies to latest versions using cargo-edit."
    )]
    async fn upgrade(
        &self,
        Parameters(req): Parameters<UpgradeRequest>,
    ) -> Result<CallToolResult, McpError> {
        use tokio::process::Command;

        let upgrade_id = self.generate_operation_id();

        // First check if cargo-edit (upgrade command) is available
        let upgrade_check = Command::new("cargo")
            .args(["upgrade", "--version"])
            .output()
            .await;

        if upgrade_check.is_err() || !upgrade_check.unwrap().status.success() {
            return Ok(CallToolResult::success(vec![Content::text(format!(
                "❌ Upgrade operation #{upgrade_id} failed: cargo-edit with upgrade command is not installed. 
📦 Install with: cargo install cargo-edit
🔄 Falling back to regular cargo update is recommended."
            ))]));
        }

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

        let output = cmd.output().await.map_err(|e| {
            McpError::internal_error(format!("Failed to execute cargo upgrade: {}", e), None)
        })?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        let working_dir_msg = format!(" in {}", &req.working_directory);

        let result_msg = if output.status.success() {
            let dry_run_msg = if req.dry_run.unwrap_or(false) {
                " (dry run - no changes made)"
            } else {
                ""
            };
            format!(
                "⬆️ Upgrade operation #{upgrade_id} completed successfully{working_dir_msg}{dry_run_msg}.\nOutput: {stdout}"
            )
        } else {
            format!(
                "❌ Upgrade operation #{upgrade_id} failed{working_dir_msg}.\nErrors: {stderr}\nOutput: {stdout}"
            )
        };

        Ok(CallToolResult::success(vec![Content::text(result_msg)]))
    }

    #[tool(
        description = "AUDIT: Safer than terminal cargo. Security vulnerability scanning. Use enable_async_notifications=true for large projects to multitask. Identifies known security vulnerabilities."
    )]
    async fn audit(
        &self,
        Parameters(req): Parameters<AuditRequest>,
    ) -> Result<CallToolResult, McpError> {
        let audit_id = self.generate_operation_id();

        // Check if async notifications are enabled
        if req.enable_async_notifications.unwrap_or(false) {
            // Use the callback-enabled version for async notifications
            let callback: Box<dyn CallbackSender> = Box::new(LoggingCallbackSender::new(format!(
                "cargo_audit_{}",
                audit_id
            )));

            match self.audit_with_callback(req, Some(callback)).await {
                Ok(result_msg) => Ok(CallToolResult::success(vec![Content::text(result_msg)])),
                Err(error_msg) => Ok(CallToolResult::success(vec![Content::text(error_msg)])),
            }
        } else {
            // Use direct execution for synchronous operation
            use tokio::process::Command;

            // First check if cargo-audit is available
            let audit_check = Command::new("cargo")
                .args(["audit", "--version"])
                .output()
                .await;

            if audit_check.is_err() || !audit_check.unwrap().status.success() {
                return Ok(CallToolResult::success(vec![Content::text(format!(
                    "❌ Audit operation #{audit_id} failed: cargo-audit is not installed. 
📦 Install with: cargo install cargo-audit
🔒 This tool scans for known security vulnerabilities in dependencies."
                ))]));
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

            cmd.current_dir(&req.working_directory);

            let output = cmd.output().await.map_err(|e| {
                McpError::internal_error(format!("Failed to execute cargo audit: {}", e), None)
            })?;

            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);

            let working_dir_msg = format!(" in {}", &req.working_directory);

            let result_msg = if output.status.success() {
                format!(
                    "🔒 Audit operation #{audit_id} completed successfully{working_dir_msg}.\nNo known vulnerabilities found.\nOutput: {stdout}"
                )
            } else {
                // Check if it's a vulnerability warning (exit code 1) vs actual error
                let vulnerability_detected = output.status.code() == Some(1) && !stdout.is_empty();

                if vulnerability_detected {
                    format!(
                        "⚠️ Audit operation #{audit_id} found security vulnerabilities{working_dir_msg}.\nVulnerabilities detected:\n{stdout}\nErrors: {stderr}"
                    )
                } else {
                    format!(
                        "❌ Audit operation #{audit_id} failed{working_dir_msg}.\nErrors: {stderr}\nOutput: {stdout}"
                    )
                }
            };

            Ok(CallToolResult::success(vec![Content::text(result_msg)]))
        }
    }

    #[tool(
        description = "FMT: Safer than terminal cargo. Format Rust code using rustfmt. Use enable_async_notifications=true for large projects to multitask while code is being formatted."
    )]
    async fn fmt(
        &self,
        Parameters(req): Parameters<FmtRequest>,
    ) -> Result<CallToolResult, McpError> {
        use tokio::process::Command;

        let fmt_id = self.generate_operation_id();

        // First check if rustfmt is available
        let fmt_check = Command::new("rustfmt").arg("--version").output().await;

        if fmt_check.is_err() || !fmt_check.unwrap().status.success() {
            return Ok(CallToolResult::success(vec![Content::text(format!(
                "❌ Format operation #{fmt_id} failed: rustfmt is not installed. 
📦 Install with: rustup component add rustfmt
✨ This tool formats Rust code according to style guidelines."
            ))]));
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
            McpError::internal_error(format!("Failed to execute cargo fmt: {}", e), None)
        })?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        let working_dir_msg = format!(" in {}", &req.working_directory);

        let result_msg = if output.status.success() {
            let check_msg = if req.check.unwrap_or(false) {
                " (check mode - no changes made)"
            } else {
                ""
            };
            format!(
                "✨ Format operation #{fmt_id} completed successfully{working_dir_msg}{check_msg}.\nOutput: {stdout}"
            )
        } else {
            // Check if it's a formatting issue (exit code 1) vs actual error
            let formatting_issues = output.status.code() == Some(1) && req.check.unwrap_or(false);

            if formatting_issues {
                format!(
                    "⚠️ Format operation #{fmt_id} found formatting issues{working_dir_msg}.\nFiles need formatting:\n{stdout}\nErrors: {stderr}"
                )
            } else {
                format!(
                    "❌ Format operation #{fmt_id} failed{working_dir_msg}.\nErrors: {stderr}\nOutput: {stdout}"
                )
            }
        };

        Ok(CallToolResult::success(vec![Content::text(result_msg)]))
    }

    #[tool(
        description = "TREE: Safer than terminal cargo. Display dependency tree. Use enable_async_notifications=true for large projects to multitask while dependency tree is being generated."
    )]
    async fn tree(
        &self,
        Parameters(req): Parameters<TreeRequest>,
    ) -> Result<CallToolResult, McpError> {
        use tokio::process::Command;

        let tree_id = self.generate_operation_id();

        let mut cmd = Command::new("cargo");
        cmd.arg("tree");

        // Add depth limit if specified
        if let Some(depth) = req.depth {
            cmd.args(["--depth", &depth.to_string()]);
        }

        // Add features if specified
        if let Some(features) = &req.features {
            if !features.is_empty() {
                cmd.args(["--features", &features.join(",")]);
            }
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
            McpError::internal_error(format!("Failed to execute cargo tree: {}", e), None)
        })?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        let working_dir_msg = format!(" in {}", &req.working_directory);

        let result_msg = if output.status.success() {
            format!(
                "🌳 Tree operation #{tree_id} completed successfully{working_dir_msg}.\nDependency tree:\n{stdout}"
            )
        } else {
            format!(
                "❌ Tree operation #{tree_id} failed{working_dir_msg}.\nErrors: {stderr}\nOutput: {stdout}"
            )
        };

        Ok(CallToolResult::success(vec![Content::text(result_msg)]))
    }

    #[tool(
        description = "VERSION: Safer than terminal cargo. Show cargo version information. Fast operation that helps LLMs understand the available cargo capabilities."
    )]
    async fn version(
        &self,
        Parameters(req): Parameters<VersionRequest>,
    ) -> Result<CallToolResult, McpError> {
        use tokio::process::Command;

        let version_id = self.generate_operation_id();

        let mut cmd = Command::new("cargo");
        cmd.arg("version");

        // Add verbose flag if requested
        if req.verbose.unwrap_or(false) {
            cmd.arg("--verbose");
        }

        let output = cmd.output().await.map_err(|e| {
            McpError::internal_error(format!("Failed to execute cargo version: {}", e), None)
        })?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        let result_msg = if output.status.success() {
            format!(
                "📋 Version operation #{version_id} completed successfully.\nCargo version information:\n{stdout}"
            )
        } else {
            format!(
                "❌ Version operation #{version_id} failed.\nErrors: {stderr}\nOutput: {stdout}"
            )
        };

        Ok(CallToolResult::success(vec![Content::text(result_msg)]))
    }

    #[tool(
        description = "FETCH: Safer than terminal cargo. Fetch dependencies without building. Use enable_async_notifications=true for large dependency sets to multitask while downloading."
    )]
    async fn fetch(
        &self,
        Parameters(req): Parameters<FetchRequest>,
    ) -> Result<CallToolResult, McpError> {
        use tokio::process::Command;

        let fetch_id = self.generate_operation_id();

        let mut cmd = Command::new("cargo");
        cmd.arg("fetch");

        // Add target if specified
        if let Some(target) = &req.target {
            cmd.args(["--target", target]);
        }

        // Add features if specified
        if let Some(features) = &req.features {
            if !features.is_empty() {
                cmd.args(["--features", &features.join(",")]);
            }
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
            McpError::internal_error(format!("Failed to execute cargo fetch: {}", e), None)
        })?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        let working_dir_msg = format!(" in {}", &req.working_directory);

        let result_msg = if output.status.success() {
            format!(
                "📦 Fetch operation #{fetch_id} completed successfully{working_dir_msg}.\nDependencies fetched:\n{stdout}"
            )
        } else {
            format!(
                "❌ Fetch operation #{fetch_id} failed{working_dir_msg}.\nErrors: {stderr}\nOutput: {stdout}"
            )
        };

        Ok(CallToolResult::success(vec![Content::text(result_msg)]))
    }

    #[tool(
        description = "RUSTC: Safer than terminal cargo. Compile with custom rustc options. Use enable_async_notifications=true for complex builds to multitask while compiling."
    )]
    async fn rustc(
        &self,
        Parameters(req): Parameters<RustcRequest>,
    ) -> Result<CallToolResult, McpError> {
        use tokio::process::Command;

        let rustc_id = self.generate_operation_id();

        let mut cmd = Command::new("cargo");
        cmd.arg("rustc");

        // Add cargo-specific arguments first
        if let Some(cargo_args) = &req.cargo_args {
            cmd.args(cargo_args);
        }

        // Add rustc-specific arguments after --
        if let Some(rustc_args) = &req.rustc_args {
            if !rustc_args.is_empty() {
                cmd.arg("--");
                cmd.args(rustc_args);
            }
        }

        cmd.current_dir(&req.working_directory);

        let output = cmd.output().await.map_err(|e| {
            McpError::internal_error(format!("Failed to execute cargo rustc: {}", e), None)
        })?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        let working_dir_msg = format!(" in {}", &req.working_directory);

        let result_msg = if output.status.success() {
            format!(
                "🔧 Rustc operation #{rustc_id} completed successfully{working_dir_msg}.\nOutput: {stdout}"
            )
        } else {
            format!(
                "❌ Rustc operation #{rustc_id} failed{working_dir_msg}.\nErrors: {stderr}\nOutput: {stdout}"
            )
        };

        Ok(CallToolResult::success(vec![Content::text(result_msg)]))
    }

    #[tool(
        description = "METADATA: Safer than terminal cargo. Output JSON metadata about the project. Fast operation that provides LLMs with comprehensive project structure information."
    )]
    async fn metadata(
        &self,
        Parameters(req): Parameters<MetadataRequest>,
    ) -> Result<CallToolResult, McpError> {
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
        if let Some(features) = &req.features {
            if !features.is_empty() {
                cmd.args(["--features", &features.join(",")]);
            }
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
            McpError::internal_error(format!("Failed to execute cargo metadata: {}", e), None)
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
                "❌ Metadata operation #{metadata_id} failed{working_dir_msg}.\nErrors: {stderr}\nOutput: {stdout}"
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
            instructions: Some("Rust cargo operations with async support. For builds/tests >1s, use enable_async_notifications=true to multitask efficiently while operations run. Safer than terminal commands.".to_string()),
        }
    }

    async fn list_resources(
        &self,
        _request: Option<PaginatedRequestParam>,
        _: RequestContext<RoleServer>,
    ) -> Result<ListResourcesResult, McpError> {
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
    ) -> Result<ReadResourceResult, McpError> {
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
            _ => Err(McpError::resource_not_found(
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
    ) -> Result<ListPromptsResult, McpError> {
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
    ) -> Result<GetPromptResult, McpError> {
        match name.as_str() {
            "example_prompt" => {
                let message = arguments
                    .and_then(|json| json.get("message")?.as_str().map(|s| s.to_string()))
                    .ok_or_else(|| {
                        McpError::invalid_params("No message provided to example_prompt", None)
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
            _ => Err(McpError::invalid_params("prompt not found", None)),
        }
    }

    async fn list_resource_templates(
        &self,
        _request: Option<PaginatedRequestParam>,
        _: RequestContext<RoleServer>,
    ) -> Result<ListResourceTemplatesResult, McpError> {
        Ok(ListResourceTemplatesResult {
            next_cursor: None,
            resource_templates: Vec::new(),
        })
    }

    async fn initialize(
        &self,
        request: InitializeRequestParam,
        context: RequestContext<RoleServer>,
    ) -> Result<InitializeResult, McpError> {
        tracing::info!("=== INITIALIZE METHOD CALLED ===");
        tracing::info!("Initialize request: {:?}", request);
        tracing::info!("Request context: {:?}", context);

        if let Some(http_request_part) = context.extensions.get::<axum::http::request::Parts>() {
            let initialize_headers = &http_request_part.headers;
            let initialize_uri = &http_request_part.uri;
            tracing::info!(?initialize_headers, %initialize_uri, "initialize from http server");
        } else {
            tracing::info!("No HTTP request parts found - this is stdio transport");
        }

        // Generate and log availability report for LLM
        let availability_report = Self::generate_availability_report().await;
        tracing::info!("Cargo Component Availability:\n{}", availability_report);

        let mut result = self.get_info();

        // Add availability information to the instructions
        let enhanced_instructions = format!(
            "{}.\n\nAVAILABILITY REPORT:\n{}",
            result.instructions.unwrap_or_default(),
            availability_report
        );
        result.instructions = Some(enhanced_instructions);

        tracing::info!("Initialize result: {:?}", result);
        tracing::info!("=== INITIALIZE METHOD RETURNING ===");
        Ok(result)
    }
}

/// Async cargo operations with callback support
impl AsyncCargo {
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
        if let Some(features) = &req.features {
            if !features.is_empty() {
                cmd.arg("--features").arg(features.join(","));
            }
        }

        // Add optional flag
        if req.optional.unwrap_or(false) {
            cmd.arg("--optional");
        }

        // Execute command and collect full output
        let output = cmd
            .output()
            .await
            .map_err(|e| format!("Failed to execute cargo add: {}", e))?;

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
                "❌ Add operation failed{working_dir_msg}.\nDependency: {}\nError: {stderr}\nOutput: {stdout}",
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
            .map_err(|e| format!("Failed to execute cargo remove: {}", e))?;

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
                "❌ Remove operation failed{working_dir_msg}.\nDependency: {}\nError: {stderr}\nOutput: {stdout}",
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
                "❌ Build operation failed in {}.\nError: Failed to execute cargo build: {}",
                &req.working_directory, e
            )
        })?;

        let duration_ms = start_time.elapsed().as_millis() as u64;
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        let working_dir_msg = format!(" in {}", &req.working_directory);

        if output.status.success() {
            let success_msg =
                format!("✅ Build completed successfully{working_dir_msg}.\nOutput: {stdout}");

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
                format!("❌ Build failed{working_dir_msg}.\nError: {stderr}\nOutput: {stdout}");

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
            let error_msg = format!(
                "❌ Audit operation failed: cargo-audit is not installed. 
📦 Install with: cargo install cargo-audit
🔒 This tool scans for known security vulnerabilities in dependencies."
            );

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
            .map_err(|e| format!("Failed to execute cargo audit: {}", e))?;

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
                    "⚠️ Audit found security vulnerabilities{working_dir_msg}.\nVulnerabilities detected:\n{stdout}\nErrors: {stderr}"
                )
            } else {
                format!("❌ Audit failed{working_dir_msg}.\nErrors: {stderr}\nOutput: {stdout}")
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
