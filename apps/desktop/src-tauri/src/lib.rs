//! gentle-desk Tauri bridge entry point.
//!
//! The bridge owns the app state (`bridge::AppStateHolder`, created at
//! startup and managed by Tauri) and talks to the web side only through the
//! generated contract (`contract`), whose TypeScript is exported below.

pub mod bridge;
pub mod contract;

use tauri_specta::Builder;

/// The whole typed surface of the bridge: the commands and events the
/// contract generates the TypeScript for. Shared with
/// `tests/contract_bindings.rs` so the drift guard sees exactly what the
/// running app registers.
pub fn contract_builder() -> Builder<tauri::Wry> {
    Builder::<tauri::Wry>::new()
        .commands(tauri_specta::collect_commands![
            contract::get_badge,
            bridge::dispatch_intent,
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
        .manage(bridge::AppStateHolder::default())
        .invoke_handler(builder.invoke_handler())
        .setup(move |app| {
            // Typed events need their registry mounted, or emitting panics.
            builder.mount_events(app);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running the gentle-desk tauri application");
}
