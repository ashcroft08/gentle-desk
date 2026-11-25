//! A chat's identity as the UI reads it back: a stored session knows its path,
//! a brand-new chat does not have one yet.

use gentle_app::ChatId;

#[test]
fn a_session_chat_knows_its_path() {
    let path = r"C:\sessions\a.jsonl";
    assert_eq!(ChatId::of_session(path).session_path(), Some(path));
}

#[test]
fn a_fresh_chat_has_no_path_yet() {
    assert_eq!(ChatId::fresh().session_path(), None);
}

#[test]
fn a_path_that_looks_like_the_id_scheme_is_still_just_a_path() {
    assert_eq!(
        ChatId::of_session("session:x").session_path(),
        Some("session:x")
    );
    assert_eq!(ChatId::of_session("new-1").session_path(), Some("new-1"));
}

#[test]
fn the_same_session_is_the_same_chat_and_a_fresh_one_never_equals_it() {
    assert_eq!(ChatId::of_session("a"), ChatId::of_session("a"));
    assert_ne!(ChatId::of_session("a"), ChatId::of_session("b"));
    assert_ne!(ChatId::fresh(), ChatId::of_session("new-1"));
}

/// The id TEXT the UI holds back as the chat it names (packages-status T7): a
/// command that starts from `foreground_chat` must reach the very chat the shell
/// opened, no matter which of the two live forms it carries.
#[test]
fn the_fresh_id_text_is_the_fresh_chat_it_names() {
    let fresh = ChatId::fresh();

    assert_eq!(
        ChatId::of_text(fresh.as_str()),
        Some(fresh.clone()),
        "a new chat travels back as ITSELF, so the host key it was given is found"
    );
}

#[test]
fn a_stored_id_text_is_the_stored_chat_it_names() {
    let path = r"C:\sessions\a.jsonl";
    let stored = ChatId::of_session(path);

    assert_eq!(ChatId::of_text(stored.as_str()), Some(stored.clone()));
    assert_eq!(
        ChatId::of_text(path),
        Some(stored),
        "a sidebar row's bare path is the same stored chat"
    );
}

#[test]
fn an_id_text_that_names_no_chat_answers_nothing() {
    assert_eq!(ChatId::of_text(""), None);
    assert_eq!(ChatId::of_text("session:"), None, "a prefix and no path");
}

#[test]
fn a_path_that_only_starts_like_a_fresh_id_is_a_stored_chat() {
    assert_eq!(
        ChatId::of_text("new-abc"),
        Some(ChatId::of_session("new-abc")),
        "the fresh form is the prefix and the counter, nothing else"
    );
}

#[test]
fn a_chat_is_labelled_by_its_session_file_or_its_own_id() {
    assert_eq!(ChatId::of_session("C:/s/a.jsonl").label(), "a.jsonl");
    assert_eq!(ChatId::of_session("C:/s/b.jsonl").label(), "b.jsonl");
    assert_eq!(ChatId::of_session("p").label(), "p");
    assert!(ChatId::fresh().label().starts_with("new-"));
}
