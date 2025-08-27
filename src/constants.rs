//! Centralized constants for LLM-facing strings, guidance, and templates.

/// Standardized addendum for async-capable tools' descriptions used in documentation and help text.
pub const ASYNC_ADDENDUM: &str = "Always use async_cargo_mcp MCP tools; do not run cargo in a terminal. For operations >1s, set enable_async_notification=true and call mcp_async_cargo_m_wait with specific operation_ids to collect results.";

/// Standardized addendum for synchronous or fast tools where async guidance is not emphasized.
pub const SYNC_ADDENDUM: &str =
    "Always use async_cargo_mcp MCP tools; do not run cargo in a terminal.";

/// Template for standardized tool-hint content displayed when async operations are started.
/// Placeholders:
/// - {operation_type}
/// - {operation_id}
pub const TOOL_HINT_TEMPLATE: &str = "\n\n### ASYNC CARGO OPERATION: {operation_type} (ID: {operation_id})\n\
1. The operation is running in the background — do not assume it’s complete.\n\
2. What to do now (pick one):\n\
 - Update the plan with what’s already achieved and list the next concrete steps.\n\
 - Do unrelated code, tests, or docs not blocked by this `{operation_type}`.\n\
 - If you’ll need these results soon, schedule a later `status` check instead of polling.\n\
 - If you have nothing else to do and need results to proceed, use `wait` with operation_ids=['{operation_id}'].\n\
3. Tips:\n\
 - Prefer `status` for non-blocking checks; avoid tight polling.\n\
 - Batch actions: start other needed tools first, then wait for all IDs at once.\n\
 - Always specify explicit operation IDs; never pass an empty list.\n\
 - You’ll also receive a completion notification via progress updates.\n\n\
Next: Continue useful work now. Check `status` later, or `wait` only if you’re blocked.\n\n";

/// Template used when detecting premature waits that harm concurrency.
/// Placeholders: {operation_id}, {gap_seconds}, {efficiency_percent}
pub const CONCURRENCY_HINT_TEMPLATE: &str = "CONCURRENCY HINT: You waited for '{operation_id}' after only {gap_seconds:.1}s (efficiency: {efficiency_percent:.0}%). \
        Consider performing other tasks while operations run in the background.";

/// Template for the status polling detection guidance.
/// Placeholders: {count}, {operation_id}
pub const STATUS_POLLING_HINT_TEMPLATE: &str = "STATUS POLLING DETECTED: You've called status {count} times for operation '{operation_id}'. \
                    Instead of repeatedly polling, consider using 'wait' with enable_async_notification=true \
                    for automatic results via progress notifications.\n";
