//! Tiny test to ensure the standardized addendum remains present in tool descriptions

#[test]
fn test_tool_description_addendum_present() {
    // Read source file content at compile time
    const SOURCE: &str = include_str!("../src/cargo_tools.rs");

    // Count number of tool definitions and addendum occurrences
    let tools_count = SOURCE.matches("#[tool(").count();
    let async_addendum = "Always use async_cargo_mcp MCP tools; do not run cargo in a terminal. For operations >1s, set enable_async_notification=true and call mcp_async_cargo_m_wait to collect results.";
    let sync_addendum = "Always use async_cargo_mcp MCP tools; do not run cargo in a terminal.";
    let async_addendum_count = SOURCE.matches(async_addendum).count();
    let sync_addendum_count = SOURCE.matches(sync_addendum).count() - async_addendum_count; // Subtract because async addendum contains sync addendum

    // Sanity: we expect many tools defined in this file
    assert!(
        tools_count >= 15,
        "Expected many #[tool] declarations, found {tools_count}"
    );

    // The sum of both addenda should equal the number of tools
    let total_addendum_count = async_addendum_count + sync_addendum_count;
    assert!(
        total_addendum_count >= tools_count,
        "Standardized addendum missing from some tool descriptions: tools={tools_count}, async_addendum_count={async_addendum_count}, sync_addendum_count={sync_addendum_count}, total={total_addendum_count}"
    );
}

#[test]
fn test_readme_like_addendum_phrase() {
    // Useful to keep around for mirroring in README
    let addendum = "Always use async_cargo_mcp MCP tools; do not run cargo in a terminal. For operations >1s, set enable_async_notification=true and call mcp_async_cargo_m_wait to collect results.";
    assert!(addendum.contains("async_cargo_mcp MCP tools"));
    assert!(addendum.contains("enable_async_notification"));
    assert!(addendum.contains("mcp_async_cargo_m_wait"));
}
