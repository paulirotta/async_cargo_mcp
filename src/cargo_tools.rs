use crate::callback_system::{CallbackSender, ProgressUpdate, no_callback};
use crate::mcp_callback::mcp_callback;
use crate::operation_monitor::OperationMonitor;
use crate::shell_pool::{ShellCommand, ShellPoolConfig, ShellPoolManager};
use crate::terminal_output::TerminalOutput;
use crate::timestamp;
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
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;
use tokio::sync::{Mutex as AsyncMutex, RwLock as AsyncRwLock};

/// Merge stdout and stderr into a unified string for the `Output:` section while preserving
/// an `Error:` section for failures. Rules:
/// - If both empty -> placeholder message (provided by caller if desired)
/// - If stdout empty and stderr non-empty -> return stderr
/// - If stderr empty and stdout non-empty -> return stdout
/// - If both non-empty -> concatenate with a separating blank line
fn merge_outputs(stdout: &str, stderr: &str, empty_placeholder: &str) -> String {
    let s = stdout.trim();
    let e = stderr.trim();
    if s.is_empty() && e.is_empty() {
        return empty_placeholder.to_string();
    }
    if s.is_empty() {
        return e.to_string();
    }
    if e.is_empty() {
        return s.to_string();
    }
    format!("{s}\n\n{e}")
}

#[derive(Debug, Clone, serde::Deserialize, schemars::JsonSchema)]
pub struct DependencyRequest {
    pub name: String,
    pub version: Option<String>,
    pub features: Option<Vec<String>>,
    pub optional: Option<bool>,
    pub working_directory: String,
    /// Enable async callback notifications for operation progress
    pub enable_async_notification: Option<bool>,
}

#[derive(Debug, Clone, serde::Deserialize, schemars::JsonSchema)]
pub struct RemoveDependencyRequest {
    pub name: String,
    pub working_directory: String,
    /// Enable async callback notifications for operation progress
    pub enable_async_notification: Option<bool>,
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
    pub enable_async_notification: Option<bool>,
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
    pub enable_async_notification: Option<bool>,
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
    pub enable_async_notification: Option<bool>,
}

#[derive(Debug, Clone, serde::Deserialize, schemars::JsonSchema)]
pub struct CheckRequest {
    pub working_directory: String,
    /// Enable async callback notifications for operation progress
    pub enable_async_notification: Option<bool>,
}

#[derive(Debug, Clone, serde::Deserialize, schemars::JsonSchema)]
pub struct UpdateRequest {
    pub working_directory: String,
}

#[derive(Debug, Clone, serde::Deserialize, schemars::JsonSchema)]
pub struct DocRequest {
    pub working_directory: String,
    /// Enable async callback notifications for operation progress
    pub enable_async_notification: Option<bool>,
}
#[derive(Debug, Clone, serde::Deserialize, schemars::JsonSchema)]
pub struct ClippyRequest {
    pub working_directory: String,
    /// Additional arguments to pass to clippy (e.g., ["--fix", "--allow-dirty"])
    pub args: Option<Vec<String>>,
    /// Enable async callback notifications for operation progress
    pub enable_async_notification: Option<bool>,
}

#[derive(Debug, Clone, serde::Deserialize, schemars::JsonSchema)]
pub struct NextestRequest {
    pub working_directory: String,
    /// Additional arguments to pass to nextest (e.g., ["--all-features"])
    pub args: Option<Vec<String>>,
    /// Enable async callback notifications for operation progress
    pub enable_async_notification: Option<bool>,
}

#[derive(Debug, Clone, serde::Deserialize, schemars::JsonSchema)]
pub struct CleanRequest {
    pub working_directory: String,
    /// Enable async callback notifications for operation progress
    pub enable_async_notification: Option<bool>,
}

#[derive(Debug, Clone, serde::Deserialize, schemars::JsonSchema)]
pub struct FixRequest {
    pub working_directory: String,
    /// Additional arguments to pass to fix (e.g., ["--allow-dirty"])
    pub args: Option<Vec<String>>,
    /// Enable async callback notifications for operation progress
    pub enable_async_notification: Option<bool>,
}

#[derive(Debug, Clone, serde::Deserialize, schemars::JsonSchema)]
pub struct SearchRequest {
    pub query: String,
    /// Limit the number of results
    pub limit: Option<u32>,
    /// Enable async callback notifications for operation progress
    pub enable_async_notification: Option<bool>,
}

#[derive(Debug, Clone, serde::Deserialize, schemars::JsonSchema)]
pub struct BenchRequest {
    pub working_directory: String,
    /// Additional arguments to pass to bench
    pub args: Option<Vec<String>>,
    /// Enable async callback notifications for operation progress
    pub enable_async_notification: Option<bool>,
}

#[derive(Debug, Clone, serde::Deserialize, schemars::JsonSchema)]
pub struct InstallRequest {
    pub package: String,
    pub version: Option<String>,
    pub working_directory: String,
    /// Enable async callback notifications for operation progress
    pub enable_async_notification: Option<bool>,
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
    pub enable_async_notification: Option<bool>,
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
    pub enable_async_notification: Option<bool>,
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
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct VersionRequest {
    /// Enable verbose output showing more version details
    pub verbose: Option<bool>,
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
    pub enable_async_notification: Option<bool>,
}

#[derive(Debug, Clone, serde::Deserialize, schemars::JsonSchema)]
pub struct RustcRequest {
    pub working_directory: String,
    /// Additional arguments to pass to rustc
    pub rustc_args: Option<Vec<String>>,
    /// Additional arguments to pass to cargo rustc
    pub cargo_args: Option<Vec<String>>,
    /// Enable async callback notifications for operation progress
    pub enable_async_notification: Option<bool>,
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
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct WaitRequest {
    /// List of operation IDs to wait for concurrently. Must contain at least one operation ID.
    pub operation_ids: Vec<String>,
}

/// Request to start a deterministic long-running async sleep operation for testing timeouts and batching.
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct SleepRequest {
    /// Unique external operation ID to assign (if omitted, one is generated)
    pub operation_id: Option<String>,
    /// Duration in milliseconds to sleep (default 1500ms)
    pub duration_ms: Option<u64>,
    /// Optional working directory (not used, but kept for symmetry with other tools)
    pub working_directory: Option<String>,
    /// Whether to enable async notifications (always true; provided to keep interface consistent)
    pub enable_async_notification: Option<bool>,
}

/// Request to query the status of running operations
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct StatusRequest {
    /// Optional operation ID to query specific operation (if omitted, shows all active operations)
    pub operation_id: Option<String>,
    /// Optional working directory filter (if provided, only show operations in this directory)
    pub working_directory: Option<String>,
    /// Filter by operation state (if provided, only show operations matching this state)
    pub state_filter: Option<String>, // "active", "completed", "failed", etc.
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "UPPERCASE")]
pub enum CargoLockAction {
    /// Delete target/.cargo-lock then run cargo clean
    A,
    /// Only delete .cargo-lock but do not clean
    B,
    /// Do nothing
    C,
}

/// Fallback remediation request for stuck Cargo lock files
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct CargoLockRemediationRequest {
    /// Project working directory containing the Cargo project
    pub working_directory: String,
    /// One of: A = delete lock then cargo clean; B = only delete lock; C = no-op
    pub action: CargoLockAction,
}

// Mark remediation request as safe for elicitation (object schema)
rmcp::elicit_safe!(CargoLockRemediationRequest);

#[derive(Clone, Debug)]
pub struct AsyncCargo {
    tool_router: ToolRouter<AsyncCargo>,
    monitor: Arc<OperationMonitor>,
    shell_pool_manager: Arc<ShellPoolManager>,
    synchronous_mode: bool,
    /// Enable the wait tool (legacy mode for debugging and specific use cases)
    enable_wait: bool,
    // Per-working-directory concurrency guard to serialize lock-file remediation
    per_dir_mutex: Arc<AsyncRwLock<HashMap<String, Arc<AsyncMutex<()>>>>>,
    disabled_tools: std::collections::HashSet<String>,
}

impl Default for AsyncCargo {
    fn default() -> Self {
        use crate::operation_monitor::MonitorConfig;
        let monitor_config = MonitorConfig::default();
        let monitor = Arc::new(OperationMonitor::new(monitor_config));
        let shell_pool_config = ShellPoolConfig::default();
        let shell_pool_manager = Arc::new(ShellPoolManager::new(shell_pool_config));
        Self::new(monitor, shell_pool_manager)
    }
}

#[tool_router]
impl AsyncCargo {
    /// Get or create a per-working-directory async mutex to serialize operations like lock-file remediation.
    async fn get_dir_mutex(&self, dir: &str) -> Arc<AsyncMutex<()>> {
        // First try a read lock for fast path
        {
            let map = self.per_dir_mutex.read().await;
            if let Some(m) = map.get(dir) {
                return m.clone();
            }
        }
        // Upgrade: acquire write lock to insert if still absent
        let mut map = self.per_dir_mutex.write().await;
        map.entry(dir.to_string())
            .or_insert_with(|| Arc::new(AsyncMutex::new(())))
            .clone()
    }
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

    /// Start a deterministic async sleep operation that just waits for a specified duration.
    /// Useful for testing timeout behavior without relying on cargo command durations.
    /// Always runs asynchronously and returns an operation ID immediately.
    #[tool(
        description = "Start a deterministic async sleep (no cargo invoked). Returns an operation ID immediately; use wait with operation_ids to retrieve completion. Useful for timeout & batching tests. Always use async_cargo_mcp MCP tools; do not run cargo in a terminal. For operations >1s, set enable_async_notification=true and call mcp_async_cargo_m_wait with specific operation_ids to collect results."
    )]
    async fn sleep(
        &self,
        Parameters(req): Parameters<SleepRequest>,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, ErrorData> {
        self.ensure_enabled("sleep")?;
        let operation_id = req
            .operation_id
            .unwrap_or_else(|| self.generate_operation_id_for("sleep"));
        let duration_ms = req.duration_ms.unwrap_or(1500);
        let description = format!("sleep {}ms", duration_ms);
        self.register_async_operation(
            &operation_id,
            "sleep",
            &description,
            req.working_directory.clone(),
        )
        .await;

        let monitor = self.monitor.clone();
        let op_clone = operation_id.clone();
        tokio::spawn(async move {
            use tokio::time::{Duration, sleep};
            sleep(Duration::from_millis(duration_ms)).await;
            let _ = monitor
                .complete_operation(&op_clone, Ok(format!("Slept for {}ms", duration_ms)))
                .await;
        });

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Sleep operation started ({}ms) with operation ID {}. Use wait with operation_ids to retrieve the result.",
            duration_ms, operation_id
        ))]))
    }
    pub fn new(monitor: Arc<OperationMonitor>, shell_pool_manager: Arc<ShellPoolManager>) -> Self {
        Self {
            tool_router: Self::tool_router(),
            monitor,
            shell_pool_manager,
            synchronous_mode: false, // Default to async mode
            enable_wait: false,      // Default to disabled (push results automatically)
            per_dir_mutex: Arc::new(AsyncRwLock::new(HashMap::new())),
            disabled_tools: Default::default(),
        }
    }

    pub fn new_with_config(
        monitor: Arc<OperationMonitor>,
        shell_pool_manager: Arc<ShellPoolManager>,
        synchronous_mode: bool,
    ) -> Self {
        Self {
            tool_router: Self::tool_router(),
            monitor,
            shell_pool_manager,
            synchronous_mode,
            enable_wait: false, // Default to disabled
            per_dir_mutex: Arc::new(AsyncRwLock::new(HashMap::new())),
            disabled_tools: Default::default(),
        }
    }

    /// Create new instance with explicit disabled tools (names normalized to lowercase)
    pub fn new_with_disabled(
        monitor: Arc<OperationMonitor>,
        shell_pool_manager: Arc<ShellPoolManager>,
        synchronous_mode: bool,
        disabled: std::collections::HashSet<String>,
    ) -> Self {
        Self::new_with_config_and_disabled(
            monitor,
            shell_pool_manager,
            synchronous_mode,
            false,
            disabled,
        )
    }

    /// Create new instance with full configuration including wait tool enablement
    pub fn new_with_config_and_disabled(
        monitor: Arc<OperationMonitor>,
        shell_pool_manager: Arc<ShellPoolManager>,
        synchronous_mode: bool,
        enable_wait: bool,
        disabled: std::collections::HashSet<String>,
    ) -> Self {
        let disabled_tools = disabled
            .into_iter()
            .map(|s| s.to_ascii_lowercase())
            .collect();
        Self {
            tool_router: Self::tool_router(),
            monitor,
            shell_pool_manager,
            synchronous_mode,
            enable_wait,
            per_dir_mutex: Arc::new(AsyncRwLock::new(HashMap::new())),
            disabled_tools,
        }
    }

    fn is_tool_disabled(&self, name: &str) -> bool {
        self.disabled_tools.contains(&name.to_ascii_lowercase())
    }

    fn ensure_enabled(&self, name: &str) -> Result<(), ErrorData> {
        if self.is_tool_disabled(name) {
            // Provide a static message (required by invalid_params signature) and structured data with tool name
            let data = Some(json!({"tool": name}));
            return Err(ErrorData::invalid_params(
                "tool_disabled: requested tool disabled via --disable flag",
                data,
            ));
        }
        Ok(())
    }

    pub fn is_tool_disabled_for_tests(&self, name: &str) -> bool {
        self.is_tool_disabled(name)
    }

    pub fn ensure_enabled_for_tests(&self, name: &str) -> Result<(), ErrorData> {
        self.ensure_enabled(name)
    }

    /// Execute a cargo command using a pre-warmed shell from the pool, with graceful fallback to direct spawn
    async fn execute_cargo_command(
        &self,
        command: String,
        working_directory: Option<String>,
        operation_id: &str,
    ) -> Result<std::process::Output, Box<dyn std::error::Error + Send + Sync>> {
        use std::path::PathBuf;
        use tokio::process::Command;

        // Try shell pool first if we have a working directory
        if let Some(ref working_dir) = working_directory {
            let working_dir_path = PathBuf::from(working_dir);

            tracing::debug!(
                operation_id = operation_id,
                command = %command,
                working_dir = %working_dir,
                "Attempting to use shell pool for cargo command"
            );

            // Try to get a shell from the pool
            if let Some(mut shell) = self.shell_pool_manager.get_shell(&working_dir_path).await {
                // Create shell command for the pool
                let shell_command = ShellCommand {
                    id: uuid::Uuid::new_v4().to_string(),
                    command: vec!["bash".to_string(), "-c".to_string(), command.clone()],
                    working_dir: working_dir.clone(),
                    timeout_ms: 300_000, // 5 minute timeout
                };
                tracing::info!(
                    operation_id = operation_id,
                    "Sending command to shell pool shell_id={} cmd={}",
                    shell.id(),
                    command
                );
                let exec_result = shell.execute_command(shell_command).await;
                let shell_id = shell.id().to_string();
                match exec_result {
                    Ok(shell_response) => {
                        tracing::info!(
                            operation_id = operation_id,
                            shell_id = %shell_id,
                            exit_code = shell_response.exit_code,
                            stdout_len = shell_response.stdout.len(),
                            stderr_len = shell_response.stderr.len(),
                            duration_ms = shell_response.duration_ms,
                            "Shell pool execution successful"
                        );

                        // Convert ShellResponse to std::process::Output
                        use std::process::Output;

                        // Get ExitStatus by running a simple command
                        let exit_status = if shell_response.exit_code == 0 {
                            Command::new("true").status().await.unwrap()
                        } else {
                            Command::new("false").status().await.unwrap()
                        };

                        let output = Output {
                            status: exit_status,
                            stdout: shell_response.stdout.into_bytes(),
                            stderr: shell_response.stderr.into_bytes(),
                        };

                        // Return shell to pool before returning
                        self.shell_pool_manager.return_shell(shell).await;
                        return Ok(output);
                    }
                    Err(e) => {
                        tracing::warn!(
                            operation_id = operation_id,
                            shell_id = %shell_id,
                            error = %e,
                            "Shell pool execution failed, will fall back"
                        );
                        // Attempt to return shell if it's still healthy
                        if shell.is_healthy() {
                            self.shell_pool_manager.return_shell(shell).await;
                        }
                        // Fall through to direct spawn
                    }
                }
            } else {
                tracing::debug!(
                    operation_id = operation_id,
                    "No shell available from pool, using direct spawn"
                );
            }
        }

        // Fallback to direct spawn (original behavior)
        tracing::debug!(
            operation_id = operation_id,
            command = %command,
            "Using direct spawn for cargo command"
        );

        let mut cmd = if cfg!(target_os = "windows") {
            let mut cmd = Command::new("cmd");
            cmd.args(["/C", &command]);
            cmd
        } else {
            let mut cmd = Command::new("bash");
            cmd.args(["-c", &command]);
            cmd
        };

        if let Some(ref working_dir) = working_directory {
            cmd.current_dir(working_dir);
        }

        cmd.output()
            .await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
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
            .push_str("* Use 'nextest' for faster execution; use 'test' when you need more complete error output for failing tests\n");
        report.push_str("* Use 'clippy' for enhanced code quality checks if available\n");
        report.push_str(
            "* Use 'upgrade' for intelligent dependency updates if cargo-edit is available\n",
        );
        report.push_str(
            "* Use 'audit' for security vulnerability scanning if cargo-audit is available\n",
        );
        report.push_str(
            "* Enable async notifications (enable_async_notification=true) for long operations\n",
        );

        report
    }

    fn _create_resource_text(&self, uri: &str, name: &str) -> Resource {
        RawResource::new(uri, name.to_string()).no_annotation()
    }

    /// Generate a descriptive operation id including the command category.
    /// Examples: op_build_12, op_test_13, op_clippy_14
    fn generate_operation_id_for(&self, kind: &str) -> String {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let counter = COUNTER.fetch_add(1, Ordering::SeqCst);
        let sanitized: String = kind
            .to_ascii_lowercase()
            .chars()
            .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
            .collect();
        format!("op_{}_{}", sanitized, counter)
    }

    /// Generate a tool hint message for LLMs when async operations are running
    fn generate_tool_hint(&self, operation_id: &str, operation_type: &str) -> String {
        crate::tool_hints::preview(operation_id, operation_type)
    }

    /// Standardized placeholder for when a command produced no combined stdout/stderr.
    /// Use for success-path Output sections when both streams are empty.
    fn no_output_placeholder(command: &str) -> String {
        format!("(no {command} output captured)")
    }

    /// Public helper to preview the standardized tool hint content.
    pub fn tool_hint_preview(operation_id: &str, operation_type: &str) -> String {
        crate::tool_hints::preview(operation_id, operation_type)
    }

    /// Determine if an operation should run synchronously based on CLI flag and request parameter
    pub fn should_run_synchronously(&self, enable_async_notification: Option<bool>) -> bool {
        self.synchronous_mode || !enable_async_notification.unwrap_or(false)
    }

    /// Helper to handle synchronous operation results with terminal output
    fn handle_sync_result(
        operation_name: &str,
        command: &str,
        description: &str,
        result: Result<String, String>,
    ) -> Result<CallToolResult, ErrorData> {
        match result {
            Ok(result_msg) => {
                if TerminalOutput::should_display(&result_msg) {
                    TerminalOutput::display_result(
                        &format!("SYNC_{}", operation_name.to_uppercase()),
                        command,
                        description,
                        &result_msg,
                    );
                }
                Ok(CallToolResult::success(vec![Content::text(result_msg)]))
            }
            Err(error_msg) => {
                if TerminalOutput::should_display(&error_msg) {
                    TerminalOutput::display_result(
                        &format!("SYNC_{}_ERROR", operation_name.to_uppercase()),
                        command,
                        &format!("{} (failed)", description),
                        &error_msg,
                    );
                }
                Ok(CallToolResult::success(vec![Content::text(error_msg)]))
            }
        }
    }

    /// Create a comprehensive final result for automatic push notifications
    pub fn create_final_result_update(
        operation_id: &str,
        command: &str,
        description: &str,
        working_directory: &str,
        result: &Result<String, String>,
        duration_ms: u64,
    ) -> ProgressUpdate {
        use crate::callback_system::ProgressUpdate;

        let (success, full_output) = match result {
            Ok(output) => {
                // Create detailed success output similar to wait command
                let mut normalized_output = output.clone();
                if (normalized_output.trim_end().ends_with("Output:")
                    || normalized_output
                        .matches("Output:")
                        .last()
                        .map(|_| {
                            if let Some(pos) = normalized_output.rfind("Output:") {
                                normalized_output[pos + 7..].trim().is_empty()
                            } else {
                                false
                            }
                        })
                        .unwrap_or(false))
                    && let Some(pos) = normalized_output.rfind("Output:")
                {
                    let (head, _) = normalized_output.split_at(pos + 7);
                    normalized_output =
                        format!("{head} (no command stdout captured – command produced no stdout)");
                }
                (true, normalized_output)
            }
            Err(error) => (false, error.clone()),
        };

        ProgressUpdate::FinalResult {
            operation_id: operation_id.to_string(),
            command: command.to_string(),
            description: description.to_string(),
            working_directory: working_directory.to_string(),
            success,
            duration_ms,
            full_output,
        }
    }

    #[tool(
        description = "LEGACY TOOL - Wait for async cargo operations to complete. This tool is deprecated in favor of automatic result push via progress notifications. Enable with --enable-wait flag if needed for debugging or specific use cases. Operations are waited for concurrently and results returned as soon as all specified operations complete. Timeout is configurable via the --timeout CLI parameter (default: 300 seconds). Always use async_cargo_mcp MCP tools; do not run cargo in a terminal. For operations >1s, set enable_async_notification=true and call mcp_async_cargo_m_wait with specific operation_ids to collect results."
    )]
    async fn wait(
        &self,
        Parameters(req): Parameters<WaitRequest>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, ErrorData> {
        // Check if wait tool is enabled
        if !self.enable_wait {
            let hint_message = "The wait tool is disabled by default to encourage more efficient AI workflows.\n\n\
                               RECOMMENDED APPROACH:\n\
                               • Async operations automatically push results via $/progress notifications\n\
                               • Continue with other tasks instead of waiting\n\
                               • Use the 'status' tool for non-blocking operation queries\n\n\
                               To enable this legacy tool, restart the server with the --enable-wait flag.";

            return Ok(CallToolResult::success(vec![Content::text(hint_message)]));
        }

        // Get the timeout from the monitor's configuration instead of hardcoding it
        let timeout_duration = {
            // Access the monitor's config to get the default timeout
            // We need to add a method to get the timeout from the monitor
            self.get_monitor_timeout().await
        };

        // Validate that we have operation IDs to wait for
        if req.operation_ids.is_empty() {
            return Err(ErrorData::invalid_params(
                "operation_ids cannot be empty. Must specify at least one operation ID to wait for.",
                None,
            ));
        }

        // Record wait calls and check for early waits
        let mut early_wait_warnings = Vec::new();
        for operation_id in &req.operation_ids {
            if let Some((gap, efficiency)) = self.monitor.record_wait_call(operation_id).await
                && gap.as_secs() < 5
                && efficiency < 0.5
            {
                early_wait_warnings.push(format!(
                        "⚡ CONCURRENCY HINT: You waited for '{}' after only {:.1}s (efficiency: {:.0}%). \
                        Consider performing other tasks while operations run in the background.",
                        operation_id, gap.as_secs_f32(), efficiency * 100.0
                    ));
            }
        }

        // Wait for each ID concurrently and collect using join handles
        let monitor = self.monitor.clone();
        let handles: Vec<_> = req
            .operation_ids
            .into_iter()
            .map(|id| {
                let monitor = monitor.clone();
                tokio::spawn(async move { monitor.wait_for_operation(&id).await })
            })
            .collect();

        let start_wait = Instant::now();
        let wait_result = tokio::time::timeout(timeout_duration, async move {
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
                            first_wait_time: None,
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
                            first_wait_time: None,
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
        .await;

        // If timeout occurred, craft a remediation-first message instead of a bare error
        let wait_result = match wait_result {
            Ok(res) => res,
            Err(_) => {
                let timeout_seconds = timeout_duration.as_secs();
                let waited = start_wait.elapsed().as_secs();
                tracing::warn!(
                    timeout_seconds,
                    waited_seconds = waited,
                    "wait timed out; preparing remediation guidance"
                );

                // Gather unique working directories from active operations at timeout moment
                let active_ops = self.monitor.get_active_operations().await;
                let mut dirs: Vec<String> = active_ops
                    .into_iter()
                    .filter_map(|op| op.working_directory)
                    .collect();
                dirs.sort();
                dirs.dedup();

                // Build remediation guidance per directory where lock file exists
                let mut guidance_blocks: Vec<String> = Vec::new();
                for dir in dirs.iter() {
                    let lock_path = std::path::Path::new(dir).join("target").join(".cargo-lock");
                    if lock_path.exists() {
                        let full = lock_path.display().to_string();
                        let mut block = format!(
                            "⏰ Wait timed out after {waited}s (limit {timeout_seconds}s) for operations in {dir}.\nDetected Cargo internal lock file: {full}. This file can persist after a crash or interrupt and block new cargo commands."
                        );

                        // Elicitation path (if supported by client)
                        let mut elicitation_done = false;
                        if context.peer.supports_elicitation() {
                            // Prepare a prefilled request asking only for action (dir is known)
                            let prompt = format!(
                                "A Cargo internal lock file was found at: {full}.\nWhy this happens: crashes or interrupts can leave this lock behind, causing future cargo commands to block.\nChoose: (A) Delete lock then cargo clean, (B) Only delete lock, (C) Do nothing."
                            );
                            match context
                                .peer
                                .elicit::<CargoLockRemediationRequest>(prompt)
                                .await
                            {
                                Ok(Some(user_req)) => {
                                    // If user didn't include working_directory, fill it
                                    let wd = if user_req.working_directory.is_empty() {
                                        dir.clone()
                                    } else {
                                        user_req.working_directory.clone()
                                    };
                                    match self
                                        .perform_cargo_lock_remediation(&wd, user_req.action)
                                        .await
                                    {
                                        Ok(sum) => {
                                            block.push_str("\n\nRemediation applied:\n");
                                            block.push_str(&sum);
                                            elicitation_done = true;
                                        }
                                        Err(e) => {
                                            block.push_str(&format!(
                                                "\n\nAttempted remediation but failed: {e}"
                                            ));
                                        }
                                    }
                                }
                                Ok(None) => {
                                    // No content provided; fall back to guidance
                                }
                                Err(_e) => {
                                    // Peer error or decline; fall back to guidance
                                }
                            }
                        }

                        if elicitation_done {
                            block.push_str("\n\nRemediation applied:\n");
                            // already appended above
                        }

                        if !elicitation_done {
                            block.push_str(&format!(
                                "\n\nChoose an option to remediate (use the 'cargo_lock_remediation' tool):\n  - action: 'A' → delete {full} then run 'cargo clean'\n  - action: 'B' → only delete {full}\n  - action: 'C' → do nothing\nExample call: cargo_lock_remediation with working_directory='{}' and action='A'|'B'|'C'",
                                dir
                            ));
                        }

                        guidance_blocks.push(block);
                    }
                }

                let mut contents = Vec::new();
                if guidance_blocks.is_empty() {
                    contents.push(Content::text(format!(
                        "Wait timed out after {waited}s (limit {timeout_seconds}s). No target/.cargo-lock files detected for active operations. You may retry 'wait' or inspect running jobs."
                    )));
                } else {
                    for b in guidance_blocks {
                        contents.push(Content::text(b));
                    }
                }

                // Also display in terminal for visibility
                if !contents.is_empty() {
                    let joined: Vec<String> = contents.iter().map(|c| format!("{:?}", c)).collect();
                    TerminalOutput::display_wait_results(&joined);
                }

                return Ok(CallToolResult::success(contents));
            }
        };

        let results = wait_result;

        // Calculate duration reporting: find the earliest start time and compute duration
        let earliest_start_time = results
            .iter()
            .map(|op_info| op_info.start_time)
            .min()
            .unwrap_or_else(std::time::Instant::now);

        let longest_duration_seconds =
            timestamp::duration_since_as_rounded_seconds(earliest_start_time);

        let content: Vec<Content> = results.clone()
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

        // Add duration summary as the first content item
        let duration_summary = if longest_duration_seconds == 0 {
            "Wait completed. Operations finished quickly (< 1 second).".to_string()
        } else {
            format!(
                "Wait completed. Longest duration: {} seconds.",
                longest_duration_seconds
            )
        };

        let mut final_content = vec![Content::text(duration_summary)];

        // Add early wait warnings if any
        if !early_wait_warnings.is_empty() {
            let warning_text = early_wait_warnings.join("\n\n");
            final_content.push(Content::text(format!("\n{}", warning_text)));
        }

        final_content.extend(content.clone());

        // Display results in terminal for all completed operations
        let terminal_results: Vec<String> = results
            .iter()
            .filter_map(|op_info| match &op_info.state {
                crate::operation_monitor::OperationState::Completed => {
                    if let Some(Ok(output)) = &op_info.result {
                        if TerminalOutput::should_display(output) {
                            Some(format!(
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
                                output
                            ))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                }
                crate::operation_monitor::OperationState::Failed => {
                    if let Some(Err(error_output)) = &op_info.result {
                        if TerminalOutput::should_display(error_output) {
                            Some(format!(
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
                            ))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                }
                _ => None,
            })
            .collect();

        // Display terminal output if we have any results
        if !terminal_results.is_empty() {
            TerminalOutput::display_wait_results(&terminal_results);
        }

        Ok(CallToolResult::success(final_content))
    }

    #[tool(
        description = "Query the status of running operations without blocking. This is the recommended way to check operation progress instead of using the wait tool. Returns current state, runtime, and other details for operations."
    )]
    async fn status(
        &self,
        Parameters(req): Parameters<StatusRequest>,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut status_lines = Vec::new();

        if let Some(operation_id) = &req.operation_id {
            // Query specific operation
            if let Some(operation) = self.monitor.get_operation(operation_id).await {
                let status_line = self.format_operation_status(&operation);
                status_lines.push(status_line);
            } else {
                // Check completion history for completed operations
                let completed_ops = self.monitor.get_completed_operations().await;
                if let Some(completed_op) = completed_ops.iter().find(|op| op.id == *operation_id) {
                    let status_line = self.format_operation_status(completed_op);
                    status_lines.push(status_line);
                } else {
                    status_lines.push(format!(
                        "❓ Operation '{}' not found (may be very old and cleaned up)",
                        operation_id
                    ));
                }
            }
        } else {
            // Query all operations with optional filtering
            let all_active = self.monitor.get_active_operations().await;
            let all_completed = self.monitor.get_completed_operations().await;
            let mut all_operations = all_active;
            all_operations.extend(all_completed);

            // Apply filters
            let filtered_operations: Vec<_> = all_operations
                .into_iter()
                .filter(|op| {
                    // Filter by working directory if specified
                    if let Some(ref filter_dir) = req.working_directory {
                        if let Some(ref op_dir) = op.working_directory {
                            if !op_dir.contains(filter_dir) {
                                return false;
                            }
                        } else {
                            return false;
                        }
                    }

                    // Filter by state if specified
                    if let Some(ref state_filter) = req.state_filter {
                        let matches_filter = match state_filter.to_lowercase().as_str() {
                            "active" => op.is_active(),
                            "completed" => matches!(
                                op.state,
                                crate::operation_monitor::OperationState::Completed
                            ),
                            "failed" => {
                                matches!(op.state, crate::operation_monitor::OperationState::Failed)
                            }
                            "cancelled" => matches!(
                                op.state,
                                crate::operation_monitor::OperationState::Cancelled
                            ),
                            "timedout" => matches!(
                                op.state,
                                crate::operation_monitor::OperationState::TimedOut
                            ),
                            _ => true, // Unknown filter, include all
                        };
                        if !matches_filter {
                            return false;
                        }
                    }

                    true
                })
                .collect();

            if filtered_operations.is_empty() {
                status_lines.push("📋 No operations match the specified criteria".to_string());
            } else {
                status_lines.push(format!(
                    "📋 Found {} operations:",
                    filtered_operations.len()
                ));
                for operation in filtered_operations {
                    let status_line = self.format_operation_status(&operation);
                    status_lines.push(format!("  {}", status_line));
                }
            }
        }

        let status_text = status_lines.join("\n");
        Ok(CallToolResult::success(vec![Content::text(status_text)]))
    }

    /// Format a single operation's status for display
    fn format_operation_status(
        &self,
        operation: &crate::operation_monitor::OperationInfo,
    ) -> String {
        use crate::operation_monitor::OperationState;

        let state_emoji = match operation.state {
            OperationState::Pending => "⏳",
            OperationState::Running => "🏃",
            OperationState::Completed => "✅",
            OperationState::Failed => "❌",
            OperationState::Cancelled => "🚫",
            OperationState::TimedOut => "⏰",
        };

        let duration = operation.duration();
        let duration_str = if duration.as_secs() > 0 {
            format!("{:.1}s", duration.as_secs_f32())
        } else {
            format!("{}ms", duration.as_millis())
        };

        let working_dir = operation.working_directory.as_deref().unwrap_or("unknown");

        let concurrency_info = if let Some(gap) = operation.concurrency_gap() {
            let efficiency = operation.concurrency_efficiency();
            format!(
                " (wait gap: {:.1}s, efficiency: {:.0}%)",
                gap.as_secs_f32(),
                efficiency * 100.0
            )
        } else {
            String::new()
        };

        format!(
            "{} [{}] {} ({}) - {} in {}{}",
            state_emoji,
            operation.id,
            operation.command,
            operation.description,
            duration_str,
            working_dir,
            concurrency_info
        )
    }

    // Helper to perform remediation with safety: cancel, delete lock, optional clean
    async fn perform_cargo_lock_remediation(
        &self,
        working_directory: &str,
        action: CargoLockAction,
    ) -> Result<String, String> {
        use std::path::PathBuf;
        use tokio::fs;

        let dir = working_directory.to_string();
        let guard = self.get_dir_mutex(&dir).await;
        let _lock = guard.lock().await;

        let lock_path = PathBuf::from(&dir).join("target").join(".cargo-lock");
        let lock_path_str = lock_path.display().to_string();

        match action {
            CargoLockAction::C => {
                tracing::info!(%lock_path_str, "Remediation choice C (do nothing)");
                Ok(format!(
                    "No action taken. If issues persist, consider deleting {lock_path_str} and optionally running cargo clean."
                ))
            }
            CargoLockAction::A | CargoLockAction::B => {
                // Cancel active ops in this directory before deletion
                let cancelled = self.monitor.cancel_by_working_directory(&dir).await;
                tracing::warn!(directory=%dir, cancelled, action=?action, "Cancelling operations before deleting .cargo-lock");

                // Delete the lock file if present
                let mut deleted = false;
                match fs::remove_file(&lock_path).await {
                    Ok(_) => {
                        deleted = true;
                    }
                    Err(e) => {
                        if e.kind() == std::io::ErrorKind::NotFound { /* already gone */
                        } else {
                            tracing::error!(%lock_path_str, error=%e, "Failed deleting .cargo-lock");
                            return Err(format!("Failed to delete {lock_path_str}: {e}"));
                        }
                    }
                }

                let delete_note = if deleted {
                    format!("Deleted {lock_path_str}.")
                } else {
                    format!("{lock_path_str} did not exist.")
                };

                if let CargoLockAction::A = action {
                    // Run cargo clean
                    let clean_req = CleanRequest {
                        working_directory: dir.clone(),
                        enable_async_notification: Some(false),
                    };
                    match Self::clean_implementation(&clean_req).await {
                        Ok(clean_msg) => Ok(format!(
                            "{delete_note}\nPerformed cargo clean. Summary:\n{clean_msg}"
                        )),
                        Err(err) => Err(format!(
                            "{delete_note}\nAttempted cargo clean but it failed:\n{err}"
                        )),
                    }
                } else {
                    Ok(delete_note)
                }
            }
        }
    }

    #[tool(
        description = "Attempt remediation for a stale Cargo lock file. Options: A = delete target/.cargo-lock then cargo clean; B = only delete .cargo-lock; C = do nothing. Cancels active jobs for the directory before deletion. Always use async_cargo_mcp MCP tools; do not run cargo in a terminal."
    )]
    async fn cargo_lock_remediation(
        &self,
        Parameters(req): Parameters<CargoLockRemediationRequest>,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, ErrorData> {
        let dir = req.working_directory.clone();
        let action = req.action.clone();
        let summary = self
            .perform_cargo_lock_remediation(&dir, action)
            .await
            .map_err(|e| ErrorData::internal_error(e, None))?;

        let explanation = format!(
            "A Cargo internal lock file can persist at <dir>/target/.cargo-lock when previous commands crashed or were interrupted, causing new cargo invocations to block.\nDirectory: {dir}\nResult: {summary}"
        );
        Ok(CallToolResult::success(vec![Content::text(explanation)]))
    }

    #[tool(
        description = "CARGO BUILD: Faster than terminal cargo. Use enable_async_notification=true for builds >1s to multitask. Structured output with isolation. Always use async_cargo_mcp MCP tools; do not run cargo in a terminal. For operations >1s, set enable_async_notification=true and call mcp_async_cargo_m_wait with specific operation_ids to collect results."
    )]
    async fn build(
        &self,
        Parameters(req): Parameters<BuildRequest>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, ErrorData> {
        self.ensure_enabled("build")?;
        let build_id = self.generate_operation_id_for("build");

        // Check if async notifications are enabled and not in synchronous mode
        if self.should_run_synchronously(req.enable_async_notification) {
            // Synchronous operation for when async notifications are disabled or synchronous mode is enabled
            let result = self.build_implementation(&req, "sync_build").await;
            return Self::handle_sync_result(
                "build",
                "cargo build",
                "Synchronous build operation",
                result,
            );
        } else {
            // TRUE 2-STAGE ASYNC PATTERN:
            // 1. Send immediate response that operation has started
            // 2. Spawn background task to do actual work and send notifications

            let peer = context.peer.clone();
            let req_clone = req.clone();
            let build_id_clone = build_id.clone();
            let monitor = self.monitor.clone();
            let shell_pool_manager = self.shell_pool_manager.clone();

            // Register operation BEFORE spawning so wait() can find it immediately
            self.register_async_operation(
                &build_id,
                "cargo build",
                "Building project in the background",
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
                        description: "Building project in the background".to_string(),
                    })
                    .await;

                let started_at = Instant::now();
                // Do the actual build work
                let result = Self::build_implementation_static(
                    &req_clone,
                    &build_id_clone,
                    shell_pool_manager,
                )
                .await;

                // Store the result in the operation monitor for later retrieval by wait
                // This ensures the full output (stdout/stderr) is available to `wait`
                let _ = monitor
                    .complete_operation(&build_id_clone, result.clone())
                    .await;

                // Send completion notification with measured duration
                let duration_ms = started_at.elapsed().as_millis() as u64;

                // Send brief completion update for legacy support
                let completion_update = match result {
                    Ok(ref msg) => ProgressUpdate::Completed {
                        operation_id: build_id_clone.clone(),
                        message: msg.clone(),
                        duration_ms,
                    },
                    Err(ref err) => ProgressUpdate::Failed {
                        operation_id: build_id_clone.clone(),
                        error: err.clone(),
                        duration_ms,
                    },
                };

                if let Err(e) = callback.send_progress(completion_update).await {
                    tracing::error!("Failed to send build completion progress update: {e:?}");
                }

                // Send comprehensive final result for AI consumption
                let final_result_update = Self::create_final_result_update(
                    &build_id_clone,
                    "cargo build",
                    "Building project in the background",
                    &req_clone.working_directory,
                    &result,
                    duration_ms,
                );

                if let Err(e) = callback.send_progress(final_result_update).await {
                    tracing::error!("Failed to send build final result update: {e:?}");
                }
            });

            // Return immediate response to LLM - this is the "first stage"
            let tool_hint = self.generate_tool_hint(&build_id, "build");
            let timestamp = timestamp::format_current_time();
            Ok(CallToolResult::success(vec![Content::text(format!(
                "Build operation {build_id} started at {timestamp} in the background.{tool_hint}"
            ))]))
        }
    }

    /// Internal implementation of build logic using shell pool
    async fn build_implementation(
        &self,
        req: &BuildRequest,
        operation_id: &str,
    ) -> Result<String, String> {
        let mut cmd_args = vec!["cargo".to_string(), "build".to_string()];

        // Add package selection
        if req.workspace.unwrap_or(false) {
            cmd_args.push("--workspace".to_string());
        }

        if let Some(exclude) = &req.exclude {
            for package in exclude {
                cmd_args.extend(vec!["--exclude".to_string(), package.clone()]);
            }
        }

        // Add target selection
        if req.lib.unwrap_or(false) {
            cmd_args.push("--lib".to_string());
        }

        if req.bins.unwrap_or(false) {
            cmd_args.push("--bins".to_string());
        }

        if let Some(bin_name) = &req.bin_name {
            cmd_args.extend(vec!["--bin".to_string(), bin_name.clone()]);
        }

        if req.examples.unwrap_or(false) {
            cmd_args.push("--examples".to_string());
        }

        if let Some(example) = &req.example {
            cmd_args.extend(vec!["--example".to_string(), example.clone()]);
        }

        if req.tests.unwrap_or(false) {
            cmd_args.push("--tests".to_string());
        }

        if let Some(test) = &req.test {
            cmd_args.extend(vec!["--test".to_string(), test.clone()]);
        }

        if req.all_targets.unwrap_or(false) {
            cmd_args.push("--all-targets".to_string());
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
                cmd_args.extend(vec!["--features".to_string(), filtered.join(",")]);
            }
        }

        if req.all_features.unwrap_or(false) {
            cmd_args.push("--all-features".to_string());
        }

        if req.no_default_features.unwrap_or(false) {
            cmd_args.push("--no-default-features".to_string());
        }

        // Add compilation options
        if req.release.unwrap_or(false) {
            cmd_args.push("--release".to_string());
        }

        if let Some(profile) = &req.profile {
            cmd_args.extend(vec!["--profile".to_string(), profile.clone()]);
        }

        if let Some(jobs) = req.jobs {
            cmd_args.extend(vec!["--jobs".to_string(), jobs.to_string()]);
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
                cmd_args.extend(vec!["--target".to_string(), target.clone()]);
            } else {
                target_note = Some(format!(
                    "[info] Requested target '{target}' not installed; building with host target instead."
                ));
            }
        }

        if let Some(target_dir) = &req.target_dir {
            cmd_args.extend(vec!["--target-dir".to_string(), target_dir.clone()]);
        }

        // Add manifest options
        if let Some(manifest_path) = &req.manifest_path {
            cmd_args.extend(vec!["--manifest-path".to_string(), manifest_path.clone()]);
        }

        // Add additional arguments
        if let Some(args) = &req.args {
            for arg in args {
                cmd_args.push(arg.clone());
            }
        }

        // Build the command string and execute using shell pool
        let command = cmd_args.join(" ");
        tracing::info!(
            operation_id = operation_id,
            "Invoking execute_cargo_command for build in {}",
            req.working_directory
        );
        let output = match self
            .execute_cargo_command(command, Some(req.working_directory.clone()), operation_id)
            .await
        {
            Ok(o) => o,
            Err(e) => {
                tracing::error!(operation_id = operation_id, error=%e, "execute_cargo_command returned error for build");
                return Err(format!(
                    "- Build operation failed in {}.\nError: Failed to execute cargo build: {}",
                    &req.working_directory, e
                ));
            }
        };
        tracing::info!(operation_id = operation_id, status = %output.status, "execute_cargo_command returned output");

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        let working_dir_msg = format!(" in {}", &req.working_directory);
        let bin_msg = if let Some(bin_name) = &req.bin_name {
            format!(" (binary: {bin_name})")
        } else {
            String::new()
        };

        // For successful builds, treat only cargo lock-wait noise as empty so placeholder still appears.
        let lock_wait_prefix = "Blocking waiting for file lock";
        let stdout_trim = stdout.trim();
        let stderr_lines: Vec<&str> = stderr.lines().collect();
        let meaningful_stderr: Vec<&str> = stderr_lines
            .iter()
            .copied()
            .filter(|l| {
                let t = l.trim();
                !(t.is_empty() || t.starts_with(lock_wait_prefix))
            })
            .collect();
        let stdout_display = if output.status.success() {
            if stdout_trim.is_empty() && meaningful_stderr.is_empty() {
                Self::no_output_placeholder("build")
            } else if stdout_trim.is_empty() {
                stderr.to_string()
            } else if meaningful_stderr.is_empty() {
                stdout.to_string()
            } else {
                format!("{stdout}\n\n{stderr}")
            }
        } else {
            merge_outputs(&stdout, &stderr, &Self::no_output_placeholder("build"))
        };
        if output.status.success() {
            let mut msg = format!(
                "+ Build completed successfully{working_dir_msg}{bin_msg}.\nOutput: {stdout_display}"
            );
            if let Some(note) = target_note {
                msg.push('\n');
                msg.push_str(&note);
            }
            Ok(msg)
        } else {
            // Keep Error: section (tests rely on it) but also include merged content in Output
            Err(format!(
                "- Build failed{working_dir_msg}{bin_msg}.\nError: {stderr}\nOutput: {stdout_display}"
            ))
        }
    }

    /// Static version of build_implementation for use in async contexts
    async fn build_implementation_static(
        req: &BuildRequest,
        operation_id: &str,
        shell_pool_manager: Arc<ShellPoolManager>,
    ) -> Result<String, String> {
        let mut cmd_args = vec!["cargo".to_string(), "build".to_string()];

        // Add package selection
        if req.workspace.unwrap_or(false) {
            cmd_args.push("--workspace".to_string());
        }

        if let Some(exclude) = &req.exclude {
            for package in exclude {
                cmd_args.extend(vec!["--exclude".to_string(), package.clone()]);
            }
        }

        // Add target selection
        if req.lib.unwrap_or(false) {
            cmd_args.push("--lib".to_string());
        }

        if req.bins.unwrap_or(false) {
            cmd_args.push("--bins".to_string());
        }

        if let Some(bin_name) = &req.bin_name {
            cmd_args.extend(vec!["--bin".to_string(), bin_name.clone()]);
        }

        if req.examples.unwrap_or(false) {
            cmd_args.push("--examples".to_string());
        }

        if let Some(example) = &req.example {
            cmd_args.extend(vec!["--example".to_string(), example.clone()]);
        }

        if req.tests.unwrap_or(false) {
            cmd_args.push("--tests".to_string());
        }

        if let Some(test) = &req.test {
            cmd_args.extend(vec!["--test".to_string(), test.clone()]);
        }

        if req.all_targets.unwrap_or(false) {
            cmd_args.push("--all-targets".to_string());
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
                cmd_args.extend(vec!["--features".to_string(), filtered.join(",")]);
            }
        }

        if req.all_features.unwrap_or(false) {
            cmd_args.push("--all-features".to_string());
        }

        if req.no_default_features.unwrap_or(false) {
            cmd_args.push("--no-default-features".to_string());
        }

        // Add compilation options
        if req.release.unwrap_or(false) {
            cmd_args.push("--release".to_string());
        }

        if let Some(profile) = &req.profile {
            cmd_args.extend(vec!["--profile".to_string(), profile.clone()]);
        }

        if let Some(jobs) = req.jobs {
            cmd_args.extend(vec!["--jobs".to_string(), jobs.to_string()]);
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
                cmd_args.extend(vec!["--target".to_string(), target.clone()]);
            } else {
                target_note = Some(format!(
                    "[info] Requested target '{target}' not installed; building with host target instead."
                ));
            }
        }

        if let Some(target_dir) = &req.target_dir {
            cmd_args.extend(vec!["--target-dir".to_string(), target_dir.clone()]);
        }

        // Add manifest options
        if let Some(manifest_path) = &req.manifest_path {
            cmd_args.extend(vec!["--manifest-path".to_string(), manifest_path.clone()]);
        }

        // Add additional arguments
        if let Some(args) = &req.args {
            for arg in args {
                cmd_args.push(arg.clone());
            }
        }

        // Build the command string and execute using shell pool
        let command = cmd_args.join(" ");
        let output = Self::execute_cargo_command_static(
            command,
            Some(req.working_directory.clone()),
            operation_id,
            shell_pool_manager,
        )
        .await
        .map_err(|e| {
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

        // For successful builds, treat only cargo lock-wait noise as empty so placeholder still appears.
        let lock_wait_prefix = "Blocking waiting for file lock";
        let stdout_trim = stdout.trim();
        let stderr_lines: Vec<&str> = stderr.lines().collect();
        let meaningful_stderr: Vec<&str> = stderr_lines
            .iter()
            .copied()
            .filter(|l| {
                let t = l.trim();
                !(t.is_empty() || t.starts_with(lock_wait_prefix))
            })
            .collect();
        let stdout_display = if output.status.success() {
            if stdout_trim.is_empty() && meaningful_stderr.is_empty() {
                Self::no_output_placeholder("build")
            } else if stdout_trim.is_empty() {
                stderr.to_string()
            } else if meaningful_stderr.is_empty() {
                stdout.to_string()
            } else {
                format!("{stdout}\n\n{stderr}")
            }
        } else {
            merge_outputs(&stdout, &stderr, &Self::no_output_placeholder("build"))
        };
        if output.status.success() {
            let mut msg = format!(
                "+ Build completed successfully{working_dir_msg}{bin_msg}.\nOutput: {stdout_display}"
            );
            if let Some(note) = target_note {
                msg.push('\n');
                msg.push_str(&note);
            }
            Ok(msg)
        } else {
            // Keep Error: section (tests rely on it) but also include merged content in Output
            Err(format!(
                "- Build failed{working_dir_msg}{bin_msg}.\nError: {stderr}\nOutput: {stdout_display}"
            ))
        }
    }

    /// Static version of execute_cargo_command for use in async contexts
    async fn execute_cargo_command_static(
        command: String,
        working_directory: Option<String>,
        operation_id: &str,
        shell_pool_manager: Arc<ShellPoolManager>,
    ) -> Result<std::process::Output, Box<dyn std::error::Error + Send + Sync>> {
        use std::path::PathBuf;
        use tokio::process::Command;

        // Try shell pool first if we have a working directory
        if let Some(ref working_dir) = working_directory {
            let working_dir_path = PathBuf::from(working_dir);

            tracing::debug!(
                operation_id = operation_id,
                command = %command,
                working_dir = %working_dir,
                "Attempting to use shell pool for cargo command"
            );

            // Try to get a shell from the pool
            if let Some(mut shell) = shell_pool_manager.get_shell(&working_dir_path).await {
                // Create shell command for the pool
                let shell_command = ShellCommand {
                    id: uuid::Uuid::new_v4().to_string(),
                    command: vec!["bash".to_string(), "-c".to_string(), command.clone()],
                    working_dir: working_dir.clone(),
                    timeout_ms: 300_000, // 5 minute timeout
                };
                tracing::info!(
                    operation_id = operation_id,
                    "[static] Sending command to shell pool shell_id={} cmd={}",
                    shell.id(),
                    command
                );
                let exec_result = shell.execute_command(shell_command).await;
                let shell_id = shell.id().to_string();
                match exec_result {
                    Ok(shell_response) => {
                        tracing::info!(
                            operation_id = operation_id,
                            shell_id = %shell_id,
                            exit_code = shell_response.exit_code,
                            stdout_len = shell_response.stdout.len(),
                            stderr_len = shell_response.stderr.len(),
                            duration_ms = shell_response.duration_ms,
                            "Shell pool execution successful"
                        );

                        // Convert ShellResponse to std::process::Output
                        use std::process::Output;

                        // Get ExitStatus by running a simple command
                        let exit_status = if shell_response.exit_code == 0 {
                            Command::new("true").status().await.unwrap()
                        } else {
                            Command::new("false").status().await.unwrap()
                        };

                        let output = Output {
                            status: exit_status,
                            stdout: shell_response.stdout.into_bytes(),
                            stderr: shell_response.stderr.into_bytes(),
                        };

                        shell_pool_manager.return_shell(shell).await;
                        return Ok(output);
                    }
                    Err(e) => {
                        tracing::warn!(
                            operation_id = operation_id,
                            shell_id = %shell_id,
                            error = %e,
                            "Shell pool execution failed, will fall back"
                        );
                        if shell.is_healthy() {
                            shell_pool_manager.return_shell(shell).await;
                        }
                        // Fall through
                    }
                }
            } else {
                tracing::debug!(
                    operation_id = operation_id,
                    "No shell available from pool, using direct spawn"
                );
            }
        }

        // Fallback to direct spawn (original behavior)
        tracing::debug!(
            operation_id = operation_id,
            command = %command,
            "Using direct spawn for cargo command"
        );

        let mut cmd = if cfg!(target_os = "windows") {
            let mut cmd = Command::new("cmd");
            cmd.args(["/C", &command]);
            cmd
        } else {
            let mut cmd = Command::new("bash");
            cmd.args(["-c", &command]);
            cmd
        };

        if let Some(ref working_dir) = working_directory {
            cmd.current_dir(working_dir);
        }

        cmd.output()
            .await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
    }

    #[tool(
        description = "CARGO RUN: Faster than terminal cargo. Use enable_async_notification=true for long-running apps to multitask. Structured output with isolation. Always use async_cargo_mcp MCP tools; do not run cargo in a terminal. For operations >1s, set enable_async_notification=true and call mcp_async_cargo_m_wait with specific operation_ids to collect results."
    )]
    async fn run(
        &self,
        Parameters(req): Parameters<RunRequest>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, ErrorData> {
        let run_id = self.generate_operation_id_for("run");

        // Check if we should run synchronously or use async notifications
        if self.should_run_synchronously(req.enable_async_notification) {
            // Synchronous operation for when async notifications are disabled
            match Self::run_implementation(&req).await {
                Ok(result_msg) => Ok(CallToolResult::success(vec![Content::text(result_msg)])),
                Err(error_msg) => Ok(CallToolResult::success(vec![Content::text(error_msg)])),
            }
        } else {
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
                "Running application in the background",
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
                        description: "Running application in the background".to_string(),
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
            let timestamp = timestamp::format_current_time();
            Ok(CallToolResult::success(vec![Content::text(format!(
                "Run operation {run_id} started at {timestamp} in the background.{tool_hint}"
            ))]))
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
            tracing::debug!("Adding binary args: {binary_args:?}");
            cmd.arg("--");
            for arg in binary_args {
                cmd.arg(arg);
                tracing::debug!("Added binary arg: {arg}");
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
            // Merge stdout+stderr so compile lines (on stderr) always appear in Output section.
            let merged = merge_outputs(&stdout, &stderr, &Self::no_output_placeholder("run"));
            Ok(format!(
                "+ Run operation completed successfully{working_dir_msg}{bin_msg}{args_msg}.\nOutput: {merged}"
            ))
        } else {
            Err(format!(
                "- Run operation failed{working_dir_msg}{bin_msg}{args_msg}.\nErrors: {stderr}\nOutput: {stdout}"
            ))
        }
    }

    #[tool(
        description = "CARGO TEST: Faster than terminal cargo. Provides complete error output but runs slower than nextest. Use when you need detailed failure information. ALWAYS use enable_async_notification=true for test suites to multitask. Always use async_cargo_mcp MCP tools; do not run cargo in a terminal. For operations >1s, set enable_async_notification=true and call mcp_async_cargo_m_wait with specific operation_ids to collect results."
    )]
    async fn test(
        &self,
        Parameters(req): Parameters<TestRequest>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, ErrorData> {
        let test_id = self.generate_operation_id_for("test");

        // Check if async notifications are enabled and not in synchronous mode
        if self.should_run_synchronously(req.enable_async_notification) {
            // Synchronous operation
            match Self::test_implementation(&req).await {
                Ok(result_msg) => Ok(CallToolResult::success(vec![Content::text(result_msg)])),
                Err(error_msg) => Ok(CallToolResult::success(vec![Content::text(error_msg)])),
            }
        } else {
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
                "Running test suite in the background",
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
                        description: "Running test suite in the background".to_string(),
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
            let timestamp = timestamp::format_current_time();
            Ok(CallToolResult::success(vec![Content::text(format!(
                "Test operation {test_id} started at {timestamp} in the background.{tool_hint}"
            ))]))
        }
    }

    /// Internal implementation of test logic
    pub async fn test_implementation(req: &TestRequest) -> Result<String, String> {
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
            let merged = merge_outputs(&stdout, &stderr, &Self::no_output_placeholder("test"));
            Ok(format!(
                "Test operation #{test_id} completed successfully{working_dir_msg}{test_filter_msg}.\nOutput: {merged}"
            ))
        } else {
            Err(format!(
                "- Test operation #{test_id} failed{working_dir_msg}{test_filter_msg}.\nErrors: {stderr}\nOutput: {stdout}"
            ))
        }
    }

    #[tool(
        description = "CARGO CHECK: Faster than terminal cargo. Fast validation - async optional for large projects. Quick compile check. Always use async_cargo_mcp MCP tools; do not run cargo in a terminal. For operations >1s, set enable_async_notification=true and call mcp_async_cargo_m_wait with specific operation_ids to collect results."
    )]
    async fn check(
        &self,
        Parameters(req): Parameters<CheckRequest>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, ErrorData> {
        let check_id = self.generate_operation_id_for("check");

        // Check if we should run synchronously or use async notifications
        if self.should_run_synchronously(req.enable_async_notification) {
            // Synchronous operation for when async notifications are disabled
            let result = Self::check_implementation(&req).await;
            return Self::handle_sync_result(
                "check",
                "cargo check",
                "Synchronous check operation",
                result,
            );
        } else {
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
                "Checking project in the background",
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
                        description: "Checking project in the background".to_string(),
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
            let timestamp = timestamp::format_current_time();
            Ok(CallToolResult::success(vec![Content::text(format!(
                "+ Check operation {check_id} started at {timestamp} in the background.{tool_hint}"
            ))]))
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
            let merged = merge_outputs(&stdout, &stderr, &Self::no_output_placeholder("check"));
            Ok(format!(
                "+ Check operation completed successfully{working_dir_msg}.\nOutput: {merged}"
            ))
        } else {
            Err(format!(
                "- Check operation failed{working_dir_msg}.\nErrors: {stderr}\nOutput: {stdout}"
            ))
        }
    }

    #[tool(
        description = "CARGO ADD: Faster than terminal cargo. Synchronous operation for Cargo.toml modifications. Handles version conflicts. Always use async_cargo_mcp MCP tools; do not run cargo in a terminal. For operations >1s, set enable_async_notification=true and call mcp_async_cargo_m_wait with specific operation_ids to collect results."
    )]
    async fn add(
        &self,
        Parameters(req): Parameters<DependencyRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let add_id = self.generate_operation_id_for("add");

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
        description = "CARGO REMOVE: Faster than terminal cargo. Synchronous operation for Cargo.toml modifications. Prevents Cargo.toml corruption. Always use async_cargo_mcp MCP tools; do not run cargo in a terminal. For operations >1s, set enable_async_notification=true and call mcp_async_cargo_m_wait with specific operation_ids to collect results."
    )]
    async fn remove(
        &self,
        Parameters(req): Parameters<RemoveDependencyRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let remove_id = self.generate_operation_id_for("remove");

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

        let merged = merge_outputs(&stdout, &stderr, "(no remove output captured)");
        let result_msg = if output.status.success() {
            format!(
                "➖ Remove operation #{remove_id} completed successfully{working_dir_msg}.\nRemoved dependency: {}\nOutput: {merged}",
                req.name
            )
        } else {
            format!(
                "- Remove operation #{remove_id} failed{working_dir_msg}.\nDependency: {}\nErrors: {stderr}\nOutput: {merged}",
                req.name
            )
        };

        Ok(CallToolResult::success(vec![Content::text(result_msg)]))
    }

    #[tool(
        description = "CARGO UPDATE: Faster than terminal cargo. Synchronous operation - returns results immediately once cargo lock is acquired. Updates dependencies to latest compatible versions. Always use async_cargo_mcp MCP tools; do not run cargo in a terminal."
    )]
    async fn update(
        &self,
        Parameters(req): Parameters<UpdateRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        // Always use synchronous execution for dependency updates
        match Self::update_implementation(&req).await {
            Ok(result_msg) => Ok(CallToolResult::success(vec![Content::text(result_msg)])),
            Err(error_msg) => Ok(CallToolResult::success(vec![Content::text(error_msg)])),
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
        let merged = merge_outputs(&stdout, &stderr, "(no update output captured)");

        if output.status.success() {
            Ok(format!(
                "Update operation completed successfully{working_dir_msg}.\nOutput: {merged}"
            ))
        } else {
            Err(format!(
                "- Update operation failed{working_dir_msg}.\nErrors: {stderr}\nOutput: {merged}"
            ))
        }
    }

    #[tool(
        description = "CARGO DOC: Faster than terminal cargo. Use enable_async_notification=true for large codebases to multitask. Creates LLM-friendly API reference. Always use async_cargo_mcp MCP tools; do not run cargo in a terminal. For operations >1s, set enable_async_notification=true and call mcp_async_cargo_m_wait with specific operation_ids to collect results."
    )]
    async fn doc(
        &self,
        Parameters(req): Parameters<DocRequest>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, ErrorData> {
        let doc_id = self.generate_operation_id_for("doc");

        // Check if we should run synchronously or use async notifications
        if self.should_run_synchronously(req.enable_async_notification) {
            // Synchronous operation for when async notifications are disabled
            match Self::doc_implementation(&req).await {
                Ok(result_msg) => Ok(CallToolResult::success(vec![Content::text(result_msg)])),
                Err(error_msg) => Ok(CallToolResult::success(vec![Content::text(error_msg)])),
            }
        } else {
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
                "Generating documentation in the background",
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
                        description: "Generating documentation in the background".to_string(),
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
            let timestamp = timestamp::format_current_time();
            Ok(CallToolResult::success(vec![Content::text(format!(
                "📚 Documentation generation {doc_id} started at {timestamp} in the background.{tool_hint}"
            ))]))
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
                let cargo_toml_path = format!("{}/Cargo.toml", &req.working_directory);
                std::fs::read_to_string(&cargo_toml_path)
                    .ok()
                    .and_then(|content| {
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

            let merged = merge_outputs(&stdout, &stderr, &Self::no_output_placeholder("doc"));
            Ok(format!(
                "📚 Documentation generation completed successfully{working_dir_msg}.\nDocumentation generated at: {doc_path}\nThe generated documentation provides comprehensive API information that can be used by LLMs for more accurate and up-to-date project understanding.\n💡 Tip: Use this documentation to get the latest API details, examples, and implementation notes that complement the source code.\n\nOutput: {merged}"
            ))
        } else {
            Err(format!(
                "- Documentation generation failed{working_dir_msg}.\nErrors: {stderr}\nOutput: {stdout}"
            ))
        }
    }
    #[tool(
        description = "CARGO CLIPPY: Faster than terminal cargo. Supports --fix via args=['--fix','--allow-dirty']. Fast operation - async optional. Always use async_cargo_mcp MCP tools; do not run cargo in a terminal. For operations >1s, set enable_async_notification=true and call mcp_async_cargo_m_wait with specific operation_ids to collect results."
    )]
    async fn clippy(
        &self,
        Parameters(req): Parameters<ClippyRequest>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, ErrorData> {
        let clippy_id = self.generate_operation_id_for("clippy");

        // Check if we should run synchronously or use async notifications
        if self.should_run_synchronously(req.enable_async_notification) {
            // Synchronous operation for when async notifications are disabled
            match Self::clippy_implementation(&req).await {
                Ok(result_msg) => Ok(CallToolResult::success(vec![Content::text(result_msg)])),
                Err(error_msg) => Ok(CallToolResult::success(vec![Content::text(error_msg)])),
            }
        } else {
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
                "Running linter in the background",
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
                        description: "Running linter in the background".to_string(),
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
            let timestamp = timestamp::format_current_time();
            Ok(CallToolResult::success(vec![Content::text(format!(
                "Clippy operation {clippy_id} started at {timestamp} in the background.{tool_hint}"
            ))]))
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
            let merged = merge_outputs(&stdout, &stderr, &Self::no_output_placeholder("clippy"));
            Ok(format!(
                "Clippy operation passed with no warnings{working_dir_msg}.\nOutput: {merged}",
            ))
        } else {
            // Even on failure, ensure stderr also visible in Output (besides Errors) for parity with other commands
            let merged = merge_outputs(&stdout, &stderr, &Self::no_output_placeholder("clippy"));
            Err(format!(
                "- Clippy operation failed{working_dir_msg}.\nErrors: {stderr}\nOutput: {merged}",
            ))
        }
    }

    #[tool(
        description = "CARGO NEXTEST: Faster than terminal cargo. Faster test runner - preferred for most testing. Use 'test' only when you need more complete error output for failing tests. ALWAYS use enable_async_notification=true for test suites to multitask. Always use async_cargo_mcp MCP tools; do not run cargo in a terminal. For operations >1s, set enable_async_notification=true and call mcp_async_cargo_m_wait with specific operation_ids to collect results."
    )]
    async fn nextest(
        &self,
        Parameters(req): Parameters<NextestRequest>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, ErrorData> {
        let nextest_id = self.generate_operation_id_for("nextest");

        // First check if nextest is available
        let nextest_check = tokio::process::Command::new("cargo")
            .args(["nextest", "--version"])
            .output()
            .await;

        if nextest_check.is_err() || !nextest_check.unwrap().status.success() {
            return Ok(CallToolResult::success(vec![Content::text(format!(
                r#"- Nextest operation #{nextest_id} failed: cargo-nextest is not installed. 
📦 Install with: cargo install cargo-nextest
🔄 Falling back to regular cargo test is recommended."#
            ))]));
        }

        // Check if we should run synchronously or use async notifications
        if self.should_run_synchronously(req.enable_async_notification) {
            // Synchronous operation for when async notifications are disabled
            match Self::nextest_implementation(&req).await {
                Ok(result_msg) => Ok(CallToolResult::success(vec![Content::text(result_msg)])),
                Err(error_msg) => Ok(CallToolResult::success(vec![Content::text(error_msg)])),
            }
        } else {
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
                "Running fast test suite in the background",
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
                        description: "Running fast test suite in the background".to_string(),
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
            let timestamp = timestamp::format_current_time();
            Ok(CallToolResult::success(vec![Content::text(format!(
                "Nextest operation {nextest_id} started at {timestamp} in the background.{tool_hint}"
            ))]))
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
            // Many nextest summaries are emitted to stderr (progress + final report). If stdout is empty but stderr has content, treat stderr as primary.
            let combined = if stdout.trim().is_empty() && !stderr.trim().is_empty() {
                stderr.to_string()
            } else if !stdout.trim().is_empty() && !stderr.trim().is_empty() {
                format!("{stdout}\n{stderr}")
            } else {
                stdout.to_string()
            };
            let final_output = if combined.trim().is_empty() {
                Self::no_output_placeholder("nextest")
            } else {
                combined
            };
            Ok(format!(
                "Nextest operation #{nextest_id} completed successfully{working_dir_msg}.\nOutput: {final_output}"
            ))
        } else {
            // Failure: include full stderr + stdout (stderr first for clarity)
            let mut err_block = String::new();
            if !stderr.trim().is_empty() {
                err_block.push_str(&stderr);
            }
            if !stdout.trim().is_empty() {
                if !err_block.is_empty() {
                    err_block.push_str("\n--- stdout ---\n");
                }
                err_block.push_str(&stdout);
            }
            Err(format!(
                "- Nextest operation #{nextest_id} failed{working_dir_msg}.\nErrors: {err_block}"
            ))
        }
    }

    #[tool(
        description = "CARGO CLEAN: Faster than terminal cargo. Fast operation - async not needed. Frees disk space. Always use async_cargo_mcp MCP tools; do not run cargo in a terminal. For operations >1s, set enable_async_notification=true and call mcp_async_cargo_m_wait with specific operation_ids to collect results."
    )]
    async fn clean(
        &self,
        Parameters(req): Parameters<CleanRequest>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, ErrorData> {
        let clean_id = self.generate_operation_id_for("clean");

        // Check if we should run synchronously or use async notifications
        if self.should_run_synchronously(req.enable_async_notification) {
            // Synchronous operation for when async notifications are disabled
            match Self::clean_implementation(&req).await {
                Ok(result_msg) => Ok(CallToolResult::success(vec![Content::text(result_msg)])),
                Err(error_msg) => Ok(CallToolResult::success(vec![Content::text(error_msg)])),
            }
        } else {
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
                "Cleaning build artifacts in the background",
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
                        description: "Cleaning build artifacts in the background".to_string(),
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
            let timestamp = timestamp::format_current_time();
            Ok(CallToolResult::success(vec![Content::text(format!(
                "Clean operation {clean_id} started at {timestamp} in the background.{tool_hint}"
            ))]))
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
            let merged = merge_outputs(&stdout, &stderr, &Self::no_output_placeholder("clean"));
            Ok(format!(
                "Clean operation completed successfully{working_dir_msg}.\nOutput: {merged}"
            ))
        } else {
            let merged = merge_outputs(&stdout, &stderr, &Self::no_output_placeholder("clean"));
            Err(format!(
                "- Clean operation failed{working_dir_msg}.\nErrors: {stderr}\nOutput: {merged}"
            ))
        }
    }

    #[tool(
        description = "CARGO FIX: Faster than terminal cargo. Automatically fix compiler warnings. Supports --allow-dirty via args. Use async for large codebases. Always use async_cargo_mcp MCP tools; do not run cargo in a terminal. For operations >1s, set enable_async_notification=true and call mcp_async_cargo_m_wait with specific operation_ids to collect results."
    )]
    async fn fix(
        &self,
        Parameters(req): Parameters<FixRequest>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, ErrorData> {
        let fix_id = self.generate_operation_id_for("fix");

        // Check if we should run synchronously or use async notifications
        if self.should_run_synchronously(req.enable_async_notification) {
            // Synchronous operation for when async notifications are disabled
            match Self::fix_implementation(&req).await {
                Ok(result_msg) => Ok(CallToolResult::success(vec![Content::text(result_msg)])),
                Err(error_msg) => Ok(CallToolResult::success(vec![Content::text(error_msg)])),
            }
        } else {
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
                "Fixing compiler warnings in the background",
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
                        description: "Fixing compiler warnings in the background".to_string(),
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
            let timestamp = timestamp::format_current_time();
            Ok(CallToolResult::success(vec![Content::text(format!(
                "Fix operation {fix_id} started at {timestamp} in the background.{tool_hint}"
            ))]))
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
            // Provide safe defaults to allow running in temp test project without VCS
            cmd.arg("--allow-dirty").arg("--allow-no-vcs");
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
            let merged = merge_outputs(&stdout, &stderr, &Self::no_output_placeholder("fix"));
            Ok(format!(
                "Fix operation completed successfully{working_dir_msg}.\nOutput: {merged}"
            ))
        } else {
            let merged = merge_outputs(&stdout, &stderr, &Self::no_output_placeholder("fix"));
            Err(format!(
                "- Fix operation failed{working_dir_msg}.\nErrors: {stderr}\nOutput: {merged}"
            ))
        }
    }

    #[tool(
        description = "CARGO SEARCH: Faster than terminal cargo. Search for crates on crates.io. Fast operation - async not needed unless searching many terms. Always use async_cargo_mcp MCP tools; do not run cargo in a terminal. For operations >1s, set enable_async_notification=true and call mcp_async_cargo_m_wait with specific operation_ids to collect results."
    )]
    async fn search(
        &self,
        Parameters(req): Parameters<SearchRequest>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, ErrorData> {
        let search_id = self.generate_operation_id_for("search");

        // Check if we should run synchronously or use async notifications
        if self.should_run_synchronously(req.enable_async_notification) {
            // Synchronous operation for when async notifications are disabled
            match Self::search_implementation(&req).await {
                Ok(result_msg) => Ok(CallToolResult::success(vec![Content::text(result_msg)])),
                Err(error_msg) => Ok(CallToolResult::success(vec![Content::text(error_msg)])),
            }
        } else {
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
                &format!("Searching crates.io for '{}' in the background", req.query),
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
                            "Searching crates.io for '{}' in the background",
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
            let timestamp = timestamp::format_current_time();
            Ok(CallToolResult::success(vec![Content::text(format!(
                "Search operation {} started at {} in the background. Searching crates.io for '{}'.{}",
                search_id, timestamp, req.query, tool_hint
            ))]))
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
        let merged = merge_outputs(&stdout, &stderr, &Self::no_output_placeholder("search"));

        if output.status.success() {
            Ok(format!(
                "Search operation completed successfully.\nQuery: {}\nResults:\n{merged}",
                req.query
            ))
        } else {
            Err(format!(
                "- Search operation failed.\nQuery: {}\nErrors: {stderr}\nOutput: {merged}",
                req.query
            ))
        }
    }

    #[tool(
        description = "CARGO BENCH: Faster than terminal cargo. ALWAYS use enable_async_notification=true for benchmark suites to multitask. Performance testing. Always use async_cargo_mcp MCP tools; do not run cargo in a terminal. For operations >1s, set enable_async_notification=true and call mcp_async_cargo_m_wait with specific operation_ids to collect results."
    )]
    async fn bench(
        &self,
        Parameters(req): Parameters<BenchRequest>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, ErrorData> {
        let bench_id = self.generate_operation_id_for("bench");

        // Check if we should run synchronously or use async notifications
        if self.should_run_synchronously(req.enable_async_notification) {
            // Synchronous operation for when async notifications are disabled
            match Self::bench_implementation(&req).await {
                Ok(result_msg) => Ok(CallToolResult::success(vec![Content::text(result_msg)])),
                Err(error_msg) => Ok(CallToolResult::success(vec![Content::text(error_msg)])),
            }
        } else {
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
                "Running benchmarks in the background",
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
                        description: "Running benchmarks in the background".to_string(),
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
            let timestamp = timestamp::format_current_time();
            Ok(CallToolResult::success(vec![Content::text(format!(
                "Benchmark operation {bench_id} started at {timestamp} in the background.{tool_hint}"
            ))]))
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
            let merged = merge_outputs(&stdout, &stderr, &Self::no_output_placeholder("bench"));
            Ok(format!(
                "🏃‍♂️ Benchmark operation completed successfully{working_dir_msg}.\nOutput: {merged}"
            ))
        } else {
            let merged = merge_outputs(&stdout, &stderr, &Self::no_output_placeholder("bench"));
            Err(format!(
                "- Benchmark operation failed{working_dir_msg}.\nErrors: {stderr}\nOutput: {merged}"
            ))
        }
    }

    #[tool(
        description = "CARGO INSTALL: Faster than terminal cargo. Use enable_async_notification=true for large packages to multitask. Global tool installation. Always use async_cargo_mcp MCP tools; do not run cargo in a terminal. For operations >1s, set enable_async_notification=true and call mcp_async_cargo_m_wait with specific operation_ids to collect results."
    )]
    async fn install(
        &self,
        Parameters(req): Parameters<InstallRequest>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, ErrorData> {
        let install_id = self.generate_operation_id_for("install");

        // Check if we should run synchronously or use async notifications
        if self.should_run_synchronously(req.enable_async_notification) {
            match Self::install_implementation(&req).await {
                Ok(result_msg) => Ok(CallToolResult::success(vec![Content::text(result_msg)])),
                Err(error_msg) => Ok(CallToolResult::success(vec![Content::text(error_msg)])),
            }
        } else {
            let peer = context.peer.clone();
            let req_clone = req.clone();
            let install_id_clone = install_id.clone();
            let monitor = self.monitor.clone();

            // Register operation before spawn
            self.register_async_operation(
                &install_id,
                "cargo install",
                &format!("Installing package '{}' in the background", req.package),
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
                            "Installing package '{}' in the background",
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
            let timestamp = timestamp::format_current_time();
            Ok(CallToolResult::success(vec![Content::text(format!(
                "Install operation {} started at {} in the background. Installing package '{}'.{}",
                install_id, timestamp, req.package, tool_hint
            ))]))
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
        let merged = merge_outputs(&stdout, &stderr, &Self::no_output_placeholder("install"));

        if output.status.success() {
            Ok(format!(
                "Install operation completed successfully{working_dir_msg}.\nInstalled package: {}\nOutput: {merged}",
                req.package
            ))
        } else {
            Err(format!(
                "- Install operation failed{working_dir_msg}.\nPackage: {}\nErrors: {stderr}\nOutput: {merged}",
                req.package
            ))
        }
    }

    #[tool(
        description = "CARGO UPGRADE: Faster than terminal cargo. Synchronous operation - returns results immediately once cargo lock is acquired. Updates dependencies to latest versions using cargo-edit. Always use async_cargo_mcp MCP tools; do not run cargo in a terminal."
    )]
    async fn upgrade(
        &self,
        Parameters(req): Parameters<UpgradeRequest>,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, ErrorData> {
        let upgrade_id = self.generate_operation_id_for("upgrade");

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
        let merged = merge_outputs(&stdout, &stderr, &Self::no_output_placeholder("upgrade"));

        if output.status.success() {
            let dry_run_msg = if req.dry_run.unwrap_or(false) {
                " (dry run - no changes made)"
            } else {
                ""
            };
            Ok(format!(
                "⬆️ Upgrade operation #{upgrade_id} completed successfully{working_dir_msg}{dry_run_msg}.\nOutput: {merged}"
            ))
        } else {
            Err(format!(
                "- Upgrade operation #{upgrade_id} failed{working_dir_msg}.\nErrors: {stderr}\nOutput: {merged}"
            ))
        }
    }

    #[tool(
        description = "CARGO AUDIT: Faster than terminal cargo. Security vulnerability scanning. Use enable_async_notification=true for large projects to multitask. Identifies known security vulnerabilities. Always use async_cargo_mcp MCP tools; do not run cargo in a terminal. For operations >1s, set enable_async_notification=true and call mcp_async_cargo_m_wait with specific operation_ids to collect results."
    )]
    async fn audit(
        &self,
        Parameters(req): Parameters<AuditRequest>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, ErrorData> {
        let audit_id = self.generate_operation_id_for("audit");

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

        // Check if we should run synchronously or use async notifications
        if self.should_run_synchronously(req.enable_async_notification) {
            // Synchronous operation for when async notifications are disabled
            match Self::audit_implementation(&req).await {
                Ok(result_msg) => Ok(CallToolResult::success(vec![Content::text(result_msg)])),
                Err(error_msg) => Ok(CallToolResult::success(vec![Content::text(error_msg)])),
            }
        } else {
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
                "Scanning for security vulnerabilities in the background",
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
                        description: "Scanning for security vulnerabilities in the background"
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
            let timestamp = timestamp::format_current_time();
            Ok(CallToolResult::success(vec![Content::text(format!(
                "Audit operation {audit_id} started at {timestamp} in the background.{tool_hint}"
            ))]))
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
        let merged = merge_outputs(&stdout, &stderr, &Self::no_output_placeholder("audit"));

        if output.status.success() {
            Ok(format!(
                "Audit operation #{audit_id} completed successfully{working_dir_msg}.\nNo known vulnerabilities found.\nOutput: {merged}"
            ))
        } else {
            // Check if it's a vulnerability warning (exit code 1) vs actual error
            let vulnerability_detected = output.status.code() == Some(1) && !stdout.is_empty();

            if vulnerability_detected {
                Err(format!(
                    "Audit operation #{audit_id} found security vulnerabilities{working_dir_msg}.\nVulnerabilities detected:\n{stdout}\nErrors: {stderr}\nOutput: {merged}"
                ))
            } else {
                Err(format!(
                    "- Audit operation #{audit_id} failed{working_dir_msg}.\nErrors: {stderr}\nOutput: {merged}"
                ))
            }
        }
    }

    #[tool(
        description = "CARGO FMT: Faster than terminal cargo. Format Rust code using rustfmt. Use enable_async_notification=true for large projects to multitask while code is being formatted. Always use async_cargo_mcp MCP tools; do not run cargo in a terminal. For operations >1s, set enable_async_notification=true and call mcp_async_cargo_m_wait with specific operation_ids to collect results."
    )]
    async fn fmt(
        &self,
        Parameters(req): Parameters<FmtRequest>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, ErrorData> {
        let fmt_id = self.generate_operation_id_for("fmt");

        // Check if we should run synchronously or use async notifications
        if self.should_run_synchronously(req.enable_async_notification) {
            // Synchronous operation for when async notifications are disabled
            match Self::fmt_implementation(&req).await {
                Ok(result_msg) => Ok(CallToolResult::success(vec![Content::text(result_msg)])),
                Err(error_msg) => Ok(CallToolResult::success(vec![Content::text(error_msg)])),
            }
        } else {
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
                "Formatting code in the background",
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
                        description: "Formatting code in the background".to_string(),
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
            let timestamp = timestamp::format_current_time();
            Ok(CallToolResult::success(vec![Content::text(format!(
                "Format operation {fmt_id} started at {timestamp} in the background.{tool_hint}"
            ))]))
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
        let merged = merge_outputs(&stdout, &stderr, &Self::no_output_placeholder("format"));

        if output.status.success() {
            let check_msg = if req.check.unwrap_or(false) {
                " (check mode - no changes made)"
            } else {
                ""
            };
            Ok(format!(
                "Format operation completed successfully{working_dir_msg}{check_msg}.\nOutput: {merged}"
            ))
        } else {
            // Check if it's a formatting issue (exit code 1) vs actual error
            let formatting_issues = output.status.code() == Some(1) && req.check.unwrap_or(false);

            if formatting_issues {
                let merged_files =
                    merge_outputs(&stdout, &stderr, &Self::no_output_placeholder("format"));
                Ok(format!(
                    "Format operation found formatting issues{working_dir_msg}.\nFiles need formatting:\n{merged_files}"
                ))
            } else {
                Err(format!(
                    "- Format operation failed{working_dir_msg}.\nErrors: {stderr}\nOutput: {merged}"
                ))
            }
        }
    }

    #[tool(
        description = "CARGO TREE: Faster than terminal cargo. Synchronous operation - returns results immediately once cargo lock is acquired. Display dependency tree. Always use async_cargo_mcp MCP tools; do not run cargo in a terminal."
    )]
    async fn tree(
        &self,
        Parameters(req): Parameters<TreeRequest>,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, ErrorData> {
        // Synchronous operation only
        match Self::tree_implementation(&req).await {
            Ok(result_msg) => Ok(CallToolResult::success(vec![Content::text(result_msg)])),
            Err(error_msg) => Ok(CallToolResult::success(vec![Content::text(error_msg)])),
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
        let merged = merge_outputs(&stdout, &stderr, &Self::no_output_placeholder("tree"));

        if output.status.success() {
            Ok(format!(
                "Tree operation completed successfully{working_dir_msg}.\nDependency tree:\n{merged}"
            ))
        } else {
            Err(format!(
                "- Tree operation failed{working_dir_msg}.\nErrors: {stderr}\nOutput: {merged}"
            ))
        }
    }

    #[tool(
        description = "CARGO VERSION: Faster than terminal cargo. Synchronous operation - returns results immediately once cargo lock is acquired. Show cargo version information. Fast operation that helps LLMs understand the available cargo capabilities. Always use async_cargo_mcp MCP tools; do not run cargo in a terminal."
    )]
    async fn version(
        &self,
        Parameters(req): Parameters<VersionRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        use tokio::process::Command;

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
        let merged = merge_outputs(&stdout, &stderr, &Self::no_output_placeholder("version"));

        let result_msg = if output.status.success() {
            format!(
                "📋 Version operation completed successfully.\nCargo version information:\n{merged}"
            )
        } else {
            format!("- Version operation failed.\nErrors: {stderr}\nOutput: {merged}")
        };

        Ok(CallToolResult::success(vec![Content::text(result_msg)]))
    }

    #[tool(
        description = "CARGO FETCH: Faster than terminal cargo. Fetch dependencies without building. Use enable_async_notification=true for large dependency sets to multitask while downloading. Always use async_cargo_mcp MCP tools; do not run cargo in a terminal. For operations >1s, set enable_async_notification=true and call mcp_async_cargo_m_wait with specific operation_ids to collect results."
    )]
    async fn fetch(
        &self,
        Parameters(req): Parameters<FetchRequest>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, ErrorData> {
        let fetch_id = self.generate_operation_id_for("fetch");

        // Check if we should run synchronously or use async notifications
        if self.should_run_synchronously(req.enable_async_notification) {
            // Synchronous operation for when async notifications are disabled
            match Self::fetch_implementation(&req).await {
                Ok(result_msg) => Ok(CallToolResult::success(vec![Content::text(result_msg)])),
                Err(error_msg) => Ok(CallToolResult::success(vec![Content::text(error_msg)])),
            }
        } else {
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
                "Fetching dependencies in the background",
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
                        description: "Fetching dependencies in the background".to_string(),
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
            let timestamp = timestamp::format_current_time();
            Ok(CallToolResult::success(vec![Content::text(format!(
                "Fetch operation {fetch_id} started at {timestamp} in the background.{tool_hint}"
            ))]))
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
        let merged = merge_outputs(&stdout, &stderr, &Self::no_output_placeholder("fetch"));

        if output.status.success() {
            Ok(format!(
                "📦 Fetch operation completed successfully{working_dir_msg}.\nDependencies fetched:\n{merged}"
            ))
        } else {
            Err(format!(
                "- Fetch operation failed{working_dir_msg}.\nErrors: {stderr}\nOutput: {merged}"
            ))
        }
    }

    #[tool(
        description = "CARGO RUSTC: Faster than terminal cargo. Compile with custom rustc options. Use enable_async_notification=true for complex builds to multitask while compiling. Always use async_cargo_mcp MCP tools; do not run cargo in a terminal. For operations >1s, set enable_async_notification=true and call mcp_async_cargo_m_wait with specific operation_ids to collect results."
    )]
    async fn rustc(
        &self,
        Parameters(req): Parameters<RustcRequest>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, ErrorData> {
        let rustc_id = self.generate_operation_id_for("rustc");

        // Check if we should run synchronously or use async notifications
        if self.should_run_synchronously(req.enable_async_notification) {
            // Synchronous operation for when async notifications are disabled
            match Self::rustc_implementation(&req).await {
                Ok(result_msg) => Ok(CallToolResult::success(vec![Content::text(result_msg)])),
                Err(error_msg) => Ok(CallToolResult::success(vec![Content::text(error_msg)])),
            }
        } else {
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
                "Compiling with custom rustc options in the background",
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
                        description: "Compiling with custom rustc options in the background"
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
            let timestamp = timestamp::format_current_time();
            Ok(CallToolResult::success(vec![Content::text(format!(
                "Rustc operation {rustc_id} started at {timestamp} in the background.{tool_hint}"
            ))]))
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
        let merged = merge_outputs(&stdout, &stderr, &Self::no_output_placeholder("rustc"));

        if output.status.success() {
            Ok(format!(
                "Rustc operation #{rustc_id} completed successfully{working_dir_msg}.\nOutput: {merged}"
            ))
        } else {
            Err(format!(
                "- Rustc operation #{rustc_id} failed{working_dir_msg}.\nErrors: {stderr}\nOutput: {merged}"
            ))
        }
    }

    #[tool(
        description = "CARGO METADATA: Faster than terminal cargo. Synchronous operation - returns results immediately once cargo lock is acquired. Output JSON metadata about the project. Fast operation that provides LLMs with comprehensive project structure information. Always use async_cargo_mcp MCP tools; do not run cargo in a terminal."
    )]
    async fn metadata(
        &self,
        Parameters(req): Parameters<MetadataRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        use tokio::process::Command;

        let metadata_id = self.generate_operation_id_for("metadata");

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
        let merged = merge_outputs(&stdout, &stderr, &Self::no_output_placeholder("metadata"));

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
                "📊 Metadata operation #{metadata_id} completed successfully{working_dir_msg}{json_validation}.\nProject metadata:\n{merged}"
            )
        } else {
            format!(
                "- Metadata operation #{metadata_id} failed{working_dir_msg}.\nErrors: {stderr}\nOutput: {merged}"
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
            instructions: Some("Rust cargo operations with async support.\n\nGolden rules for LLM agents:\n1) Always use async_cargo_mcp MCP tools for ALL cargo operations. Do not run cargo in a terminal.\n2) For builds/tests >1s, set enable_async_notification=true to multitask while work runs.\n3) After starting an async operation, you MUST call mcp_async_cargo_m_wait to retrieve results before making decisions.\n\nThese tools are Faster than terminal commands and provide structured output, progress callbacks, and isolation.".to_string()),
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

        // Best-effort startup lock check in current CWD
        if let Ok(cwd) = std::env::current_dir() {
            let lock_path = cwd.join("target").join(".cargo-lock");
            if lock_path.exists() {
                let full = lock_path.display().to_string();
                tracing::warn!(%full, "Detected existing Cargo lock file at startup");
                // Append a short notice to instructions
                let notice = format!(
                    "\n[Notice] Detected Cargo lock file: {full}. If you encounter blocked cargo commands, you may delete this file and optionally run cargo clean. You can also call the 'cargo_lock_remediation' tool."
                );
                result.instructions = result.instructions.map(|mut s| {
                    s.push_str(&notice);
                    s
                });
            }
        }

        tracing::debug!("Initialize result: {result:?}");
        tracing::debug!("=== INITIALIZE METHOD RETURNING ===");
        Ok(result)
    }
}

/// Async cargo operations with callback support
impl AsyncCargo {
    /// Get the timeout duration from the monitor's configuration
    pub async fn get_monitor_timeout(&self) -> std::time::Duration {
        // We need to access the monitor's configuration
        // Since MonitorConfig doesn't have a public getter, we'll need to add one
        // For now, let's implement a simple approach by adding a method to OperationMonitor
        self.monitor.get_default_timeout().await
    }

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

        let operation_id = self.generate_operation_id_for("add");
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

        let operation_id = self.generate_operation_id_for("remove");
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
        let operation_id = self.generate_operation_id_for("build");
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

        let operation_id = self.generate_operation_id_for("audit");
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
            let error_msg = r#"- Audit operation failed: cargo-audit is not installed. 
📦 Install with: cargo install cargo-audit
🔒 This tool scans for known security vulnerabilities in dependencies."#
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
