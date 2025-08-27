//! Pre-warmed shell pool for instant cargo commands
//!
//! This module provides a pool of pre-warmed bash shells that can execute cargo commands
//! with minimal startup latency. The pool maintains separate shell collections per working
//! directory to ensure proper isolation while maximizing performance.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::{Mutex, RwLock};
use tokio::time::timeout;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Configuration for shell pool behavior
#[derive(Debug, Clone)]
pub struct ShellPoolConfig {
    pub enabled: bool,
    pub shells_per_directory: usize,
    pub max_total_shells: usize,
    pub shell_idle_timeout: Duration,
    pub pool_cleanup_interval: Duration,
    pub shell_spawn_timeout: Duration,
    pub command_timeout: Duration,
    pub health_check_interval: Duration,
}

impl Default for ShellPoolConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            shells_per_directory: 2,
            max_total_shells: 20,
            shell_idle_timeout: Duration::from_secs(1800),
            pool_cleanup_interval: Duration::from_secs(300),
            shell_spawn_timeout: Duration::from_secs(5),
            command_timeout: Duration::from_secs(300),
            health_check_interval: Duration::from_secs(60),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShellCommand {
    pub id: String,
    pub command: Vec<String>,
    pub working_dir: String,
    pub timeout_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShellResponse {
    pub id: String,
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub duration_ms: u64,
}

/// Error types for shell operations
#[derive(Debug, thiserror::Error)]
pub enum ShellError {
    #[error("Failed to spawn shell process: {0}")]
    SpawnError(#[from] std::io::Error),

    #[error("Shell communication timeout")]
    Timeout,

    #[error("Shell process died unexpectedly")]
    ProcessDied,

    #[error("Failed to serialize command: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Shell pool is at capacity")]
    PoolFull,

    #[error("Working directory access error: {0}")]
    WorkingDirectoryError(String),
}

impl ShellError {
    /// Check if this error represents a potentially recoverable condition
    pub fn is_recoverable(&self) -> bool {
        matches!(
            self,
            ShellError::Timeout | ShellError::PoolFull | ShellError::ProcessDied
        )
    }

    /// Check if this error indicates resource exhaustion
    pub fn is_resource_exhaustion(&self) -> bool {
        matches!(self, ShellError::PoolFull | ShellError::Timeout)
    }

    /// Check if this error is related to I/O operations
    pub fn is_io_error(&self) -> bool {
        matches!(
            self,
            ShellError::SpawnError(_) | ShellError::WorkingDirectoryError(_)
        )
    }

    /// Get error category for programmatic handling
    pub fn error_category(&self) -> &'static str {
        match self {
            ShellError::SpawnError(_) | ShellError::WorkingDirectoryError(_) => "IO",
            ShellError::Timeout => "TIMEOUT",
            ShellError::ProcessDied => "PROCESS",
            ShellError::SerializationError(_) => "SERIALIZATION",
            ShellError::PoolFull => "RESOURCE",
        }
    }

    /// Get severity level for logging
    pub fn severity_level(&self) -> &'static str {
        match self {
            ShellError::SpawnError(_)
            | ShellError::ProcessDied
            | ShellError::SerializationError(_)
            | ShellError::WorkingDirectoryError(_) => "ERROR",
            ShellError::Timeout | ShellError::PoolFull => "WARN", // Might be temporary
        }
    }
}

/// A single prewarmed shell process
pub struct PrewarmedShell {
    /// Unique identifier for this shell
    pub id: String,
    /// The shell process
    process: Child,
    /// Writer for sending commands via stdin
    stdin: tokio::process::ChildStdin,
    /// Reader for receiving responses via stdout
    stdout_reader: BufReader<tokio::process::ChildStdout>,
    /// Working directory for this shell
    working_dir: PathBuf,
    /// Configuration for this shell
    config: ShellPoolConfig,
    /// Last time this shell was used
    last_used: Instant,
    /// Whether this shell is currently healthy
    is_healthy: bool,
    /// Lock to ensure only one command runs at a time
    command_lock: Mutex<()>,
}

impl std::fmt::Debug for PrewarmedShell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PrewarmedShell")
            .field("id", &self.id)
            .field("working_dir", &self.working_dir)
            .field("config", &self.config)
            .field("last_used", &self.last_used)
            .field("is_healthy", &self.is_healthy)
            .field("process_id", &self.process.id())
            .finish_non_exhaustive()
    }
}

impl PrewarmedShell {
    /// Create a new prewarmed shell for the specified working directory
    pub async fn new(
        working_dir: impl AsRef<Path>,
        _config: &ShellPoolConfig,
    ) -> Result<Self, ShellError> {
        let working_dir = working_dir.as_ref().to_path_buf();
        let shell_id = Uuid::new_v4().to_string();

        debug!(
            "Spawning new shell {} for directory: {:?}",
            shell_id, &working_dir
        );

        // Spawn bash process with JSON communication
        let mut process = Command::new("bash")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped()) // capture stderr for diagnostics
            .current_dir(&working_dir)
            .spawn()?;

        let stdin = process.stdin.take().ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::BrokenPipe, "Failed to get stdin")
        })?;

        let stdout = process.stdout.take().ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::BrokenPipe, "Failed to get stdout")
        })?;

        let stdout_reader = BufReader::new(stdout);
        // Spawn a background task to read and log stderr lines for diagnostics
        if let Some(stderr) = process.stderr.take() {
            tokio::spawn(async move {
                let mut reader = BufReader::new(stderr);
                loop {
                    let mut line = String::new();
                    match reader.read_line(&mut line).await {
                        Ok(0) => break, // EOF
                        Ok(_) => {
                            if !line.trim().is_empty() {
                                warn!(target: "shell_stderr", "shell stderr: {}", line.trim_end());
                            }
                        }
                        Err(e) => {
                            warn!(target: "shell_stderr", "error reading shell stderr: {}", e);
                            break;
                        }
                    }
                }
            });
        }

        let mut shell = Self {
            id: shell_id.clone(),
            process,
            stdin,
            stdout_reader,
            working_dir: working_dir.clone(),
            config: _config.clone(),
            last_used: Instant::now(),
            is_healthy: true,
            command_lock: Mutex::new(()),
        };

        // Initialize the shell with our command protocol handler
        shell.initialize_protocol().await?;

        info!(
            "Successfully spawned shell {} for directory: {:?}",
            shell_id, &working_dir
        );
        Ok(shell)
    }

    /// Initialize the shell with our JSON command protocol
    async fn initialize_protocol(&mut self) -> Result<(), ShellError> {
        // Send initial setup commands to prepare shell for JSON protocol
        let setup_script = r#"
# Portable minimal shell setup for async_cargo_mcp (macOS bash 3.2 compatible)
set +e

command -v jq >/dev/null 2>&1 || {
    echo 'MCP_DIAG: jq not found in PATH' >&2
}

json_escape_file() {
    # Use jq -Rs . to JSON-encode entire file contents
    jq -Rs . 2>/dev/null || {
        # Fallback: basic escaping (quotes and newlines)
        sed 's/\\/\\\\/g; s/"/\\"/g; s/$/\\n/' | tr -d '\n' | sed 's/\\n$//' | sed 's/^/"/;s/$/"/'
    }
}

execute_command() {
    cmd_json="$1"
    id=$(echo "$cmd_json" | jq -r '.id')
    working_dir=$(echo "$cmd_json" | jq -r '.working_dir')
    timeout_ms=$(echo "$cmd_json" | jq -r '.timeout_ms')
    # Build command line joining args safely (shell-escaped)
    cmd_line=$(echo "$cmd_json" | jq -r '.command | map(@sh) | join(" ")')

    cd "$working_dir" 2>/dev/null || {
        echo '{"id":"'"$id"'","exit_code":1,"stdout":"","stderr":"Failed to change directory","duration_ms":0}'
        return
    }

    start_time=$(date +%s)
    temp_stdout=$(mktemp)
    temp_stderr=$(mktemp)

    # Execute command via bash -c using safe assembled line
    bash -c "$cmd_line" >"$temp_stdout" 2>"$temp_stderr"
    exit_code=$?
    end_time=$(date +%s)
    duration=$(((end_time - start_time)*1000))

    stdout_json=$(cat "$temp_stdout" | json_escape_file)
    stderr_json=$(cat "$temp_stderr" | json_escape_file)
    rm -f "$temp_stdout" "$temp_stderr"
    echo '{"id":"'"$id"'","exit_code":'"$exit_code"',"stdout":'"$stdout_json"',"stderr":'"$stderr_json"',"duration_ms":'"$duration"'}'
}

echo "SHELL_READY"

while IFS= read -r line; do
    [ -z "$line" ] && continue
    if [ "$line" = "HEALTH_CHECK" ]; then
        echo "HEALTHY"
    elif [ "$line" = "SHUTDOWN" ]; then
        break
    else
        execute_command "$line"
    fi
done
"#;

        // Send setup script to shell
        self.stdin.write_all(setup_script.as_bytes()).await?;
        self.stdin.flush().await?;

        // Wait for ready signal
        let mut ready_line = String::new();
        self.stdout_reader.read_line(&mut ready_line).await?;

        if ready_line.trim() != "SHELL_READY" {
            error!(
                "Shell {} failed to emit SHELL_READY, got: '{}'",
                self.id,
                ready_line.trim()
            );
            return Err(ShellError::ProcessDied);
        }

        debug!("Shell {} initialized and ready", self.id);
        Ok(())
    }

    /// Execute a command in this shell
    pub async fn execute_command(
        &mut self,
        command: ShellCommand,
    ) -> Result<ShellResponse, ShellError> {
        let _lock = self.command_lock.lock().await;
        self.last_used = Instant::now();

        info!(
            "Executing command {} in shell {}: {:?}",
            command.id, self.id, command.command
        );

        // Serialize command as JSON
        let command_json = serde_json::to_string(&command)?;

        // Send command to shell
        self.stdin
            .write_all(command_json.as_bytes())
            .await
            .map_err(|_| ShellError::ProcessDied)?;
        info!("shell {} wrote command bytes", self.id);
        self.stdin
            .write_all(b"\n")
            .await
            .map_err(|_| ShellError::ProcessDied)?;
        info!("shell {} wrote newline", self.id);
        self.stdin
            .flush()
            .await
            .map_err(|_| ShellError::ProcessDied)?;
        info!("shell {} flushed stdin", self.id);

        // Read response with timeout
        let response_future = async {
            let mut attempts = 0usize;
            let mut last_err: Option<serde_json::Error> = None;
            loop {
                attempts += 1;
                if attempts > 50 {
                    // Prevent infinite loop on persistent junk
                    if let Some(err) = last_err {
                        return Err(ShellError::from(err));
                    }
                    return Err(ShellError::ProcessDied);
                }
                let mut response_line = String::new();
                let bytes = self
                    .stdout_reader
                    .read_line(&mut response_line)
                    .await
                    .map_err(|_| ShellError::ProcessDied)?;
                if bytes == 0 {
                    return Err(ShellError::ProcessDied);
                }
                let trimmed = response_line.trim();
                info!("shell {} raw line: '{}'", self.id, trimmed);
                if trimmed.is_empty() {
                    continue;
                }
                match serde_json::from_str::<ShellResponse>(trimmed) {
                    Ok(resp) => break Ok(resp),
                    Err(e) => {
                        info!("Shell {} skipping non-JSON line: '{}'", self.id, trimmed);
                        last_err = Some(e);
                        continue;
                    }
                }
            }
        };

        let timeout_duration = Duration::from_millis(command.timeout_ms);
        let response = timeout(timeout_duration, response_future)
            .await
            .map_err(|_| ShellError::Timeout)??;

        info!(
            "Command {} completed with exit code {} in {}ms",
            response.id, response.exit_code, response.duration_ms
        );

        Ok(response)
    }

    /// Check if this shell is healthy
    pub async fn health_check(&mut self) -> bool {
        let _lock = self.command_lock.lock().await;

        debug!("Performing health check on shell {}", self.id);

        // Send health check command
        if let Err(e) = self.stdin.write_all(b"HEALTH_CHECK\n").await {
            warn!("Health check failed for shell {}: {}", self.id, e);
            self.is_healthy = false;
            return false;
        }

        if let Err(e) = self.stdin.flush().await {
            warn!("Health check failed for shell {}: {}", self.id, e);
            self.is_healthy = false;
            return false;
        }

        // Read health response with short timeout
        let health_future = async {
            let mut response = String::new();
            self.stdout_reader.read_line(&mut response).await?;
            Ok::<String, std::io::Error>(response)
        };

        match timeout(Duration::from_secs(2), health_future).await {
            Ok(Ok(response)) if response.trim() == "HEALTHY" => {
                debug!("Shell {} is healthy", self.id);
                self.is_healthy = true;
                true
            }
            _ => {
                warn!("Shell {} failed health check", self.id);
                self.is_healthy = false;
                false
            }
        }
    }

    /// Get the working directory this shell is configured for
    pub fn working_dir(&self) -> &Path {
        &self.working_dir
    }

    /// Get when this shell was last used
    pub fn last_used(&self) -> Instant {
        self.last_used
    }

    /// Check if this shell is healthy
    pub fn is_healthy(&self) -> bool {
        self.is_healthy
    }

    /// Get the shell ID
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Gracefully shutdown this shell
    pub async fn shutdown(&mut self) {
        debug!("Shutting down shell {}", self.id);

        // Try to send shutdown signal
        if (self.stdin.write_all(b"SHUTDOWN\n").await).is_ok() {
            let _ = self.stdin.flush().await;
        }

        // Kill the process
        if let Err(e) = self.process.kill().await {
            warn!("Failed to kill shell process {}: {}", self.id, e);
        }

        // Wait for process to exit
        if let Err(e) = self.process.wait().await {
            warn!("Error waiting for shell {} to exit: {}", self.id, e);
        }

        info!("Shell {} has been shut down", self.id);
    }
}

impl Drop for PrewarmedShell {
    fn drop(&mut self) {
        // Attempt to kill the process on drop
        let _ = self.process.start_kill();
    }
}

/// Pool of shells for a specific working directory
#[derive(Debug)]
pub struct ShellPool {
    working_dir: PathBuf,
    shells: Mutex<Vec<PrewarmedShell>>,
    config: ShellPoolConfig,
    last_accessed: Mutex<Instant>,
}

impl ShellPool {
    /// Create a new shell pool for the specified working directory
    pub fn new(working_dir: impl AsRef<Path>, config: ShellPoolConfig) -> Self {
        let working_dir = working_dir.as_ref().to_path_buf();
        info!("Creating shell pool for directory: {:?}", working_dir);

        Self {
            working_dir,
            shells: Mutex::new(Vec::new()),
            config,
            last_accessed: Mutex::new(Instant::now()),
        }
    }

    /// Get a shell from the pool, creating one if necessary
    pub async fn get_shell(&self) -> Result<PrewarmedShell, ShellError> {
        let mut last_accessed = self.last_accessed.lock().await;
        *last_accessed = Instant::now();
        drop(last_accessed);

        let mut shells = self.shells.lock().await;

        // Try to find a healthy shell
        while let Some(shell) = shells.pop() {
            if shell.is_healthy() {
                debug!("Reusing healthy shell {} from pool", shell.id());
                return Ok(shell);
            } else {
                debug!("Discarding unhealthy shell {} from pool", shell.id());
                // Shell is unhealthy, let it drop and try next
            }
        }

        drop(shells);

        // No healthy shells available, create a new one
        debug!(
            "Creating new shell for pool (directory: {:?})",
            self.working_dir
        );
        PrewarmedShell::new(&self.working_dir, &self.config).await
    }

    /// Return a shell to the pool
    pub async fn return_shell(&self, shell: PrewarmedShell) {
        let mut shells = self.shells.lock().await;

        // Only return healthy shells and respect pool size limits
        if shell.is_healthy() && shells.len() < self.config.shells_per_directory {
            debug!("Returning shell {} to pool", shell.id());
            shells.push(shell);
        } else {
            debug!("Discarding shell {} (unhealthy or pool full)", shell.id());
            // Shell will be dropped and process killed
        }
    }

    /// Check if this pool has been idle for too long
    pub async fn is_idle(&self) -> bool {
        let last_accessed = self.last_accessed.lock().await;
        last_accessed.elapsed() > self.config.shell_idle_timeout
    }

    /// Get the working directory for this pool
    pub fn working_dir(&self) -> &Path {
        &self.working_dir
    }

    /// Perform health checks on all shells in the pool
    pub async fn health_check(&self) {
        let mut shells = self.shells.lock().await;
        let mut healthy_shells = Vec::new();

        for mut shell in shells.drain(..) {
            if shell.health_check().await {
                healthy_shells.push(shell);
            } else {
                debug!("Removing unhealthy shell {} from pool", shell.id());
                // Unhealthy shell will be dropped
            }
        }

        *shells = healthy_shells;
    }

    /// Shutdown all shells in this pool
    pub async fn shutdown(&self) {
        let mut shells = self.shells.lock().await;
        for mut shell in shells.drain(..) {
            shell.shutdown().await;
        }
        info!("Shut down shell pool for directory: {:?}", self.working_dir);
    }

    /// Get the current number of shells in the pool
    pub async fn shell_count(&self) -> usize {
        let shells = self.shells.lock().await;
        shells.len()
    }
}

/// Manager for multiple shell pools across different working directories
#[derive(Debug)]
pub struct ShellPoolManager {
    pools: RwLock<HashMap<PathBuf, Arc<ShellPool>>>,
    config: ShellPoolConfig,
    total_shells: Mutex<usize>,
}

impl ShellPoolManager {
    /// Create a new shell pool manager
    pub fn new(config: ShellPoolConfig) -> Self {
        info!("Creating shell pool manager with config: {:#?}", config);

        Self {
            pools: RwLock::new(HashMap::new()),
            config,
            total_shells: Mutex::new(0),
        }
    }

    /// Start background monitoring tasks (call this after creating the manager)
    pub fn start_background_tasks(self: Arc<Self>) {
        if self.config.enabled {
            let manager_for_cleanup = Arc::clone(&self);
            let manager_for_health = Arc::clone(&self);
            let cleanup_interval = self.config.pool_cleanup_interval;
            let health_interval = self.config.health_check_interval;

            // Start periodic cleanup task
            tokio::spawn(async move {
                let mut interval = tokio::time::interval(cleanup_interval);

                loop {
                    interval.tick().await;
                    manager_for_cleanup.cleanup_idle_shells().await;
                }
            });

            // Start periodic health check task
            tokio::spawn(async move {
                let mut interval = tokio::time::interval(health_interval);

                loop {
                    interval.tick().await;
                    manager_for_health.health_check_all_pools().await;
                }
            });

            info!("Started background tasks for shell pool monitoring");
        }
    }

    /// Get a shell for the specified working directory
    pub async fn get_shell(&self, working_dir: impl AsRef<Path>) -> Option<PrewarmedShell> {
        if !self.config.enabled {
            debug!("Shell pooling is disabled");
            return None;
        }

        let working_dir = working_dir.as_ref().to_path_buf();

        // Check if we're at capacity
        {
            let total_shells = self.total_shells.lock().await;
            if *total_shells >= self.config.max_total_shells {
                warn!("Shell pool manager at capacity ({} shells)", *total_shells);
                return None;
            }
        }

        // Get or create pool for this directory
        let pool = {
            let pools = self.pools.read().await;
            if let Some(pool) = pools.get(&working_dir) {
                Arc::clone(pool)
            } else {
                drop(pools);
                self.create_pool_for_dir(&working_dir).await
            }
        };

        // Get shell from pool
        match pool.get_shell().await {
            Ok(shell) => {
                let mut total_shells = self.total_shells.lock().await;
                *total_shells += 1;
                debug!("Got shell from pool, total shells: {}", *total_shells);
                Some(shell)
            }
            Err(e) => {
                warn!("Failed to get shell from pool for {:?}: {}", working_dir, e);
                None
            }
        }
    }

    /// Return a shell to its appropriate pool
    pub async fn return_shell(&self, shell: PrewarmedShell) {
        let working_dir = shell.working_dir().to_path_buf();

        // Find the pool for this working directory
        let pools = self.pools.read().await;
        if let Some(pool) = pools.get(&working_dir) {
            let pool = Arc::clone(pool);
            drop(pools);

            pool.return_shell(shell).await;

            let mut total_shells = self.total_shells.lock().await;
            *total_shells = total_shells.saturating_sub(1);
            debug!("Returned shell to pool, total shells: {}", *total_shells);
        } else {
            warn!("No pool found for working directory: {:?}", working_dir);
            // Shell will be dropped
        }
    }

    /// Create a new pool for the specified working directory
    async fn create_pool_for_dir(&self, working_dir: &Path) -> Arc<ShellPool> {
        let mut pools = self.pools.write().await;

        // Double-check that pool wasn't created while we were waiting for write lock
        if let Some(existing_pool) = pools.get(working_dir) {
            return Arc::clone(existing_pool);
        }

        let pool = Arc::new(ShellPool::new(working_dir, self.config.clone()));
        pools.insert(working_dir.to_path_buf(), Arc::clone(&pool));

        info!("Created new shell pool for directory: {:?}", working_dir);
        pool
    }

    /// Clean up idle pools and perform health checks
    pub async fn cleanup_idle_pools(&self) {
        debug!("Starting cleanup of idle pools");

        let mut pools = self.pools.write().await;
        let mut pools_to_remove = Vec::new();

        // Check each pool for idleness and health
        for (working_dir, pool) in pools.iter() {
            if pool.is_idle().await {
                debug!("Pool for {:?} is idle, marking for removal", working_dir);
                pools_to_remove.push(working_dir.clone());
            } else {
                // Perform health check on active pools
                pool.health_check().await;
            }
        }

        // Remove idle pools
        for working_dir in pools_to_remove {
            if let Some(pool) = pools.remove(&working_dir) {
                pool.shutdown().await;
            }
        }

        debug!("Completed cleanup, {} pools remaining", pools.len());
    }

    /// Shutdown all pools and shells
    pub async fn shutdown_all(&self) {
        info!("Shutting down all shell pools");

        let mut pools = self.pools.write().await;
        let pool_count = pools.len();

        for (_, pool) in pools.drain() {
            pool.shutdown().await;
        }

        let mut total_shells = self.total_shells.lock().await;
        *total_shells = 0;

        info!("Shut down {} shell pools", pool_count);
    }

    /// Get configuration
    pub fn config(&self) -> &ShellPoolConfig {
        &self.config
    }

    /// Get current statistics
    pub async fn get_stats(&self) -> ShellPoolStats {
        let pools = self.pools.read().await;
        let total_shells = *self.total_shells.lock().await;

        ShellPoolStats {
            total_pools: pools.len(),
            total_shells,
            max_shells: self.config.max_total_shells,
        }
    }

    /// Clean up idle shells across all pools
    async fn cleanup_idle_shells(&self) {
        let pools = self.pools.read().await;
        let mut cleaned_count = 0;

        for (path, pool) in pools.iter() {
            let before_count = pool.shell_count().await;
            // Shells will be cleaned up based on their idle timeout
            // This is a placeholder - actual cleanup logic would be in ShellPool
            debug!("Checking pool {:?} for idle shells", path);
            let after_count = pool.shell_count().await;
            cleaned_count += before_count.saturating_sub(after_count);
        }

        if cleaned_count > 0 {
            info!("Cleaned up {} idle shells", cleaned_count);
        }
    }

    /// Perform health checks on all pools
    async fn health_check_all_pools(&self) {
        let pools = self.pools.read().await;

        for (path, pool) in pools.iter() {
            debug!("Health checking pool {:?}", path);
            pool.health_check().await;
        }
    }
}

/// Statistics about shell pool usage
#[derive(Debug, Clone)]
pub struct ShellPoolStats {
    pub total_pools: usize,
    pub total_shells: usize,
    pub max_shells: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_shell_pool_config_defaults() {
        let config = ShellPoolConfig::default();
        assert!(config.enabled); // Should be enabled by default for production use
        assert_eq!(config.shells_per_directory, 2);
        assert_eq!(config.max_total_shells, 20);
    }

    #[tokio::test]
    async fn test_shell_command_serialization() {
        let command = ShellCommand {
            id: "test123".to_string(),
            command: vec!["cargo".to_string(), "build".to_string()],
            working_dir: "/tmp".to_string(),
            timeout_ms: 30000,
        };

        let json = serde_json::to_string(&command).unwrap();
        let deserialized: ShellCommand = serde_json::from_str(&json).unwrap();

        assert_eq!(command.id, deserialized.id);
        assert_eq!(command.command, deserialized.command);
    }

    #[tokio::test]
    async fn test_shell_pool_manager_disabled() {
        let config = ShellPoolConfig {
            enabled: false,
            ..Default::default()
        };

        let manager = ShellPoolManager::new(config);
        let shell = manager.get_shell("/tmp").await;
        assert!(shell.is_none());
    }

    #[tokio::test]
    async fn test_shell_pool_creation() {
        let temp_dir = TempDir::new().unwrap();
        let config = ShellPoolConfig::default();

        let pool = ShellPool::new(temp_dir.path(), config);
        assert_eq!(pool.working_dir(), temp_dir.path());
    }
}
