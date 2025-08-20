//! Logging configuration for the application.
use anyhow::Result;
use directories::ProjectDirs;
use tracing_subscriber::{EnvFilter, Layer, fmt, layer::SubscriberExt, util::SubscriberInitExt};

/// Initializes the tracing subscriber with optional file and stderr logging.
///
/// # Arguments
///
/// * `log_to_file` - If `true`, logs will be written to a file in the project's log directory.
/// * `verbose` - If `true`, info-level logs will be written to stderr.
///
/// # Returns
///
/// * `Ok(())` if the subscriber was initialized successfully.
/// * `Err(anyhow::Error)` if the project directories could not be determined.
pub fn init_subscriber(log_to_file: bool, verbose: bool) -> Result<()> {
    let mut layers = Vec::new();

    // File logger layer
    if log_to_file {
        if let Some(proj_dirs) = ProjectDirs::from("com", "async_cargo_mcp", "async_cargo_mcp") {
            let log_dir = proj_dirs.data_local_dir();
            let file_appender = tracing_appender::rolling::daily(log_dir, "async_cargo_mcp.log");
            let file_layer = fmt::layer()
                .with_writer(file_appender)
                .with_ansi(false)
                .with_filter(EnvFilter::new("info"));
            layers.push(file_layer.boxed());
        } else {
            return Err(anyhow::anyhow!("Could not determine project directories"));
        }
    }

    // Stderr logger layer
    let stderr_filter = if verbose {
        EnvFilter::new("info")
    } else {
        EnvFilter::new("warn") // Only show warnings and errors by default
    };

    let stderr_layer = fmt::layer()
        .with_writer(std::io::stderr)
        .with_ansi(true)
        .with_target(false)
        .with_filter(stderr_filter);
    layers.push(stderr_layer.boxed());

    tracing_subscriber::registry().with(layers).init();

    Ok(())
}
