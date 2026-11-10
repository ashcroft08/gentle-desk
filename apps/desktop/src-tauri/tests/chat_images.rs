//! Images attached to a chat: the paths the drop names are read and validated
//! HERE, held per chat, and merged into that chat's send when the executor runs
//! it. Nothing of an image ever travels the other way: the web side sends paths
//! and the shell answers with a preview URL and a name.
//!
//! Windowless: the store, the merge and the preview scheme are plain items, so
//! every refusal is asserted on the value the bridge pushes. The one test that
//! needs a live child drives the REAL executor against the stand-in
//! (`tools/fake-gentle-shell/fake.mjs` via `GENTLE_SHELL_BIN`), so what is
//! asserted is the prompt the child actually received.

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::{Duration, Instant};

use gentle_app::{ChatCommand, ChatId};
use gentle_core::rpc::{encode_command, ImageContent, RpcCommand};
use gentle_desk_lib::bridge::AppStateHolder;
use gentle_desk_lib::contract::{ChatActivityMark, MessageRole, UiIntent};
use gentle_desk_lib::effects::{self, RuntimeExecutor};
use gentle_desk_lib::events::{EventPump, Push};
use gentle_desk_lib::images::{self, Images};
use gentle_desk_lib::pending::PendingFolders;

/// A chat id the store is asked about: the bridge keys attachments by the text
/// `ViewState::foreground_chat` carries.
const CHAT: &str = "/sessions/a.jsonl";

/// The bytes of a "png" for these tests: the store never decodes an image, it
/// reads it and validates its extension, so the content is whatever the test
/// writes.
const PNG: &[u8] = b"\x89PNG\r\n\x1a\none";

fn chat_id() -> ChatId {
    ChatId::of_session(CHAT)
}

/// A temp directory of the system tempdir, removed when the test ends.
struct Scratch(tempfile::TempDir);

impl Scratch {
    fn new() -> Self {
        Self(tempfile::tempdir().expect("a temp dir outside the worktree"))
    }

    /// A file of `name` holding `bytes`, as the drop would name it.
    fn file(&self, name: &str, bytes: &[u8]) -> String {
        let path = self.0.path().join(name);
        std::fs::write(&path, bytes).expect("the file is writable");
        text(&path)
    }

    /// A file of `name` that CLAIMS `size` bytes without writing them.
    fn sparse(&self, name: &str, size: u64) -> String {
        let path = self.0.path().join(name);
        let file = std::fs::File::create(&path).expect("the file is creatable");
        file.set_len(size).expect("the file takes its length");
        text(&path)
    }
}

fn text(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

// The store and the merge.

#[test]
fn a_dropped_png_is_read_once_and_held_for_its_chat() {
    let scratch = Scratch::new();
    let path = scratch.file("shot.png", PNG);
    let images = Images::default();

    images.attach(CHAT, &[path]);

    let held = images.pending(CHAT);
    assert_eq!(held.len(), 1, "the drop attaches one image");
    assert_eq!(held[0].name, "shot.png", "the name is the file's own");
    assert!(
        held[0].preview_url.ends_with(&held[0].id),
        "the preview URL names the image: {}",
        held[0].preview_url
    );
    assert!(
        !held[0].preview_url.contains("shot.png"),
        "the URL carries no filesystem path: {}",
        held[0].preview_url
    );

    let taken = images.take(CHAT);
    assert_eq!(taken.len(), 1, "the send takes what waited");
    assert_eq!(
        taken[0].data,
        gentle_core::rpc::base64::encode(PNG),
        "the file is read here, once, and base64 encoded for pi's field"
    );
    assert_eq!(taken[0].mime.essence(), "image/png");
    assert!(images.pending(CHAT).is_empty(), "a taken image is gone");
}

#[test]
fn a_file_that_is_not_an_image_is_refused_with_a_plain_reason_and_nothing_attaches() {
    let scratch = Scratch::new();
    let notes = scratch.file("notes.txt", b"not an image");
    let images = Images::default();

    images.attach(CHAT, &[notes]);

    assert!(images.pending(CHAT).is_empty(), "nothing is attached");
    let notice = images.notice().expect("a refusal says why");
    assert!(notice.is_ascii(), "got {notice:?}");
    assert!(notice.contains("notes.txt"), "names the file: {notice:?}");
    assert!(
        notice.contains("was not attached"),
        "says nothing happened: {notice:?}"
    );
    assert!(
        notice.contains("png"),
        "says what would have worked: {notice:?}"
    );
}

#[test]
fn a_file_that_is_no_longer_there_is_refused() {
    let scratch = Scratch::new();
    let gone = text(&scratch.0.path().join("gone.png"));
    let images = Images::default();

    images.attach(CHAT, &[gone]);

    assert!(images.pending(CHAT).is_empty());
    let notice = images.notice().expect("a refusal says why");
    assert!(notice.is_ascii(), "got {notice:?}");
    assert!(notice.contains("gone.png"), "names the file: {notice:?}");
}

#[test]
fn a_folder_is_refused_even_when_it_is_named_like_an_image() {
    let scratch = Scratch::new();
    let folder = scratch.0.path().join("shot.png");
    std::fs::create_dir(&folder).unwrap();
    let images = Images::default();

    images.attach(CHAT, &[text(&folder)]);

    assert!(images.pending(CHAT).is_empty());
    let notice = images.notice().expect("a refusal says why");
    assert!(notice.is_ascii(), "got {notice:?}");
    assert!(notice.contains("folder"), "got {notice:?}");
}

#[test]
fn a_message_holds_at_most_eight_images() {
    let scratch = Scratch::new();
    let paths: Vec<String> = (0..10)
        .map(|at| scratch.file(&format!("shot-{at}.png"), PNG))
        .collect();
    let images = Images::default();

    images.attach(CHAT, &paths);

    assert_eq!(
        images.pending(CHAT).len(),
        images::MAX_IMAGES,
        "the cap is the eight a message takes"
    );
    let notice = images.notice().expect("the ninth is refused with a reason");
    assert!(notice.is_ascii(), "got {notice:?}");
    assert!(notice.contains("8 images"), "got {notice:?}");
    assert!(
        notice.contains("shot-8.png"),
        "names the first file the cap refused: {notice:?}"
    );
}

#[test]
fn a_message_holds_at_most_the_64_mb_of_the_cap() {
    let scratch = Scratch::new();
    let big = scratch.sparse("big.png", images::MAX_TOTAL / 2 + 1);
    let second = scratch.sparse("second.png", images::MAX_TOTAL / 2 + 1);
    let images = Images::default();

    images.attach(CHAT, &[big, second]);

    assert_eq!(
        images.pending(CHAT).len(),
        1,
        "the file that would pass the cap is refused"
    );
    let notice = images.notice().expect("a refusal says why");
    assert!(notice.is_ascii(), "got {notice:?}");
    assert!(notice.contains("second.png"), "got {notice:?}");
    assert!(notice.contains("64 MB"), "got {notice:?}");
}

#[test]
fn a_file_larger_than_the_whole_cap_is_refused_without_being_read() {
    let scratch = Scratch::new();
    let huge = scratch.sparse("huge.png", images::MAX_TOTAL + 1);
    let images = Images::default();

    images.attach(CHAT, &[huge]);

    assert!(images.pending(CHAT).is_empty());
    let notice = images.notice().expect("a refusal says why");
    assert!(notice.contains("64 MB"), "got {notice:?}");
}

#[test]
fn removing_an_image_takes_it_away_and_its_send_no_longer_carries_it() {
    let scratch = Scratch::new();
    let one = scratch.file("one.png", PNG);
    let two = scratch.file("two.png", PNG);
    let images = Images::default();
    images.attach(CHAT, &[one, two]);
    let first = images.pending(CHAT)[0].id.clone();

    let removed = images.remove(&first);

    assert!(removed, "the composer's remove names an image it holds");
    let left = images.pending(CHAT);
    assert_eq!(left.len(), 1, "one image is left");
    assert_eq!(left[0].name, "two.png");
    assert!(!images.remove(&first), "a removed id is gone");
}

// What the send carries (S3, S5).

#[test]
fn the_send_carries_the_path_as_text_and_the_image_in_pis_field() {
    let scratch = Scratch::new();
    let path = scratch.file("shot.png", PNG);
    let images = Images::default();
    images.attach(CHAT, std::slice::from_ref(&path));
    let attachments = images.take(CHAT);

    let sent = images::with_attachments(
        ChatCommand::Send {
            chat: chat_id(),
            rpc: RpcCommand::prompt("look at this"),
        },
        attachments,
    );

    let ChatCommand::Send { rpc, .. } = sent else {
        panic!("a send stays a send");
    };
    assert_eq!(
        rpc,
        RpcCommand::prompt_with_images(
            format!("look at this\n\nAttached image: {path}"),
            vec![ImageContent {
                mime_type: "image/png".to_owned(),
                data: gentle_core::rpc::base64::encode(PNG),
            }],
        ),
        "the transcript reads the same and pi's own field carries the image"
    );
    // The exact line the child is written, so the shape is pinned to pi's.
    let line = encode_command(&rpc);
    assert!(
        line.contains("\"mimeType\":\"image/png\""),
        "pi's own image shape: {line}"
    );
    assert!(
        line.contains(&path.replace('\\', "\\\\")),
        "the path is text"
    );
}

#[test]
fn a_send_with_no_images_is_the_prompt_it_always_was() {
    // PRESERVE: a chat that attached nothing sends the line it always sent, so
    // every existing prompt stays byte identical.
    let command = ChatCommand::Send {
        chat: chat_id(),
        rpc: RpcCommand::Prompt {
            message: "hola".to_owned(),
        },
    };

    let sent = images::with_attachments(command.clone(), Vec::new());

    assert_eq!(sent, command);
    let ChatCommand::Send { rpc, .. } = &sent else {
        panic!("a send stays a send");
    };
    assert_eq!(
        encode_command(rpc),
        "{\"message\":\"hola\",\"type\":\"prompt\"}\n"
    );
}

#[test]
fn a_send_for_another_chat_carries_no_images() {
    let scratch = Scratch::new();
    let path = scratch.file("shot.png", PNG);
    let images = Images::default();
    images.attach(CHAT, &[path]);

    let other = images.take("/sessions/b.jsonl");

    assert!(other.is_empty(), "another chat's send takes nothing");
    assert_eq!(
        images.pending(CHAT).len(),
        1,
        "the chat they were attached to still holds them"
    );
}

// The preview the web draws (S2), served by the shell and never sent up.

#[test]
fn the_preview_scheme_serves_the_dropped_files_and_nothing_else() {
    let scratch = Scratch::new();
    let path = scratch.file("shot.png", PNG);
    let images = Images::default();
    images.attach(CHAT, &[path]);
    let id = images.pending(CHAT)[0].id.clone();

    let served = images.read(&id).expect("the dropped file is served");

    assert_eq!(served.0, "image/png", "with the type the browser needs");
    assert_eq!(served.1, PNG, "the bytes are the file's");
    assert!(
        images.read("img-9999").is_none(),
        "an id this run never attached is not served"
    );
    assert!(
        images.read("../config.json").is_none(),
        "a path is never a key: only ids the user's own drops created"
    );
}

#[test]
fn a_preview_url_is_the_scheme_of_this_platform() {
    let url = images::preview_url("img-1");

    if cfg!(windows) {
        assert_eq!(
            url, "http://gentle-image.localhost/img-1",
            "the form Tauri serves a custom scheme under on Windows"
        );
    } else {
        assert_eq!(url, "gentle-image://localhost/img-1");
    }
}

// The send through the executor, against the stand-in child.

/// The stand-in child, relative to the repository root.
const FAKE: &str = "tools/fake-gentle-shell/fake.mjs";

const WAIT: Duration = Duration::from_secs(15);

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(3)
        .expect("the workspace root is three levels above apps/desktop/src-tauri")
        .to_path_buf()
}

/// The environment is process-global and the tests must not see each other's
/// child, so only one of them runs at a time.
static ENV_LOCK: Mutex<()> = Mutex::new(());

/// One test's isolated world: its own home, its own log, never the user's.
struct Harness {
    _serial: MutexGuard<'static, ()>,
    root: PathBuf,
    log: PathBuf,
}

impl Harness {
    fn new() -> Option<Self> {
        if !node_on_path() {
            eprintln!("skipping: node is not on PATH, so the fake gentle-shell cannot run");
            return None;
        }
        let serial = ENV_LOCK.lock().unwrap_or_else(|poison| poison.into_inner());
        let root = std::env::temp_dir().join(format!(
            "gentle-desk-chat-images-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|since| since.as_nanos())
                .unwrap_or(0)
        ));
        std::fs::create_dir_all(root.join("home")).unwrap();
        let log = root.join("fake.log");
        std::env::set_var("GENTLE_SHELL_BIN", repo_root().join(FAKE));
        std::env::set_var("GENTLE_SHELL_HOME", root.join("home"));
        std::env::set_var("FAKE_SHELL_LOG", &log);
        Some(Self {
            _serial: serial,
            root,
            log,
        })
    }

    /// The commands the child received, oldest first.
    fn commands(&self) -> Vec<serde_json::Value> {
        let Ok(text) = std::fs::read_to_string(&self.log) else {
            return Vec::new();
        };
        text.lines()
            .filter_map(|line| serde_json::from_str::<serde_json::Value>(line).ok())
            .filter(|entry| entry["event"] == "command")
            .collect()
    }
}

impl Drop for Harness {
    fn drop(&mut self) {
        for name in ["GENTLE_SHELL_BIN", "GENTLE_SHELL_HOME", "FAKE_SHELL_LOG"] {
            std::env::remove_var(name);
        }
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

fn node_on_path() -> bool {
    let name = if cfg!(windows) { "node.exe" } else { "node" };
    let Some(path) = std::env::var_os("PATH") else {
        return false;
    };
    std::env::split_paths(&path).any(|dir| dir.join(name).is_file())
}

/// Waits for what a bounded wait is about, failing the test instead of hanging.
fn wait_for<T>(what: &str, mut probe: impl FnMut() -> Option<T>) -> T {
    let deadline = Instant::now() + WAIT;
    loop {
        if let Some(value) = probe() {
            return value;
        }
        assert!(
            Instant::now() < deadline,
            "waited {WAIT:?} for {what} and it never came"
        );
        std::thread::sleep(Duration::from_millis(20));
    }
}

/// The bridge pieces `Bridge::dispatch` drives, inline instead of on worker
/// threads so the order is fixed: no window is needed and none is built.
struct Bridge {
    holder: Arc<AppStateHolder>,
    executor: Arc<RuntimeExecutor>,
}

impl Bridge {
    fn new() -> Self {
        let holder = Arc::new(AppStateHolder::default());
        let pump = Arc::new(EventPump::new(Arc::clone(&holder)));
        // The push itself is the app's business; here only the folding matters.
        let push: Push = Box::new(|_view| {});
        pump.arm(push);
        let folders = Arc::new(PendingFolders::default());
        let executor = Arc::new(RuntimeExecutor::new(
            Arc::clone(&pump),
            folders,
            Arc::clone(&holder),
        ));
        Self { holder, executor }
    }

    fn dispatch(&self, intent: UiIntent) {
        let (_, commands) = self.holder.dispatch_commands(intent);
        for command in commands {
            let ran = effects::run_one(command, self.executor.as_ref());
            for input in ran.inputs {
                self.holder.fold_input(input);
            }
        }
    }
}

#[test]
fn the_executor_merges_the_chat_images_into_the_send_the_child_receives() {
    let Some(harness) = Harness::new() else {
        return;
    };
    let scratch = Scratch::new();
    let path = scratch.file("shot.png", PNG);
    let bridge = Bridge::new();

    bridge.dispatch(UiIntent::OpenNewChat);
    let chat = bridge
        .holder
        .view()
        .foreground_chat
        .expect("the new chat is the foreground one");
    wait_for("the child to start", || {
        (!harness.commands().is_empty()).then_some(())
    });
    // The chat is confirmed open only by its own first event, which the stand-in
    // sends at once; a prompt to a chat that is still opening is refused.
    wait_for("the chat to be ready", || {
        bridge
            .holder
            .view()
            .activity
            .iter()
            .all(|entry| entry.chat_id != chat || entry.activity != ChatActivityMark::Opening)
            .then_some(())
    });

    let view = bridge.holder.attach_images(std::slice::from_ref(&path));
    assert_eq!(view.images.len(), 1, "the chat holds the dropped image");
    bridge.dispatch(UiIntent::SendPrompt {
        text: "look".to_owned(),
    });

    let prompts: Vec<String> = wait_for("the prompt to reach the child", || {
        let prompts: Vec<String> = harness
            .commands()
            .into_iter()
            .filter(|entry| entry["type"] == "prompt")
            .map(|entry| entry["message"].as_str().unwrap_or_default().to_owned())
            .collect();
        (!prompts.is_empty()).then_some(prompts)
    });
    assert_eq!(
        prompts,
        vec![format!("look\n\nAttached image: {path}")],
        "the child receives the path as text, as the shell's transcript reads"
    );
    assert!(
        bridge.holder.view().images.is_empty(),
        "the send took the chat's images: they cannot travel twice"
    );
    assert!(
        bridge.holder.view().images_notice.is_none(),
        "nothing was refused"
    );
}

#[test]
fn the_sent_message_carries_the_thumbnails_of_its_images() {
    let Some(_harness) = Harness::new() else {
        return;
    };
    let scratch = Scratch::new();
    let path = scratch.file("shot.png", PNG);
    let bridge = Bridge::new();
    bridge.dispatch(UiIntent::OpenNewChat);
    let chat = wait_for("the chat to be ready", || {
        let view = bridge.holder.view();
        let chat = view.foreground_chat?;
        let opening = view
            .activity
            .iter()
            .any(|entry| entry.chat_id == chat && entry.activity == ChatActivityMark::Opening);
        (!opening).then_some(chat)
    });
    bridge.holder.attach_images(&[path]);
    let held = bridge.holder.view().images;
    assert_eq!(held.len(), 1);
    let id = held[0].id.clone();

    bridge.dispatch(UiIntent::SendPrompt {
        text: "look".to_owned(),
    });

    let view = bridge.holder.view();
    let sent = view
        .messages
        .iter()
        .find(|message| message.role == MessageRole::User)
        .expect("the prompt is in the thread");
    assert_eq!(
        sent.images.len(),
        1,
        "the sent message draws the image it carried (chat {chat})"
    );
    assert_eq!(
        sent.images[0].id, id,
        "the same image, so the same preview URL"
    );
    assert_eq!(sent.images[0].name, "shot.png");
}
