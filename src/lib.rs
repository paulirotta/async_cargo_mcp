//! Model Control Protocol (MCP) for Cargo with asynchronous respon handling to allow the LLM to continue processing while waiting for responses.

pub mod callback_system;
pub mod cargo_tools;
pub mod mcp_callback;
pub mod operation_monitor;
pub mod test_cargo_tools;
pub mod tool_hints;
