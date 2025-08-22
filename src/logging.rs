//! Logging initialization utilities.
//!
//! Provides a single entry point `init_subscriber` that configures tracing for either
//! stderr (default) or a rolling daily log file. Verbose mode enables debug level.
//! The function is idempotent: subsequent calls after the first are no-ops so tests
//! or library reuse won't panic on duplicate initialization.

use directories::ProjectDirs;
use std::{fs, path::PathBuf, sync::Once};
use tracing_subscriber::{EnvFilter, fmt};

static INIT: Once = Once::new();

/// Initialize the global tracing subscriber.
///
/// * `log_to_file` - if true, write logs to a rolling daily file under the user's cache dir
/// * `verbose` - if true, set global log level to `debug`, otherwise `info`
pub fn init_subscriber(log_to_file: bool, verbose: bool) {
    // Only allow one-time initialization; ignore later calls.
    INIT.call_once(|| {
        let level = if verbose { "debug" } else { "info" };
        let env_filter =
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(level));

        if log_to_file
            && let Some(proj) = ProjectDirs::from("dev", "async_cargo_mcp", "async_cargo_mcp")
        {
            let mut log_dir = PathBuf::from(proj.cache_dir());
            log_dir.push("logs");
            if let Err(e) = fs::create_dir_all(&log_dir) {
                eprintln!("Failed to create log dir {:?}: {e}", log_dir);
            }
            let file_appender = tracing_appender::rolling::daily(&log_dir, "server.log");
            let (nb, guard) = tracing_appender::non_blocking(file_appender);
            // Keep guard alive for program lifetime to ensure flushing.
            Box::leak(Box::new(guard));
            fmt()
                .with_env_filter(env_filter)
                .with_writer(nb)
                .with_ansi(false)
                .with_target(false)
                .init();
            tracing::debug!(
                "Logging initialized (file mode) verbose={} dir={:?}",
                verbose,
                log_dir
            );
            return;
        }

        fmt()
            .with_env_filter(env_filter)
            .with_writer(std::io::stderr)
            .with_ansi(true)
            .with_target(false)
            .init();
        tracing::debug!("Logging initialized (stderr mode) verbose={}", verbose);
    });
}
