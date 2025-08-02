//! Cargo command handling module with asynchronous support

use anyhow::Result;
use rmcp::tool;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, Clone)]
pub enum CargoCommand {
    Increment,
    Decriment,
    GetValue,
    Build,
    Run,
    Test,
    Check,
    Clippy,
    Add,
    Remove,
    Update,
}

#[derive(Debug, Clone)]
pub struct CargoCommandRequest {
    pub command: CargoCommand,
    pub description: String,
    pub args: Vec<String>,
    pub counter: Arc<Mutex<i32>>,
}

#[derive(Debug, Clone)]
pub enum CargoCommandResult {
    RunningAsync,
    Success(String),
    Failure(String),
}

impl CargoCommandRequest {
    pub fn new(command: CargoCommand, description: String, args: Vec<String>) -> Self {
        CargoCommandRequest {
            command,
            description,
            args,
            counter: Arc::new(Mutex::new(0)),
        }
    }

    pub async fn execute(&self, spawn: bool) -> Result<CargoCommandResult> {
        if spawn {
            match self.command {
                CargoCommand::Increment => {
                    let req = self.clone();
                    tokio::spawn(async move {
                        let _ = req.increment().await;
                    });
                }
                CargoCommand::Decriment => {
                    let req = self.clone();
                    tokio::spawn(async move {
                        let _ = req.decriment().await;
                    });
                }
                CargoCommand::GetValue => {
                    let req = self.clone();
                    tokio::spawn(async move {
                        let _ = req.get_value().await;
                    });
                }
                CargoCommand::Build => {
                    let req = self.clone();
                    tokio::spawn(async move {
                        let _ = req.build().await;
                    });
                }
                CargoCommand::Run => {
                    let req = self.clone();
                    tokio::spawn(async move {
                        let _ = req.run().await;
                    });
                }
                CargoCommand::Test => {
                    let req = self.clone();
                    tokio::spawn(async move {
                        let _ = req.test().await;
                    });
                }
                CargoCommand::Check => {
                    let req = self.clone();
                    tokio::spawn(async move {
                        let _ = req.check().await;
                    });
                }
                CargoCommand::Add => {
                    let req = self.clone();
                    tokio::spawn(async move {
                        let _ = req.add().await;
                    });
                }
                CargoCommand::Remove => {
                    let req = self.clone();
                    tokio::spawn(async move {
                        let _ = req.remove().await;
                    });
                }
                CargoCommand::Update => {
                    let req = self.clone();
                    tokio::spawn(async move {
                        let _ = req.update().await;
                    });
                }
                _ => {}
            }
            Ok(CargoCommandResult::RunningAsync)
        } else {
            match self.command {
                CargoCommand::Increment => self.increment().await,
                CargoCommand::Decriment => self.decriment().await,
                CargoCommand::GetValue => self.get_value().await,
                CargoCommand::Build => self.build().await,
                CargoCommand::Run => self.run().await,
                CargoCommand::Test => self.test().await,
                CargoCommand::Check => self.check().await,
                CargoCommand::Add => self.add().await,
                CargoCommand::Remove => self.remove().await,
                CargoCommand::Update => self.update().await,
                _ => Ok(CargoCommandResult::Failure("Unknown command".to_string())),
            }
        }
    }

    #[tool(description = "Decrement the counter and return the result")]
    async fn decriment(&self) -> Result<CargoCommandResult> {
        let mut counter = self.counter.lock().await;
        *counter -= 1;
        let msg = format!(
            "'Decriment' command executed: {:?}, counter: {}",
            self, *counter
        );
        println!("{}", msg);

        Ok(CargoCommandResult::Success(msg))
    }

    #[tool(description = "Get the current counter value")]
    async fn get_value(&self) -> Result<CargoCommandResult> {
        let counter = self.counter.lock().await;
        let msg = format!(
            "'GetValue' command executed: {:?}, counter: {}",
            self, *counter
        );
        println!("{}", msg);

        Ok(CargoCommandResult::Success(msg))
    }

    #[tool(description = "Increment the counter and return the result")]
    async fn increment(&self) -> Result<CargoCommandResult> {
        let mut counter = self.counter.lock().await;
        *counter += 1;
        let msg = format!(
            "'Increment' command executed: {:?}, counter: {}",
            self, *counter
        );
        println!("{}", msg);

        Ok(CargoCommandResult::Success(msg))
    }

    #[tool(description = "Build the Rust project using cargo build")]
    async fn build(&self) -> Result<CargoCommandResult> {
        // Simulate build operation synchronously
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        let mut counter = self.counter.lock().await;
        *counter += 1;
        let msg = format!(
            "'Build' command executed synchronously: {:?}, counter: {}",
            self, *counter
        );
        println!("{}", msg);

        Ok(CargoCommandResult::Success(msg))
    }

    #[tool(description = "Run the Rust project using cargo run")]
    async fn run(&self) -> Result<CargoCommandResult> {
        let msg = format!("'Run' command executed: {:?}", self);
        println!("{}", msg);

        Ok(CargoCommandResult::Success(msg))
    }

    #[tool(description = "Run tests for the Rust project using cargo test")]
    async fn test(&self) -> Result<CargoCommandResult> {
        let msg = format!("'Test' command executed: {:?}", self);
        println!("{}", msg);

        Ok(CargoCommandResult::Success(msg))
    }

    #[tool(description = "Check the Rust project for errors using cargo check")]
    async fn check(&self) -> Result<CargoCommandResult> {
        let msg = format!("'Check' command executed: {:?}", self);
        println!("{}", msg);

        Ok(CargoCommandResult::Success(msg))
    }

    #[tool(description = "Add a dependency to the Rust project using cargo add")]
    async fn add(&self) -> Result<CargoCommandResult> {
        let msg = format!("'Add' command executed: {:?}", self);
        println!("{}", msg);

        Ok(CargoCommandResult::Success(msg))
    }

    #[tool(description = "Remove a dependency from the Rust project using cargo remove")]
    async fn remove(&self) -> Result<CargoCommandResult> {
        let msg = format!("'Remove' command executed: {:?}", self);
        println!("{}", msg);

        Ok(CargoCommandResult::Success(msg))
    }

    #[tool(description = "Update dependencies in the Rust project using cargo update")]
    async fn update(&self) -> Result<CargoCommandResult> {
        let msg = format!("'Update' command executed: {:?}", self);
        println!("{}", msg);

        Ok(CargoCommandResult::Success(msg))
    }
}
