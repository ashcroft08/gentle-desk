//! Removing a workspace (S1, S2, S4, S5, S7): the flow's own rules, windowless.
//!
//! What reaches the disk is `gentle-backend`'s trash-first removal, tested
//! there; what this file tests is the ORDER and the gate the maintainer ruled
//! (2026-11-16): a workspace whose chat is running a turn refuses the whole
//! removal, a workspace with no chat is forgotten and touches nothing, and the
//! chats of the workspace are closed first, deleted second, and only then is the
//! bookmark forgotten. A fake index answers the chats, a fake disk answers the
//! removal and a fake close records what was closed, so no pi home, no child and
//! no window is involved.

use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard};

use gentle_backend::session_delete::SessionRemovals;
use gentle_core::i18n::{messages, Language};
use gentle_desk_lib::bridge::AppStateHolder;
use gentle_desk_lib::chat_rename::WorkingChats;
use gentle_desk_lib::config::ConfigSnapshot;
use gentle_desk_lib::contract::WorkspaceRemovalView;
use gentle_desk_lib::workspace_remove::{self, offer, ChatFile, CloseFirst, WorkspaceChats};
use gentle_runtime::app_config::{folder_key, AppConfig};

const REPO: &str = r"C:\dev\repo";
const NOTES: &str = r"C:\dev\notes";
const CHAT_ONE: &str = r"C:\pi\sessions\--dev-repo--\2026_s1.jsonl";
const CHAT_TWO: &str = r"C:\pi\sessions\--dev-repo--\2026_s2.jsonl";
const THIRD_CHAT: &str = r"C:\pi\sessions\--dev-repo--\2026_s3.jsonl";
const OTHER_CHAT: &str = r"C:\pi\sessions\--dev-notes--\2026_s3.jsonl";

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|poison| poison.into_inner())
}

/// A holder whose config holds `paths` as workspaces, all expanded.
fn holder_holding(paths: &[&str]) -> AppStateHolder {
    let holder = AppStateHolder::default();
    holder.update_config(|config| {
        let mut changed = false;
        for path in paths {
            changed |= config.add_workspace(path);
        }
        changed
    });
    holder
}

/// A holder that speaks Spanish, which is the language its notices must be in.
fn spanish_holder(paths: &[&str]) -> AppStateHolder {
    let config = AppConfig {
        language: Some(Language::Es),
        ..AppConfig::default()
    };
    let holder = AppStateHolder::with_config(ConfigSnapshot {
        config,
        notice: None,
    });
    holder.update_config(|config| {
        let mut changed = false;
        for path in paths {
            changed |= config.add_workspace(path);
        }
        changed
    });
    holder
}

/// The chats a fake index reports, keyed by the folder the sessions name.
#[derive(Default)]
struct FakeIndex {
    chats: Vec<(&'static str, &'static str, u64)>,
}

impl FakeIndex {
    fn holding(chats: &[(&'static str, &'static str, u64)]) -> Self {
        Self {
            chats: chats.to_vec(),
        }
    }
}

impl WorkspaceChats for FakeIndex {
    fn chats(&self, workspace: &str) -> Vec<ChatFile> {
        let key = folder_key(workspace);
        self.chats
            .iter()
            .filter(|(cwd, _, _)| folder_key(cwd) == key)
            .map(|(_, path, bytes)| ChatFile {
                path: PathBuf::from(path),
                bytes: *bytes,
            })
            .collect()
    }
}

/// The session files a fake disk holds, and every path it was asked to remove,
/// in order.
#[derive(Default)]
struct FakeDisk {
    files: Mutex<Vec<PathBuf>>,
    calls: Mutex<Vec<String>>,
    refusal: Mutex<Option<String>>,
}

impl FakeDisk {
    fn holding(paths: &[&str]) -> Self {
        let disk = FakeDisk::default();
        *lock(&disk.files) = paths.iter().map(PathBuf::from).collect();
        disk
    }

    /// The same, holding `paths` and refusing to remove the file named `refused`.
    fn refusing(paths: &[&str], refused: &str) -> Self {
        let disk = FakeDisk::holding(paths);
        *lock(&disk.refusal) = Some(refused.to_owned());
        disk
    }

    fn calls(&self) -> Vec<String> {
        lock(&self.calls).clone()
    }
}

impl SessionRemovals for FakeDisk {
    fn remove(&self, path: &Path) -> io::Result<()> {
        lock(&self.calls).push(path.display().to_string());
        if lock(&self.refusal).as_deref() == Some(&path.display().to_string()) {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "access is denied",
            ));
        }
        let mut files = lock(&self.files);
        match files.iter().position(|held| held.as_path() == path) {
            Some(at) => {
                files.remove(at);
                Ok(())
            }
            None => Err(io::Error::new(
                io::ErrorKind::NotFound,
                "no such session file",
            )),
        }
    }
}

/// Which chats are mid-turn, and every chat the removal asked to close.
#[derive(Default)]
struct FakeChats {
    working: Vec<&'static str>,
    closed: Mutex<Vec<String>>,
    events: Mutex<Vec<String>>,
}

impl FakeChats {
    fn with_working(working: &[&'static str]) -> Self {
        Self {
            working: working.to_vec(),
            ..Self::default()
        }
    }

    fn closed(&self) -> Vec<String> {
        lock(&self.closed).clone()
    }

    fn events(&self) -> Vec<String> {
        lock(&self.events).clone()
    }
}

impl WorkingChats for FakeChats {
    fn is_working(&self, session: &str) -> bool {
        self.working.contains(&session)
    }
}

/// The fake's own close: it records the close and folds it into the event log,
/// which is what proves the order the disk sees.
struct RecordClose<'a>(&'a FakeChats);

impl CloseFirst for RecordClose<'_> {
    fn close(&self, session: &str) {
        lock(&self.0.closed).push(session.to_owned());
        lock(&self.0.events).push(format!("close {session}"));
    }
}

/// The disk of the order test: every removal is folded into the same log.
struct RecordDisk<'a>(&'a FakeDisk, &'a FakeChats);

impl SessionRemovals for RecordDisk<'_> {
    fn remove(&self, path: &Path) -> io::Result<()> {
        lock(&self.1.events).push(format!("delete {}", path.display()));
        self.0.remove(path)
    }
}

/// The whole flow over the fakes, on the holders the tests build.
fn run(
    holder: &AppStateHolder,
    index: &dyn WorkspaceChats,
    files: &dyn SessionRemovals,
    chats: &FakeChats,
    path: &str,
    confirmed: bool,
) -> workspace_remove::Removed {
    workspace_remove::remove(
        holder,
        index,
        files,
        chats,
        &RecordClose(chats),
        path,
        confirmed,
    )
}

#[test]
fn a_workspace_without_chats_is_forgotten_and_the_disk_is_never_touched() {
    let holder = holder_holding(&[REPO, NOTES]);
    let index = FakeIndex::default();
    let disk = FakeDisk::default();
    let chats = FakeChats::default();

    let removed = run(&holder, &index, &disk, &chats, REPO, true);

    assert!(
        holder.config_value().workspace(REPO).is_none(),
        "the bookmark is forgotten"
    );
    assert!(
        holder.config_value().workspace(NOTES).is_some(),
        "another workspace is untouched"
    );
    assert!(
        disk.calls().is_empty(),
        "a workspace with no chat deletes nothing"
    );
    assert!(chats.closed().is_empty(), "and closes nothing");
    assert!(removed.view.workspace_notice.is_none(), "nothing to report");
    assert!(!removed.removed, "no session file changed");
}

#[test]
fn a_working_chat_refuses_the_whole_removal() {
    let holder = holder_holding(&[REPO]);
    let index = FakeIndex::holding(&[(REPO, CHAT_ONE, 10), (REPO, CHAT_TWO, 20)]);
    let disk = FakeDisk::holding(&[CHAT_ONE, CHAT_TWO]);
    let chats = FakeChats::with_working(&[CHAT_ONE]);

    let removed = run(&holder, &index, &disk, &chats, REPO, true);

    assert_eq!(
        removed.view.workspace_notice.as_deref(),
        Some(messages::remove_workspace_working_refusal(Language::En).as_str()),
        "the cataloged working-chat wording, and nothing of this feature's own"
    );
    assert!(chats.closed().is_empty(), "nothing is closed");
    assert!(disk.calls().is_empty(), "nothing is deleted");
    assert!(
        holder.config_value().workspace(REPO).is_some(),
        "the bookmark stays: the workspace still holds chats"
    );
}

#[test]
fn the_children_of_the_workspace_are_closed_before_their_files_are_deleted() {
    let holder = holder_holding(&[REPO]);
    let index = FakeIndex::holding(&[(REPO, CHAT_ONE, 10), (REPO, CHAT_TWO, 20)]);
    let disk = FakeDisk::holding(&[CHAT_ONE, CHAT_TWO]);
    let chats = FakeChats::default();

    let removed = run(
        &holder,
        &index,
        &RecordDisk(&disk, &chats),
        &chats,
        REPO,
        true,
    );

    assert_eq!(
        chats.events(),
        [
            format!("close {CHAT_ONE}"),
            format!("close {CHAT_TWO}"),
            format!("delete {CHAT_ONE}"),
            format!("delete {CHAT_TWO}"),
        ],
        "every child is gone before the first file is deleted, and none is closed twice"
    );
    assert!(
        holder.config_value().workspace(REPO).is_none(),
        "the bookmark goes last, once the chats are gone"
    );
    assert!(removed.view.workspace_notice.is_none());
    assert!(removed.removed, "the list is asked to heal");
}

#[test]
fn a_delete_the_disk_refuses_stops_the_removal_and_keeps_the_bookmark() {
    let holder = holder_holding(&[REPO]);
    let index = FakeIndex::holding(&[
        (REPO, CHAT_ONE, 10),
        (REPO, CHAT_TWO, 20),
        (REPO, THIRD_CHAT, 30),
    ]);
    let disk = FakeDisk::refusing(&[CHAT_ONE, CHAT_TWO, THIRD_CHAT], CHAT_TWO);
    let chats = FakeChats::default();

    let removed = run(&holder, &index, &disk, &chats, REPO, true);

    let notice = removed
        .view
        .workspace_notice
        .expect("a failed removal says so");
    assert!(
        notice.starts_with("the removal could not run: "),
        "the cataloged line, got {notice:?}"
    );
    assert!(
        notice.contains("access is denied"),
        "the disk's own reason travels as it arrived, got {notice:?}"
    );
    assert_eq!(
        disk.calls(),
        [CHAT_ONE, CHAT_TWO],
        "the removal stops at the file the disk refused, so a retry re-offers the rest"
    );
    assert!(
        holder.config_value().workspace(REPO).is_some(),
        "a workspace that could not be swept clean is not forgotten"
    );
    assert!(!removed.removed, "the list has nothing new to read back");
}

#[test]
fn the_refusal_is_worded_in_the_language_the_run_speaks() {
    let holder = spanish_holder(&[REPO]);
    let index = FakeIndex::holding(&[(REPO, CHAT_ONE, 10)]);
    let disk = FakeDisk::holding(&[CHAT_ONE]);
    let chats = FakeChats::with_working(&[CHAT_ONE]);

    let removed = run(&holder, &index, &disk, &chats, REPO, true);

    assert_eq!(
        removed.view.workspace_notice.as_deref(),
        Some("Este chat está trabajando. Detenlo primero, luego quita el espacio de trabajo."),
        "the Spanish column of the same rule"
    );
    assert!(disk.calls().is_empty());
}

#[test]
fn nothing_happens_without_the_confirmation_the_dialog_sends() {
    let holder = holder_holding(&[REPO]);
    let index = FakeIndex::holding(&[(REPO, CHAT_ONE, 10)]);
    let disk = FakeDisk::holding(&[CHAT_ONE]);
    let chats = FakeChats::default();

    let removed = run(&holder, &index, &disk, &chats, REPO, false);

    assert!(chats.closed().is_empty(), "no child is closed");
    assert!(disk.calls().is_empty(), "no file is deleted");
    assert!(
        holder.config_value().workspace(REPO).is_some(),
        "the bookmark stays"
    );
    assert!(removed.view.workspace_notice.is_none());
}

#[test]
fn a_workspace_no_row_names_is_left_alone() {
    let holder = holder_holding(&[REPO]);
    let index = FakeIndex::default();
    let disk = FakeDisk::default();
    let chats = FakeChats::default();

    let removed = run(&holder, &index, &disk, &chats, NOTES, true);

    assert!(
        holder.config_value().workspace(REPO).is_some(),
        "nothing of the config moves"
    );
    assert!(disk.calls().is_empty());
    assert!(
        removed.view.workspace_notice.is_none(),
        "and no line is left"
    );
}

#[test]
fn the_offer_counts_the_chats_of_one_workspace_and_adds_up_their_bytes() {
    let holder = holder_holding(&[REPO, NOTES]);
    let index = FakeIndex::holding(&[
        (REPO, CHAT_ONE, 700_000),
        (REPO, CHAT_TWO, 1_000_000),
        (NOTES, OTHER_CHAT, 5_000_000),
    ]);

    let offer = offer(&holder, &index, REPO);

    assert_eq!(
        offer,
        WorkspaceRemovalView {
            key: folder_key(REPO),
            path: REPO.to_owned(),
            label: "repo".to_owned(),
            chats: 2,
            bytes: 1_700_000.0,
        },
        "the count and the size are the workspace's own, never another one's"
    );
}

#[test]
fn the_shells_close_asks_for_the_one_command_that_stops_that_chats_child() {
    use std::sync::Arc;

    use gentle_app::{ChatId, Input, Intent};
    use gentle_desk_lib::workspace_remove::{CloseFirst, ShellClose};

    let holder = Arc::new(AppStateHolder::default());
    // The chat of one stored session, opened the way a click on its row opens it.
    holder.fold_input(Input::Intent(Intent::OpenChat(ChatId::of_session(
        CHAT_ONE,
    ))));
    assert_eq!(
        holder.view().open_chats,
        1,
        "the chat is open to begin with"
    );
    let ran: Arc<Mutex<Vec<String>>> = Arc::default();
    let seen = Arc::clone(&ran);
    let close = ShellClose::new(
        Arc::clone(&holder),
        Arc::new(move |commands| {
            for command in commands {
                lock(&seen).push(format!("{command:?}"));
            }
        }),
    );

    // The chat the removal is about to delete, closed through the app's own rule.
    close.close(CHAT_ONE);

    let commands = lock(&ran).clone();
    assert_eq!(commands.len(), 1, "one command, got {commands:?}");
    assert!(
        commands[0].contains("CloseProcess"),
        "the reducer's own close, which is what stops the child, got {commands:?}"
    );
    assert_eq!(
        holder.view().open_chats,
        0,
        "the chat left the open set before its file is deleted"
    );
}

#[test]
fn the_offer_of_a_folder_the_config_spells_another_way_uses_the_stored_spelling() {
    let holder = holder_holding(&[REPO]);
    let index = FakeIndex::default();

    let offer = offer(&holder, &index, r"\\?\C:\Dev\Repo\");

    assert_eq!(offer.path, REPO, "the row's spelling, from the config");
    assert_eq!(offer.chats, 0);
    assert_eq!(offer.bytes, 0.0, "no chat, no size");
}
