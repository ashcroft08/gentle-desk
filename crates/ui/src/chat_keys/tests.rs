use std::path::PathBuf;

use super::*;

fn saved(path: &str) -> ChatKey {
    ChatKey::Saved(PathBuf::from(path))
}

#[test]
fn a_stored_chat_maps_to_its_session_path_and_back() {
    let keys = ChatKeys::default();
    let id = ChatId::of_session(r"C:\s\a.jsonl");

    assert_eq!(keys.key_of(&id), Some(saved(r"C:\s\a.jsonl")));
    assert_eq!(keys.id_of(&saved(r"C:\s\a.jsonl")), Some(id));
}

#[test]
fn a_fresh_chat_has_no_key_until_it_is_registered() {
    let mut keys = ChatKeys::default();
    let id = ChatId::fresh();
    assert_eq!(keys.key_of(&id), None);

    let key = keys.register_new(&id).expect("a fresh chat registers");

    assert!(matches!(key, ChatKey::New(_)));
    assert_eq!(keys.key_of(&id), Some(key.clone()));
    assert_eq!(keys.id_of(&key), Some(id));
}

#[test]
fn registering_the_same_fresh_chat_twice_gives_the_same_key() {
    let mut keys = ChatKeys::default();
    let id = ChatId::fresh();

    assert_eq!(keys.register_new(&id), keys.register_new(&id));
}

#[test]
fn two_fresh_chats_get_distinct_keys() {
    let mut keys = ChatKeys::default();

    let first = keys.register_new(&ChatId::fresh());
    let second = keys.register_new(&ChatId::fresh());

    assert_ne!(first, second);
}

#[test]
fn a_stored_chat_is_never_registered_as_new() {
    let mut keys = ChatKeys::default();

    assert_eq!(keys.register_new(&ChatId::of_session("a")), None);
}

#[test]
fn an_unknown_new_key_has_no_chat() {
    let keys = ChatKeys::default();

    assert_eq!(keys.id_of(&ChatKey::new_chat()), None);
}

#[test]
fn learning_the_session_path_turns_the_new_chat_into_the_stored_one() {
    let mut keys = ChatKeys::default();
    let id = ChatId::fresh();
    let from = keys.register_new(&id).unwrap();

    let moved = keys
        .rekey(&id, "p.jsonl")
        .expect("a registered chat rekeys");

    assert_eq!(moved.from, from);
    assert_eq!(moved.to, saved("p.jsonl"));
    assert_eq!(moved.id, ChatId::of_session("p.jsonl"));
    // The pairing is gone: the old identity names nothing any more.
    assert_eq!(keys.key_of(&id), None);
    assert_eq!(keys.id_of(&from), None);
    // And the stored chat resolves like any other.
    assert_eq!(keys.key_of(&moved.id), Some(saved("p.jsonl")));
}

#[test]
fn a_rekey_of_an_unknown_chat_or_an_empty_path_changes_nothing() {
    let mut keys = ChatKeys::default();
    let known = ChatId::fresh();
    let key = keys.register_new(&known).unwrap();

    assert_eq!(keys.rekey(&ChatId::fresh(), "p.jsonl"), None);
    assert_eq!(keys.rekey(&ChatId::of_session("a"), "p.jsonl"), None);
    assert_eq!(keys.rekey(&known, ""), None);

    assert_eq!(keys.key_of(&known), Some(key));
}
