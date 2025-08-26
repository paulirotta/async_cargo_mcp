//! Pure helpers for standardized tool-hint content shown to LLMs

/// Public helper to preview the standardized tool hint content.
/// This is a pure function (no async runtime needed) so tests can call it in #[test] contexts.
pub fn preview(operation_id: &str, operation_type: &str) -> String {
    // Encourage status over wait and promote productive work while operation runs
    format!(
        "\n\n### ASYNC CARGO OPERATION: {operation_type} (ID: {operation_id})\n\
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
Next: Continue useful work now. Check `status` later, or `wait` only if you’re blocked.\n\n"
    )
}
