//! Tiny test to ensure the standardized addendum remains present in tool descriptions

#[test]
fn test_tool_description_addendum_present() {
    // Read source file content at compile time
    const SOURCE: &str = include_str!("../src/cargo_tools.rs");

    // Count number of tool definitions and addendum occurrences
    let tools_count = SOURCE.matches("#[tool(").count();
    let addendum = "Always use async_cargo_mcp MCP tools; do not run cargo in a terminal. For operations >1s, set enable_async_notifications=true and call mcp_async_cargo_m_wait to collect results.";
    let addendum_count = SOURCE.matches(addendum).count();

    // Sanity: we expect many tools defined in this file
    assert!(
        tools_count >= 15,
        "Expected many #[tool] declarations, found {tools_count}"
    );

    // The addendum should appear for every tool description
    assert!(
        addendum_count >= tools_count,
        "Standardized addendum missing from some tool descriptions: tools={tools_count}, addendum_count={addendum_count}"
    );
}

#[test]
fn test_readme_like_addendum_phrase() {
    // Useful to keep around for mirroring in README
    let addendum = "Always use async_cargo_mcp MCP tools; do not run cargo in a terminal. For operations >1s, set enable_async_notifications=true and call mcp_async_cargo_m_wait to collect results.";
    assert!(addendum.contains("async_cargo_mcp MCP tools"));
    assert!(addendum.contains("enable_async_notifications"));
    assert!(addendum.contains("mcp_async_cargo_m_wait"));
}
