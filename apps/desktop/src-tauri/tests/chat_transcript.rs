//! The thread's first frame, painted from the session FILE while a stored
//! chat's child is still starting (`chat-open-latency.md` T1, T2, S1-S4).
//!
//! The rules this file pins are the ones the previous slice's review lost
//! (R3-001 of `chat-transcript.md`): a chat id that closes and reopens is a NEW
//! incarnation, so a read that started before the close is dropped instead of
//! installed, and once the child's own `get_messages` answer has been folded the
//! file's rows are neither drawn nor re-installed. Nothing here starts a child,
//! opens a window or reads a real file: the session-file seam is the fake of
//! `settings::FileRead`, and one case exercises the worker the shell really
//! spawns.

use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::sync::{Arc, Mutex, Weak};
use std::time::Duration;

use gentle_app::{ChatId, Input, Intent};
use gentle_core::chat::SESSION_FILE_MAX_BYTES;
use gentle_core::i18n::Language;
use gentle_core::rpc::{decode_line, Decoded};
use gentle_desk_lib::bridge::AppStateHolder;
use gentle_desk_lib::contract::{ChatItemView, UiIntent, ViewState};
use gentle_desk_lib::settings::FileRead;

const SESSION_PATH: &str = "/sessions/a.jsonl";
const STORE: &str = "/sessions";

/// One `message` line of a session file, the shape the child writes (no line
/// ending of its own: [`file`] and the partial-file case join them).
fn line(role: &str, text: &str) -> String {
    format!(
        r#"{{"type":"message","id":"m","timestamp":"2026-01-01T00:01:00.000Z","message":{{"role":"{role}","content":[{{"type":"text","text":"{text}"}}]}}}}"#
    )
}

/// A file of one exchange, the header included: what a stored chat holds.
fn file() -> String {
    format!(
        "{}\n{}\n{}\n",
        r#"{"type":"session","version":3,"id":"s1","timestamp":"2026-01-01T00:00:00.000Z","cwd":"C:\\dev\\x"}"#,
        line("user", "a pregunta"),
        line("assistant", "a resposta")
    )
}

/// The chat of that stored session.
fn stored() -> ChatId {
    ChatId::of_session(SESSION_PATH)
}

fn lock<T>(mutex: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|poison| poison.into_inner())
}

/// A session-file seam that answers `answer` and remembers every path it was
/// asked for.
fn files(answer: Result<Option<String>, String>) -> (FileRead, Arc<Mutex<Vec<PathBuf>>>) {
    let asked: Arc<Mutex<Vec<PathBuf>>> = Arc::new(Mutex::new(Vec::new()));
    let kept = Arc::clone(&asked);
    let read: FileRead = Arc::new(move |path: &Path, _language: Language| {
        lock(&kept).push(path.to_path_buf());
        answer.clone()
    });
    (read, asked)
}

/// The holder with the seam armed and the stored chat OPENED, exactly as the
/// web's `OpenChat` leaves it: foreground, opening, and no row of its own.
fn opened_with(answer: Result<Option<String>, String>) -> (AppStateHolder, Arc<Mutex<Vec<PathBuf>>>) {
    let holder = AppStateHolder::default();
    let (read, asked) = files(answer);
    holder.arm_session_files(read, PathBuf::from(STORE));
    holder.fold_input(Input::Intent(Intent::OpenChat(stored())));
    (holder, asked)
}

/// The answer the child gives to `get_messages`: one user turn, no failure.
fn history(user_text: &str) -> gentle_core::rpc::RpcEvent {
    let text = format!(
        r#"{{"type":"response","command":"get_messages","success":true,"data":{{"messages":[{{"role":"user","content":[{{"type":"text","text":"{user_text}"}}]}}]}}}}"#
    );
    match decode_line(&text) {
        Decoded::Event(event) => event,
        _ => panic!("a `get_messages` answer is an event"),
    }
}

/// One event of that chat's child, as the pump folds it.
fn folded(holder: &AppStateHolder, chat: &ChatId, event: gentle_core::rpc::RpcEvent) -> ViewState {
    holder.fold_input(Input::ChatEvent {
        chat: chat.clone(),
        event,
    })
}

/// Every drawn row as (id, text), whichever shape it has: what a test compares
/// without caring whether the row came from the file or from the child.
fn rows(view: &ViewState) -> Vec<(String, String)> {
    view.items
        .iter()
        .map(|item| match item {
            ChatItemView::Message(message) => (message.id.clone(), message.text.clone()),
            ChatItemView::Thinking(thinking) => (thinking.id.clone(), thinking.text.clone()),
            ChatItemView::Tool(tool) => (tool.id.clone(), tool.tool_name.clone()),
        })
        .collect()
}

#[test]
fn the_open_of_a_stored_chat_claims_one_read_and_the_file_paints_the_thread() {
    let (holder, asked) = opened_with(Ok(Some(file())));

    let read = holder
        .claim_transcript()
        .expect("a stored chat with no rows of its own claims one read");
    assert_eq!(read.chat, stored());
    assert!(
        lock(&asked).is_empty(),
        "claiming reads no byte: the read is the caller's, on its own thread"
    );
    let drawn = holder
        .note_transcript(&read.read())
        .expect("the read's rows are pushed");

    assert_eq!(
        drawn.transcript_provisional, true,
        "what is drawn right now came from the file"
    );
    assert_eq!(
        rows(&drawn),
        [
            ("prov-0".to_owned(), "a pregunta".to_owned()),
            ("prov-1".to_owned(), "a resposta".to_owned()),
        ],
        "the file's turns, in its own order, under ids of their own"
    );
    assert_eq!(
        drawn
            .messages
            .iter()
            .map(|message| (message.role, message.text.as_str()))
            .collect::<Vec<_>>(),
        [
            (
                gentle_desk_lib::contract::MessageRole::User,
                "a pregunta"
            ),
            (
                gentle_desk_lib::contract::MessageRole::Assistant,
                "a resposta"
            ),
        ],
        "both lists carry the same rows, as every push does"
    );
    assert_eq!(
        *lock(&asked),
        [PathBuf::from(SESSION_PATH)],
        "read exactly once"
    );

    assert!(
        holder.claim_transcript().is_none(),
        "one open asks for one read, and only one"
    );
}

#[test]
fn the_childs_answer_replaces_the_rows_and_a_late_read_is_dropped() {
    let (holder, _asked) = opened_with(Ok(Some(file())));
    let read = holder.claim_transcript().expect("the one read");

    // The child answers first: its history is the thread from now on.
    let answered = folded(&holder, &stored(), history("the child's own turn"));
    assert_eq!(
        rows(&answered),
        [("msg-0".to_owned(), "the child's own turn".to_owned())]
    );
    assert_eq!(
        answered.transcript_provisional, false,
        "the child's answer is the authority, always"
    );

    // The read that started before it lands now. It may not re-install itself.
    assert_eq!(
        holder.note_transcript(&read.read()),
        None,
        "a read that lands after the answer is dropped, never folded over it"
    );
    let now = holder.view();
    assert_eq!(
        rows(&now),
        [("msg-0".to_owned(), "the child's own turn".to_owned())],
        "the child's row is still the only one drawn"
    );
    assert_eq!(now.transcript_provisional, false);
}

#[test]
fn a_file_row_also_loses_to_an_answer_that_carries_no_row_at_all() {
    // The answer is an ANSWER even when it is empty: emptiness cannot stand for
    // "not loaded yet", or a chat whose file exists and whose child says it has
    // nothing would paint the file for ever.
    let (holder, _asked) = opened_with(Ok(Some(file())));
    let read = holder.claim_transcript().expect("the one read");
    let answered = folded(&holder, &stored(), history(""));
    assert!(answered.items.is_empty(), "the child sent no turn");
    assert_eq!(answered.transcript_provisional, false);

    assert_eq!(holder.note_transcript(&read.read()), None);
    assert!(holder.view().items.is_empty());
}

#[test]
fn a_read_of_a_chat_that_closed_and_reopened_is_dropped() {
    // The lesson of R3-001: the same chat id coming back is a NEW incarnation,
    // and the read of the old one belongs to a chat that is gone.
    let (holder, _asked) = opened_with(Ok(Some(file())));
    let old = holder.claim_transcript().expect("the first incarnation's read");

    holder.fold_input(Input::Intent(Intent::CloseChat(stored())));
    holder.fold_input(Input::Intent(Intent::OpenChat(stored())));
    let new = holder
        .claim_transcript()
        .expect("the new incarnation asks for its own read");
    assert_ne!(
        old.started_in, new.started_in,
        "the two incarnations are told apart"
    );

    assert_eq!(
        holder.note_transcript(&old.read()),
        None,
        "the old read is dropped: its chat closed"
    );
    assert!(
        holder.view().items.is_empty(),
        "and it drew nothing at all on the chat that is open now"
    );

    let drawn = holder
        .note_transcript(&new.read())
        .expect("the new read is the one this chat draws");
    assert_eq!(drawn.transcript_provisional, true);
    assert_eq!(drawn.items.len(), 2);
}

#[test]
fn a_file_that_draws_nothing_draws_nothing_and_says_nothing() {
    // Every way a read comes up empty: no file, one that cannot be read, one
    // holding nothing but the line being written, and one past the cap. A file
    // being written draws the turns that ARE whole (that is the parser's case);
    // what it must never do is draw the half-written line or say anything about
    // it.
    let oversize = format!("{}{}", file(), "x".repeat(SESSION_FILE_MAX_BYTES));
    let being_written = r#"{"type":"message","message":{"role":"assistant","content":[{"type":"te"#.to_owned();
    let cases: [(&str, Result<Option<String>, String>); 4] = [
        ("a file that is not there", Ok(None)),
        ("a file that cannot be read", Err("denied".to_owned())),
        ("a file holding one half-written line", Ok(Some(being_written))),
        ("a file past the cap", Ok(Some(oversize))),
    ];

    for (case, answer) in cases {
        let (holder, _asked) = opened_with(answer);
        let read = holder.claim_transcript().expect("the read is claimed");

        assert_eq!(
            holder.note_transcript(&read.read()),
            None,
            "{case}: nothing to draw is no push at all"
        );
        let view = holder.view();
        assert!(view.items.is_empty(), "{case}");
        assert_eq!(view.transcript_provisional, false, "{case}");
        assert!(
            view.notices.is_empty(),
            "{case}: a file the app could not use is not news for the user"
        );
    }
}

#[test]
fn a_chat_that_already_holds_rows_never_reads_its_file() {
    let holder = AppStateHolder::default();
    let (read, _asked) = files(Ok(Some(file())));
    holder.arm_session_files(read, PathBuf::from(STORE));

    // The chat opens, its child is up and the user's own prompt becomes a row.
    holder.fold_input(Input::Intent(Intent::OpenChat(stored())));
    holder.fold_input(Input::ChatOpened { chat: stored() });
    holder.fold_input(Input::Intent(Intent::SendPrompt("typed here".to_owned())));
    let mine = holder.view();
    assert_eq!(
        rows(&mine),
        [("msg-0".to_owned(), "typed here".to_owned())],
        "the row of the prompt the user really sent"
    );

    // Away to a fresh chat and back: the thread it kept is what it draws.
    holder.fold_input(Input::Intent(Intent::OpenNewChat));
    holder.fold_input(Input::Intent(Intent::OpenChat(stored())));

    assert!(
        holder.claim_transcript().is_none(),
        "a chat that already holds rows of its own asks its file for none"
    );
    let back = holder.view();
    assert_eq!(
        rows(&back),
        [("msg-0".to_owned(), "typed here".to_owned())]
    );
    assert_eq!(
        back.transcript_provisional, false,
        "rows the reducer holds are never called provisional"
    );
}

#[test]
fn the_read_runs_on_a_worker_and_pushes_its_answer_exactly_once() {
    // The path the app really takes: the open claims the read in the holder and
    // the read runs on the shell's worker thread, which pushes once - not per
    // line, and not never.
    let holder = Arc::new(AppStateHolder::default());
    let (read, asked) = files(Ok(Some(file())));
    holder.arm_session_files(read, PathBuf::from(STORE));
    let (pushed, wait) = mpsc::channel::<ViewState>();
    let weak: Weak<AppStateHolder> = Arc::downgrade(&holder);
    let once = Arc::new(Mutex::new(0usize));
    let counted = Arc::clone(&once);
    holder.arm_transcript_push(Arc::new(move |read| {
        *lock(&counted) += 1;
        if let Some(holder) = weak.upgrade() {
            if let Some(view) = holder.note_transcript(&read) {
                let _ = pushed.send(view);
            }
        }
    }));

    let (opening, _commands) = holder.dispatch_commands(UiIntent::OpenChat {
        chat_id: SESSION_PATH.to_owned(),
    });
    assert!(
        opening.items.is_empty(),
        "the intent path ANSWERS before the file is read: it never reads it itself"
    );

    let drawn = wait
        .recv_timeout(Duration::from_secs(30))
        .expect("the read never pushed its answer");
    assert_eq!(
        rows(&drawn),
        [
            ("prov-0".to_owned(), "a pregunta".to_owned()),
            ("prov-1".to_owned(), "a resposta".to_owned()),
        ]
    );
    assert_eq!(drawn.transcript_provisional, true);
    assert_eq!(
        lock(&asked).len(),
        1,
        "the file is read once for the one open"
    );
    assert_eq!(*lock(&once), 1, "and its answer is pushed exactly once");
}
