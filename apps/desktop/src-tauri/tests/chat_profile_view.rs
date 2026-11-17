//! The Profile tab's view of one chat (profiles-editor T2b-2, L8, L9, L11): the
//! shell owns it as a view of its own, a chat's own child is what answers it,
//! and a read that reached nothing changes nothing.
//!
//! No child and no window run here: the three stores arrive as the channel's
//! documented frames (`crates/gentle-core/tests/fixtures/desktop-rpc-profiles.json`,
//! parsed by the same parser a real answer goes through), the session file as its
//! documented fixture, and the executor is the recording fake of `chat_seam`.

mod chat_seam;
use chat_seam::{stored, Answer, Fake};

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use gentle_app::{ChatCommand, ChatId, Input, Intent};
use gentle_core::i18n::Language;
use gentle_core::rpc::config_store::{self, StoreId, StoreValue};
use gentle_desk_lib::bridge::AppStateHolder;
use gentle_desk_lib::contract::UiIntent;
use gentle_desk_lib::effects;
use gentle_desk_lib::settings::FileRead;
use gentle_runtime::chat_runtime::{ChatConfigRead, StoreRead};

/// The channel's own documented frames: the profiles, the fallback chains and the
/// pins, quoted from `desktop-rpc.md` in that fixture.
const FRAMES: &str =
    include_str!("../../../../crates/gentle-core/tests/fixtures/desktop-rpc-profiles.json");

/// One session file that bound a profile, with the terminal entry last: the
/// documented fixture of the reader this view reuses (L9).
const SESSION: &str =
    include_str!("../../../../crates/runtime/tests/fixtures/session-profile.jsonl");

/// The fixture's frames as one JSON document.
fn frames() -> serde_json::Value {
    serde_json::from_str(FRAMES).expect("the documented frames are JSON")
}

/// One store's value, read by the channel's own parser - the same parse a real
/// child's answer goes through.
fn typed<T>(
    value: &serde_json::Value,
    store: StoreId,
    of: impl FnOnce(StoreValue) -> Option<T>,
) -> StoreRead<T> {
    let parsed = config_store::parse_store_value(store, value).expect("a documented value");
    StoreRead::Read(of(parsed).expect("the value of the store that was asked for"))
}

/// The documented read: all three stores answered, each from its own frame.
fn documented() -> ChatConfigRead {
    let frames = frames();
    ChatConfigRead {
        profiles: typed(
            &frames["profiles_data"],
            StoreId::Profiles,
            |value| match value {
                StoreValue::Profiles(document) => Some(document),
                _ => None,
            },
        ),
        fallbacks: typed(
            &frames["fallbacks_data"],
            StoreId::ProfileFallbacks,
            |value| match value {
                StoreValue::Fallbacks(document) => Some(document),
                _ => None,
            },
        ),
        pins: typed(
            &frames["pins_value"]["value"],
            StoreId::Pins,
            |value| match value {
                StoreValue::Pins(pins) => Some(pins),
                _ => None,
            },
        ),
    }
}

/// One session-file reader: the text it answers, and the paths it was asked for.
fn session_files(text: &'static str) -> FileRead {
    Arc::new(move |_path: &Path, _language: Language| Ok(Some(text.to_owned())))
}

/// The sessions store `stored` lives in: the `<agent>/sessions` folder the
/// session list of this run enumerates (`sessions::startup`).
fn sessions_store() -> PathBuf {
    PathBuf::from("/sessions")
}

/// A session-file reader that answers `text` and records every path it was
/// asked for, so a test can prove the seam was never reached.
fn recording_files(text: &'static str) -> (FileRead, Arc<Mutex<Vec<PathBuf>>>) {
    let asked: Arc<Mutex<Vec<PathBuf>>> = Arc::new(Mutex::new(Vec::new()));
    let kept = Arc::clone(&asked);
    let files: FileRead = Arc::new(move |path: &Path, _language: Language| {
        kept.lock().expect("asked mutex").push(path.to_path_buf());
        Ok(Some(text.to_owned()))
    });
    (files, asked)
}

/// The holder with the stored chat opened and confirmed, so it is the foreground
/// chat and it is idle.
fn open_stored() -> (AppStateHolder, ChatId) {
    let holder = AppStateHolder::default();
    let chat = stored();
    holder.fold_input(Input::Intent(Intent::OpenChat(chat.clone())));
    holder.fold_input(Input::ChatOpened { chat: chat.clone() });
    (holder, chat)
}

#[test]
fn the_fold_stores_the_view_of_that_chat_and_answers_the_slice_to_push() {
    let (holder, chat) = open_stored();
    holder.arm_session_files(session_files(SESSION), sessions_store());

    let pushed = holder
        .note_chat_profile(&chat, &documented())
        .expect("a read that reached a store is a view");

    let view = pushed
        .chat_profile
        .expect("the slice carries the view the fold just stored");
    assert_eq!(view.chat_id, chat.as_str());
    assert_eq!(
        view.bound.as_deref(),
        Some("team"),
        "the last bind of the chat's own session file is the profile this chat uses"
    );
    assert!(
        view.bound_from_session_file,
        "read from the file, as L9 ruled"
    );
    assert_eq!(
        view.default_profile.as_deref(),
        Some("team"),
        "the profiles store's own `active`"
    );
    assert_eq!(
        holder
            .view()
            .chat_profile
            .as_ref()
            .map(|v| v.chat_id.clone()),
        Some(chat.as_str().to_owned()),
        "the view lives in the shell state, not in the reducer"
    );
}

#[test]
fn the_refresh_intent_cannot_read_a_session_file_outside_the_store() {
    let holder = AppStateHolder::default();
    let outside = ChatId::of_session("/etc/a.jsonl");
    holder.fold_input(Input::Intent(Intent::OpenChat(outside.clone())));
    holder.fold_input(Input::ChatOpened {
        chat: outside.clone(),
    });
    let (files, asked) = recording_files(SESSION);
    holder.arm_session_files(files, sessions_store());
    let fake = Fake::answering(Answer::ChatProfile(Box::new(documented())));

    // The web names the chat by the id it holds, and the mapping turns any
    // string into a session path (`chat_for`). The read must not follow that
    // fallback out of the sessions store.
    let (_view, commands) = holder.dispatch_commands(UiIntent::RefreshChatProfile {
        chat_id: outside.as_str().to_owned(),
    });
    let ran = effects::run_one(commands[0].clone(), &fake);
    let (named, read) = ran
        .chat_profile
        .into_iter()
        .next()
        .expect("the child's answer travels");
    let pushed = holder
        .note_chat_profile(&named, &read)
        .expect("the channel read reached its stores");

    assert!(
        asked.lock().expect("asked mutex").is_empty(),
        "the file seam is never asked for a path outside the sessions store"
    );
    let view = pushed.chat_profile.expect("the open chat's own view");
    assert_eq!(
        view.bound, None,
        "nothing was read, so the chat uses the default - never an error"
    );
}

#[test]
fn a_read_that_reached_no_store_keeps_the_previous_view_and_pushes_nothing() {
    let (holder, chat) = open_stored();
    holder.arm_session_files(session_files(SESSION), sessions_store());
    holder
        .note_chat_profile(&chat, &documented())
        .expect("the first read");
    let kept = holder.view().chat_profile;

    let pushed = holder.note_chat_profile(&chat, &ChatConfigRead::unasked());

    assert!(
        pushed.is_none(),
        "nothing is emitted for a read that reached no store (L11-4)"
    );
    assert_eq!(
        holder.view().chat_profile,
        kept,
        "the tab keeps what it drew instead of blanking"
    );
}

#[test]
fn only_the_foreground_chats_view_is_carried() {
    let holder = AppStateHolder::default();
    let viewed = stored();
    let other = ChatId::of_session("/sessions/b.jsonl");
    for chat in [&viewed, &other] {
        holder.fold_input(Input::Intent(Intent::OpenChat(chat.clone())));
        holder.fold_input(Input::ChatOpened { chat: chat.clone() });
    }
    holder.arm_session_files(session_files(SESSION), sessions_store());
    holder
        .note_chat_profile(&viewed, &documented())
        .expect("the read of the chat that is not in front");

    assert!(
        holder.view().chat_profile.is_none(),
        "another chat's profile is not the one the window draws"
    );

    holder.fold_input(Input::Intent(Intent::OpenChat(viewed.clone())));

    assert_eq!(
        holder.view().chat_profile.map(|view| view.chat_id),
        Some(viewed.as_str().to_owned()),
        "in front of its own chat, the view is carried"
    );
}

#[test]
fn closing_the_chat_drops_its_view() {
    let (holder, chat) = open_stored();
    holder.arm_session_files(session_files(SESSION), sessions_store());
    holder
        .note_chat_profile(&chat, &documented())
        .expect("the read");

    holder.fold_input(Input::ChatClosed {
        chat: chat.clone(),
        expected: true,
    });

    assert!(
        holder.view().chat_profile.is_none(),
        "a closed chat's view is dropped, so a stalled read cannot stick (L11-4)"
    );
    holder.fold_input(Input::Intent(Intent::OpenChat(chat.clone())));
    assert!(
        holder.view().chat_profile.is_none(),
        "reopening the same chat asks again: the dropped view is not resurrected"
    );
}

#[test]
fn a_read_for_a_chat_that_is_still_open_installs_its_view() {
    let (holder, chat) = open_stored();
    holder.arm_session_files(session_files(SESSION), sessions_store());

    let pushed = holder
        .note_chat_profile(&chat, &documented())
        .expect("an open chat's read is noted");

    assert_eq!(
        pushed.chat_profile.map(|view| view.chat_id),
        Some(chat.as_str().to_owned()),
        "the open chat's own read still draws its tab"
    );
}

#[test]
fn a_read_that_answers_after_a_separate_close_installs_nothing_and_a_reopen_shows_no_stale_view() {
    let (holder, chat) = open_stored();
    holder.arm_session_files(session_files(SESSION), sessions_store());
    holder
        .note_chat_profile(&chat, &documented())
        .expect("the read that draws the tab while the chat is open");

    // The user's Close chat takes the chat out of the open set, and the fake
    // answers the host's expected close for the command it asked for.
    holder.fold_input(Input::Intent(Intent::CloseChat(chat.clone())));
    let closer = Fake::answering(Answer::Closed);
    let closed = effects::run_one(ChatCommand::CloseProcess { chat: chat.clone() }, &closer);
    holder.fold_inputs(closed.inputs);

    // The read that was in flight answers now. Its chat is gone, so it may not
    // re-install the view the close dropped (review R4-late-read).
    assert!(
        holder.note_chat_profile(&chat, &documented()).is_none(),
        "a read for a chat that is no longer open is dropped, with no push"
    );
    assert!(
        holder.view().chat_profile.is_none(),
        "the late answer leaves the shell with no view of the closed chat"
    );

    // The row opens the same chat again: it asks its child afresh, and the
    // stale view may not reappear under the reused id.
    holder.fold_input(Input::Intent(Intent::OpenChat(chat.clone())));
    assert!(
        holder.view().chat_profile.is_none(),
        "a reopen shows nothing until its own read answers"
    );
}

#[test]
fn the_refresh_intent_asks_the_named_chat_for_its_own_profile() {
    let (holder, chat) = open_stored();

    let (view, commands) = holder.dispatch_commands(UiIntent::RefreshChatProfile {
        chat_id: chat.as_str().to_owned(),
    });

    assert_eq!(
        commands,
        vec![ChatCommand::ReadChatProfile { chat }],
        "one read, of the one chat the intent named"
    );
    assert!(
        view.chat_profile.is_none(),
        "nothing is read on this thread: the slice is what it was"
    );
}

#[test]
fn the_intent_reads_the_chat_it_names_and_not_the_foreground_one() {
    let holder = AppStateHolder::default();
    let named = stored();
    let other = ChatId::of_session("/sessions/b.jsonl");
    for chat in [&named, &other] {
        holder.fold_input(Input::Intent(Intent::OpenChat(chat.clone())));
        holder.fold_input(Input::ChatOpened { chat: chat.clone() });
    }

    let (_view, commands) = holder.dispatch_commands(UiIntent::RefreshChatProfile {
        chat_id: named.as_str().to_owned(),
    });

    assert_eq!(
        commands,
        vec![ChatCommand::ReadChatProfile {
            chat: named.clone()
        }],
        "the named chat is the one asked, whatever is in front"
    );
}

#[test]
fn the_childs_answer_reaches_the_shell_view_through_the_effects_run() {
    let (holder, chat) = open_stored();
    holder.arm_session_files(session_files(SESSION), sessions_store());
    let fake = Fake::answering(Answer::ChatProfile(Box::new(documented())));

    let ran = effects::run_one(ChatCommand::ReadChatProfile { chat: chat.clone() }, &fake);

    assert!(
        ran.inputs.is_empty(),
        "the reducer knows nothing about a chat's profile (L11-1)"
    );
    assert_eq!(ran.chat_profile.len(), 1, "one command, one read");
    let (named, read) = ran
        .chat_profile
        .into_iter()
        .next()
        .expect("the read travels beside the inputs");
    assert_eq!(named, chat, "the read stays that chat's");
    let view = holder
        .note_chat_profile(&named, &read)
        .expect("one store was read");
    assert_eq!(
        view.chat_profile.and_then(|view| view.bound),
        Some("team".to_owned())
    );
    assert_eq!(
        fake.asked(),
        vec![ChatCommand::ReadChatProfile { chat }],
        "one read, of that chat"
    );
}

#[test]
fn a_child_that_never_answered_changes_nothing_in_the_view() {
    let (holder, chat) = open_stored();
    holder.arm_session_files(session_files(SESSION), sessions_store());
    holder
        .note_chat_profile(&chat, &documented())
        .expect("the read that drew the tab");
    let kept = holder.view().chat_profile;
    let fake = Fake::answering(Answer::ChatProfile(Box::new(ChatConfigRead::unasked())));

    let ran = effects::run_one(ChatCommand::ReadChatProfile { chat: chat.clone() }, &fake);
    let (named, read) = ran
        .chat_profile
        .into_iter()
        .next()
        .expect("the read travels");

    assert!(
        holder.note_chat_profile(&named, &read).is_none(),
        "a chat whose child never answered pushes nothing (S6, L11-4)"
    );
    assert_eq!(holder.view().chat_profile, kept, "the tab is as it was");
}

/// The chat WITH a profile view carries it, and the chat WITHOUT one is the slice
/// it was before this slice: `chat_profile` is null and nothing else moved.
#[test]
fn a_chat_with_no_profile_view_is_unchanged() {
    let (holder, chat) = open_stored();

    let slice = serde_json::to_value(holder.view()).expect("the slice is JSON");

    assert_eq!(
        slice["chat_profile"],
        serde_json::Value::Null,
        "no read answered yet, so the tab has nothing of its own to draw"
    );
    assert_eq!(slice["foreground_chat"], serde_json::json!(chat.as_str()));
}
