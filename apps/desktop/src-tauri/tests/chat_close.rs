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
        close_working(),
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
