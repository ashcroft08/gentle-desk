//! View A's data (welcome-screen T1, S1, S3, R3, R4): the workspace the user
//! worked in last, that workspace's recent chats and the other configured
//! workspaces, as the slice the web side receives.
//!
//! Nothing here decides a grouping, a name or an order of its own: the folders,
//! their labels and their order come from the runtime's workspace list, the same
//! one the sidebar's tree is flattened from, so these tests only assert what the
//! slice carries for the state they seed. No child process runs and no window
//! exists: the holder is a `Mutex<AppState>` and a config.

use std::fs;
use std::path::Path;

use gentle_app::{ChatId, Input, Intent, SessionItem};
use gentle_core::i18n::{messages, Language};
use gentle_desk_lib::bridge::AppStateHolder;
use gentle_desk_lib::config::ConfigSnapshot;
use gentle_desk_lib::contract::{
    SessionTreeRow, UiIntent, WelcomeChat, WelcomeLink, WelcomeView, WorkspaceRowView,
};
use gentle_desk_lib::sessions::window::MAX_WINDOW;
use gentle_runtime::app_config::AppConfig;

/// One folder of the scratch home, created so the sidebar's own probe finds it.
fn folder(home: &Path, name: &str) -> String {
    let path = home.join(name);
    fs::create_dir_all(&path).expect("a scratch workspace folder");
    path.to_string_lossy().into_owned()
}

/// One session header of `cwd`, last written at `modified_ms`, as the first pass
/// reports it. Its file is the id the web sends back, and its title is what a
/// sidebar row draws.
fn session(cwd: &str, modified_ms: u64) -> SessionItem {
    SessionItem {
        path: format!("sessions/{modified_ms}.jsonl"),
        title: format!("chat {modified_ms}"),
        cwd: cwd.to_owned(),
        modified_ms,
        message_count: 1,
    }
}

/// The recent chat of that same session, as view A carries it.
fn recent(modified_ms: u64) -> WelcomeChat {
    WelcomeChat {
        chat_id: format!("sessions/{modified_ms}.jsonl"),
        title: format!("chat {modified_ms}"),
        // The contract carries the timestamp as a double (the bindings refuse a
        // 64-bit integer on the wire).
        modified_ms: modified_ms as f64,
    }
}

/// The quiet link of one configured workspace.
fn link(path: &str, label: &str) -> WelcomeLink {
    WelcomeLink {
        path: path.to_owned(),
        label: label.to_owned(),
    }
}

/// A holder whose config holds `paths` as workspaces IN THAT ORDER, with the
/// session list seeded the way a finished pass seeds it.
fn holder(paths: &[&str], sessions: Vec<SessionItem>) -> AppStateHolder {
    let mut config = AppConfig::default();
    for path in paths {
        assert!(config.add_workspace(path), "add_workspace({path})");
    }
    let holder = AppStateHolder::with_config(ConfigSnapshot {
        config,
        notice: None,
    });
    holder.fold_input(Input::SessionsLoaded(sessions));
    holder
}

/// The slice of a holder with no chat in the foreground, which is the only state
/// that carries view A at all.
fn welcome(holder: &AppStateHolder) -> WelcomeView {
    holder
        .view()
        .welcome
        .expect("no chat is open: view A carries its data")
}

/// The workspace header row the sidebar serves for `path`: the flag and the very
/// words view A has to match, because A's primary action is disabled exactly
/// when the sidebar disables new chats there.
fn sidebar(holder: &AppStateHolder, path: &str) -> WorkspaceRowView {
    holder
        .sessions_window(0, MAX_WINDOW)
        .rows
        .into_iter()
        .find_map(|row| match row {
            SessionTreeRow::Workspace(row) if row.path == path => Some(row),
            _ => None,
        })
        .unwrap_or_else(|| panic!("the sidebar serves a row for {path}"))
}

#[test]
fn with_no_configured_workspace_view_a_names_no_last_workspace() {
    let home = tempfile::tempdir().unwrap();
    let loose = folder(home.path(), "loose");
    // A folder only the history names is a row of the sidebar, but it was never
    // configured: nothing is invented as the user's last workspace (R3, R4).
    let holder = holder(&[], vec![session(&loose, 700)]);

    let welcome = welcome(&holder);

    assert_eq!(welcome.last, None);
    assert_eq!(welcome.recents, Vec::new());
    assert_eq!(welcome.others, Vec::new());
}

#[test]
fn the_workspace_of_the_newest_session_is_the_last_one_with_its_three_newest_chats() {
    let home = tempfile::tempdir().unwrap();
    let desk = folder(home.path(), "desk");
    let proyecto = folder(home.path(), "proyecto");
    let curriculum = folder(home.path(), "curriculum");
    let holder = holder(
        // Config order is desk, proyecto, curriculum; the activity order is
        // curriculum, proyecto, desk.
        &[&desk, &proyecto, &curriculum],
        vec![
            session(&curriculum, 500),
            session(&curriculum, 400),
            session(&curriculum, 300),
            session(&curriculum, 100),
            session(&proyecto, 200),
        ],
    );

    let welcome = welcome(&holder);

    let last = welcome.last.expect("curriculum holds the newest session");
    assert_eq!(last.path, curriculum);
    assert_eq!(last.label, "curriculum", "the label the sidebar row draws");
    assert!(last.available, "the folder is there");
    assert_eq!(last.reason, None, "nothing to warn about");
    assert_eq!(last.chat_count, 4, "every session the tree holds for it");
    assert_eq!(last.last_activity_ms, Some(500.0));

    assert_eq!(
        welcome.recents,
        vec![recent(500), recent(400), recent(300)],
        "the three newest chats, newest first, with the row's title and the raw \
         session path the web sends back"
    );
    assert_eq!(
        welcome.others,
        vec![link(&desk, "desk"), link(&proyecto, "proyecto"),],
        "the other configured workspaces in config order, without the last one"
    );
}

#[test]
fn a_last_workspace_with_no_chats_says_so_and_offers_no_list() {
    let home = tempfile::tempdir().unwrap();
    let fresh = folder(home.path(), "fresh");
    let other = folder(home.path(), "other");
    // No session at all: there is no newest one to read the workspace from, so
    // the first configured workspace is the last one the user worked in.
    let holder = holder(&[&fresh, &other], Vec::new());

    let welcome = welcome(&holder);

    let last = welcome.last.expect("the first configured workspace");
    assert_eq!(last.path, fresh);
    assert_eq!(last.chat_count, 0);
    assert_eq!(last.last_activity_ms, None);
    assert!(last.available, "the folder is there; it holds no chat yet");
    assert_eq!(last.reason, None);
    assert_eq!(
        welcome.recents,
        Vec::new(),
        "the list is absent, never empty"
    );
    assert_eq!(welcome.others, vec![link(&other, "other")]);
}

#[test]
fn the_last_workspace_reports_what_the_sidebar_reports_about_its_folder() {
    let home = tempfile::tempdir().unwrap();
    let there = folder(home.path(), "there");
    // A FILE where a workspace folder was: the sidebar's other refusal.
    let file = home.path().join("a-file");
    fs::write(&file, b"not a folder").unwrap();
    let file = file.to_string_lossy().into_owned();
    // A folder that is not there at all.
    let gone = home.path().join("gone").to_string_lossy().into_owned();

    // A folder that is there: a chat can start, and there is nothing to say.
    let holder = self::holder(&[&there], Vec::new());
    let last = welcome(&holder).last.expect("the configured workspace");
    assert!(last.available);
    assert_eq!(last.reason, None);

    // A folder that no longer exists: unavailable, with the sidebar's own line.
    let holder = self::holder(&[&gone, &there], Vec::new());
    let last = welcome(&holder)
        .last
        .expect("the first configured workspace");
    assert_eq!(last.path, gone);
    assert_eq!(last.label, "gone", "the very name the sidebar row draws");
    assert!(!last.available, "the primary action is disabled here");
    assert_eq!(
        last.reason,
        Some(messages::workspace_folder_missing(Language::En).to_owned()),
        "the words the sidebar itself draws"
    );
    let row = sidebar(&holder, &gone);
    assert_eq!(last.reason, row.reason);
    assert_eq!(last.available, row.available);

    // A path holding a file: the sidebar refuses new chats there too, so view A
    // must, with the other line it words for this case.
    let holder = self::holder(&[&file], Vec::new());
    let last = welcome(&holder).last.expect("the configured workspace");
    assert_eq!(last.path, file);
    assert!(!last.available);
    assert_eq!(
        last.reason,
        Some(messages::workspace_not_a_folder(Language::En).to_owned())
    );
    let row = sidebar(&holder, &file);
    assert_eq!(last.reason, row.reason);
    assert_eq!(last.available, row.available);
}

#[test]
fn a_chat_in_the_foreground_takes_view_a_away() {
    let home = tempfile::tempdir().unwrap();
    let workspace = folder(home.path(), "workspace");
    let holder = holder(&[&workspace], vec![session(&workspace, 100)]);
    assert!(holder.view().welcome.is_some(), "nothing is open at start");

    let view = holder.fold_input(Input::Intent(Intent::OpenNewChat));

    assert!(view.foreground_chat.is_some());
    assert_eq!(view.welcome, None, "a chat on screen draws no welcome");
}

#[test]
fn a_session_that_belongs_to_no_configured_workspace_never_names_the_last_one() {
    let home = tempfile::tempdir().unwrap();
    let workspace = folder(home.path(), "workspace");
    let elsewhere = folder(home.path(), "elsewhere");
    let holder = holder(
        &[&workspace],
        vec![
            // Newest of the whole list, in a folder the config does not hold:
            // it is not a configured workspace, so it decides nothing.
            session(&elsewhere, 900),
            // And a session reporting no cwd belongs to no workspace at all.
            session("", 800),
            session(&workspace, 100),
        ],
    );

    let welcome = welcome(&holder);

    let last = welcome
        .last
        .expect("the configured workspace that owns a session");
    assert_eq!(last.path, workspace);
    assert_eq!(
        last.last_activity_ms,
        Some(100.0),
        "another folder's session is not this workspace's activity"
    );
    assert_eq!(last.chat_count, 1);
    assert_eq!(welcome.recents, vec![recent(100)]);
    assert_eq!(welcome.others, Vec::new(), "no OTHER configured workspace");
}

#[test]
fn closing_the_open_chat_brings_view_a_back() {
    let home = tempfile::tempdir().unwrap();
    let workspace = folder(home.path(), "workspace");
    let holder = holder(&[&workspace], vec![session(&workspace, 100)]);
    let path = "sessions/100.jsonl".to_owned();
    let chat = ChatId::of_session(&path);
    let before = welcome(&holder);

    holder.fold_input(Input::Intent(Intent::OpenChat(chat.clone())));
    holder.fold_input(Input::ChatOpened { chat });
    assert_eq!(holder.view().welcome, None, "the chat is in the foreground");

    let view = holder.dispatch(UiIntent::CloseChat {
        chat_id: path.clone(),
    });

    assert_eq!(view.foreground_chat, None);
    assert_eq!(
        view.welcome,
        Some(before),
        "the same data A showed before the chat opened"
    );
}
