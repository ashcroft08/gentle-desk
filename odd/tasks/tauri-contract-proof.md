# tauri-contract-proof: choosing the Rust-to-TypeScript contract generator (T2b, S5)

Tried the three candidates in `odd/tasks/tauri-migration-m19.md` S5 against the Tauri
version already in `Cargo.lock` (tauri 2.12.5, tauri-build 2.7.4), with one tiny sample
contract in `apps/desktop/src-tauri/src/contract.rs`:

```rust
pub struct ChatBadge {
    pub chat_id: String,
    pub activity: ChatActivityMark, // enum: Working, Queued, Opening, NewReply, NeedsAttention, Ended
    pub unread: bool,
}
#[tauri::command] pub fn get_badge(chat_id: String) -> ChatBadge
```

The winner is wired in on `feat/desktop-skeleton`; the other candidates were removed after
the trial. See "Recommendation" and "Caveats".

## Candidates

1. `ts-rs` (v12.0.1) - derive-based type export; types only.
2. `specta` standalone (2.0.0-rc.28 + specta-serde/specta-typescript 0.0.15) - spectas type
   layer without the tauri glue; types only.
3. `tauri-specta` (tauri-specta 2.0.0-rc.26 over specta 2.0.0-rc.26 / specta-serde and
   specta-typescript 0.0.13) - the full glue: derives the same types AND typed command
   invocation wrappers from the `#[tauri::command]` functions themselves.

## Results

| axis | ts-rs 12.0.1 | specta standalone (rc.28 / 0.0.15) | tauri-specta rc.26 (ts 0.0.13) |
|---|---|---|---|
| works with tauri 2.12.5 in lock | yes (independent of tauri) | yes (independent of tauri) | yes (needs the `common-controls-v6` tweak below) |
| extra crates in Cargo.lock (delta) | 2 (ts-rs, ts-rs-macros) | 5 (specta, specta-macros, small_ctor, specta-serde, specta-typescript) | 7 (Inflector, specta, specta-macros, specta-serde, specta-typescript, specta-util, tauri-specta) |
| compile time added (rough) | ts-rs + ts-rs-macros among ~16 s wall for the app-crate rebuild; small | ~21 s wall first build of its 5-crates + app rebuild | 13 crates rebuilt in ~18 s wall including the app crate (measured by `cargo clean` on the stack + `cargo build -p gentle-desk-app`) |
| output shape | one file per type (`ChatBadge.ts`, `ChatActivityMark.ts`, imports between them) | one file with all types | one file: types AND a typed `commands` object |
| enum variants typed | `"Working" \| "Queued" \| ...` | same union tuple | same union tuple |
| command typed | no - the `.ts` contains no command; invoke calls stay hand-written | no | yes - `getBadge: (chatId: string) => __TAURI_INVOKE<ChatBadge>("get_badge", { chatId })` |
| drift test support (regenerate to temp path in a plain `cargo test`) | yes (programmatic `TS::export_all` with a `Config` export dir) | yes (`Typescript::export_to`) | yes (`Builder::export` to a temp path) |
| stability | stable 12.0.1, slow-moving API | pre-1.0 release candidates; API churned within the rc series (0.0.15 needs a different API than 0.0.13) | pre-1.0 release candidates too, but version-pinned (`specta = "=2.0.0-rc.26"`) inside tauri-specta so its internal pair is locked |
| blocker | commands invisible to the generator | same, plus rustcrc churn across 0.0.13 -> 0.0.15 | only one: default tauri features break plain test binaries (see Caveats) |

Sample of the generated TS (tauri-specta, `apps/desktop/src/contract/bindings.ts`):

```ts
export const commands = {
	getBadge: (chatId: string) => __TAURI_INVOKE<ChatBadge>("get_badge", { chatId }),
};

export type ChatActivityMark = "Working" | "Queued" | "Opening" | "NewReply" | "NeedsAttention" | "Ended";

export type ChatBadge = {
	chat_id: string,
	activity: ChatActivityMark,
	unread: boolean,
};
```

ts-rs / standalone-specta produce the same two type declarations, one file per type / one
flat file respectively, and nothing else.

## Recommendation

`tauri-specta`, wired in on `feat/desktop-skeleton`:

- It is the only candidate whose generated output covers the WHOLE bridge surface, not
  just the payload types: command names, the argument-name conversion Tauri does
  (`chat_id` -> `chatId`), and the return type all come from the Rust definition. With two
  of the three candidates, every `invoke()` call would still be re-declared by hand on
  the web side - exactly the drift S5 tries to close, one level up.
- Works with the tauri 2.12.5 in the lock; the drift check runs as a plain, deterministic
  `cargo test` (tests/contract_bindings.rs regenerates into a temp dir and fails if the
  committed `bindings.ts` differs - demonstrated RED by adding an enum variant and seeing
  the mismatch, GREEN after regeneration).
- The typed `commands` object slots into the layout rule (S6: bridge calls live in their
  own files, never inside components) instead of fighting it.

Not ts-rs: smallest payload and stable, but blind to commands. Not standalone specta:
same blind spot, plus the 0.0.x/rc API churn itself (0.0.15 needs
a different export call (`specta_serde::Format` plumbing) than 0.0.13, which
tauri-specta's own pin does not force us through)

## Caveats

- `tauri` must be used with `default-features = false, features = ["wry"]`. The default
  feature `common-controls-v6` makes tauri import `TaskDialogIndirect` (comctl32 v6) and
  plain console test binaries carry no manifest activating common controls v6, so ANY
  test binary that links tauri fails to load with STATUS_ENTRYPOINT_NOT_FOUND
  (0xc0000139) on Windows. Dropping the feature removes the import; it is only used for
  NATIVE dialogs (the migration's dialogs are Svelte, per S6), and the GUI binary itself
  is unaffected (proven: `cargo run` opens the window and exports the bindings).
- tauri-specta/specta/specta-typescript/specta-serde are 2.0.0-rc / 0.0.13: a future rc
  bump may change the export API; tauri-specta pins its internal specta pair, so a
  version upgrade moves all parts together through `cargo update`.
- The generated file imports `@tauri-apps/api/core`; `@tauri-apps/api` was added to
  `apps/desktop` devDependencies (it is the runtime dependency of any Tauri 2 frontend
  and would be needed by T3 regardless).
- The trial exposed two rough edges, both removed in the final wiring: the debug-build
  export path must be anchored on `CARGO_MANIFEST_DIR` (a cwd-relative path silently
  writes nowhere when the app is launched from the wrong directory), and peeled-off
  trial artifacts were all removed.
- Extra crates delta vs base: 7 (Inflector, specta, specta-macros, specta-serde,
  specta-typescript, specta-util, tauri-specta). Wall-clock compile-time numbers above
  are one machine, warm caches, dev profile - treat as order-of-magnitude only.
