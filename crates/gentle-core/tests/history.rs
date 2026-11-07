//! Rebuilding the thread from a `get_messages` response (reopening a session).

use gentle_core::chat::{messages_from_history, ChatMessage, ChatRole};
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
        messages_from_history(&data),
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
    let texts: Vec<_> = messages_from_history(&data)
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
    assert_eq!(messages_from_history(&data)[0].text, "uno dos");
}

#[test]
fn an_assistant_turn_with_only_tools_is_dropped_like_the_live_reducer_does() {
    let data = json!({"messages": [
        {"role": "user", "content": "q"},
        {"role": "assistant", "content": [{"type": "toolCall", "id": "t", "name": "bash"}]},
        {"role": "assistant", "content": [{"type": "text", "text": "listo"}]},
    ]});
    let ids: Vec<_> = messages_from_history(&data)
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
    let messages = messages_from_history(&data);
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
        assert!(messages_from_history(&data).is_empty(), "{data}");
    }
}
