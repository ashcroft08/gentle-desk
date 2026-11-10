# gentle-desk

A desktop frontend (Tauri 2 shell in Rust, Svelte 5 + TypeScript view) for [gentle-shell](https://github.com/Gentleman-Programming/gentle-shell). It does not reimplement the agent: it spawns `gentle-shell --mode rpc` and speaks pi's JSONL RPC.

Personal project, early stage. Progress and decisions live in [`odd/tasks/`](odd/tasks/).

## Layout

```
crates/gentle-core   UI-free core: RPC protocol, codec, chat state (pure, testable)
crates/backend       Launcher resolution (npm .cmd shims) and the rpc child process (no orphans)
crates/app           Application state, reducers and commands (no window framework)
crates/runtime       UI-agnostic runtime: chat children, session refresh, startup,
                     app config, workspaces
apps/desktop         The Tauri 2 app: src-tauri (Rust bridge, commands, events) and
                     src (Svelte view, one component per file)
crates/term          gentle-term: terminal client, prompt -> streamed reply
```

Layers, with dependencies in one direction only:
`apps/desktop` (Tauri bridge and Svelte view) -> `runtime` (chat children, session refresh, startup; no window framework) -> `app` (state, reducers, commands) -> `backend` (process spawner, session index, config) -> `gentle-core` (RPC protocol).

## Development

Requires Rust stable and, on Windows, the MSVC Build Tools.

```sh
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

The web side lives in `apps/desktop` (Node and npm required):

```sh
cd apps/desktop
npm ci
npm run lint && npm run check && npm run test
npm run build   # the Rust app embeds the built dist/
```

To build the Windows installer (NSIS, per-user, no admin rights needed):

```sh
cd apps/desktop
npx tauri build   # writes target/release/bundle/nsis/gentle-desk_<version>_x64-setup.exe
```

The installer shows the rose on its welcome and header pages: `tools/installer/make-images.ps1` draws
`apps/desktop/src-tauri/installer/sidebar.bmp` and `header.bmp` from the app icon (run it again after changing the icon).
The icon set under `apps/desktop/src-tauri/icons/` is generated from `apps/desktop/app-icon.png`
with `npx tauri icon app-icon.png -o src-tauri/icons`.

To run the app against the stand-in shell instead of a real gentle-shell, set
`GENTLE_SHELL_BIN` to `tools/fake-gentle-shell/fake.mjs`.

The fixtures under `crates/gentle-core/tests/fixtures/` are recorded RPC streams copied from the reference TypeScript app (`gentle-shell-desktop`).

## Code organization

One responsibility per file; no file that collects everything. Rules of thumb:

- A file that grows past roughly 150-200 lines, or starts mixing concerns (state, wiring, drawing, I/O), is split before the next feature lands in it.
- UI (`apps/desktop/src/`): one Svelte component per file (eslint `max-lines` 200 enforces it). Rust owns the state; the web side is a pure view over the pushed state and sends intents back. New UI is a new component file.
- Logic that can be pure goes in `gentle-core` or `gentle-app` (testable without a window or a process); I/O stays in `gentle-backend`.
- Runtime logic (chat children, session refresh, startup) lives in `gentle-runtime`, with no window-framework imports; `apps/desktop/src-tauri` keeps only the Tauri wiring.
- Tests sit next to the code (`foo/tests.rs` or `tests/`), and a refactor keeps them green unchanged.

## License

MIT, see [LICENSE](LICENSE). Material derived from the reference app is credited in [NOTICE.md](NOTICE.md).
