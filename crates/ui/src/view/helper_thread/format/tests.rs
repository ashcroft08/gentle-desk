use super::*;
use serde_json::json;

#[test]
fn a_running_tool_is_running_even_if_an_error_flag_is_stale() {
    assert_eq!(tool_status(Some(true), Some(true)), ToolStatus::Running);
    assert_eq!(tool_status(Some(true), None), ToolStatus::Running);
}

#[test]
fn a_finished_tool_is_failed_only_when_flagged() {
    assert_eq!(tool_status(Some(false), Some(true)), ToolStatus::Failed);
    assert_eq!(tool_status(None, Some(true)), ToolStatus::Failed);
    assert_eq!(tool_status(Some(false), Some(false)), ToolStatus::Ok);
    assert_eq!(tool_status(None, None), ToolStatus::Ok);
}

#[test]
fn truncate_keeps_short_text_and_cuts_long_text_on_a_character_boundary() {
    assert_eq!(truncate("hola", 4), "hola");
    assert_eq!(truncate("hola mundo", 4), "hola…");
    let long = "ñ".repeat(10);
    assert_eq!(truncate(&long, 3), "ñññ…");
}

#[test]
fn string_arguments_are_shown_unquoted_as_key_value_pairs() {
    assert_eq!(args_line(&json!({"path": "rpc.md"})), "path=rpc.md");
    assert_eq!(
        args_line(&json!({"command": "cat README.md", "timeout": 30, "all": true})),
        // JSON objects are unordered; keys come out alphabetically, always the same way.
        "all=true, command=cat README.md, timeout=30"
    );
}

#[test]
fn nested_values_are_compact_json_and_newlines_become_spaces() {
    assert_eq!(args_line(&json!({"opts": {"a": 1}})), r#"opts={"a":1}"#);
    assert_eq!(args_line(&json!({"text": "uno\ndos"})), "text=uno dos");
}

#[test]
fn missing_or_odd_arguments_are_handled() {
    assert_eq!(args_line(&Value::Null), "");
    assert_eq!(args_line(&json!({})), "");
    assert_eq!(args_line(&json!("bare")), "bare");
    assert_eq!(args_line(&json!([1, 2])), "[1,2]");
}

#[test]
fn long_arguments_are_cut() {
    let line = args_line(&json!({"command": "x".repeat(500)}));
    assert_eq!(line.chars().count(), 101);
    assert!(line.ends_with('…'));
}

#[test]
fn string_output_is_shown_as_is_and_other_json_compactly() {
    assert_eq!(output_preview(&json!("listo")), "listo");
    assert_eq!(output_preview(&json!({"ok": true})), r#"{"ok":true}"#);
    assert_eq!(output_preview(&Value::Null), "");
}

#[test]
fn long_output_keeps_the_first_lines_only() {
    let output: String = (1..=30).map(|n| format!("line {n}\n")).collect();
    let preview = output_preview(&json!(output));
    let lines: Vec<_> = preview.lines().collect();
    assert_eq!(lines.len(), 13);
    assert_eq!(lines[0], "line 1");
    assert_eq!(lines[11], "line 12");
    assert_eq!(lines[12], "…");
}

#[test]
fn a_single_huge_line_is_cut_by_size() {
    let preview = output_preview(&json!("y".repeat(5000)));
    assert_eq!(preview.chars().count(), 601);
    assert!(preview.ends_with('…'));
}
