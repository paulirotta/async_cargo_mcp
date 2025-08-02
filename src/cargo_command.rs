//! Cargo command handling module with asynchronous support

use anyhow::Result;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, Clone)]
pub enum CargoCommand {
    Increment,
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
    pub fn new(command: CargoCommand, args: Vec<String>) -> Self {
        CargoCommandRequest {
            command,
            args,
            counter: Arc::new(Mutex::new(0)),
        }
    }

    pub async fn execute(&self, spawn: bool) -> Result<()> {
        if spawn {
            match self.command {
                CargoCommand::Increment => {
                    let counter = self.counter.clone();
                    tokio::spawn(async move {
                        let mut counter = counter.lock().await;
                        *counter += 1;
                        println!("Increment command executed, counter: {}", *counter);
                    });
                }

                CargoCommand::Build => {
                    let args = self.args.clone();
                    let counter = self.counter.clone();
                    tokio::spawn(async move {
                        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                        let mut counter = counter.lock().await;
                        *counter += 1;
                        println!(
                            "Build command executed concurrently with args: {:?}, counter: {}",
                            args, *counter
                        );
                    });
                }

                CargoCommand::Run => {
                    let args = self.args.clone();
                    tokio::spawn(async move {
                        println!("Run command executed with args: {:?}", args);
                    });
                }

                CargoCommand::Test => {
                    let args = self.args.clone();
                    tokio::spawn(async move {
                        println!("Test command executed with args: {:?}", args);
                    });
                }

                CargoCommand::Check => {
                    let args = self.args.clone();
                    tokio::spawn(async move {
                        println!("Check command executed with args: {:?}", args);
                    });
                }

                CargoCommand::Add => {
                    let args = self.args.clone();
                    tokio::spawn(async move {
                        println!("Add command executed with args: {:?}", args);
                    });
                }

                CargoCommand::Remove => {
                    let args = self.args.clone();
                    tokio::spawn(async move {
                        println!("Remove command executed with args: {:?}", args);
                    });
                }

                CargoCommand::Update => {
                    let args = self.args.clone();
                    tokio::spawn(async move {
                        println!("Update command executed with args: {:?}", args);
                    });
                }
                _ => {}
            }

            Ok(())
        } else {
            match self.command {
                CargoCommand::Increment => self.increment().await,
                CargoCommand::Build => self.build().await,
                CargoCommand::Run => self.run().await,
                CargoCommand::Test => self.test().await,
                CargoCommand::Check => self.check().await,
                CargoCommand::Add => self.add().await,
                CargoCommand::Remove => self.remove().await,
                CargoCommand::Update => self.update().await,
                _ => Ok(()),
            }
        }
    }

    async fn increment(&self) -> Result<()> {
        let mut counter = self.counter.lock().await;
        *counter += 1;
        println!("Increment command executed, counter: {}", *counter);
        Ok(())
    }

    async fn build(&self) -> Result<()> {
        // Simulate build operation synchronously
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        let mut counter = self.counter.lock().await;
        *counter += 1;
        println!(
            "Build command executed synchronously with args: {:?}, counter: {}",
            self.args, *counter
        );
        Ok(())
    }

    async fn run(&self) -> Result<()> {
        println!("Run command executed with args: {:?}", self.args);
        Ok(())
    }

    async fn test(&self) -> Result<()> {
        println!("Test command executed with args: {:?}", self.args);
        Ok(())
    }

    async fn check(&self) -> Result<()> {
        println!("Check command executed with args: {:?}", self.args);
        Ok(())
    }

    async fn add(&self) -> Result<()> {
        println!("Add command executed with args: {:?}", self.args);
        Ok(())
    }

    async fn remove(&self) -> Result<()> {
        println!("Remove command executed with args: {:?}", self.args);
        Ok(())
    }

    async fn update(&self) -> Result<()> {
        println!("Update command executed with args: {:?}", self.args);
        Ok(())
    }
}
