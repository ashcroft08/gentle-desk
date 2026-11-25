//! Close chat through the bridge (0.1.14): the row menu's intent takes the chat
//! off the open set the Updates pane's gate counts, asks the shell to stop that
//! chat's child, and leaves the session file exactly where it was.
//!
//! The reducer's own rules are tested in `gentle-app`; what this file adds is
//! the contract's intent on the real dispatch path - the id round trip, the
//! slice the web side receives, and the one file that must NOT change.

use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};

use gentle_app::{close_working, ChatCommand, ChatId, Input, Intent, SessionItem};
use gentle_core::i18n::Language;
use gentle_core::rpc::RpcEvent;
use gentle_desk_lib::bridge::AppStateHolder;
use gentle_desk_lib::contract::UiIntent;

const CWD: &str = "/dev/repo";

/// A directory of its own under the system temp directory, removed when the test
/// ends, so a session file can be watched without touching the repository.
struct Scratch {
    dir: PathBuf,
}

impl Scratch {
    fn new(name: &str) -> Self {
        static NEXT: AtomicU32 = AtomicU32::new(0);
        let dir = std::env::temp_dir().join(format!(
            "gentle-desk-chat-close-{}-{name}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&dir).expect("a scratch directory");
        Self { dir }
    }

    /// The session file of one chat, as the list would name it.
    fn session(&self, name: &str) -> String {
        let path = self.dir.join(name);
        std::fs::write(&path, "{\"type\":\"session_info\"}\n").expect("a session file");
        path.to_string_lossy().into_owned()
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

/// One session file of the list, as the first pass reports it.
fn session(path: &str) -> SessionItem {
    SessionItem {
        path: path.to_owned(),
        title: "A chat".to_owned(),
        cwd: CWD.to_owned(),
        modified_ms: 0,
        message_count: 1,
    }
}

/// The holder with the chat stored at `path` opened the way a click on its row
/// opens it: foreground, child up.
fn open_chat(path: &str) -> AppStateHolder {
    let holder = AppStateHolder::default();
    holder.fold_input(Input::SessionsLoaded(vec![session(path)]));
    holder.fold_input(Input::Intent(Intent::OpenChat(ChatId::of_session(path))));
    holder.fold_input(Input::ChatOpened {
        chat: ChatId::of_session(path),
    });
    holder
}

#[test]
fn close_chat_drops_it_from_the_open_set_and_leaves_the_file_in_place() {
    let scratch = Scratch::new("drops");
    let path = scratch.session("a.jsonl");
    let holder = open_chat(&path);
    let before = holder.view();
    assert_eq!(before.open_chats, 1, "the gate counts it as open");

    let (view, commands) = holder.dispatch_commands(UiIntent::CloseChat {
        chat_id: path.clone(),
    });

    assert_eq!(
        view.open_chats, 0,
        "the Updates gate has nothing to wait for"
    );
    assert_eq!(view.foreground_chat, None, "the chat is off the screen");
    assert_eq!(
        commands,
        vec![ChatCommand::CloseProcess {
            chat: ChatId::of_session(&path)
        }],
        "the shell is asked to stop exactly that chat's child"
    );
    assert!(
        std::path::Path::new(&path).exists(),
        "closing a chat is not deleting it: the session file is untouched"
    );
    assert_eq!(
        view.total_sessions, before.total_sessions,
        "the session list keeps its row"
    );
    assert_eq!(
        view.sessions_revision, before.sessions_revision,
        "no list pass was asked for: no file changed"
    );
    assert_eq!(
        view.notices.len(),
        before.notices.len(),
        "and nothing is reported"
    );
}

#[test]
fn a_working_chat_is_refused_and_stays_open_with_the_delete_flows_words() {
    let scratch = Scratch::new("working");
    let path = scratch.session("a.jsonl");
    let holder = open_chat(&path);
    holder.fold_input(Input::ChatEvent {
        chat: ChatId::of_session(&path),
        event: RpcEvent::AgentStart,
    });

    let (view, commands) = holder.dispatch_commands(UiIntent::CloseChat {
        chat_id: path.clone(),
    });

    assert_eq!(view.open_chats, 1, "a chat that is working stays open");
    assert_eq!(
        commands,
        Vec::new(),
        "its child is never stopped from here: the user stops it first"
    );
    let notice = view.notices.last().expect("the refusal is one notice");
    assert_eq!(
        notice.text,
        close_working(Language::En),
        "the refusal is the sentence the delete flow refuses with"
    );
    assert_eq!(
        notice.level,
        gentle_desk_lib::contract::NoticeLevel::Warning
    );
    assert!(
        std::path::Path::new(&path).exists(),
        "a refusal touches no file"
    );
}

#[test]
fn closing_a_chat_this_run_never_opened_asks_for_nothing() {
    let scratch = Scratch::new("unopened");
    let path = scratch.session("a.jsonl");
    let holder = AppStateHolder::default();
    holder.fold_input(Input::SessionsLoaded(vec![session(&path)]));
    let before = holder.view();

    let (view, commands) = holder.dispatch_commands(UiIntent::CloseChat { chat_id: path });

    assert_eq!(
        commands,
        Vec::new(),
        "a chat with no child of this run needs no stop"
    );
    assert_eq!(view.open_chats, before.open_chats);
    assert_eq!(view.notices, before.notices);
}

/// The gate the Updates pane's terminal buttons read (`open_chats`) counts the
/// chats the app has OPEN - not every id something mentioned. A chat that left
/// the set must never come back from the last things its child said.
///
/// This is the maintainer's second report (L3, 0.9.0): one chat opened and
/// closed, and the terminal buttons stayed disabled with "1 chat is open" for the
/// rest of the run. The chat was not on screen and had no child: a last input of
/// the child that had just been stopped (a stderr line, a reply that was already
/// in flight, the death of the process as it went) recreated the map entry, and
/// the entry is the count.
#[test]
fn a_closed_chats_last_inputs_never_reopen_it() {
    let scratch = Scratch::new("phantom");
    let path = scratch.session("a.jsonl");
    let chat = ChatId::of_session(&path);
    let holder = open_chat(&path);
    holder.fold_input(Input::ChatEvent {
        chat: chat.clone(),
        event: RpcEvent::AgentSettled,
    });
    let (closed, _commands) = holder.dispatch_commands(UiIntent::CloseChat {
        chat_id: path.clone(),
    });
    assert_eq!(closed.open_chats, 0, "the close took it off the open set");
    assert_eq!(closed.foreground_chat, None);

    // What a stopped child can still leave behind, in the order it can arrive.
    holder.fold_input(Input::ChatStderr {
        chat: chat.clone(),
        line: "gentle-shell: goodbye".to_owned(),
    });
    assert_eq!(
        holder.view().open_chats,
        0,
        "a stderr line of a stopped child is not a chat the app has open"
    );

    holder.fold_input(Input::ChatEvent {
        chat: chat.clone(),
        event: RpcEvent::Response(gentle_core::rpc::RpcResponse {
            id: None,
            command: "get_session_stats".to_owned(),
            outcome: Ok(serde_json::json!({"tokens": 10, "contextWindow": 100})),
        }),
    });
    assert_eq!(
        holder.view().open_chats,
        0,
        "a reply that was already in flight does not reopen the chat"
    );

    holder.fold_input(Input::ChatClosed {
        chat: chat.clone(),
        expected: false,
    });
    let view = holder.view();
    assert_eq!(
        view.open_chats, 0,
        "the stop of a closed chat's child is not a chat back on the open set"
    );
    assert_eq!(view.foreground_chat, None, "nothing took the screen back");
    assert!(
        view.notices.is_empty(),
        "and it is not a Restart offered for a chat that is not open: {:?}",
        view.notices
    );
}

/// The other side of the same memory: a chat opened AGAIN after a close is a live
/// chat like any other, so what its new child says is folded and kept.
#[test]
fn a_chat_opened_again_after_a_close_is_live_again() {
    let scratch = Scratch::new("reopened");
    let path = scratch.session("a.jsonl");
    let chat = ChatId::of_session(&path);
    let holder = open_chat(&path);
    let (closed, _commands) = holder.dispatch_commands(UiIntent::CloseChat {
        chat_id: path.clone(),
    });
    assert_eq!(closed.open_chats, 0);

    let (reopened, _commands) = holder.dispatch_commands(UiIntent::OpenChat {
        chat_id: path.clone(),
    });
    assert_eq!(reopened.open_chats, 1, "the row opens it again");
    holder.fold_input(Input::ChatStderr {
        chat: chat.clone(),
        line: "gentle-shell: back".to_owned(),
    });
    holder.fold_input(Input::ChatEvent {
        chat: chat.clone(),
        event: RpcEvent::AgentStart,
    });
    assert_eq!(
        holder.view().activity.first().map(|mark| mark.activity),
        Some(gentle_desk_lib::contract::ChatActivityMark::Working),
        "the new child's own event marks it working"
    );
}

/// The other half of the same rule, for the LAZY start (welcome-screen T4): a
/// fresh chat this run allocated and never started has no child at all, so
/// closing it asks for no stop, leaves the gate at zero, and nothing of its own
/// reopens it.
#[test]
fn a_fresh_chat_that_never_started_closes_to_zero_chats() {
    let holder = AppStateHolder::default();
    let (allocated, _commands) = holder.dispatch_commands(UiIntent::OpenNewChat);
    let fresh = allocated.foreground_chat.expect("a new chat is in front");
    assert_eq!(allocated.open_chats, 1, "the click opened one chat");

    let (closed, commands) = holder.dispatch_commands(UiIntent::CloseChat {
        chat_id: fresh.clone(),
    });

    assert_eq!(
        closed.open_chats, 0,
        "the Updates gate has nothing left to wait for"
    );
    assert_eq!(closed.foreground_chat, None);
    assert!(
        commands
            .iter()
            .all(|command| matches!(command, ChatCommand::CloseProcess { chat } if chat == &ChatId::of_text(&fresh).expect("the fresh form"))),
        "a chat whose child never started asks for no other chat's process: {commands:?}"
    );
    assert!(closed.notices.is_empty(), "and nothing is reported");
}

/// A fresh chat that WAS started and closed leaves its child's own inputs behind
/// too: the key the runtime registered for `new-N` is not the app's open set, and
/// nothing it says brings the chat back.
#[test]
fn a_closed_fresh_chat_stays_closed_whatever_its_child_says() {
    let holder = AppStateHolder::default();
    let (allocated, _commands) = holder.dispatch_commands(UiIntent::OpenNewChat);
    let fresh = allocated.foreground_chat.expect("a new chat is in front");
    let (_started, commands) = holder.dispatch_commands(UiIntent::EnsureStarted {
        chat_id: fresh.clone(),
    });
    assert_eq!(
        commands,
        vec![ChatCommand::OpenProcess {
            chat: ChatId::of_text(&fresh).expect("the fresh form")
        }],
        "the composer's ask starts its child"
    );

    let (closed, _commands) = holder.dispatch_commands(UiIntent::CloseChat {
        chat_id: fresh.clone(),
    });
    assert_eq!(closed.open_chats, 0);

    holder.fold_input(Input::ChatClosed {
        chat: ChatId::of_text(&fresh).expect("the fresh form"),
        expected: true,
    });
    assert_eq!(
        holder.view().open_chats,
        0,
        "an expected stop of a closed chat's child reopens nothing"
    );
}
