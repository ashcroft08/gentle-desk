//! The id scheme read back as a file: a command that starts from the id text the
//! UI holds must reach the session file it names, and a chat that has no file yet
//! must reach nothing at all rather than a path the disk will refuse.
//!
//! Both forms are pinned because both are live: the chat header sends the
//! [`ChatId`] text itself (`foreground_chat`) and a sidebar row still sends its
//! bare session path, and the rename they share has to land for either.

use super::ChatId;

#[test]
fn the_id_form_resolves_to_its_session_file() {
    let path = r"C:\sessions\a.jsonl";
    let id = ChatId::of_session(path);
    assert_eq!(
        ChatId::session_path_of(id.as_str()),
        Some(path),
        "the prefix is stripped, the path is the identity"
    );
}

#[test]
fn the_bare_path_a_row_sends_resolves_to_the_same_file() {
    let path = r"C:\sessions\a.jsonl";
    assert_eq!(
        ChatId::session_path_of(path),
        Some(path),
        "a row's id is already the path, and its rename keeps working"
    );
}

#[test]
fn a_fresh_chats_id_names_no_file() {
    let fresh = ChatId::fresh();
    assert!(
        fresh.as_str().starts_with("new-"),
        "the fresh form is the scheme's, got {fresh}"
    );
    assert_eq!(
        ChatId::session_path_of(fresh.as_str()),
        None,
        "a chat with no session file yet has nothing to resolve"
    );
}

#[test]
fn an_empty_id_names_no_file_either() {
    assert_eq!(ChatId::session_path_of(""), None);
    assert_eq!(
        ChatId::session_path_of("session:"),
        None,
        "a prefix and no path names nothing to write"
    );
}

#[test]
fn a_session_whose_file_is_called_like_a_fresh_chat_is_still_a_session() {
    assert_eq!(
        ChatId::session_path_of("session:new-1"),
        Some("new-1"),
        "the prefixed form wins, so `of_session` never reads as `fresh`"
    );
}

#[test]
fn anything_else_is_read_as_the_path_it_already_was() {
    // Whatever a row sends is opaque here: a string in neither form is still
    // handed to the disk exactly as before, which refuses in its own words.
    assert_eq!(
        ChatId::session_path_of("C:/no-such-file"),
        Some("C:/no-such-file")
    );
}
