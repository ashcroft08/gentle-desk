//! Rebuilding the thread from a `get_messages` response (reopening a session),
//! and from the session FILE itself, which paints the first frame while the
//! child is still starting (`chat-open-latency.md` T1).

use gentle_core::chat::{
    messages_from_history, messages_from_session_file, ChatMessage, ChatRole,
    SESSION_FILE_MAX_BYTES,
};
use gentle_core::i18n::Language;
use serde_json::json;

fn msg(id: &str, role: ChatRole, text: &str) -> ChatMessage {
    ChatMessage {
        id: id.into(),
        role,
        text: text.into(),
        streaming: false,
        error: None,
    }
}

#[test]
fn user_and_assistant_text_become_messages_with_contiguous_ids() {
    let data = json!({"messages": [
        {"role": "user", "content": [{"type": "text", "text": "hola"}]},
        {"role": "assistant", "content": [{"type": "text", "text": "**ey**"}]},
        {"role": "user", "content": "plain string"},
    ]});
    assert_eq!(
        messages_from_history(Language::En, &data),
        vec![
            msg("msg-0", ChatRole::User, "hola"),
            msg("msg-1", ChatRole::Assistant, "**ey**"),
            msg("msg-2", ChatRole::User, "plain string"),
        ]
    );
}

#[test]
fn system_messages_and_tool_results_are_not_shown() {
    let data = json!({"messages": [
        {"role": "system", "content": "", "sections": {"preamble": "x"}},
        {"role": "user", "content": "q"},
        {"role": "toolResult", "content": [{"type": "text", "text": "output"}]},
        {"role": "assistant", "content": [{"type": "text", "text": "a"}]},
    ]});
    let texts: Vec<_> = messages_from_history(Language::En, &data)
        .into_iter()
        .map(|m| m.text)
        .collect();
    assert_eq!(texts, ["q", "a"]);
}

#[test]
fn text_blocks_are_concatenated_and_other_blocks_ignored() {
    let data = json!({"messages": [{"role": "assistant", "content": [
        {"type": "thinking", "thinking": "hmm"},
        {"type": "text", "text": "uno "},
        {"type": "toolCall", "id": "t", "name": "bash"},
        {"type": "text", "text": "dos"},
    ]}]});
    assert_eq!(
        messages_from_history(Language::En, &data)[0].text,
        "uno dos"
    );
}

#[test]
fn an_assistant_turn_with_only_tools_is_dropped_like_the_live_reducer_does() {
    let data = json!({"messages": [
        {"role": "user", "content": "q"},
        {"role": "assistant", "content": [{"type": "toolCall", "id": "t", "name": "bash"}]},
        {"role": "assistant", "content": [{"type": "text", "text": "listo"}]},
    ]});
    let ids: Vec<_> = messages_from_history(Language::En, &data)
        .into_iter()
        .map(|m| (m.id, m.text))
        .collect();
    assert_eq!(
        ids,
        [
            ("msg-0".to_owned(), "q".to_owned()),
            ("msg-1".to_owned(), "listo".to_owned())
        ]
    );
}

#[test]
fn a_failed_assistant_message_is_kept_and_flagged_even_without_text() {
    let data = json!({"messages": [{"role": "assistant", "content": [], "stopReason": "error", "errorMessage": "quota"}]});
    let messages = messages_from_history(Language::En, &data);
    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0].error.as_deref(), Some("quota"));
}

#[test]
fn malformed_data_gives_an_empty_thread() {
    for data in [
        json!(null),
        json!({}),
        json!({"messages": "no"}),
        json!({"messages": [1, "x", null]}),
    ] {
        assert!(
            messages_from_history(Language::En, &data).is_empty(),
            "{data}"
        );
    }
}

// --- The session file as the thread's first frame (chat-open-latency T1) ---

/// One `message` entry of a session file, the shape `session.rs` scans and
/// `tools/perf/gen-home.mjs` writes.
fn line(role: &str, text: &str) -> String {
    format!(
        r#"{{"type":"message","id":"m","timestamp":"2026-01-01T00:01:00.000Z","message":{{"role":"{role}","content":[{{"type":"text","text":"{text}"}}]}}}}"#
    )
}

/// A realistic file: the header, a name, a compaction marker, the entries a
/// thread does not show, and the turns it does.
fn realistic() -> String {
    let lines = [
        r#"{"type":"session","version":3,"id":"s1","timestamp":"2026-01-01T00:00:00.000Z","cwd":"C:\\dev\\x"}"#.to_owned(),
        r#"{"type":"session_info","name":"a chat"}"#.to_owned(),
        line("system", "you are a helpful agent"),
        line("user", "hola"),
        line("toolResult", "total 0"),
        line("assistant", "ey"),
        r#"{"type":"compaction","summary":"so far so good"}"#.to_owned(),
        line("user", "otra"),
        line("assistant", "listo"),
    ];
    lines.join("\n") + "\n"
}

#[test]
fn a_session_file_projects_its_turns_and_nothing_else() {
    let messages = messages_from_session_file(&realistic());

    assert_eq!(
        messages
            .iter()
            .map(|message| (message.role, message.text.as_str()))
            .collect::<Vec<_>>(),
        [
            (ChatRole::User, "hola"),
            (ChatRole::Assistant, "ey"),
            (ChatRole::User, "otra"),
            (ChatRole::Assistant, "listo"),
        ],
        "the header, the name, the compaction and the tool result are not the thread"
    );
    assert!(
        messages.iter().all(|message| !message.streaming),
        "a file line is a closed turn, never a streaming one"
    );
}

#[test]
fn a_file_row_gets_a_provisional_id_the_child_never_uses() {
    let messages = messages_from_session_file(&realistic());

    assert_eq!(
        messages
            .iter()
            .map(|message| message.id.as_str())
            .collect::<Vec<_>>(),
        ["prov-0", "prov-1", "prov-2", "prov-3"],
        "the ids are this file's own: the child's `msg-<n>` rows are a different identity"
    );
    let child = messages_from_history(
        Language::En,
        &json!({"messages": [{"role": "user", "content": "hola"}]}),
    );
    assert!(
        messages
            .iter()
            .all(|row| child.iter().all(|own| own.id != row.id)),
        "a provisional id is never one of the child's"
    );
}

#[test]
fn a_trailing_partial_line_is_ignored_never_an_error() {
    // A file being appended to: the last line is still half written, so it is
    // not JSON yet. The rows before it are what the thread can draw.
    let text = format!(
        "{}\n{}",
        line("user", "hola"),
        r#"{"type":"message","message":{"role":"assistant","content":[{"type":"te"#
    );

    let messages = messages_from_session_file(&text);

    assert_eq!(
        messages
            .iter()
            .map(|message| message.text.as_str())
            .collect::<Vec<_>>(),
        ["hola"],
        "the half-written line draws nothing and breaks nothing"
    );
}

#[test]
fn a_malformed_line_in_the_middle_is_skipped() {
    let text = format!(
        "{}\n{{{{\n{}\nnot json at all\n{}\n",
        line("user", "uno"),
        line("assistant", "dos"),
        line("user", "tres")
    );

    let messages = messages_from_session_file(&text);

    assert_eq!(
        messages
            .iter()
            .map(|message| message.text.as_str())
            .collect::<Vec<_>>(),
        ["uno", "dos", "tres"]
    );
}

#[test]
fn an_oversized_file_draws_nothing() {
    // The file is read whole, once, before the child replaces it: a session of
    // many megabytes is not parsed into rows the answer is about to discard.
    let mut text = realistic();
    text.push_str(&"x".repeat(SESSION_FILE_MAX_BYTES));

    assert!(messages_from_session_file(&text).is_empty());
    assert_eq!(
        SESSION_FILE_MAX_BYTES,
        8 * 1024 * 1024,
        "the cap the shell reads against is the one this parser applies"
    );
}

#[test]
fn an_empty_or_blank_file_draws_nothing() {
    for text in ["", "\n", "  \n\t\n"] {
        assert!(messages_from_session_file(text).is_empty(), "{text:?}");
    }
}

#[test]
fn a_file_and_the_childs_own_answer_project_the_same_roles_and_texts() {
    // Parity is the whole point of reusing the projection: the rows the file
    // paints are the rows the child is about to send, only their ids differ.
    let data = json!({"messages": [
        {"role": "system", "content": ""},
        {"role": "user", "content": [{"type": "text", "text": "hola"}]},
        {"role": "toolResult", "content": [{"type": "text", "text": "output"}]},
        {"role": "assistant", "content": [
            {"type": "thinking", "thinking": "hmm"},
            {"type": "text", "text": "uno "},
            {"type": "text", "text": "dos"},
        ]},
        {"role": "assistant", "content": [], "stopReason": "error", "errorMessage": "quota"},
        {"role": "assistant", "content": [{"type": "toolCall", "id": "t", "name": "bash"}]},
    ]});
    let entries = data["messages"].as_array().expect("the messages array");
    let file: String = entries
        .iter()
        .map(|entry| {
            format!(
                "{}\n",
                serde_json::to_string(&json!({"type": "message", "message": entry}))
                    .expect("the message entry is valid JSON"),
            )
        })
        .collect();

    let from_file: Vec<_> = messages_from_session_file(&file)
        .into_iter()
        .map(|message| (message.role, message.text, message.error))
        .collect();
    let from_child: Vec<_> = messages_from_history(Language::En, &data)
        .into_iter()
        .map(|message| (message.role, message.text, message.error))
        .collect();

    assert_eq!(from_file, from_child);
    assert_eq!(from_file.len(), 3, "two turns and one failure");
}
