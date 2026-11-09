//! gentle-desk Tauri bridge entry point.
//!
//! The bridge owns the app state (`bridge::AppStateHolder`, created at startup
//! and managed by Tauri) and talks to the web side only through the generated
//! contract (`contract`), whose TypeScript is exported below. The per-chat
//! commands the reducer answers are run by `effects`, off the window's thread,
//! from the command layer in `commands`.

pub mod bridge;
pub mod commands;
pub mod config;
pub mod contract;
pub mod effects;
pub mod events;
pub mod intent;
pub mod sessions;
pub mod state_view;
pub mod view;

use commands::Bridge;
use tauri::Manager;
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
        .manage(Bridge::new())
        .invoke_handler(builder.invoke_handler())
        .setup(move |app| {
            // Typed events need their registry mounted, or emitting panics.
            builder.mount_events(app);
            // The pump's emit step needs the window handle, which only setup
            // has: this is the one place the child events are armed. The pump's
            // threads start the first time the executor builds its host.
            app.state::<Bridge>().arm_events(app.handle());
            // The session list is armed the same way, and its first pass asked
            // for here: the window opens with a list that says it is reading.
            app.state::<Bridge>().start_sessions(app.handle());
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running the gentle-desk tauri application");
}
