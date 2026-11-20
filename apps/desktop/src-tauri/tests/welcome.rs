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
    SessionRowView, SessionTreeRow, UiIntent, WelcomeChat, WelcomeLink, WelcomeView,
    WorkspaceRowView,
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

/// The quiet link of one configured workspace, with whether new chats can still
/// start in it.
fn link(path: &str, label: &str, available: bool) -> WelcomeLink {
    WelcomeLink {
        path: path.to_owned(),
        label: label.to_owned(),
        available,
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

/// The chat row the sidebar serves for the session at `path`: the row of the
/// tree a view A recent names, drawn the width a chat row really has.
fn chat_row(holder: &AppStateHolder, path: &str) -> SessionRowView {
    holder
        .sessions_window(0, MAX_WINDOW)
        .rows
        .into_iter()
        .find_map(|row| match row {
            SessionTreeRow::Chat(row) if row.id == path => Some(row),
            _ => None,
        })
        .unwrap_or_else(|| panic!("the sidebar serves a chat row for {path}"))
}

/// A stored title longer than a sidebar chat row holds: the 32 characters that
/// row keeps, and this one has 115.
const LONG_TITLE: &str = "Retomar gentle-desk en la rama feat/0.8.0-welcome con la pantalla \
                          de bienvenida y los chats recientes del workspace";

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
        "the three newest chats, newest first, with the session's own title and \
         the raw session path the web sends back"
    );
    assert_eq!(
        welcome.others,
        vec![link(&desk, "desk", true), link(&proyecto, "proyecto", true),],
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
    assert_eq!(welcome.others, vec![link(&other, "other", true)]);
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
fn a_quiet_link_says_whether_its_folder_can_still_host_a_chat() {
    let home = tempfile::tempdir().unwrap();
    // A link whose folder is gone: the sidebar disables its button, so view A
    // must not offer a new chat there either.
    let gone = home.path().join("gone").to_string_lossy().into_owned();
    let working = folder(home.path(), "working");
    let other = folder(home.path(), "other");
    let holder = holder(
        // `working` holds the newest session, so it is the last workspace and
        // the two others are the links, in config order.
        &[&gone, &working, &other],
        vec![session(&working, 500)],
    );

    let welcome = welcome(&holder);

    assert_eq!(
        welcome.last.expect("the workspace of the session").path,
        working
    );
    assert_eq!(
        welcome.others,
        vec![link(&gone, "gone", false), link(&other, "other", true)],
        "the folder that is gone cannot host a chat; the one that is there can"
    );
    // And each answer is the one the sidebar itself gives for that workspace.
    for quiet in &welcome.others {
        let row = sidebar(&holder, &quiet.path);
        assert_eq!(
            quiet.available, row.available,
            "the sidebar's own answer for {}",
            quiet.label
        );
    }
}

#[test]
fn a_long_title_reaches_view_a_whole_where_the_sidebar_row_cuts_it() {
    let home = tempfile::tempdir().unwrap();
    let workspace = folder(home.path(), "workspace");
    let path = "sessions/500.jsonl".to_owned();
    let holder = holder(
        &[&workspace],
        vec![SessionItem {
            path: path.clone(),
            title: LONG_TITLE.to_owned(),
            cwd: workspace.clone(),
            modified_ms: 500,
            message_count: 1,
        }],
    );

    // View A's column is wide enough for the name the session stores, and the
    // page cuts it for the width it really has: the slice carries it whole.
    assert_eq!(
        welcome(&holder).recents,
        vec![WelcomeChat {
            chat_id: path.clone(),
            title: LONG_TITLE.to_owned(),
            modified_ms: 500.0,
        }],
        "the stored title whole and unchanged, never the sidebar row's cut"
    );

    // The same session beside it in the sidebar: one chat, and the title that
    // row's own narrow column still holds.
    let cut: String = LONG_TITLE
        .chars()
        .take(32)
        .chain(std::iter::once('…'))
        .collect();
    assert_eq!(
        chat_row(&holder, &path).title,
        cut,
        "the sidebar row keeps the width it has"
    );
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
