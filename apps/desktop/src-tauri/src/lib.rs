//! gentle-desk Tauri bridge entry point.

pub mod contract;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri_specta::Builder::<tauri::Wry>::new()
        .commands(tauri_specta::collect_commands![contract::get_badge,]);

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
        .invoke_handler(builder.invoke_handler())
        .run(tauri::generate_context!())
        .expect("error while running the gentle-desk tauri application");
}
