//! The `open_link` command of the thread (T5a): one link the user clicked.
//!
//! It is a COMMAND and not a `UiIntent` variant, because it folds nothing: no
//! reducer input describes "open this url", and nothing of the app state
//! changes when a browser opens. The view sends the url it rendered, the shell
//! decides whether it may be opened ([`crate::link`]) and opens it.
//!
//! The open runs on a blocking thread - starting the system's default program
//! is I/O and may take as long as the desktop takes - and never under a lock:
//! the opener is cloned out of the bridge first.

use std::sync::Arc;

use crate::commands::Bridge;
use crate::link_opener::{self, LinkOpener};

/// Opens `url` in the system's default browser, or refuses it here.
///
/// Only a plain `http`/`https` url with a host and no credentials is opened;
/// everything else is refused with the reason, and the browser is never called.
#[tauri::command]
#[specta::specta]
pub async fn open_link(url: String, bridge: tauri::State<'_, Bridge>) -> Result<(), String> {
    // Taken off the bridge before the await: no lock of it is held while the
    // browser starts.
    let opener: Arc<dyn LinkOpener> = bridge.opener();
    tauri::async_runtime::spawn_blocking(move || link_opener::open(opener.as_ref(), &url))
        .await
        .map_err(|error| format!("the link could not be opened: {error}"))?
}
