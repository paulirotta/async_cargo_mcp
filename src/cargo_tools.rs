use crate::callback_system::{CallbackSender, LoggingCallbackSender, ProgressUpdate, no_callback};
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

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct BuildRequest {
    pub working_directory: String,
    /// Enable async callback notifications for operation progress
    pub enable_async_notifications: Option<bool>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct RunRequest {
    pub working_directory: String,
    /// Enable async callback notifications for operation progress
    pub enable_async_notifications: Option<bool>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct TestRequest {
    pub working_directory: String,
    /// Enable async callback notifications for operation progress
    pub enable_async_notifications: Option<bool>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct CheckRequest {
    pub working_directory: String,
    /// Enable async callback notifications for operation progress
    pub enable_async_notifications: Option<bool>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct UpdateRequest {
    pub working_directory: String,
    /// Enable async callback notifications for operation progress
    pub enable_async_notifications: Option<bool>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct DocRequest {
    pub working_directory: String,
    /// Enable async callback notifications for operation progress
    pub enable_async_notifications: Option<bool>,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ClippyRequest {
    pub working_directory: String,
    /// Additional arguments to pass to clippy (e.g., ["--fix", "--allow-dirty"])
    pub args: Option<Vec<String>>,
    /// Enable async callback notifications for operation progress
    pub enable_async_notifications: Option<bool>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct NextestRequest {
    pub working_directory: String,
    /// Additional arguments to pass to nextest (e.g., ["--all-features"])
    pub args: Option<Vec<String>>,
    /// Enable async callback notifications for operation progress
    pub enable_async_notifications: Option<bool>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct CleanRequest {
    pub working_directory: String,
    /// Enable async callback notifications for operation progress
    pub enable_async_notifications: Option<bool>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct FixRequest {
    pub working_directory: String,
    /// Additional arguments to pass to fix (e.g., ["--allow-dirty"])
    pub args: Option<Vec<String>>,
    /// Enable async callback notifications for operation progress
    pub enable_async_notifications: Option<bool>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct SearchRequest {
    pub query: String,
    /// Limit the number of results
    pub limit: Option<u32>,
    /// Enable async callback notifications for operation progress
    pub enable_async_notifications: Option<bool>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
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
        report.push_str("✅ • build, test, run, check, doc, add, remove, update, clean, fix, search, bench, install\n\n");

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

    fn generate_operation_id(&self) -> u64 {
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
        (now.timestamp_millis() - midnight.timestamp_millis()) as u64
    }

    #[tool(
        description = "BUILD: Safer than terminal cargo. Use enable_async_notifications=true for builds >1s to multitask. Structured output + isolation."
    )]
    async fn build(
        &self,
        Parameters(req): Parameters<BuildRequest>,
    ) -> Result<CallToolResult, McpError> {
        let build_id = self.generate_operation_id();

        // Check if async notifications are enabled
        if req.enable_async_notifications.unwrap_or(false) {
            // Use the callback-enabled version for async notifications
            let callback: Box<dyn CallbackSender> = Box::new(LoggingCallbackSender::new(format!(
                "cargo_build_{}",
                build_id
            )));

            match self.build_with_callback(req, Some(callback)).await {
                Ok(result_msg) => Ok(CallToolResult::success(vec![Content::text(result_msg)])),
                Err(error_msg) => Ok(CallToolResult::success(vec![Content::text(error_msg)])),
            }
        } else {
            // Use direct execution for synchronous operation
            use tokio::process::Command;

            // TODO: Add asynchronous callback mechanism here for build progress updates
            // Implementation plan:
            // 1. Use tokio::process::Command::spawn() instead of output() to get a Child process
            // 2. Read stdout/stderr streams line by line using BufReader
            // 3. Send progress messages via MCP notifications or progress callbacks to the LLM
            // 4. Include compilation warnings, errors, and progress percentage if available
            // 5. Allow LLM to receive real-time feedback during long compilation processes

            let mut cmd = Command::new("cargo");
            cmd.arg("build");

            // Set working directory
            cmd.current_dir(&req.working_directory);

            let output = cmd.output().await.map_err(|e| {
                McpError::internal_error(format!("Failed to execute cargo build: {}", e), None)
            })?;

            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);

            let working_dir_msg = format!(" in {}", &req.working_directory);

            let result_msg = if output.status.success() {
                format!(
                    "✅ Build operation #{build_id} completed successfully{working_dir_msg}.\nOutput: {stdout}"
                )
            } else {
                format!(
                    "❌ Build operation #{build_id} failed{working_dir_msg}.\nErrors: {stderr}\nOutput: {stdout}"
                )
            };

            Ok(CallToolResult::success(vec![Content::text(result_msg)]))
        }
    }

    #[tool(
        description = "RUN: Safer than terminal cargo. Use enable_async_notifications=true for long apps to multitask. Structured output + isolation."
    )]
    async fn run(
        &self,
        Parameters(req): Parameters<RunRequest>,
    ) -> Result<CallToolResult, McpError> {
        use tokio::process::Command;

        let run_id = self.generate_operation_id();

        // TODO: Add asynchronous callback mechanism here for runtime output streaming
        // Implementation plan:
        // 1. Use tokio::process::Command::spawn() to start the process without blocking
        // 2. Stream stdout/stderr in real-time to provide live output to the LLM
        // 3. Handle long-running processes that might need user interaction
        // 4. Provide option to terminate running processes via MCP commands
        // 5. Support for interactive applications through bidirectional communication

        let mut cmd = Command::new("cargo");
        cmd.arg("run");

        // Set working directory
        cmd.current_dir(&req.working_directory);

        let output = cmd.output().await.map_err(|e| {
            McpError::internal_error(format!("Failed to execute cargo run: {}", e), None)
        })?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        let working_dir_msg = format!(" in {}", &req.working_directory);

        let result_msg = if output.status.success() {
            format!(
                "✅ Run operation #{run_id} completed successfully{working_dir_msg}.\nOutput: {stdout}"
            )
        } else {
            format!(
                "❌ Run operation #{run_id} failed{working_dir_msg}.\nErrors: {stderr}\nOutput: {stdout}"
            )
        };

        Ok(CallToolResult::success(vec![Content::text(result_msg)]))
    }

    #[tool(
        description = "TEST: Safer than terminal cargo. ALWAYS use enable_async_notifications=true for test suites to multitask. Real-time progress + isolation."
    )]
    async fn test(
        &self,
        Parameters(req): Parameters<TestRequest>,
    ) -> Result<CallToolResult, McpError> {
        use tokio::process::Command;

        let test_id = self.generate_operation_id();

        let mut cmd = Command::new("cargo");
        cmd.arg("test");

        // Set working directory
        cmd.current_dir(&req.working_directory);

        let output = cmd.output().await.map_err(|e| {
            McpError::internal_error(format!("Failed to execute cargo test: {}", e), None)
        })?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        let working_dir_msg = format!(" in {}", &req.working_directory);

        let result_msg = if output.status.success() {
            format!(
                "🧪 Test operation #{test_id} completed successfully{working_dir_msg}.\nOutput: {stdout}"
            )
        } else {
            format!(
                "❌ Test operation #{test_id} failed{working_dir_msg}.\nErrors: {stderr}\nOutput: {stdout}"
            )
        };

        Ok(CallToolResult::success(vec![Content::text(result_msg)]))
    }

    #[tool(
        description = "CHECK: Safer than terminal cargo. Fast validation - async optional for large projects. Quick compile check."
    )]
    async fn check(
        &self,
        Parameters(req): Parameters<CheckRequest>,
    ) -> Result<CallToolResult, McpError> {
        use tokio::process::Command;

        let check_id = self.generate_operation_id();

        // TODO: Add asynchronous callback mechanism here for check progress updates
        // Implementation plan:
        // 1. Stream compilation check results as they become available
        // 2. Send immediate warnings and errors to the LLM during checking
        // 3. Provide progress indicators for large projects
        // 4. Allow early termination on first error if requested
        // 5. Include suggestion hints from compiler alongside error messages

        let mut cmd = Command::new("cargo");
        cmd.arg("check");

        // Set working directory
        cmd.current_dir(&req.working_directory);

        let output = cmd.output().await.map_err(|e| {
            McpError::internal_error(format!("Failed to execute cargo check: {}", e), None)
        })?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        let working_dir_msg = format!(" in {}", &req.working_directory);

        let result_msg = if output.status.success() {
            format!(
                "✅ Check operation #{check_id} completed successfully{working_dir_msg}.\nOutput: {stdout}"
            )
        } else {
            format!(
                "❌ Check operation #{check_id} failed{working_dir_msg}.\nErrors: {stderr}\nOutput: {stdout}"
            )
        };

        Ok(CallToolResult::success(vec![Content::text(result_msg)]))
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
    ) -> Result<CallToolResult, McpError> {
        use tokio::process::Command;

        let update_id = self.generate_operation_id();

        // TODO: Add asynchronous callback mechanism here for dependency update progress
        // Implementation plan:
        // 1. Stream update progress and version changes to the LLM in real-time
        // 2. Show which dependencies are being updated and to what versions
        // 3. Provide warnings about breaking changes or compatibility issues
        // 4. Allow selective updates if the LLM requests specific package updates
        // This would allow streaming update progress and version changes to the LLM

        let mut cmd = Command::new("cargo");
        cmd.arg("update");

        // Set working directory
        cmd.current_dir(&req.working_directory);

        let output = cmd.output().await.map_err(|e| {
            McpError::internal_error(format!("Failed to execute cargo update: {}", e), None)
        })?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        let working_dir_msg = format!(" in {}", &req.working_directory);

        let result_msg = if output.status.success() {
            format!(
                "⬆️ Update operation #{update_id} completed successfully{working_dir_msg}.\nOutput: {stdout}"
            )
        } else {
            format!(
                "❌ Update operation #{update_id} failed{working_dir_msg}.\nErrors: {stderr}\nOutput: {stdout}"
            )
        };

        Ok(CallToolResult::success(vec![Content::text(result_msg)]))
    }

    #[tool(
        description = "DOC: Safer than terminal cargo. Use enable_async_notifications=true for large codebases to multitask. Creates LLM-friendly API reference."
    )]
    async fn doc(
        &self,
        Parameters(req): Parameters<DocRequest>,
    ) -> Result<CallToolResult, McpError> {
        use tokio::process::Command;

        let doc_id = self.generate_operation_id();

        // TODO: Add asynchronous callback mechanism here for documentation generation progress
        // Implementation plan:
        // 1. Stream documentation generation progress to the LLM in real-time
        // 2. Show which crates and modules are being documented
        // 3. Provide warnings about missing documentation or broken doc links
        // 4. Report the final location of generated documentation files
        // This would allow streaming doc generation progress and warnings to the LLM

        let mut cmd = Command::new("cargo");
        cmd.arg("doc").arg("--no-deps");

        // Set working directory
        cmd.current_dir(&req.working_directory);

        let output = cmd.output().await.map_err(|e| {
            McpError::internal_error(format!("Failed to execute cargo doc: {}", e), None)
        })?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        let working_dir_msg = format!(" in {}", &req.working_directory);

        let result_msg = if output.status.success() {
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

            format!(
                "📚 Documentation generation #{doc_id} completed successfully{working_dir_msg}.
Documentation generated at: {}
The generated documentation provides comprehensive API information that can be used by LLMs for more accurate and up-to-date project understanding.
💡 Tip: Use this documentation to get the latest API details, examples, and implementation notes that complement the source code.

Output: {stdout}",
                doc_path
            )
        } else {
            format!(
                "❌ Documentation generation #{doc_id} failed{working_dir_msg}.\nErrors: {stderr}\nOutput: {stdout}"
            )
        };

        Ok(CallToolResult::success(vec![Content::text(result_msg)]))
    }
    #[tool(
        description = "CLIPPY: Safer than terminal cargo. Supports --fix via args=['--fix','--allow-dirty']. Fast operation - async optional."
    )]
    async fn clippy(
        &self,
        Parameters(req): Parameters<ClippyRequest>,
    ) -> Result<CallToolResult, McpError> {
        use tokio::process::Command;

        let clippy_id = self.generate_operation_id();

        let mut cmd = Command::new("cargo");
        cmd.arg("clippy");

        // Add any additional arguments passed to clippy
        if let Some(args) = &req.args {
            cmd.args(args);
        }

        cmd.current_dir(&req.working_directory);

        let output = cmd.output().await.map_err(|e| {
            McpError::internal_error(format!("Failed to execute cargo clippy: {}", e), None)
        })?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        let working_dir_msg = format!(" in {}", &req.working_directory);

        let result_msg = if output.status.success() {
            format!(
                "🔍 Clippy operation #{clippy_id} passed with no warnings{working_dir_msg}.\nOutput: {stdout}",
            )
        } else {
            format!(
                "❌ Clippy operation #{clippy_id} failed{working_dir_msg}.\nErrors: {stderr}\nOutput: {stdout}",
            )
        };

        Ok(CallToolResult::success(vec![Content::text(result_msg)]))
    }

    #[tool(
        description = "NEXTEST: Safer than terminal cargo. Faster test runner. ALWAYS use enable_async_notifications=true for test suites to multitask. Real-time progress + isolation."
    )]
    async fn nextest(
        &self,
        Parameters(req): Parameters<NextestRequest>,
    ) -> Result<CallToolResult, McpError> {
        use tokio::process::Command;

        let nextest_id = self.generate_operation_id();

        // First check if nextest is available
        let nextest_check = Command::new("cargo")
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

        let mut cmd = Command::new("cargo");
        cmd.args(["nextest", "run"]);

        // Add any additional arguments
        if let Some(args) = &req.args {
            cmd.args(args);
        }

        cmd.current_dir(&req.working_directory);

        let output = cmd.output().await.map_err(|e| {
            McpError::internal_error(format!("Failed to execute cargo nextest: {}", e), None)
        })?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        let working_dir_msg = format!(" in {}", &req.working_directory);

        let result_msg = if output.status.success() {
            format!(
                "⚡ Nextest operation #{nextest_id} completed successfully{working_dir_msg}.\nOutput: {stdout}"
            )
        } else {
            format!(
                "❌ Nextest operation #{nextest_id} failed{working_dir_msg}.\nErrors: {stderr}\nOutput: {stdout}"
            )
        };

        Ok(CallToolResult::success(vec![Content::text(result_msg)]))
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
        let output = cmd
            .output()
            .await
            .map_err(|e| format!("Failed to execute cargo build: {}", e))?;

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
