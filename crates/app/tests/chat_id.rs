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
