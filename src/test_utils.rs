//! Test helpers for resilient string assertions and comparisons.

/// Returns true if `response` includes `pattern`, allowing common formatting variants.
/// Strategy:
/// - Direct substring check
/// - Decode simple debug escapes (\n, \r, \t, \", \\) in both text and pattern, then check
/// - Whitespace-normalized contains (collapse runs of whitespace to single spaces)
pub fn includes(response: &str, pattern: &str) -> bool {
    // 1) Direct fast path
    if response.contains(pattern) {
        return true;
    }

    // 2) Decode common debug-escaped sequences in both sides
    let response = decode_escapes(response);
    let pattern = decode_escapes(pattern);

    if response.contains(&pattern) {
        return true;
    }

    // 3) Whitespace-normalized compare
    let response = normalize_ws(&response);
    let pattern = normalize_ws(&pattern);

    response.contains(&pattern)
}

fn decode_escapes(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut it = s.chars().peekable();
    while let Some(c) = it.next() {
        if c == '\\' {
            match it.next() {
                Some('n') => out.push('\n'),
                Some('r') => out.push('\r'),
                Some('t') => out.push('\t'),
                Some('"') => out.push('"'),
                Some('\\') => out.push('\\'),
                Some(other) => {
                    // Unknown escape: keep as-is
                    out.push('\\');
                    out.push(other);
                }
                None => out.push('\\'),
            }
        } else {
            out.push(c);
        }
    }
    out
}

// Whitespace-normalized check
fn normalize_ws(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut last_space = false;
    for ch in s.chars() {
        if ch.is_whitespace() {
            if !last_space {
                out.push(' ');
                last_space = true;
            }
        } else {
            out.push(ch);
            last_space = false;
        }
    }
    out.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::includes;

    #[test]
    fn includes_direct_substring() {
        let resp = "Hello world. Async cargo is running.";
        assert!(includes(resp, "Async cargo"));
    }

    #[test]
    fn includes_debug_escaped_newlines() {
        let resp = "Line1\nLine2\nLine3";
        let pattern = "Line1\\nLine2"; // debug-escaped input
        assert!(includes(resp, pattern));
    }

    #[test]
    fn includes_quotes_and_backslashes() {
        // Response contains typical Windows path and quoted filename
        let resp = r#"Path: C:\Temp\file "name".txt"#;
        // Debug-escaped pattern (common in debug prints/logs)
        let pattern_debug = r#"C:\\Temp\\file \"name\".txt"#;
        // Raw pattern
        let pattern_raw = r#"C:\Temp\file "name".txt"#;
        assert!(includes(resp, pattern_debug));
        assert!(includes(resp, pattern_raw));
    }

    #[test]
    fn includes_whitespace_normalized() {
        let resp = "One\n  two\tthree   four";
        assert!(includes(resp, "two three   four"));
        assert!(includes(resp, "two\nthree four"));
    }

    #[test]
    fn includes_is_case_sensitive() {
        let resp = "CaseSensitive";
        assert!(!includes(resp, "casesensitive"));
    }

    #[test]
    fn includes_absent_pattern_false() {
        let resp = "No match here";
        assert!(!includes(resp, "definitely not present"));
    }
}
