//! Asynchronous callback system for monitoring cargo operation progress
//!
//! This module provides a flexible callback architecture for tracking the progress of
//! long-running cargo operations. It enables real-time progress updates, output streaming,
//! and completion notifications through various callback mechanisms.
//!
//! ## Key Components
//!
//! - [`ProgressUpdate`]: Enumeration of different progress event types
//! - [`CallbackSender`]: Trait for implementing custom progress callback handlers
//! - [`ChannelCallbackSender`]: Channel-based callback implementation for async communication
//! - [`LoggingCallbackSender`]: Simple logging-based callback for debugging and monitoring
//!
//! ## Usage Example
//!
//! ```rust,no_run
//! use async_cargo_mcp::callback_system::{CallbackSender, LoggingCallbackSender, ProgressUpdate};
//!
//! #[tokio::main]
//! async fn main() {
//!     let callback: Box<dyn CallbackSender> = Box::new(
//!         LoggingCallbackSender::new("cargo_build_001".to_string())
//!     );
//!     
//!     // Send progress updates during a cargo operation
//!     let update = ProgressUpdate::Started {
//!         operation_id: "cargo_build_001".to_string(),
//!         command: "cargo build".to_string(),
//!         description: "Building project dependencies".to_string(),
//!     };
//!     
//!     callback.send_progress(update).await;
//! }
//! ```

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::fmt;
use tokio::sync::mpsc;

/// Represents different types of progress updates during cargo operations
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum ProgressUpdate {
    /// Operation has started
    Started {
        operation_id: String,
        command: String,
        description: String,
    },
    /// Progress update with optional percentage and message
    Progress {
        operation_id: String,
        message: String,
        percentage: Option<f64>,
        current_step: Option<String>,
    },
    /// Output line from the cargo command
    Output {
        operation_id: String,
        line: String,
        is_stderr: bool,
    },
    /// Operation completed successfully
    Completed {
        operation_id: String,
        message: String,
        duration_ms: u64,
    },
    /// Operation failed with error
    Failed {
        operation_id: String,
        error: String,
        duration_ms: u64,
    },
    /// Operation was cancelled
    Cancelled {
        operation_id: String,
        message: String,
        duration_ms: u64,
    },
    /// Final comprehensive result with all details (like wait command output)
    FinalResult {
        operation_id: String,
        command: String,
        description: String,
        working_directory: String,
        success: bool,
        duration_ms: u64,
        full_output: String,
    },
}

impl fmt::Display for ProgressUpdate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProgressUpdate::Started {
                operation_id,
                command,
                description,
            } => {
                write!(f, "[{operation_id}] Started: {command} - {description}")
            }
            ProgressUpdate::Progress {
                operation_id,
                message,
                percentage,
                current_step,
            } => {
                let progress_str = match percentage {
                    Some(p) => format!(" ({p:.1}%)"),
                    None => String::new(),
                };
                let step_str = match current_step {
                    Some(s) => format!(" [{s}]"),
                    None => String::new(),
                };
                write!(
                    f,
                    "[{operation_id}] Progress{progress_str}: {message}{step_str}"
                )
            }
            ProgressUpdate::Output {
                operation_id,
                line,
                is_stderr,
            } => {
                let stream = if *is_stderr { "stderr" } else { "stdout" };
                write!(f, "[{operation_id}] {stream}: {line}")
            }
            ProgressUpdate::Completed {
                operation_id,
                message,
                duration_ms,
            } => {
                write!(
                    f,
                    "[{operation_id}] Completed in {duration_ms}ms: {message}"
                )
            }
            ProgressUpdate::Failed {
                operation_id,
                error,
                duration_ms,
            } => {
                write!(f, "[{operation_id}] Failed after {duration_ms}ms: {error}")
            }
            ProgressUpdate::Cancelled {
                operation_id,
                message,
                duration_ms,
            } => {
                write!(
                    f,
                    "[{operation_id}] CANCELLED after {duration_ms}ms: {message}"
                )
            }
            ProgressUpdate::FinalResult {
                operation_id,
                command,
                success,
                full_output,
                ..
            } => {
                let status = if *success { "COMPLETED" } else { "FAILED" };
                write!(f, "[{operation_id}] {status}: {command}\n{full_output}")
            }
        }
    }
}

impl ProgressUpdate {
    /// Check if this update represents a terminal state (operation is complete)
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            ProgressUpdate::Completed { .. }
                | ProgressUpdate::Failed { .. }
                | ProgressUpdate::Cancelled { .. }
                | ProgressUpdate::FinalResult { .. }
        )
    }

    /// Check if this update represents a successful completion
    pub fn is_success(&self) -> bool {
        match self {
            ProgressUpdate::Completed { .. } => true,
            ProgressUpdate::FinalResult { success, .. } => *success,
            _ => false,
        }
    }

    /// Check if this update represents a failure (failed or cancelled)
    pub fn is_failure(&self) -> bool {
        match self {
            ProgressUpdate::Failed { .. } | ProgressUpdate::Cancelled { .. } => true,
            ProgressUpdate::FinalResult { success, .. } => !*success,
            _ => false,
        }
    }

    /// Get the operation ID from any update variant
    pub fn operation_id(&self) -> &str {
        match self {
            ProgressUpdate::Started { operation_id, .. }
            | ProgressUpdate::Progress { operation_id, .. }
            | ProgressUpdate::Output { operation_id, .. }
            | ProgressUpdate::Completed { operation_id, .. }
            | ProgressUpdate::Failed { operation_id, .. }
            | ProgressUpdate::Cancelled { operation_id, .. }
            | ProgressUpdate::FinalResult { operation_id, .. } => operation_id,
        }
    }

    /// Get the duration in milliseconds if this update contains timing information
    pub fn duration_ms(&self) -> Option<u64> {
        match self {
            ProgressUpdate::Completed { duration_ms, .. }
            | ProgressUpdate::Failed { duration_ms, .. }
            | ProgressUpdate::Cancelled { duration_ms, .. }
            | ProgressUpdate::FinalResult { duration_ms, .. } => Some(*duration_ms),
            _ => None,
        }
    }

    /// Get the variant name as a string for logging and debugging
    pub fn variant_name(&self) -> &'static str {
        match self {
            ProgressUpdate::Started { .. } => "Started",
            ProgressUpdate::Progress { .. } => "Progress",
            ProgressUpdate::Output { .. } => "Output",
            ProgressUpdate::Completed { .. } => "Completed",
            ProgressUpdate::Failed { .. } => "Failed",
            ProgressUpdate::Cancelled { .. } => "Cancelled",
            ProgressUpdate::FinalResult { .. } => "FinalResult",
        }
    }
}

/// Trait for sending progress updates during cargo operations
/// This allows for different callback implementations (MCP notifications, logging, etc.)
#[async_trait]
pub trait CallbackSender: Send + Sync {
    /// Send a progress update
    async fn send_progress(&self, update: ProgressUpdate) -> Result<(), CallbackError>;

    /// Check if the operation should be cancelled
    async fn should_cancel(&self) -> bool;

    /// Send multiple progress updates in sequence
    async fn send_batch(&self, updates: Vec<ProgressUpdate>) -> Result<(), CallbackError> {
        for update in updates {
            self.send_progress(update).await?;
        }
        Ok(())
    }
}

/// Errors that can occur when sending callbacks
#[derive(Debug, thiserror::Error)]
pub enum CallbackError {
    #[error("Failed to send progress update: {0}")]
    SendFailed(String),
    #[error("Callback receiver disconnected")]
    Disconnected,
    #[error("Operation was cancelled")]
    Cancelled,
    #[error("Callback timeout: {0}")]
    Timeout(String),
}

impl CallbackError {
    /// Check if this error represents a potentially recoverable condition
    pub fn is_recoverable(&self) -> bool {
        matches!(
            self,
            CallbackError::SendFailed(_) | CallbackError::Timeout(_)
        )
    }

    /// Check if this error was caused by user action
    pub fn is_user_initiated(&self) -> bool {
        matches!(self, CallbackError::Cancelled)
    }

    /// Get error code for programmatic handling
    pub fn error_code(&self) -> &'static str {
        match self {
            CallbackError::SendFailed(_) => "SEND_FAILED",
            CallbackError::Disconnected => "DISCONNECTED",
            CallbackError::Cancelled => "CANCELLED",
            CallbackError::Timeout(_) => "TIMEOUT",
        }
    }

    /// Get severity level for logging
    pub fn severity(&self) -> &'static str {
        match self {
            CallbackError::SendFailed(_)
            | CallbackError::Disconnected
            | CallbackError::Timeout(_) => "ERROR",
            CallbackError::Cancelled => "WARN", // User action, less severe
        }
    }

    /// Get detailed message if available
    pub fn message_detail(&self) -> Option<&str> {
        match self {
            CallbackError::SendFailed(msg) | CallbackError::Timeout(msg) => Some(msg),
            CallbackError::Disconnected | CallbackError::Cancelled => None,
        }
    }
}

/// Channel-based callback sender for async communication
pub struct ChannelCallbackSender {
    sender: mpsc::UnboundedSender<ProgressUpdate>,
    cancellation_token: tokio_util::sync::CancellationToken,
}

impl ChannelCallbackSender {
    pub fn new(
        sender: mpsc::UnboundedSender<ProgressUpdate>,
        cancellation_token: tokio_util::sync::CancellationToken,
    ) -> Self {
        Self {
            sender,
            cancellation_token,
        }
    }
}

#[async_trait]
impl CallbackSender for ChannelCallbackSender {
    async fn send_progress(&self, update: ProgressUpdate) -> Result<(), CallbackError> {
        self.sender
            .send(update)
            .map_err(|_| CallbackError::Disconnected)?;
        Ok(())
    }

    async fn should_cancel(&self) -> bool {
        self.cancellation_token.is_cancelled()
    }
}

/// No-op callback sender for when progress updates are not needed
pub struct NoOpCallbackSender;

#[async_trait]
impl CallbackSender for NoOpCallbackSender {
    async fn send_progress(&self, _update: ProgressUpdate) -> Result<(), CallbackError> {
        Ok(())
    }

    async fn should_cancel(&self) -> bool {
        false
    }
}

/// Logging callback sender that writes progress to the log
pub struct LoggingCallbackSender {
    operation_name: String,
}

impl LoggingCallbackSender {
    pub fn new(operation_name: String) -> Self {
        Self { operation_name }
    }
}

#[async_trait]
impl CallbackSender for LoggingCallbackSender {
    async fn send_progress(&self, update: ProgressUpdate) -> Result<(), CallbackError> {
        tracing::debug!("{}: {update}", self.operation_name);
        Ok(())
    }

    async fn should_cancel(&self) -> bool {
        false
    }
}

/// Utility function to create a no-op callback sender
pub fn no_callback() -> Box<dyn CallbackSender> {
    Box::new(NoOpCallbackSender)
}

/// Utility function to create a logging callback sender
pub fn logging_callback(operation_name: String) -> Box<dyn CallbackSender> {
    Box::new(LoggingCallbackSender::new(operation_name))
}

/// Utility function to create a channel-based callback sender with receiver
pub fn channel_callback(
    cancellation_token: tokio_util::sync::CancellationToken,
) -> (
    Box<dyn CallbackSender>,
    mpsc::UnboundedReceiver<ProgressUpdate>,
) {
    let (sender, receiver) = mpsc::unbounded_channel();
    let callback = Box::new(ChannelCallbackSender::new(sender, cancellation_token));
    (callback, receiver)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{Duration, timeout};

    #[tokio::test]
    async fn test_no_op_callback() {
        let callback = no_callback();
        let update = ProgressUpdate::Started {
            operation_id: "test".to_string(),
            command: "cargo build".to_string(),
            description: "Building project".to_string(),
        };

        assert!(callback.send_progress(update).await.is_ok());
        assert!(!callback.should_cancel().await);
    }

    #[tokio::test]
    async fn test_channel_callback() {
        let token = tokio_util::sync::CancellationToken::new();
        let (callback, mut receiver) = channel_callback(token.clone());

        let update = ProgressUpdate::Progress {
            operation_id: "test".to_string(),
            message: "Building...".to_string(),
            percentage: Some(50.0),
            current_step: Some("Compiling".to_string()),
        };

        callback.send_progress(update.clone()).await.unwrap();

        let received = timeout(Duration::from_millis(100), receiver.recv())
            .await
            .unwrap()
            .unwrap();

        match (&update, &received) {
            (
                ProgressUpdate::Progress {
                    operation_id: id1, ..
                },
                ProgressUpdate::Progress {
                    operation_id: id2, ..
                },
            ) => {
                assert_eq!(id1, id2);
            }
            _ => panic!("Unexpected update type"),
        }
    }

    #[tokio::test]
    async fn test_cancellation() {
        let token = tokio_util::sync::CancellationToken::new();
        let (callback, _receiver) = channel_callback(token.clone());

        assert!(!callback.should_cancel().await);

        token.cancel();

        assert!(callback.should_cancel().await);
    }
}
