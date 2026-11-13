//! gentle-desk Tauri bridge entry point.
//!
//! The bridge owns the app state (`bridge::AppStateHolder`, created at startup
//! and managed by Tauri) and talks to the web side only through the generated
//! contract (`contract`), whose TypeScript is exported below. The per-chat
//! commands the reducer answers are run by `effects`, off the window's thread,
//! from the command layer in `commands`.

pub mod bridge;
pub mod chat_delete;
pub mod chat_rename;
pub mod clipboard;
pub mod close;
pub mod commands;
pub mod config;
pub mod contract;
pub mod dialogs;
pub mod effects;
pub mod events;
pub mod folder_name;
pub mod helper_thread;
pub mod helpers;
pub mod images;
pub mod intent;
pub mod link;
pub mod link_commands;
pub mod link_opener;
pub mod new_folder;
pub mod pending;
pub mod picker;
pub mod release;
pub mod sessions;
pub mod settings;
pub mod shell_install;
pub mod state_view;
pub mod terminal;
pub mod terminal_commands;
pub mod theme;
pub mod view;
pub mod window;
pub mod workspace;
pub mod workspace_add;
pub mod workspace_commands;
pub mod workspace_create;

use commands::Bridge;
use tauri::{Manager, WindowEvent};
use tauri_specta::Builder;

/// The whole typed surface of the bridge: the commands and events the
/// contract generates the TypeScript for. Shared with
/// `tests/contract_bindings.rs` so the drift guard sees exactly what the
/// running app registers.
pub fn contract_builder() -> Builder<tauri::Wry> {
    Builder::<tauri::Wry>::new()
        .commands(tauri_specta::collect_commands![
            contract::get_badge,
            commands::dispatch_intent,
            sessions::commands::sessions_window,
            sessions::commands::get_view_state,
            link_commands::open_link,
            workspace_commands::add_workspace,
            workspace_commands::create_workspace,
            chat_rename::rename_session,
            chat_delete::delete_session,
            commands::attach_images,
            commands::remove_image,
            commands::store_last_run_version,
            terminal_commands::open_terminal,
        ])
        .events(tauri_specta::collect_events![contract::StateChanged,])
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = contract_builder();

    // Regenerate the web-side bindings on every dev build so they cannot go
    // stale silently; the committed file is verified by tests/contract_bindings.rs.
    // Anchored on CARGO_MANIFEST_DIR so the path is correct from any cwd.
    #[cfg(debug_assertions)]
    builder
        .export(
            specta_typescript::Typescript::default(),
            concat!(env!("CARGO_MANIFEST_DIR"), "/../src/contract/bindings.ts"),
        )
        .expect("failed to export TypeScript bindings to ../src/contract");

    tauri::Builder::default()
        // The updater, which the app side reads from the web (see `update/`). On
        // Windows it closes the app itself to hand over to the installer.
        .plugin(tauri_plugin_updater::Builder::new().build())
        .manage(Bridge::new())
        // The thumbnails the composer and the thread draw: the shell's own
        // scheme, served from the files the user dropped. An `<img>` loads it
        // like any URL, so not one byte of an image travels through the
        // contract, and the request names an ID, never a path.
        .register_uri_scheme_protocol(images::SCHEME, |ctx, request| {
            preview(ctx.app_handle(), request.uri().path())
        })
        .invoke_handler(builder.invoke_handler())
        // The close flow (T8a). The window's own close request - the OS close
        // button, Alt+F4 - is the same event iced's `window::close_requests()`
        // delivered, and it is prevented here so the window stays up and
        // repainting while the shutdown runs on a thread of its own; its last
        // step exits the process. `RunEvent::ExitRequested` is not the hook: it
        // arrives once the window is already gone, which is the freeze this slice
        // avoids, and it cannot refuse the close.
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                // The close is prevented ONLY when there is a shutdown to finish
                // it: a window that cannot be closed is worse than one that closes
                // without the orderly shutdown, so an unarmed run (a request
                // before the setup step armed it) keeps the default close. So does
                // a close whose shutdown could not get a thread: the flow runs
                // nothing then (`CloseFlow::request`), and preventing this one
                // would leave a window nobody can close. A second request while
                // shutting down is ignored by the flow's own gate, but its close
                // is still prevented: the flow's exit step is what ends the
                // process.
                if let Some(close) = window.app_handle().state::<Bridge>().close_flow() {
                    close.request();
                    if close.shutdown_started() {
                        api.prevent_close();
                    }
                }
            }
        })
        .setup(move |app| {
            // Typed events need their registry mounted, or emitting panics.
            builder.mount_events(app);
            // The bridge's emit step and the pump's need the window handle,
            // which only setup has: this is the one place they are armed, from
            // the one handle. The pump's threads start the first time the
            // executor builds its host.
            app.state::<Bridge>().arm_events(app.handle());
            // Where the window opens: the stored rectangle when it still lands on
            // a screen the machine reports right now, the centre otherwise. Only
            // this step has the window and its screens, so it runs here, before
            // anything is drawn. The window is created hidden (`tauri.conf.json`),
            // so this is also the step that shows it - last, once it is placed.
            app.state::<Bridge>().place_window(app.handle());
            // The window's own chrome (the native title bar) follows the theme
            // of this run from the first frame; only this step has the window.
            app.state::<Bridge>().arm_window_theme(app.handle());
            // The session list is armed the same way, and its first pass asked
            // for here: the window opens with a list that says it is reading.
            app.state::<Bridge>().start_sessions();
            // The close is armed last and for the same reason: its exit step is
            // the one thing here that needs the handle.
            app.state::<Bridge>().arm_close(app.handle());
            // The what's-new step of this run (what's-new S6): last, once the
            // window is placed and armed, and never on this thread - it reads the
            // config and, only when this run follows an update, asks the release
            // server for the notes on a thread of its own, pushing what it
            // answers through the emit step above. Nothing here waits for the
            // network, so the first frame never depends on it.
            app.state::<Bridge>().start_whats_new(
                &app.package_info().version.to_string(),
                release::endpoint(app.handle()).as_deref(),
                release::fetcher(),
            );
            // The update check of this run (0.1.12, S3): once, when the app
            // opens, on a thread of its own, from the same manifest endpoint.
            // It only CHECKS - nothing is installed from here - and it runs
            // after the what's-new step so the version this run runs as is
            // known to it.
            app.state::<Bridge>()
                .start_updates(release::endpoint(app.handle()).as_deref());
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running the gentle-desk tauri application");
}

/// The reply of the preview scheme: the file the id names, with the type the
/// browser needs, or a 404 for an id this run never attached.
///
/// Only IDs the user's own drops created are served (`images::Images::read`), so
/// a page cannot name a file it was never given.
fn preview(app: &tauri::AppHandle<tauri::Wry>, path: &str) -> tauri::http::Response<Vec<u8>> {
    use tauri::http::{header::CONTENT_TYPE, HeaderValue, Response, StatusCode};

    let served = app
        .state::<Bridge>()
        .holder
        .images()
        .read(path.trim_start_matches('/'));
    let (status, mime, body) = match served {
        Some((mime, bytes)) => (StatusCode::OK, mime, bytes),
        None => (StatusCode::NOT_FOUND, "text/plain", Vec::new()),
    };
    let mut reply = Response::new(body);
    *reply.status_mut() = status;
    reply
        .headers_mut()
        .insert(CONTENT_TYPE, HeaderValue::from_static(mime));
    reply
}
