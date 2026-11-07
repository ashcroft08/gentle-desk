# core-m0: toolchain, workspace and RPC core

Native Rust desktop client for gentle-shell (pi RPC over JSONL). M0 = toolchain + a UI-free core crate validated against the reference app's fixtures. Reference clone: `C:\Users\josep\dev\gentle-shell-desktop\repo`.

## Specs

- S1: "vamos desde cero" — a new Rust project, not a fork of the TypeScript app. Location: `C:\Users\josep\dev\gentle-desk`.
- S2: "UI 100% nativa en Rust (egui, iced, Slint, Dioxus)" — the final UI is native Rust; the framework is chosen later by prototype, never Electron or a webview.
- S3: "la logica la maneja gentle-shell" — this project is only the frontend: it spawns `gentle-shell --mode rpc` and speaks pi's JSONL RPC; it does not reimplement agent logic.
- S4: Architecture in three one-way layers: `ui` (renders state, emits intents, no I/O) -> `app` (state, reducers, commands; pure and testable) -> `backend` (process spawner, session index, config, launcher resolution). M0 builds the core crate only: no UI crate yet.
- S5: Core behaviour is validated against the reference fixtures (`src/main/domain/rpc/__fixtures__/*.jsonl` of the reference clone), copied into this repo.
- S6: Toolchain: "rustup + MSVC Build Tools (recomendado)" installed through winget.

## Tasks

- [x] T1 (S6) Install rustup + MSVC Build Tools; `cargo --version` works (cargo 1.99.1, rustc 1.99.1) — inline — commit: n/a (system install)
- [x] T2 (S1, S4) Cargo workspace + `gentle-core` crate skeleton, `.gitignore`, README — inline — commit: see git log (feat/core-m0)
- [x] T3 (S3, S5) Codec: JSONL line splitter, decode events/responses, encode commands, tested with fixtures — inline — 20 tests pass (incl. 41 fixture lines), clippy -D warnings clean; RED not observed: code was written before the compiler existed — commit: see git log (feat/core-m0)
- [ ] T4 (S5) Chat reducer over the fixtures (messages, working, dialogs) — inline — commit: pending

## Log

- L1: "vamos desde cero"
- L2: Decisions: location `C:\Users\josep\dev\gentle-desk`; toolchain "rustup + MSVC Build Tools (recomendado)". Context: the user prefers a UI 100% native in Rust, taken as a slow personal project; two bugs of the reference app were found (sidebar loaded once, "New chat" no-op) and a Windows `.cmd` spawn bug (EINVAL), which the Rust design must avoid.
