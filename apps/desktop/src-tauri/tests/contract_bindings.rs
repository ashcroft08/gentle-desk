//! Drift guard: the committed `apps/desktop/src/contract/bindings.ts` must
//! match what the generator produces from the Rust definitions in
//! `src/contract.rs`. If this test fails, a Rust-side contract change was
//! committed without regenerating the TypeScript side (or vice versa).
//!
//! Regenerate with `GENTLE_DESK_REGEN_BINDINGS=1 cargo test -p
//! gentle-desk-app --test contract_bindings`.

use gentle_desk_lib::contract_builder;
use specta_typescript::Typescript;

const COMMITTED_BINDINGS: &str = "../src/contract/bindings.ts";

#[test]
fn committed_bindings_match_the_rust_contract() {
    let scratch_dir =
        std::env::temp_dir().join(format!("gentle-desk-bindings-check-{}", std::process::id()));
    std::fs::create_dir_all(&scratch_dir).expect("failed to create scratch directory");
    let regenerated_path = scratch_dir.join("bindings.ts");

    let builder = contract_builder();
    builder
        .export(Typescript::default(), &regenerated_path)
        .expect("failed to generate bindings into a scratch directory");

    // The regen switch writes the fresh bindings over the committed file; the
    // run without the switch is the guard itself and never writes into src.
    if std::env::var("GENTLE_DESK_REGEN_BINDINGS").as_deref() == Ok("1") {
        std::fs::copy(&regenerated_path, COMMITTED_BINDINGS)
            .expect("failed to write the committed bindings file");
    }

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
    // A Windows checkout may convert the committed file to CRLF; compare content, not line endings.
    let normalize = |text: &str| text.replace("\r\n", "\n");
    if normalize(&committed) == normalize(&regenerated) {
        Ok(())
    } else {
        Err(format!(
            "committed {COMMITTED_BINDINGS} differs from freshly generated bindings"
        ))
    }
}
