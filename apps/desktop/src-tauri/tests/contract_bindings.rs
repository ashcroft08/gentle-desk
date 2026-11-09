//! Drift guard: the committed `apps/desktop/src/contract/bindings.ts` must
//! match what the generator produces from the Rust definitions in
//! `src/contract.rs`. If this test fails, a Rust-side contract change was
//! committed without regenerating the TypeScript side (or vice versa).

use gentle_desk_lib::contract;
use specta_typescript::Typescript;
use tauri_specta::{collect_commands, Builder};

const COMMITTED_BINDINGS: &str = "../src/contract/bindings.ts";

#[test]
fn committed_bindings_match_the_rust_contract() {
    let scratch_dir =
        std::env::temp_dir().join(format!("gentle-desk-bindings-check-{}", std::process::id()));
    std::fs::create_dir_all(&scratch_dir).expect("failed to create scratch directory");
    let regenerated_path = scratch_dir.join("bindings.ts");

    let builder = Builder::<tauri::Wry>::new().commands(collect_commands![contract::get_badge,]);
    builder
        .export(Typescript::default(), &regenerated_path)
        .expect("failed to generate bindings into a scratch directory");

    let result = fg_check(&regenerated_path);

    let _ = std::fs::remove_file(&regenerated_path);
    let _ = std::fs::remove_dir(&scratch_dir);

    result.expect(
        "src/contract/bindings.ts is out of date: regenerate it from the Rust contract \
         (run the app once in debug, or cargo test after updating the committed file)",
    );
}

fn fg_check(regenerated_path: &std::path::Path) -> Result<(), String> {
    let committed = std::fs::read_to_string(COMMITTED_BINDINGS)
        .map_err(|e| format!("committed bindings file is missing or unreadable: {e}"))?;
    let regenerated = std::fs::read_to_string(regenerated_path)
        .map_err(|e| format!("regenerated bindings unreadable: {e}"))?;
    if committed == regenerated {
        Ok(())
    } else {
        Err(format!(
            "committed {COMMITTED_BINDINGS} differs from freshly generated bindings"
        ))
    }
}
