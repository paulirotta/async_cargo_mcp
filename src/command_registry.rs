//! Dynamic command registration and discovery system for cargo commands
//!
//! This module provides an extensible architecture for dynamically registering
//! and discovering cargo commands and subcommands. It enables auto-discovery
//! of available cargo functionality and provides a trait-based interface for
//! easy extension.

use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;
use std::process::Stdio;
use tokio::process::Command;
use tracing::{error, info, warn};

/// Result type for command operations
pub type CommandResult = Result<String, String>;

/// Trait defining the interface for cargo command implementations
#[async_trait]
pub trait CargoCommand: Send + Sync {
    /// Get the command name (e.g., "build", "test", "add")
    fn name(&self) -> &str;

    /// Get the command description for help text
    fn description(&self) -> &str;

    /// Get the JSON schema for command parameters
    fn parameter_schema(&self) -> Value;

    /// Execute the command with given parameters and optional working directory
    async fn execute(&self, params: Value, working_directory: Option<&str>) -> CommandResult;

    /// Validate parameters before execution (optional)
    async fn validate_params(&self, params: &Value) -> Result<(), String> {
        // Default implementation does no validation
        let _ = params;
        Ok(())
    }

    /// Get command aliases (optional)
    fn aliases(&self) -> Vec<&str> {
        vec![]
    }

    /// Check if this command is available on the current system
    async fn is_available(&self) -> bool {
        // Default: check if cargo is available
        Command::new("cargo")
            .arg("--version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .await
            .map(|status| status.success())
            .unwrap_or(false)
    }
}

/// Registry for managing cargo commands
pub struct CommandRegistry {
    commands: HashMap<String, Box<dyn CargoCommand>>,
    aliases: HashMap<String, String>, // alias -> command_name
}

impl CommandRegistry {
    /// Create a new command registry
    pub fn new() -> Self {
        Self {
            commands: HashMap::new(),
            aliases: HashMap::new(),
        }
    }

    /// Register a command in the registry
    pub fn register<T: CargoCommand + 'static>(&mut self, command: T) {
        let name = command.name().to_string();

        // Register aliases
        for alias in command.aliases() {
            self.aliases.insert(alias.to_string(), name.clone());
        }

        info!("Registered cargo command: {}", name);
        self.commands.insert(name, Box::new(command));
    }

    /// Get a command by name or alias
    pub fn get_command(&self, name: &str) -> Option<&dyn CargoCommand> {
        // First try direct lookup
        if let Some(command) = self.commands.get(name) {
            return Some(command.as_ref());
        }

        // Then try alias lookup
        if let Some(real_name) = self.aliases.get(name) {
            return self.commands.get(real_name).map(|c| c.as_ref());
        }

        None
    }

    /// Get all registered command names
    pub fn command_names(&self) -> Vec<&str> {
        self.commands.keys().map(|s| s.as_str()).collect()
    }

    /// Get all registered commands
    pub fn commands(&self) -> Vec<&dyn CargoCommand> {
        self.commands.values().map(|c| c.as_ref()).collect()
    }

    /// Execute a command by name
    pub async fn execute_command(
        &self,
        name: &str,
        params: Value,
        working_directory: Option<&str>,
    ) -> CommandResult {
        match self.get_command(name) {
            Some(command) => {
                // Validate parameters first
                if let Err(e) = command.validate_params(&params).await {
                    return Err(format!("Parameter validation failed: {}", e));
                }

                // Execute the command
                command.execute(params, working_directory).await
            }
            None => Err(format!("Unknown command: {}", name)),
        }
    }

    /// Auto-discover available cargo subcommands
    pub async fn auto_discover_commands(&mut self) {
        info!("Auto-discovering available cargo commands...");

        let output = match Command::new("cargo").arg("--list").output().await {
            Ok(output) if output.status.success() => {
                String::from_utf8_lossy(&output.stdout).to_string()
            }
            Ok(output) => {
                warn!(
                    "cargo --list failed: {}",
                    String::from_utf8_lossy(&output.stderr)
                );
                return;
            }
            Err(e) => {
                error!("Failed to run cargo --list: {}", e);
                return;
            }
        };

        let mut discovered_count = 0;
        for line in output.lines() {
            if let Some(command_name) = self.parse_cargo_list_line(line) {
                if !self.commands.contains_key(&command_name) {
                    // Create a generic command for discovered commands
                    let generic_command = GenericCargoCommand::new(command_name.clone());
                    if generic_command.is_available().await {
                        self.register(generic_command);
                        discovered_count += 1;
                    }
                }
            }
        }

        info!("Auto-discovered {} new cargo commands", discovered_count);
    }

    /// Parse a line from `cargo --list` output
    fn parse_cargo_list_line(&self, line: &str) -> Option<String> {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with("Installed Commands:") {
            return None;
        }

        // Lines typically look like: "    build            Compile the current package"
        if let Some(first_word) = trimmed.split_whitespace().next() {
            // Skip if it's not a valid command name (contains special chars)
            if first_word
                .chars()
                .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
            {
                return Some(first_word.to_string());
            }
        }

        None
    }

    /// Get help text for a command
    pub async fn get_command_help(&self, name: &str) -> Option<String> {
        if let Some(command) = self.get_command(name) {
            Some(format!(
                "Command: {}\nDescription: {}\nSchema: {}",
                command.name(),
                command.description(),
                serde_json::to_string_pretty(&command.parameter_schema()).unwrap_or_default()
            ))
        } else {
            None
        }
    }

    /// Check availability of all registered commands
    pub async fn check_availability(&self) -> HashMap<String, bool> {
        let mut results = HashMap::new();

        for (name, command) in &self.commands {
            let available = command.is_available().await;
            results.insert(name.clone(), available);
            if !available {
                warn!("Command '{}' is not available", name);
            }
        }

        results
    }
}

impl Default for CommandRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Generic command implementation for auto-discovered cargo commands
pub struct GenericCargoCommand {
    name: String,
    description: String,
}

impl GenericCargoCommand {
    pub fn new(name: String) -> Self {
        let description = format!("Auto-discovered cargo command: {}", name);
        Self { name, description }
    }
}

#[async_trait]
impl CargoCommand for GenericCargoCommand {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn parameter_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "args": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Additional arguments to pass to the cargo command"
                },
                "working_directory": {
                    "type": "string",
                    "description": "Working directory for the command"
                }
            }
        })
    }

    async fn execute(&self, params: Value, working_directory: Option<&str>) -> CommandResult {
        let mut cmd = Command::new("cargo");
        cmd.arg(&self.name);

        // Add additional arguments if provided
        if let Some(args) = params.get("args").and_then(|v| v.as_array()) {
            for arg in args {
                if let Some(arg_str) = arg.as_str() {
                    cmd.arg(arg_str);
                }
            }
        }

        // Set working directory
        if let Some(wd) =
            working_directory.or_else(|| params.get("working_directory").and_then(|v| v.as_str()))
        {
            cmd.current_dir(wd);
        }

        let output = cmd
            .output()
            .await
            .map_err(|e| format!("Failed to execute cargo {}: {}", self.name, e))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        if output.status.success() {
            Ok(format!(
                "cargo {} completed successfully.\nOutput: {}",
                self.name, stdout
            ))
        } else {
            Err(format!(
                "❌ cargo {} failed.\nError: {}\nOutput: {}",
                self.name, stderr, stdout
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    struct MockCommand {
        name: String,
        available: bool,
    }

    impl MockCommand {
        fn new(name: &str, available: bool) -> Self {
            Self {
                name: name.to_string(),
                available,
            }
        }
    }

    #[async_trait]
    impl CargoCommand for MockCommand {
        fn name(&self) -> &str {
            &self.name
        }

        fn description(&self) -> &str {
            "Mock command for testing"
        }

        fn parameter_schema(&self) -> Value {
            json!({ "type": "object" })
        }

        async fn execute(&self, _params: Value, _working_directory: Option<&str>) -> CommandResult {
            Ok(format!("Mock execution of {}", self.name))
        }

        async fn is_available(&self) -> bool {
            self.available
        }

        fn aliases(&self) -> Vec<&str> {
            if self.name == "build" {
                vec!["b"]
            } else {
                vec![]
            }
        }
    }

    #[tokio::test]
    async fn test_command_registration() {
        let mut registry = CommandRegistry::new();

        let mock_command = MockCommand::new("test", true);
        registry.register(mock_command);

        assert!(registry.get_command("test").is_some());
        assert!(registry.get_command("nonexistent").is_none());
    }

    #[tokio::test]
    async fn test_command_aliases() {
        let mut registry = CommandRegistry::new();

        let mock_command = MockCommand::new("build", true);
        registry.register(mock_command);

        // Test both name and alias
        assert!(registry.get_command("build").is_some());
        assert!(registry.get_command("b").is_some());
    }

    #[tokio::test]
    async fn test_command_execution() {
        let mut registry = CommandRegistry::new();

        let mock_command = MockCommand::new("test", true);
        registry.register(mock_command);

        let result = registry.execute_command("test", json!({}), None).await;
        assert!(result.is_ok());
        assert!(result.unwrap().contains("Mock execution of test"));
    }

    #[tokio::test]
    async fn test_availability_check() {
        let mut registry = CommandRegistry::new();

        registry.register(MockCommand::new("available", true));
        registry.register(MockCommand::new("unavailable", false));

        let availability = registry.check_availability().await;

        assert_eq!(availability.get("available"), Some(&true));
        assert_eq!(availability.get("unavailable"), Some(&false));
    }

    #[test]
    fn test_cargo_list_parsing() {
        let registry = CommandRegistry::new();

        // Test various cargo --list output formats
        assert_eq!(
            registry.parse_cargo_list_line("    build            Compile the current package"),
            Some("build".to_string())
        );
        assert_eq!(
            registry.parse_cargo_list_line("    test"),
            Some("test".to_string())
        );
        assert_eq!(
            registry.parse_cargo_list_line("    cargo-expand     Expand macros"),
            Some("cargo-expand".to_string())
        );
        assert_eq!(registry.parse_cargo_list_line(""), None);
        assert_eq!(registry.parse_cargo_list_line("Installed Commands:"), None);
    }
}
