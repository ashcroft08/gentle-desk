//! Windowless tests of the Copy action (T5a): the intent the web side sends
//! back names ONE message, and the shell writes exactly that message's text to
//! the clipboard it holds, off the caller's thread.
//!
//! The clipboard is the seam ([`Clipboard`]), so a fake records every write and
//! nothing here touches the system clipboard of the machine running the tests.

use std::sync::{Arc, Mutex, MutexGuard};
use std::time::{Duration, Instant};

use gentle_app::{ChatId, Input, Intent};
use gentle_desk_lib::bridge::AppStateHolder;
use gentle_desk_lib::clipboard::Clipboard;
use gentle_desk_lib::commands::command_sink;
use gentle_desk_lib::contract::UiIntent;

/// Every wait of these tests: one spawned thread, never minutes.
const WAIT: Duration = Duration::from_secs(10);

/// How long a "writes nothing" case is watched before it is believed.
const QUIET: Duration = Duration::from_millis(300);

/// The clipboard a test injects: it remembers every text it was asked to write,
/// including the ones it failed to write.
#[derive(Default)]
struct FakeClipboard {
    written: Mutex<Vec<String>>,
    /// Whether every write fails, which is what the real clipboard does when
    /// another program holds it.
    failing: bool,
}

impl FakeClipboard {
    fn failing() -> Self {
        Self {
            written: Mutex::new(Vec::new()),
            failing: true,
        }
    }

    fn written(&self) -> Vec<String> {
        lock(&self.written).clone()
    }
}

impl Clipboard for FakeClipboard {
    fn copy_text(&self, text: &str) -> Result<(), String> {
        lock(&self.written).push(text.to_owned());
        if self.failing {
            Err("the clipboard is held by another program".to_owned())
        } else {
            Ok(())
        }
    }
}

/// A holder wired to `clipboard` the way startup wires the real one, with no
/// session service: the list branch of the sink answers nothing here.
fn armed_holder(clipboard: Arc<FakeClipboard>) -> AppStateHolder {
    let holder = AppStateHolder::default();
    let seam: Arc<dyn Clipboard> = clipboard;
    holder.arm_commands(command_sink(None, seam));
    holder
}

/// A holder with one finished turn in it: the user's `msg-0` in the foreground
/// chat, which the reducer opens and confirms the way the app does before a
/// prompt.
fn holder_with_a_turn(clipboard: Arc<FakeClipboard>) -> AppStateHolder {
    let holder = armed_holder(clipboard);
    open_and_prompt(&holder);
    holder
}

/// Opens the stored chat, confirms it and sends `hello` in it.
fn open_and_prompt(holder: &AppStateHolder) {
    let chat = ChatId::of_session("/sessions/a.jsonl");
    holder.fold_input(Input::Intent(Intent::OpenChat(chat.clone())));
    // A chat that is still opening takes no prompt, exactly as in the app.
    holder.fold_input(Input::ChatOpened { chat });
    holder.fold_input(Input::Intent(Intent::SendPrompt("hello".to_owned())));
}

#[test]
fn copying_a_finished_message_writes_exactly_its_text_once() {
    let fake = Arc::new(FakeClipboard::default());
    let holder = holder_with_a_turn(Arc::clone(&fake));
    let before = holder.view();
    assert_eq!(before.messages[0].id, "msg-0");

    let after = holder.dispatch(UiIntent::CopyMessage {
        message_id: "msg-0".to_owned(),
    });

    assert_eq!(
        wait_for(&fake),
        vec!["hello".to_owned()],
        "the message's own text reaches the clipboard, once"
    );
    assert_eq!(after, before, "copying changes nothing the view renders");
    std::thread::sleep(QUIET);
    assert_eq!(
        fake.written(),
        vec!["hello".to_owned()],
        "one copy, one write: the sink answers the command once"
    );
}

#[test]
fn copying_an_unknown_message_writes_nothing() {
    let fake = Arc::new(FakeClipboard::default());
    let holder = holder_with_a_turn(Arc::clone(&fake));
    let before = holder.view();

    let after = holder.dispatch(UiIntent::CopyMessage {
        message_id: "msg-77".to_owned(),
    });

    assert_eq!(after, before, "an unknown id folds to no change");
    std::thread::sleep(QUIET);
    assert_eq!(
        fake.written(),
        Vec::<String>::new(),
        "an id the thread never named writes nothing at all"
    );
}

#[test]
fn a_failing_clipboard_is_a_logged_line_and_never_a_panic() {
    let fake = Arc::new(FakeClipboard::failing());
    let holder = holder_with_a_turn(Arc::clone(&fake));

    let after = holder.dispatch(UiIntent::CopyMessage {
        message_id: "msg-0".to_owned(),
    });

    // The write was attempted, and its failure neither panicked the copy thread
    // nor reached the view: the slice is the one the fold answered.
    assert_eq!(wait_for(&fake), vec!["hello".to_owned()]);
    assert_eq!(after, holder.view());
}

/// Pinned: arming the sink with a clipboard leaves the other paths of the
/// reducer exactly as they were.
#[test]
fn the_other_commands_of_the_sink_still_fold() {
    let fake = Arc::new(FakeClipboard::default());
    let holder = armed_holder(Arc::clone(&fake));
    let chat = ChatId::of_session("/sessions/a.jsonl");
    holder.fold_input(Input::Intent(Intent::OpenChat(chat.clone())));
    holder.fold_input(Input::ChatOpened { chat });

    let prompted = holder.fold_input(Input::Intent(Intent::SendPrompt("hello".to_owned())));
    assert_eq!(
        prompted
            .messages
            .first()
            .map(|message| message.text.as_str()),
        Some("hello"),
        "a prompt still reaches the thread"
    );

    let dismissed = holder.dispatch(UiIntent::DismissNotice { id: 0 });
    assert_eq!(dismissed.notices, Vec::new());
    std::thread::sleep(QUIET);
    assert_eq!(
        fake.written(),
        Vec::<String>::new(),
        "no command wrote to the clipboard by itself"
    );
}

/// Waits for the copy thread to reach the clipboard, answering what it wrote.
fn wait_for(fake: &FakeClipboard) -> Vec<String> {
    let deadline = Instant::now() + WAIT;
    loop {
        let written = fake.written();
        if !written.is_empty() {
            return written;
        }
        assert!(
            Instant::now() < deadline,
            "waited {WAIT:?} for the copy thread to reach the clipboard"
        );
        std::thread::sleep(Duration::from_millis(10));
    }
}

/// A lock that survives a poisoned mutex: a test that failed while holding one
/// must not turn into a second failure.
fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|poison| poison.into_inner())
}
