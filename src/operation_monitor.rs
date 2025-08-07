//! Operation monitoring and management system
//!
//! This module provides comprehensive monitoring, timeout handling, and cancellation
//! support for long-running cargo operations. It enables tracking of operation state,
//! automatic cleanup, and detailed logging for debugging.

use crate::callback_system::{CallbackSender, ProgressUpdate};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tokio::time::timeout;
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Represents the current state of an operation
#[derive(Debug, Clone, PartialEq)]
pub enum OperationState {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
    TimedOut,
}

/// Information about a running operation
#[derive(Debug, Clone)]
pub struct OperationInfo {
    pub id: String,
    pub command: String,
    pub description: String,
    pub state: OperationState,
    pub start_time: Instant,
    pub end_time: Option<Instant>,
    pub timeout_duration: Option<Duration>,
    pub working_directory: Option<String>,
    pub result: Option<Result<String, String>>,
    pub cancellation_token: CancellationToken,
}

impl OperationInfo {
    /// Create a new operation info
    pub fn new(
        command: String,
        description: String,
        timeout_duration: Option<Duration>,
        working_directory: Option<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            command,
            description,
            state: OperationState::Pending,
            start_time: Instant::now(),
            end_time: None,
            timeout_duration,
            working_directory,
            result: None,
            cancellation_token: CancellationToken::new(),
        }
    }

    /// Get the duration since the operation started
    pub fn duration(&self) -> Duration {
        match self.end_time {
            Some(end) => end.duration_since(self.start_time),
            None => self.start_time.elapsed(),
        }
    }

    /// Check if the operation is still active (running or pending)
    pub fn is_active(&self) -> bool {
        matches!(
            self.state,
            OperationState::Pending | OperationState::Running
        )
    }

    /// Mark the operation as completed with a result
    pub fn complete(&mut self, result: Result<String, String>) {
        self.end_time = Some(Instant::now());
        self.state = match &result {
            Ok(_) => OperationState::Completed,
            Err(_) => OperationState::Failed,
        };
        self.result = Some(result);
    }

    /// Mark the operation as cancelled
    pub fn cancel(&mut self) {
        self.end_time = Some(Instant::now());
        self.state = OperationState::Cancelled;
        self.cancellation_token.cancel();
    }

    /// Mark the operation as timed out
    pub fn timeout(&mut self) {
        self.end_time = Some(Instant::now());
        self.state = OperationState::TimedOut;
        self.cancellation_token.cancel();
    }

    /// Start the operation (change state from Pending to Running)
    pub fn start(&mut self) {
        if self.state == OperationState::Pending {
            self.state = OperationState::Running;
        }
    }
}

/// Configuration for operation monitoring
#[derive(Debug, Clone)]
pub struct MonitorConfig {
    /// Default timeout for operations
    pub default_timeout: Duration,
    /// How often to check for operation timeouts
    pub cleanup_interval: Duration,
    /// Maximum number of completed operations to keep in history
    pub max_history_size: usize,
    /// Maximum number of completed operations to keep in completion history (for wait operations)
    pub max_completion_history_size: usize,
    /// Whether to automatically clean up completed operations
    pub auto_cleanup: bool,
}

impl Default for MonitorConfig {
    fn default() -> Self {
        Self {
            default_timeout: Duration::from_secs(300),    // 5 minutes
            cleanup_interval: Duration::from_secs(21600), // 6 hours - less aggressive for long user sessions
            max_history_size: 1000,
            max_completion_history_size: 5000, // Keep more completion history for wait operations
            auto_cleanup: true,
        }
    }
}

/// Operation monitor that tracks and manages cargo operations
#[derive(Debug)]
pub struct OperationMonitor {
    operations: Arc<RwLock<HashMap<String, OperationInfo>>>,
    /// Completion history to track operations that have finished, even after cleanup
    completion_history: Arc<RwLock<HashMap<String, OperationInfo>>>,
    config: MonitorConfig,
    cleanup_token: CancellationToken,
}

impl OperationMonitor {
    /// Create a new operation monitor
    pub fn new(config: MonitorConfig) -> Self {
        let monitor = Self {
            operations: Arc::new(RwLock::new(HashMap::new())),
            completion_history: Arc::new(RwLock::new(HashMap::new())),
            config,
            cleanup_token: CancellationToken::new(),
        };

        // Start the cleanup task
        if monitor.config.auto_cleanup {
            monitor.start_cleanup_task();
        }

        monitor
    }

    /// Register a new operation for monitoring
    pub async fn register_operation(
        &self,
        command: String,
        description: String,
        timeout_duration: Option<Duration>,
        working_directory: Option<String>,
    ) -> String {
        let operation = OperationInfo::new(
            command,
            description.clone(),
            timeout_duration.or(Some(self.config.default_timeout)),
            working_directory,
        );
        let id = operation.id.clone();

        debug!("Registering operation: {} - {}", id, operation.description);

        let mut operations = self.operations.write().await;
        operations.insert(id.clone(), operation);

        tracing::debug!("Registered operation {id}: {description}");
        id
    }
    /// Start monitoring an operation
    pub async fn start_operation(&self, operation_id: &str) -> Result<(), String> {
        let mut operations = self.operations.write().await;

        if let Some(operation) = operations.get_mut(operation_id) {
            operation.start();
            tracing::debug!(
                "Started operation {operation_id}: {}",
                operation.description
            );
            Ok(())
        } else {
            Err(format!("Operation not found: {operation_id}"))
        }
    }

    /// Complete an operation with a result
    pub async fn complete_operation(
        &self,
        operation_id: &str,
        result: Result<String, String>,
    ) -> Result<(), String> {
        let mut operations = self.operations.write().await;

        if let Some(operation) = operations.get_mut(operation_id) {
            operation.complete(result.clone());

            // Store completed operation in completion history for future wait operations
            let completed_operation = operation.clone();
            drop(operations); // Release the write lock early

            let mut completion_history = self.completion_history.write().await;
            completion_history.insert(operation_id.to_string(), completed_operation);

            match &result {
                Ok(msg) => {
                    tracing::debug!("Operation {operation_id} completed successfully: {msg}")
                }
                Err(err) => error!("Operation {} failed: {}", operation_id, err),
            }

            Ok(())
        } else {
            Err(format!("Operation not found: {operation_id}"))
        }
    }

    /// Cancel an operation
    pub async fn cancel_operation(&self, operation_id: &str) -> Result<(), String> {
        let mut operations = self.operations.write().await;

        if let Some(operation) = operations.get_mut(operation_id) {
            operation.cancel();
            warn!("Operation {} was cancelled", operation_id);
            Ok(())
        } else {
            Err(format!("Operation not found: {operation_id}"))
        }
    }

    /// Get information about an operation
    pub async fn get_operation(&self, operation_id: &str) -> Option<OperationInfo> {
        let operations = self.operations.read().await;
        operations.get(operation_id).cloned()
    }

    /// Get all operations matching a predicate
    pub async fn get_operations<F>(&self, predicate: F) -> Vec<OperationInfo>
    where
        F: Fn(&OperationInfo) -> bool,
    {
        let operations = self.operations.read().await;
        operations
            .values()
            .filter(|op| predicate(op))
            .cloned()
            .collect()
    }

    /// Get all active operations
    pub async fn get_active_operations(&self) -> Vec<OperationInfo> {
        self.get_operations(|op| op.is_active()).await
    }

    /// Get all completed operations
    pub async fn get_completed_operations(&self) -> Vec<OperationInfo> {
        self.get_operations(|op| !op.is_active()).await
    }

    /// Get operation statistics
    pub async fn get_statistics(&self) -> OperationStatistics {
        let operations = self.operations.read().await;
        let mut stats = OperationStatistics::default();

        for operation in operations.values() {
            stats.total += 1;

            match operation.state {
                OperationState::Pending => stats.pending += 1,
                OperationState::Running => stats.running += 1,
                OperationState::Completed => stats.completed += 1,
                OperationState::Failed => stats.failed += 1,
                OperationState::Cancelled => stats.cancelled += 1,
                OperationState::TimedOut => stats.timed_out += 1,
            }

            if !operation.is_active() {
                stats.total_duration += operation.duration();
            }
        }

        if stats.completed + stats.failed > 0 {
            stats.average_duration = stats.total_duration / (stats.completed + stats.failed) as u32;
        }

        stats
    }

    /// Execute an operation with monitoring, timeout, and cancellation support
    pub async fn execute_with_monitoring<F, Fut>(
        &self,
        command: String,
        description: String,
        timeout_duration: Option<Duration>,
        working_directory: Option<String>,
        callback: Option<Box<dyn CallbackSender>>,
        operation: F,
    ) -> Result<String, String>
    where
        F: FnOnce(String, CancellationToken) -> Fut,
        Fut: std::future::Future<Output = Result<String, String>>,
    {
        // Register the operation
        let operation_id = self
            .register_operation(
                command.clone(),
                description.clone(),
                timeout_duration,
                working_directory,
            )
            .await;

        // Get the cancellation token
        let cancellation_token = {
            let operations = self.operations.read().await;
            operations
                .get(&operation_id)
                .map(|op| op.cancellation_token.clone())
                .ok_or_else(|| "Operation registration failed".to_string())?
        };

        // Start the operation
        self.start_operation(&operation_id).await?;

        // Send start notification via callback
        if let Some(ref callback) = callback {
            let _ = callback
                .send_progress(ProgressUpdate::Started {
                    operation_id: operation_id.clone(),
                    command: command.clone(),
                    description: description.clone(),
                })
                .await;
        }

        // Execute with timeout and cancellation
        let timeout_duration = timeout_duration.unwrap_or(self.config.default_timeout);
        let result = timeout(
            timeout_duration,
            operation(operation_id.clone(), cancellation_token.clone()),
        )
        .await;

        let final_result = match result {
            Ok(operation_result) => operation_result,
            Err(_) => {
                // Timeout occurred
                self.complete_operation(&operation_id, Err("Operation timed out".to_string()))
                    .await?;

                if let Some(ref callback) = callback {
                    let duration = {
                        let operations = self.operations.read().await;
                        operations
                            .get(&operation_id)
                            .map(|op| op.duration().as_millis() as u64)
                            .unwrap_or(0)
                    };

                    let _ = callback
                        .send_progress(ProgressUpdate::Failed {
                            operation_id: operation_id.clone(),
                            error: "Operation timed out".to_string(),
                            duration_ms: duration,
                        })
                        .await;
                }

                return Err("Operation timed out".to_string());
            }
        };

        // Complete the operation
        self.complete_operation(&operation_id, final_result.clone())
            .await?;

        // Send completion notification via callback
        if let Some(ref callback) = callback {
            let duration = {
                let operations = self.operations.read().await;
                operations
                    .get(&operation_id)
                    .map(|op| op.duration().as_millis() as u64)
                    .unwrap_or(0)
            };

            let progress_update = match &final_result {
                Ok(msg) => ProgressUpdate::Completed {
                    operation_id: operation_id.clone(),
                    message: msg.clone(),
                    duration_ms: duration,
                },
                Err(err) => ProgressUpdate::Failed {
                    operation_id: operation_id.clone(),
                    error: err.clone(),
                    duration_ms: duration,
                },
            };

            let _ = callback.send_progress(progress_update).await;
        }

        final_result
    }

    /// Start the cleanup task for removing old operations
    fn start_cleanup_task(&self) {
        let operations = Arc::clone(&self.operations);
        let completion_history = Arc::clone(&self.completion_history);
        let config = self.config.clone();
        let cleanup_token = self.cleanup_token.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(config.cleanup_interval);

            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        Self::cleanup_operations(&operations, &completion_history, &config).await;
                    }
                    _ = cleanup_token.cancelled() => {
                        debug!("Operation cleanup task cancelled");
                        break;
                    }
                }
            }
        });
    }

    /// Clean up old completed operations
    async fn cleanup_operations(
        operations: &Arc<RwLock<HashMap<String, OperationInfo>>>,
        completion_history: &Arc<RwLock<HashMap<String, OperationInfo>>>,
        config: &MonitorConfig,
    ) {
        let mut ops = operations.write().await;
        let initial_count = ops.len();

        // Check for timed-out operations
        let now = Instant::now();
        let mut timed_out_ops = Vec::new();

        for (id, operation) in ops.iter_mut() {
            if operation.is_active() {
                if let Some(timeout_duration) = operation.timeout_duration {
                    if operation.start_time.elapsed() > timeout_duration {
                        operation.timeout();
                        timed_out_ops.push(id.clone());
                    }
                }
            }
        }

        for id in timed_out_ops {
            warn!("Operation {} timed out and was cancelled", id);
        }

        // Remove old completed operations if we exceed the history limit
        if ops.len() > config.max_history_size {
            let mut completed_ops: Vec<_> = ops
                .iter()
                .filter(|(_, op)| !op.is_active())
                .map(|(id, op)| (id.clone(), op.end_time.unwrap_or(now)))
                .collect();

            // Sort by end time (oldest first)
            completed_ops.sort_by_key(|(_, end_time)| *end_time);

            let to_remove = ops.len() - config.max_history_size;
            for (id, _) in completed_ops.into_iter().take(to_remove) {
                // Before removing from operations, ensure it's in completion history
                if let Some(operation) = ops.get(&id) {
                    if !operation.is_active() {
                        let mut completion_history = completion_history.write().await;
                        completion_history.insert(id.clone(), operation.clone());
                    }
                }
                ops.remove(&id);
            }

            let final_count = ops.len();
            if final_count < initial_count {
                debug!(
                    "Cleaned up {} old operations (from {} to {})",
                    initial_count - final_count,
                    initial_count,
                    final_count
                );
            }
        }

        // Also limit completion history size to prevent unbounded growth
        let mut completion_history = completion_history.write().await;
        if completion_history.len() > config.max_completion_history_size {
            let mut completed_ops: Vec<_> = completion_history
                .iter()
                .map(|(id, op)| (id.clone(), op.end_time.unwrap_or(now)))
                .collect();

            // Sort by end time (oldest first)
            completed_ops.sort_by_key(|(_, end_time)| *end_time);

            let to_remove = completion_history.len() - config.max_completion_history_size;
            for (id, _) in completed_ops.into_iter().take(to_remove) {
                completion_history.remove(&id);
            }

            debug!("Cleaned up {} old completion history entries", to_remove);
        }
    }

    /// Wait for a specific operation to complete
    /// This method never fails - it always returns helpful information even for unknown operations
    pub async fn wait_for_operation(
        &self,
        operation_id: &str,
    ) -> Result<Vec<OperationInfo>, String> {
        // Validate operation ID
        if operation_id.is_empty() || operation_id.trim().is_empty() {
            // Create a helpful info message for empty/invalid IDs
            let info = OperationInfo {
                id: operation_id.to_string(),
                command: "unknown".to_string(),
                description:
                    "No operation found with empty ID. Please provide a valid operation ID."
                        .to_string(),
                state: OperationState::Failed,
                start_time: Instant::now(),
                end_time: Some(Instant::now()),
                timeout_duration: None,
                result: Some(Err(
                    "Invalid operation ID: empty or whitespace-only ID provided".to_string(),
                )),
                working_directory: None,
                cancellation_token: CancellationToken::new(),
            };
            return Ok(vec![info]);
        }

        // First check if the operation is already completed in the completion history
        {
            let completion_history = self.completion_history.read().await;
            if let Some(completed_operation) = completion_history.get(operation_id) {
                return Ok(vec![completed_operation.clone()]);
            }
        }

        // If not in completion history, check active operations
        loop {
            if let Some(operation) = self.get_operation(operation_id).await {
                match operation.state {
                    OperationState::Completed
                    | OperationState::Failed
                    | OperationState::Cancelled => {
                        return Ok(vec![operation]);
                    }
                    _ => {
                        // Operation is still in progress, wait a bit
                        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                    }
                }
            } else {
                // Check completion history one more time in case operation completed
                // and was cleaned up while we were checking
                let completion_history = self.completion_history.read().await;
                if let Some(completed_operation) = completion_history.get(operation_id) {
                    return Ok(vec![completed_operation.clone()]);
                }

                // Operation not found - provide helpful information instead of error
                let info = OperationInfo {
                    id: operation_id.to_string(),
                    command: "unknown".to_string(),
                    description: format!(
                        "No operation found with ID '{}'. It may be very old and cleaned up, or the ID may be incorrect. Please check the operation ID or use wait without operation_id to see all active operations.",
                        operation_id
                    ),
                    state: OperationState::Failed,
                    start_time: Instant::now(),
                    end_time: Some(Instant::now()),
                    timeout_duration: None,
                    result: Some(Ok(format!(
                        "No operation found with ID '{}'. This could mean:\n\
                        • The operation completed long ago and was cleaned up\n\
                        • The operation ID is incorrect or mistyped\n\
                        • The operation never existed\n\
                        To see current operations, use wait without specifying an operation ID.",
                        operation_id
                    ))),
                    working_directory: None,
                    cancellation_token: CancellationToken::new(),
                };
                return Ok(vec![info]);
            }
        }
    }

    /// Wait for all active operations to complete
    pub async fn wait_for_all_operations(&self) -> Result<Vec<OperationInfo>, String> {
        loop {
            let active_ops = self.get_active_operations().await;
            if active_ops.is_empty() {
                // Return all completed operations
                return Ok(self.get_completed_operations().await);
            }
            // Wait a bit before checking again
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
    }

    /// Stop the operation monitor and clean up resources
    pub async fn shutdown(&self) {
        info!("Shutting down operation monitor");

        // Cancel all active operations
        let active_ops = self.get_active_operations().await;
        for operation in active_ops {
            let _ = self.cancel_operation(&operation.id).await;
        }

        // Stop the cleanup task
        self.cleanup_token.cancel();

        info!("Operation monitor shutdown complete");
    }
}

impl Drop for OperationMonitor {
    fn drop(&mut self) {
        // Cancel the cleanup task
        self.cleanup_token.cancel();
    }
}

/// Statistics about operations
#[derive(Debug, Default, Clone)]
pub struct OperationStatistics {
    pub total: usize,
    pub pending: usize,
    pub running: usize,
    pub completed: usize,
    pub failed: usize,
    pub cancelled: usize,
    pub timed_out: usize,
    pub total_duration: Duration,
    pub average_duration: Duration,
}

impl OperationStatistics {
    /// Get the success rate as a percentage
    pub fn success_rate(&self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            (self.completed as f64 / self.total as f64) * 100.0
        }
    }

    /// Get the failure rate as a percentage
    pub fn failure_rate(&self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            ((self.failed + self.cancelled + self.timed_out) as f64 / self.total as f64) * 100.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_operation_registration() {
        let monitor = OperationMonitor::new(MonitorConfig::default());

        let id = monitor
            .register_operation("test".to_string(), "Test operation".to_string(), None, None)
            .await;

        let operation = monitor.get_operation(&id).await.unwrap();
        assert_eq!(operation.command, "test");
        assert_eq!(operation.description, "Test operation");
        assert_eq!(operation.state, OperationState::Pending);
    }

    #[tokio::test]
    async fn test_operation_lifecycle() {
        let monitor = OperationMonitor::new(MonitorConfig::default());

        let id = monitor
            .register_operation("test".to_string(), "Test operation".to_string(), None, None)
            .await;

        // Start operation
        monitor.start_operation(&id).await.unwrap();
        let operation = monitor.get_operation(&id).await.unwrap();
        assert_eq!(operation.state, OperationState::Running);

        // Complete operation
        monitor
            .complete_operation(&id, Ok("Success".to_string()))
            .await
            .unwrap();
        let operation = monitor.get_operation(&id).await.unwrap();
        assert_eq!(operation.state, OperationState::Completed);
        assert!(operation.result.is_some());
    }

    #[tokio::test]
    async fn test_operation_cancellation() {
        let monitor = OperationMonitor::new(MonitorConfig::default());

        let id = monitor
            .register_operation("test".to_string(), "Test operation".to_string(), None, None)
            .await;

        monitor.start_operation(&id).await.unwrap();
        monitor.cancel_operation(&id).await.unwrap();

        let operation = monitor.get_operation(&id).await.unwrap();
        assert_eq!(operation.state, OperationState::Cancelled);
        assert!(operation.cancellation_token.is_cancelled());
    }

    #[tokio::test]
    async fn test_execute_with_monitoring() {
        let monitor = OperationMonitor::new(MonitorConfig::default());

        let result = monitor
            .execute_with_monitoring(
                "test".to_string(),
                "Test operation".to_string(),
                Some(Duration::from_secs(1)),
                None,
                None,
                |_id, _token| async { Ok("Success".to_string()) },
            )
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Success");
    }

    #[tokio::test]
    async fn test_operation_timeout() {
        let monitor = OperationMonitor::new(MonitorConfig::default());

        let result = monitor
            .execute_with_monitoring(
                "test".to_string(),
                "Test operation".to_string(),
                Some(Duration::from_millis(100)), // Very short timeout
                None,
                None,
                |_id, _token| async {
                    sleep(Duration::from_millis(200)).await; // Longer than timeout
                    Ok("Should not complete".to_string())
                },
            )
            .await;

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("timed out"));
    }

    #[tokio::test]
    async fn test_operation_statistics() {
        let monitor = OperationMonitor::new(MonitorConfig::default());

        // Register and complete some operations
        let id1 = monitor
            .register_operation("test1".to_string(), "Test 1".to_string(), None, None)
            .await;
        monitor.start_operation(&id1).await.unwrap();
        monitor
            .complete_operation(&id1, Ok("Success".to_string()))
            .await
            .unwrap();

        let id2 = monitor
            .register_operation("test2".to_string(), "Test 2".to_string(), None, None)
            .await;
        monitor.start_operation(&id2).await.unwrap();
        monitor
            .complete_operation(&id2, Err("Failed".to_string()))
            .await
            .unwrap();

        let stats = monitor.get_statistics().await;
        assert_eq!(stats.total, 2);
        assert_eq!(stats.completed, 1);
        assert_eq!(stats.failed, 1);
        assert_eq!(stats.success_rate(), 50.0);
        assert_eq!(stats.failure_rate(), 50.0);
    }

    #[tokio::test]
    async fn test_monitor_shutdown() {
        let monitor = OperationMonitor::new(MonitorConfig::default());

        let id = monitor
            .register_operation("test".to_string(), "Test".to_string(), None, None)
            .await;
        monitor.start_operation(&id).await.unwrap();

        monitor.shutdown().await;

        let operation = monitor.get_operation(&id).await.unwrap();
        assert_eq!(operation.state, OperationState::Cancelled);
    }

    #[tokio::test]
    async fn test_wait_for_already_completed_operation() {
        // This test reproduces the race condition bug where waiting for an operation
        // that has already completed returns an error instead of success.
        let monitor = OperationMonitor::new(MonitorConfig::default());

        // Register and immediately complete an operation
        let id = monitor
            .register_operation("test".to_string(), "Test operation".to_string(), None, None)
            .await;

        monitor.start_operation(&id).await.unwrap();
        monitor
            .complete_operation(&id, Ok("Success".to_string()))
            .await
            .unwrap();

        // Simulate the case where automatic cleanup might remove the operation
        // For now, just test that waiting for a completed operation works
        let result = monitor.wait_for_operation(&id).await;

        // This should succeed and return the completed operation info
        // Currently it will succeed, but let's test the more complex case
        assert!(result.is_ok());
        let operations = result.unwrap();
        assert_eq!(operations.len(), 1);
        assert_eq!(operations[0].state, OperationState::Completed);
    }

    #[tokio::test]
    async fn test_wait_for_operation_cleaned_up_by_automatic_cleanup() {
        // This test specifically reproduces the race condition where an operation
        // is completed and then cleaned up by the automatic cleanup process
        // before a wait call is made.
        let config = MonitorConfig {
            max_history_size: 0, // Force immediate cleanup of completed operations from main map
            max_completion_history_size: 100, // But keep completion history for wait operations
            auto_cleanup: true,
            ..Default::default()
        };

        let monitor = OperationMonitor::new(config);

        // Register, start, and complete an operation
        let id = monitor
            .register_operation("test".to_string(), "Test operation".to_string(), None, None)
            .await;

        monitor.start_operation(&id).await.unwrap();
        monitor
            .complete_operation(&id, Ok("Success".to_string()))
            .await
            .unwrap();

        // Force cleanup by manually calling it (simulating what the background task does)
        {
            let operations = std::sync::Arc::clone(&monitor.operations);
            let completion_history = std::sync::Arc::clone(&monitor.completion_history);
            let config = monitor.config.clone();
            OperationMonitor::cleanup_operations(&operations, &completion_history, &config).await;
        }

        // Now try to wait for the operation - this should return success indicating
        // it was already completed, not an error
        let result = monitor.wait_for_operation(&id).await;

        // After our fix, this should succeed and return the completed operation
        assert!(result.is_ok());
        let operations = result.unwrap();
        assert_eq!(operations.len(), 1);
        assert_eq!(operations[0].state, OperationState::Completed);
        assert_eq!(operations[0].id, id);
    }

    #[tokio::test]
    async fn test_wait_for_nonexistent_operation_never_fails() {
        let monitor = OperationMonitor::new(MonitorConfig::default());

        // This should never return an error, even for non-existent operation IDs
        let result = monitor.wait_for_operation("nonexistent_id").await;

        // Should now succeed with helpful information instead of returning an error
        assert!(result.is_ok(), "wait_for_operation should never fail");
        let operations = result.unwrap();
        assert_eq!(operations.len(), 1);
        
        // Should contain helpful information about the missing operation
        let op_info = &operations[0];
        assert_eq!(op_info.id, "nonexistent_id");
        assert!(op_info.description.contains("No operation found"));
        assert!(op_info.description.contains("nonexistent_id"));
        
        // Should have a helpful result message
        if let Some(Ok(message)) = &op_info.result {
            assert!(message.contains("No operation found"));
            assert!(message.contains("nonexistent_id"));
        } else {
            panic!("Expected helpful result message for missing operation");
        }
    }

    #[tokio::test]
    async fn test_wait_for_invalid_operation_id_never_fails() {
        let monitor = OperationMonitor::new(MonitorConfig::default());

        // Test with various invalid operation IDs
        let invalid_ids = vec!["", "   ", "invalid-id", "123", "op_!@#$%"];

        for invalid_id in invalid_ids {
            let result = monitor.wait_for_operation(invalid_id).await;
            // Should now succeed with helpful information instead of returning an error
            assert!(result.is_ok(), "wait_for_operation should never fail for invalid ID '{}'", invalid_id);
            
            let operations = result.unwrap();
            assert_eq!(operations.len(), 1);
            
            let op_info = &operations[0];
            assert_eq!(op_info.id, invalid_id);
            
            // Empty/whitespace IDs should get specific handling
            if invalid_id.trim().is_empty() {
                assert!(op_info.description.contains("empty ID"));
            } else {
                assert!(op_info.description.contains("No operation found"));
            }
        }
    }

    #[tokio::test]
    async fn test_wait_returns_full_result_for_completed_operation() {
        let monitor = OperationMonitor::new(MonitorConfig::default());

        let id = monitor
            .register_operation("test".to_string(), "Test operation".to_string(), None, None)
            .await;

        monitor.start_operation(&id).await.unwrap();
        let test_result = "Operation completed successfully with detailed output";
        monitor
            .complete_operation(&id, Ok(test_result.to_string()))
            .await
            .unwrap();

        // Wait should return the full result even if operation is already completed
        let result = monitor.wait_for_operation(&id).await.unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].state, OperationState::Completed);

        // Should contain the full result content
        if let Some(Ok(message)) = &result[0].result {
            assert_eq!(message, test_result);
        } else {
            panic!("Expected completed operation result");
        }
    }

    #[tokio::test]
    async fn test_wait_handles_operation_cleaned_from_completion_history() {
        let config = MonitorConfig {
            max_history_size: 0,
            max_completion_history_size: 0, // Force immediate cleanup from completion history too
            auto_cleanup: true,
            ..Default::default()
        };

        let monitor = OperationMonitor::new(config);

        let id = monitor
            .register_operation("test".to_string(), "Test operation".to_string(), None, None)
            .await;

        monitor.start_operation(&id).await.unwrap();
        monitor
            .complete_operation(&id, Ok("Success".to_string()))
            .await
            .unwrap();

        // Force cleanup of both main operations and completion history
        {
            let operations = std::sync::Arc::clone(&monitor.operations);
            let completion_history = std::sync::Arc::clone(&monitor.completion_history);
            let config_ref = monitor.config.clone();
            OperationMonitor::cleanup_operations(&operations, &completion_history, &config_ref).await;
        }

        // Wait should handle this gracefully and never return an error
        let result = monitor.wait_for_operation(&id).await;
        // Should now succeed with helpful information instead of returning an error
        assert!(result.is_ok(), "wait_for_operation should never fail even when operation is cleaned up");
        
        let operations = result.unwrap();
        assert_eq!(operations.len(), 1);
        
        // Should contain helpful information about the missing/cleaned operation
        let op_info = &operations[0];
        assert_eq!(op_info.id, id);
        assert!(op_info.description.contains("No operation found"));
    }

    #[tokio::test]
    async fn test_concurrent_wait_operations_same_id() {
        let monitor = std::sync::Arc::new(OperationMonitor::new(MonitorConfig::default()));

        let id = monitor
            .register_operation("test".to_string(), "Test operation".to_string(), None, None)
            .await;

        monitor.start_operation(&id).await.unwrap();

        // Launch multiple concurrent wait operations for the same ID
        let monitor_clone1 = std::sync::Arc::clone(&monitor);
        let monitor_clone2 = std::sync::Arc::clone(&monitor);
        let monitor_clone3 = std::sync::Arc::clone(&monitor);
        let id_clone1 = id.clone();
        let id_clone2 = id.clone();
        let id_clone3 = id.clone();

        let wait1 =
            tokio::spawn(async move { monitor_clone1.wait_for_operation(&id_clone1).await });
        let wait2 =
            tokio::spawn(async move { monitor_clone2.wait_for_operation(&id_clone2).await });
        let wait3 =
            tokio::spawn(async move { monitor_clone3.wait_for_operation(&id_clone3).await });

        // Give waits time to start
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        // Complete the operation
        monitor
            .complete_operation(&id, Ok("Success".to_string()))
            .await
            .unwrap();

        // All waits should succeed
        let result1 = wait1.await.unwrap().unwrap();
        let result2 = wait2.await.unwrap().unwrap();
        let result3 = wait3.await.unwrap().unwrap();

        assert_eq!(result1.len(), 1);
        assert_eq!(result2.len(), 1);
        assert_eq!(result3.len(), 1);
        assert_eq!(result1[0].state, OperationState::Completed);
        assert_eq!(result2[0].state, OperationState::Completed);
        assert_eq!(result3[0].state, OperationState::Completed);
    }

    #[tokio::test]
    async fn test_long_cleanup_timeout_config() {
        // Test that 6-hour cleanup timeout can be set
        let config = MonitorConfig {
            cleanup_interval: Duration::from_secs(21600), // 6 hours
            ..Default::default()
        };

        let monitor = OperationMonitor::new(config);

        let id = monitor
            .register_operation("test".to_string(), "Test operation".to_string(), None, None)
            .await;

        monitor.start_operation(&id).await.unwrap();
        monitor
            .complete_operation(&id, Ok("Success".to_string()))
            .await
            .unwrap();

        // Operation should still be accessible since cleanup interval is very long
        let result = monitor.wait_for_operation(&id).await.unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].state, OperationState::Completed);
    }
}
