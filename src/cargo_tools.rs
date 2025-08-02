#![allow(dead_code)]
use std::sync::Arc;

use rmcp::{
    ErrorData as McpError, RoleServer, ServerHandler,
    handler::server::{router::tool::ToolRouter, tool::Parameters},
    model::*,
    schemars,
    service::RequestContext,
    tool, tool_handler, tool_router,
};
use serde_json::json;
use tokio::sync::Mutex;

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct StructRequest {
    pub a: i32,
    pub b: i32,
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
    async fn build(&self) -> Result<CallToolResult, McpError> {
        use tokio::process::Command;

        let mut counter = self.counter.lock().await;
        *counter += 1;
        let build_id = *counter;
        drop(counter);

        let output = Command::new("cargo")
            .arg("build")
            .output()
            .await
            .map_err(|e| {
                McpError::internal_error(format!("Failed to execute cargo build: {}", e), None)
            })?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        let result_msg = if output.status.success() {
            format!(
                "Build #{} completed successfully.\nOutput: {}",
                build_id, stdout
            )
        } else {
            format!(
                "Build #{} failed.\nStderr: {}\nStdout: {}",
                build_id, stderr, stdout
            )
        };

        Ok(CallToolResult::success(vec![Content::text(result_msg)]))
    }

    #[tool(description = "Run the Rust project using cargo run")]
    async fn run(&self) -> Result<CallToolResult, McpError> {
        use tokio::process::Command;

        let mut counter = self.counter.lock().await;
        *counter += 1;
        let run_id = *counter;
        drop(counter);

        let output = Command::new("cargo")
            .arg("run")
            .output()
            .await
            .map_err(|e| {
                McpError::internal_error(format!("Failed to execute cargo run: {}", e), None)
            })?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        let result_msg = if output.status.success() {
            format!(
                "Run #{} completed successfully.\nOutput: {}",
                run_id, stdout
            )
        } else {
            format!(
                "Run #{} failed.\nStderr: {}\nStdout: {}",
                run_id, stderr, stdout
            )
        };

        Ok(CallToolResult::success(vec![Content::text(result_msg)]))
    }

    #[tool(description = "Run tests for the Rust project using cargo test")]
    async fn test(&self) -> Result<CallToolResult, McpError> {
        use tokio::process::Command;

        let mut counter = self.counter.lock().await;
        *counter += 1;
        let test_id = *counter;
        drop(counter);

        let output = Command::new("cargo")
            .arg("test")
            .output()
            .await
            .map_err(|e| {
                McpError::internal_error(format!("Failed to execute cargo test: {}", e), None)
            })?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        let result_msg = if output.status.success() {
            format!(
                "Test #{} completed successfully.\nOutput: {}",
                test_id, stdout
            )
        } else {
            format!(
                "Test #{} failed.\nStderr: {}\nStdout: {}",
                test_id, stderr, stdout
            )
        };

        Ok(CallToolResult::success(vec![Content::text(result_msg)]))
    }

    #[tool(description = "Check the Rust project for errors using cargo check")]
    async fn check(&self) -> Result<CallToolResult, McpError> {
        use tokio::process::Command;

        let mut counter = self.counter.lock().await;
        *counter += 1;
        let check_id = *counter;
        drop(counter);

        let output = Command::new("cargo")
            .arg("check")
            .output()
            .await
            .map_err(|e| {
                McpError::internal_error(format!("Failed to execute cargo check: {}", e), None)
            })?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        let result_msg = if output.status.success() {
            format!(
                "Check #{} completed successfully.\nOutput: {}",
                check_id, stdout
            )
        } else {
            format!(
                "Check #{} failed.\nStderr: {}\nStdout: {}",
                check_id, stderr, stdout
            )
        };

        Ok(CallToolResult::success(vec![Content::text(result_msg)]))
    }

    #[tool(description = "Add a dependency to the Rust project using cargo add")]
    async fn add(&self) -> Result<CallToolResult, McpError> {
        let mut counter = self.counter.lock().await;
        *counter += 1;
        let add_id = *counter;
        drop(counter);

        // Note: This is a simple implementation. In a real scenario, you'd want to accept
        // the dependency name as a parameter
        let result_msg = format!(
            "Add #{} - This tool needs to be called with a dependency name parameter",
            add_id
        );

        Ok(CallToolResult::success(vec![Content::text(result_msg)]))
    }

    #[tool(description = "Remove a dependency from the Rust project using cargo remove")]
    async fn remove(&self) -> Result<CallToolResult, McpError> {
        let mut counter = self.counter.lock().await;
        *counter += 1;
        let remove_id = *counter;
        drop(counter);

        // Note: This is a simple implementation. In a real scenario, you'd want to accept
        // the dependency name as a parameter
        let result_msg = format!(
            "Remove #{} - This tool needs to be called with a dependency name parameter",
            remove_id
        );

        Ok(CallToolResult::success(vec![Content::text(result_msg)]))
    }

    #[tool(description = "Update dependencies in the Rust project using cargo update")]
    async fn update(&self) -> Result<CallToolResult, McpError> {
        use tokio::process::Command;

        let mut counter = self.counter.lock().await;
        *counter += 1;
        let update_id = *counter;
        drop(counter);

        let output = Command::new("cargo")
            .arg("update")
            .output()
            .await
            .map_err(|e| {
                McpError::internal_error(format!("Failed to execute cargo update: {}", e), None)
            })?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        let result_msg = if output.status.success() {
            format!(
                "Update #{} completed successfully.\nOutput: {}",
                update_id, stdout
            )
        } else {
            format!(
                "Update #{} failed.\nStderr: {}\nStdout: {}",
                update_id, stderr, stdout
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
