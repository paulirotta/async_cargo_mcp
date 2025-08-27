//! Tests for the includes() util to make assertions less brittle.

use async_cargo_mcp::test_utils;

#[test]
fn includes_direct_substring() {
    let resp = "Hello world. Async cargo is running.";
    assert!(test_utils::includes(resp, "Async cargo"));
}

#[test]
fn includes_debug_escaped_newlines() {
    let resp = "Line1\nLine2\nLine3";
    // pattern comes from debug-escaped form
    let pattern = "Line1\\nLine2";
    assert!(test_utils::includes(resp, pattern));
}

#[test]
fn includes_whitespace_normalized() {
    let resp = "One\n  two\tthree   four";
    let pattern = "one two three"; // different case? keep exact; we'll not case-fold
    assert!(test_utils::includes(resp, "two three   four"));
    assert!(test_utils::includes(resp, "two\nthree four"));
    assert!(!test_utils::includes(resp, pattern)); // case-sensitive by design
}
