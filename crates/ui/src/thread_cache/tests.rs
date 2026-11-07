use super::*;
use gentle_core::chat::ChatMessage;

fn message(role: ChatRole, text: &str) -> ChatMessage {
    ChatMessage {
        id: String::new(),
        role,
        text: text.to_owned(),
        streaming: false,
        error: None,
    }
}

fn chat(messages: Vec<ChatMessage>) -> ChatState {
    ChatState {
        messages,
        ..ChatState::default()
    }
}

const MARKDOWN: &str = "# Plan\n\n- uno\n- **dos**\n\n```rust\nfn main() {}\n```\n";

#[test]
fn streaming_deltas_end_up_identical_to_a_fresh_parse() {
    let mut cache = ThreadCache::default();
    let mut text = String::new();
    for piece in MARKDOWN.as_bytes().chunks(3) {
        text.push_str(std::str::from_utf8(piece).unwrap());
        cache.sync(&chat(vec![message(ChatRole::Assistant, &text)]));
    }
    let fresh = markdown::Content::parse(MARKDOWN);
    assert_eq!(shape(cache.content(0).unwrap()), shape(&fresh));
}

#[test]
fn text_that_is_not_an_extension_is_reparsed() {
    let mut cache = ThreadCache::default();
    cache.sync(&chat(vec![message(ChatRole::Assistant, "draft")]));
    // `message_end` replaced the streamed text with the authoritative one.
    cache.sync(&chat(vec![message(ChatRole::Assistant, "# Final")]));
    let fresh = markdown::Content::parse("# Final");
    assert_eq!(shape(cache.content(0).unwrap()), shape(&fresh));
}

#[test]
fn user_messages_have_no_markdown_content() {
    let mut cache = ThreadCache::default();
    cache.sync(&chat(vec![
        message(ChatRole::User, "**not markdown**"),
        message(ChatRole::Assistant, "ok"),
    ]));
    assert!(cache.content(0).is_none());
    assert!(cache.content(1).is_some());
}

#[test]
fn a_shorter_thread_drops_the_stale_entries() {
    let mut cache = ThreadCache::default();
    cache.sync(&chat(vec![
        message(ChatRole::User, "hi"),
        message(ChatRole::Assistant, "hello"),
    ]));
    cache.sync(&ChatState::default());
    assert!(cache.content(1).is_none());
    // A new chat reusing the slot must not see the old content.
    cache.sync(&chat(vec![
        message(ChatRole::User, "again"),
        message(ChatRole::Assistant, "fresh"),
    ]));
    let fresh = markdown::Content::parse("fresh");
    assert_eq!(shape(cache.content(1).unwrap()), shape(&fresh));
}

#[test]
fn a_slot_that_changes_role_is_rebuilt() {
    let mut cache = ThreadCache::default();
    cache.sync(&chat(vec![message(ChatRole::Assistant, "a")]));
    cache.sync(&chat(vec![message(ChatRole::User, "u")]));
    assert!(cache.content(0).is_none());
    cache.sync(&chat(vec![message(ChatRole::Assistant, "b")]));
    assert!(cache.content(0).is_some());
}

/// `markdown::Item` is not `PartialEq`; its `Debug` output captures the whole parsed structure.
fn shape(content: &markdown::Content) -> String {
    format!("{:?}", content.items())
}
