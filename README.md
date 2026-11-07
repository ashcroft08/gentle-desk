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

## Code organization

One responsibility per file; no file that collects everything. Rules of thumb:

- A file that grows past roughly 150-200 lines, or starts mixing concerns (state, wiring, drawing, I/O), is split before the next feature lands in it.
- UI (`crates/ui/src/view/`): one file per visual component, each a pure `view(...)` over plain data plus its own `Event` enum. A component never touches app state or another component; `app.rs` maps events to intents. New UI is a new file there.
- Logic that can be pure goes in `gentle-core` or `gentle-app` (testable without a window or a process); I/O stays in `gentle-backend`.
- Tests sit next to the code (`foo/tests.rs` or `tests/`), and a refactor keeps them green unchanged.

## License

MIT, see [LICENSE](LICENSE). Material derived from the reference app is credited in [NOTICE.md](NOTICE.md).
