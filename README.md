# gentle-desk

A native Rust desktop frontend for [gentle-shell](https://github.com/Gentleman-Programming/gentle-shell). It does not reimplement the agent: it spawns `gentle-shell --mode rpc` and speaks pi's JSONL RPC.

Personal project, early stage. Progress and decisions live in [`odd/tasks/`](odd/tasks/).

## Layout

```
crates/gentle-core   UI-free core: RPC protocol, codec, chat state (pure, testable)
crates/backend       Launcher resolution (npm .cmd shims) and the rpc child process (no orphans)
crates/term          gentle-term: terminal client, prompt -> streamed reply
```

Planned layers, with dependencies in one direction only:
`ui` (renders state, emits intents) -> `app` (state, reducers, commands) -> `backend` (process spawner, session index, config).

## Development

Requires Rust stable and, on Windows, the MSVC Build Tools.

```sh
cargo test
cargo clippy --all-targets -- -D warnings
```

The fixtures under `crates/gentle-core/tests/fixtures/` are recorded RPC streams copied from the reference TypeScript app (`gentle-shell-desktop`).
