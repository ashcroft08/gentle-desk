//! Close every chat through the bridge (close-chats T1, S2, S3): the contract's
//! `CloseAllChats` intent takes every open chat off the open set the Updates
//! pane's gate counts, asks the shell to stop each of those children, and leaves
//! every session file exactly where it was. A chat that is working is skipped
//! silently - the guard `CloseChat` has is kept, never bypassed.
//!
//! The reducer's own rules are tested in `gentle-app`; what this file adds is the
//! contract's intent on the real dispatch path - the slice the web side receives
//! and the one file that must NOT change.

use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};

use gentle_app::{ChatCommand, ChatId, Input, Intent, SessionItem};
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
            "gentle-desk-chat-close-all-{}-{name}-{}",
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

/// The holder with every stored chat opened the way a click on its row opens it:
/// the last one foreground, all of them with their child up.
fn opened(paths: &[&str]) -> AppStateHolder {
    let holder = AppStateHolder::default();
    holder.fold_input(Input::SessionsLoaded(
        paths.iter().map(|path| session(path)).collect(),
    ));
    for path in paths {
        let chat = ChatId::of_session(path);
        holder.fold_input(Input::Intent(Intent::OpenChat(chat.clone())));
        holder.fold_input(Input::ChatOpened { chat });
    }
    holder
}

#[test]
fn a_bulk_close_takes_every_open_chat_off_the_gate_and_leaves_the_files_in_place() {
    let scratch = Scratch::new("all");
    let path = scratch.session("a.jsonl");
    let holder = opened(&[&path]);
    let before = holder.view();
    assert_eq!(before.open_chats, 1, "the gate counts it as open");

    let (view, commands) = holder.dispatch_commands(UiIntent::CloseAllChats);

    assert_eq!(
        view.open_chats, 0,
        "the Updates gate has nothing to wait for"
    );
    assert_eq!(view.foreground_chat, None, "and nothing is on screen");
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
fn a_bulk_close_skips_the_working_chat_and_closes_the_others() {
    let scratch = Scratch::new("working");
    let idle = scratch.session("a.jsonl");
    let working = scratch.session("b.jsonl");
    let holder = opened(&[&idle, &working]);
    // The chat in front is mid-turn: CloseChat refuses it, so this must too.
    holder.fold_input(Input::ChatEvent {
        chat: ChatId::of_session(&working),
        event: RpcEvent::AgentStart,
    });
    let before = holder.view();
    assert_eq!(before.open_chats, 2);

    let (view, commands) = holder.dispatch_commands(UiIntent::CloseAllChats);

    assert_eq!(
        view.open_chats, 1,
        "the working chat is still open, the idle one is not"
    );
    assert_eq!(
        commands,
        vec![ChatCommand::CloseProcess {
            chat: ChatId::of_session(&idle)
        }],
        "only the idle chat's child is stopped"
    );
    assert_eq!(
        view.notices, before.notices,
        "the skipped chat is not reported one by one"
    );
    assert!(
        std::path::Path::new(&working).exists() && std::path::Path::new(&idle).exists(),
        "a refusal and a close both leave every file alone"
    );
}

#[test]
fn a_bulk_close_with_nothing_open_changes_nothing() {
    let holder = AppStateHolder::default();
    let before = holder.view();

    let (view, commands) = holder.dispatch_commands(UiIntent::CloseAllChats);

    assert_eq!(commands, Vec::new(), "no child to stop");
    assert_eq!(view.open_chats, 0);
    assert_eq!(view.notices, before.notices, "and nothing is reported");
}
