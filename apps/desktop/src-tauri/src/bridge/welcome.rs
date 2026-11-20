//! View A's data (welcome-screen T1, S1, S3, R3, R4): the workspace the user
//! worked in last, its recent chats and the other configured workspaces.
//!
//! The rule is pure over what the sidebar already holds: the runtime's workspace
//! list (the very one [`crate::sessions::tree::Tree`] flattens into rows) and the
//! session list that list was built from. No grouping, no naming and no order is
//! decided here - the labels, the paths and the membership of a session in a
//! workspace are the runtime's, so the welcome can never name a workspace or a
//! chat differently from the row beside it.
//!
//! The one thing this module asks the outside world is whether the last
//! workspace's folder can host a new chat, through the probe the caller brings:
//! the same [`folder_status`](crate::sessions::window::folder_status) a served
//! page of rows runs, asked for ONE folder and never for the whole list. Its two
//! answers are carried the way a sidebar workspace row carries them, so the
//! welcome disables the primary action exactly when the sidebar disables new
//! chats, and prints the line the sidebar already words.
//!
//! "The last workspace" is R3's: the CONFIGURED workspace that owns the most
//! recent session, `last_activity_ms` being the newest `modified_ms` the runtime
//! gathered for it. A session of a folder the config does not hold is a sidebar
//! row of its own and decides nothing here, and neither does a session reporting
//! no `cwd` at all.

use std::collections::HashSet;

use gentle_app::SessionItem;
use gentle_runtime::app_config::{folder_key, AppConfig, WorkspaceStatus};
use gentle_runtime::workspaces::{Workspace, WorkspaceList};

use crate::contract::{WelcomeChat, WelcomeLink, WelcomeView, WelcomeWorkspace};
use crate::sessions::view;

/// How many chats of the last workspace the screen lists: the recent ones, not
/// the workspace's history.
const RECENTS: usize = 3;

/// View A of `config`, `workspaces` and `sessions` (welcome-screen T1), with the
/// folder of the last workspace asked through `status`.
///
/// `workspaces` and `sessions` are one pair: the runtime built the list from
/// that very slice, so every index the list holds names the session it names
/// there.
pub(crate) fn welcome(
    config: &AppConfig,
    workspaces: &WorkspaceList,
    sessions: &[SessionItem],
    status: &dyn Fn(&str) -> WorkspaceStatus,
) -> WelcomeView {
    match last_workspace(workspaces) {
        Some(last) => {
            // What the sidebar draws for this very folder, asked once and with
            // no lock held by the caller: the one question this module puts to
            // the outside world.
            let folder = status(&last.path);
            WelcomeView {
                recents: recents(last, sessions),
                others: others(config, workspaces, last),
                last: Some(WelcomeWorkspace {
                    path: last.path.clone(),
                    label: last.label.clone(),
                    // The two fields the sidebar's own workspace row carries,
                    // from the same probe: the primary action is disabled
                    // exactly when the sidebar refuses a new chat here, and the
                    // reason is the line the sidebar already words for that
                    // folder (a folder that is gone and a path that holds
                    // something else have their own texts there, so nothing is
                    // re-worded here).
                    available: folder.is_available(),
                    reason: folder.reason().map(str::to_owned),
                    chat_count: view::narrow(last.sessions.len()),
                    last_activity_ms: last.last_activity_ms.map(millis),
                }),
            }
        }
        // No configured workspace: there is nothing to have been last in, and
        // nothing to link to either (R3).
        None => WelcomeView {
            last: None,
            recents: Vec::new(),
            others: Vec::new(),
        },
    }
}

/// The configured workspace that owns the most recent session (R3): the one with
/// the newest `last_activity_ms`, and - with none of them holding a chat - the
/// first the config lists, which is what the runtime's order answers for folders
/// with no activity.
fn last_workspace(workspaces: &WorkspaceList) -> Option<&Workspace> {
    // Strictly greater keeps the first of two equally recent workspaces, so the
    // order this reads is the runtime's own tie-break and never a later row.
    workspaces
        .workspaces
        .iter()
        .filter(|workspace| workspace.from_config)
        .fold(None, |best: Option<&Workspace>, workspace| match best {
            Some(best) if best.last_activity_ms >= workspace.last_activity_ms => Some(best),
            _ => Some(workspace),
        })
}

/// Up to [`RECENTS`] chats of `workspace`, newest first.
///
/// The runtime gathers a workspace's sessions in the order of the list it was
/// given, which is the app's own (newest first, the order the sidebar draws), so
/// the first indices ARE the newest ones and no sort runs here.
fn recents(workspace: &Workspace, sessions: &[SessionItem]) -> Vec<WelcomeChat> {
    workspace
        .sessions
        .iter()
        .filter_map(|index| sessions.get(*index))
        .take(RECENTS)
        .map(|session| WelcomeChat {
            // The RAW session path: `UiIntent::OpenChat` takes exactly this
            // string and `ChatId::of_session` prefixes it once, over there.
            chat_id: session.path.clone(),
            title: view::row_title(session),
            modified_ms: millis(session.modified_ms),
        })
        .collect()
}

/// The other CONFIGURED workspaces, in the order the config lists them, without
/// `last` (welcome-screen T1).
///
/// The config's own order is what the quiet links keep, which is not the sidebar
/// tree's order (that one is by last activity); the label and the canonical path
/// are still the ones the runtime gave the workspace, never a second spelling.
/// A folder the config names twice keeps its first entry, exactly as
/// [`gentle_runtime::workspaces::build`] gathers it.
fn others(config: &AppConfig, workspaces: &WorkspaceList, last: &Workspace) -> Vec<WelcomeLink> {
    let mut seen: HashSet<String> = HashSet::new();
    config
        .workspaces
        .iter()
        .filter(|entry| !entry.path.trim().is_empty())
        .filter_map(|entry| {
            let key = folder_key(&entry.path);
            if !seen.insert(key.clone()) {
                return None;
            }
            let workspace = workspaces
                .workspaces
                .iter()
                .find(|workspace| workspace.key == key)?;
            (workspace.key != last.key).then(|| WelcomeLink {
                path: workspace.path.clone(),
                label: workspace.label.clone(),
            })
        })
        .collect()
}

/// A timestamp as the double the contract carries: the generated bindings refuse
/// a 64-bit integer on the wire, and a real millisecond count is exact in a
/// double.
fn millis(ms: u64) -> f64 {
    ms as f64
}
