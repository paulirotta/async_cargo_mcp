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
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Mutex;

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct StructRequest {
    pub a: i32,
    pub b: i32,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct DependencyRequest {
    pub name: String,
    pub version: Option<String>,
    pub features: Option<Vec<String>>,
    pub optional: Option<bool>,
    pub working_directory: Option<String>,
    /// Enable async callback notifications for operation progress
    pub enable_async_notifications: Option<bool>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct RemoveDependencyRequest {
    pub name: String,
    pub working_directory: Option<String>,
    /// Enable async callback notifications for operation progress
    pub enable_async_notifications: Option<bool>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct BuildRequest {
    pub working_directory: Option<String>,
    /// Enable async callback notifications for operation progress
    pub enable_async_notifications: Option<bool>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct RunRequest {
    pub working_directory: Option<String>,
    /// Enable async callback notifications for operation progress
    pub enable_async_notifications: Option<bool>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct TestRequest {
    pub working_directory: Option<String>,
    /// Enable async callback notifications for operation progress
    pub enable_async_notifications: Option<bool>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct CheckRequest {
    pub working_directory: Option<String>,
    /// Enable async callback notifications for operation progress
    pub enable_async_notifications: Option<bool>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct UpdateRequest {
    pub working_directory: Option<String>,
    /// Enable async callback notifications for operation progress
    pub enable_async_notifications: Option<bool>,
}

#[derive(Clone, Debug)]
pub struct AsyncCargo {
    counter: Arc<Mutex<i32>>,
    tool_router: ToolRouter<AsyncCargo>,
}

#[tool_router]
impl AsyncCargo {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            counter: Arc::new(Mutex::new(0)),
            tool_router: Self::tool_router(),
        }
    }

    fn _create_resource_text(&self, uri: &str, name: &str) -> Resource {
        RawResource::new(uri, name.to_string()).no_annotation()
    }

    /// Increment the counter and return the new value
    async fn next_operation_id(&self) -> i32 {
        let mut counter = self.counter.lock().await;
        *counter += 1;
        *counter
    }

    #[tool(description = "Increment the counter by 1")]
    async fn increment(&self) -> Result<CallToolResult, McpError> {
        tracing::info!("=== INCREMENT TOOL CALLED ===");
        let mut counter = self.counter.lock().await;
        *counter += 1;
        let result = CallToolResult::success(vec![Content::text(counter.to_string())]);
        tracing::info!("Increment result: {:?}", result);
        tracing::info!("=== INCREMENT TOOL RETURNING ===");
        Ok(result)
    }

    #[tool(description = "Decrement the counter by 1")]
    async fn decrement(&self) -> Result<CallToolResult, McpError> {
        let mut counter = self.counter.lock().await;
        *counter -= 1;
        Ok(CallToolResult::success(vec![Content::text(
            counter.to_string(),
        )]))
    }

    #[tool(description = "Get the current counter value")]
    async fn get_value(&self) -> Result<CallToolResult, McpError> {
        let counter = self.counter.lock().await;
        Ok(CallToolResult::success(vec![Content::text(
            counter.to_string(),
        )]))
    }

    #[tool(description = "Say hello to the client")]
    fn say_hello(&self) -> Result<CallToolResult, McpError> {
        Ok(CallToolResult::success(vec![Content::text("hello")]))
    }

    #[tool(description = "Repeat what you say")]
    fn echo(&self, Parameters(object): Parameters<JsonObject>) -> Result<CallToolResult, McpError> {
        Ok(CallToolResult::success(vec![Content::text(
            serde_json::Value::Object(object).to_string(),
        )]))
    }

    #[tool(description = "Calculate the sum of two numbers")]
    fn sum(
        &self,
        Parameters(StructRequest { a, b }): Parameters<StructRequest>,
    ) -> Result<CallToolResult, McpError> {
        Ok(CallToolResult::success(vec![Content::text(
            (a + b).to_string(),
        )]))
    }

    #[tool(description = "Build the Rust project using cargo build")]
    async fn build(
        &self,
        Parameters(req): Parameters<BuildRequest>,
    ) -> Result<CallToolResult, McpError> {
        let build_id = self.next_operation_id().await;

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

            // Set working directory if provided
            if let Some(working_dir) = &req.working_directory {
                cmd.current_dir(working_dir);
            }

            let output = cmd.output().await.map_err(|e| {
                McpError::internal_error(format!("Failed to execute cargo build: {}", e), None)
            })?;

            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);

            let working_dir_msg = req
                .working_directory
                .as_ref()
                .map(|dir| format!(" in {}", dir))
                .unwrap_or_default();

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

    #[tool(description = "Run the Rust project using cargo run")]
    async fn run(
        &self,
        Parameters(req): Parameters<RunRequest>,
    ) -> Result<CallToolResult, McpError> {
        use tokio::process::Command;

        let run_id = self.next_operation_id().await;

        // TODO: Add asynchronous callback mechanism here for runtime output streaming
        // Implementation plan:
        // 1. Use tokio::process::Command::spawn() to start the process without blocking
        // 2. Stream stdout/stderr in real-time to provide live output to the LLM
        // 3. Handle long-running processes that might need user interaction
        // 4. Provide option to terminate running processes via MCP commands
        // 5. Support for interactive applications through bidirectional communication

        let mut cmd = Command::new("cargo");
        cmd.arg("run");

        // Set working directory if provided
        if let Some(working_dir) = &req.working_directory {
            cmd.current_dir(working_dir);
        }

        let output = cmd.output().await.map_err(|e| {
            McpError::internal_error(format!("Failed to execute cargo run: {}", e), None)
        })?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        let working_dir_msg = req
            .working_directory
            .as_ref()
            .map(|dir| format!(" in {}", dir))
            .unwrap_or_default();

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

    #[tool(description = "Run tests for the Rust project using cargo test")]
    async fn test(
        &self,
        Parameters(req): Parameters<TestRequest>,
    ) -> Result<CallToolResult, McpError> {
        use tokio::process::Command;

        let test_id = self.next_operation_id().await;

        // TODO: Add asynchronous callback mechanism here for test progress updates
        // Implementation plan:
        // 1. Stream test execution results in real-time as they complete
        // 2. Provide progress indicators for test suites (e.g., "Running 15/30 tests")
        // 3. Send immediate notifications for test failures with detailed error info
        // 4. Allow LLM to see which specific tests are running/passing/failing
        // 5. Support for parallel test execution feedback

        let mut cmd = Command::new("cargo");
        cmd.arg("test");

        // Set working directory if provided
        if let Some(working_dir) = &req.working_directory {
            cmd.current_dir(working_dir);
        }

        let output = cmd.output().await.map_err(|e| {
            McpError::internal_error(format!("Failed to execute cargo test: {}", e), None)
        })?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        let working_dir_msg = req
            .working_directory
            .as_ref()
            .map(|dir| format!(" in {}", dir))
            .unwrap_or_default();

        let result_msg = if output.status.success() {
            format!(
                "✅ Test operation #{test_id} completed successfully{working_dir_msg}.\nOutput: {stdout}"
            )
        } else {
            format!(
                "❌ Test operation #{test_id} failed{working_dir_msg}.\nErrors: {stderr}\nOutput: {stdout}"
            )
        };

        Ok(CallToolResult::success(vec![Content::text(result_msg)]))
    }

    #[tool(description = "Check the Rust project for errors using cargo check")]
    async fn check(
        &self,
        Parameters(req): Parameters<CheckRequest>,
    ) -> Result<CallToolResult, McpError> {
        use tokio::process::Command;

        let check_id = self.next_operation_id().await;

        // TODO: Add asynchronous callback mechanism here for check progress updates
        // Implementation plan:
        // 1. Stream compilation check results as they become available
        // 2. Send immediate warnings and errors to the LLM during checking
        // 3. Provide progress indicators for large projects
        // 4. Allow early termination on first error if requested
        // 5. Include suggestion hints from compiler alongside error messages

        let mut cmd = Command::new("cargo");
        cmd.arg("check");

        // Set working directory if provided
        if let Some(working_dir) = &req.working_directory {
            cmd.current_dir(working_dir);
        }

        let output = cmd.output().await.map_err(|e| {
            McpError::internal_error(format!("Failed to execute cargo check: {}", e), None)
        })?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        let working_dir_msg = req
            .working_directory
            .as_ref()
            .map(|dir| format!(" in {}", dir))
            .unwrap_or_default();

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

    #[tool(description = "Add a dependency to the Rust project using cargo add")]
    async fn add(
        &self,
        Parameters(req): Parameters<DependencyRequest>,
    ) -> Result<CallToolResult, McpError> {
        let add_id = self.next_operation_id().await;

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

            // Set working directory if provided
            if let Some(working_dir) = &req.working_directory {
                cmd.current_dir(working_dir);
            }

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

            let working_dir_msg = req
                .working_directory
                .as_ref()
                .map(|dir| format!(" in {}", dir))
                .unwrap_or_default();

            let result_msg = if output.status.success() {
                format!(
                    "✅ Add operation #{add_id} completed successfully{working_dir_msg}.\nAdded dependency: {}\nOutput: {stdout}",
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

    #[tool(description = "Remove a dependency from the Rust project using cargo remove")]
    async fn remove(
        &self,
        Parameters(req): Parameters<RemoveDependencyRequest>,
    ) -> Result<CallToolResult, McpError> {
        let remove_id = self.next_operation_id().await;

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

            // Set working directory if provided
            if let Some(working_dir) = &req.working_directory {
                cmd.current_dir(working_dir);
            }

            let output = cmd.output().await.map_err(|e| {
                McpError::internal_error(format!("Failed to execute cargo remove: {}", e), None)
            })?;

            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);

            let working_dir_msg = req
                .working_directory
                .as_ref()
                .map(|dir| format!(" in {}", dir))
                .unwrap_or_default();

            let result_msg = if output.status.success() {
                format!(
                    "✅ Remove operation #{remove_id} completed successfully{working_dir_msg}.\nRemoved dependency: {}\nOutput: {stdout}",
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

    #[tool(description = "Update dependencies in the Rust project using cargo update")]
    async fn update(
        &self,
        Parameters(req): Parameters<UpdateRequest>,
    ) -> Result<CallToolResult, McpError> {
        use tokio::process::Command;

        let update_id = self.next_operation_id().await;

        // TODO: Add asynchronous callback mechanism here for dependency update progress
        // Implementation plan:
        // 1. Stream update progress and version changes to the LLM in real-time
        // 2. Show which dependencies are being updated and to what versions
        // 3. Provide warnings about breaking changes or compatibility issues
        // 4. Allow selective updates if the LLM requests specific package updates
        // This would allow streaming update progress and version changes to the LLM

        let mut cmd = Command::new("cargo");
        cmd.arg("update");

        // Set working directory if provided
        if let Some(working_dir) = &req.working_directory {
            cmd.current_dir(working_dir);
        }

        let output = cmd.output().await.map_err(|e| {
            McpError::internal_error(format!("Failed to execute cargo update: {}", e), None)
        })?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        let working_dir_msg = req
            .working_directory
            .as_ref()
            .map(|dir| format!(" in {}", dir))
            .unwrap_or_default();

        let result_msg = if output.status.success() {
            format!(
                "✅ Update operation #{update_id} completed successfully{working_dir_msg}.\nOutput: {stdout}"
            )
        } else {
            format!(
                "❌ Update operation #{update_id} failed{working_dir_msg}.\nErrors: {stderr}\nOutput: {stdout}"
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
            instructions: Some("This server provides a counter tool that can increment and decrement values. The counter starts at 0 and can be modified using the 'increment' and 'decrement' tools. Use 'get_value' to check the current count.".to_string()),
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

        let result = self.get_info();
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

        let operation_id = self.next_operation_id().await.to_string();
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

        // Set working directory if provided
        if let Some(working_dir) = &req.working_directory {
            cmd.current_dir(working_dir);
        }

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

        let working_dir_msg = req
            .working_directory
            .as_ref()
            .map(|dir| format!(" in {}", dir))
            .unwrap_or_default();

        if output.status.success() {
            let success_msg = format!(
                "✅ Add operation completed successfully{working_dir_msg}.\nAdded dependency: {}\nOutput: {stdout}",
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

        let operation_id = self.next_operation_id().await.to_string();
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

        // Set working directory if provided
        if let Some(working_dir) = &req.working_directory {
            cmd.current_dir(working_dir);
        }

        // Execute command and collect full output
        let output = cmd
            .output()
            .await
            .map_err(|e| format!("Failed to execute cargo remove: {}", e))?;

        let duration_ms = start_time.elapsed().as_millis() as u64;
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        let working_dir_msg = req
            .working_directory
            .as_ref()
            .map(|dir| format!(" in {}", dir))
            .unwrap_or_default();

        if output.status.success() {
            let success_msg = format!(
                "✅ Remove operation completed successfully{working_dir_msg}.\nRemoved dependency: {}\nOutput: {stdout}",
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

        let operation_id = self.next_operation_id().await.to_string();
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

        // Set working directory if provided
        if let Some(working_dir) = &req.working_directory {
            cmd.current_dir(working_dir);
        }

        // Execute command and collect full output
        let output = cmd
            .output()
            .await
            .map_err(|e| format!("Failed to execute cargo build: {}", e))?;

        let duration_ms = start_time.elapsed().as_millis() as u64;
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        let working_dir_msg = req
            .working_directory
            .as_ref()
            .map(|dir| format!(" in {}", dir))
            .unwrap_or_default();

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
}
