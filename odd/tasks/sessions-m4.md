# sessions-m4: incremental session index

List pi's saved sessions without re-reading every file, keep the list fresh, and (last task) reopen one in the UI.

## Specs

- S1: "índice de sesiones incremental (cabeceras + mtime con caché, no parsear cada .jsonl)"
- S2: Bugs of the reference app not to repeat: "la lista de chats se cargaba una sola vez" (the list must refresh) and "listAll() parseaba todos los archivos de sesión".
- S3: Layers (core-m0 S4): parsing is pure (`gentle-core`), file system and cache live in `gentle-backend`, state/intents in `gentle-app`, rendering in `gentle-ui`.
- S4: Same semantics as pi's own `SessionManager` listing (`buildSessionInfo`): sessions live in `<home>/sessions/--<encoded-cwd>--/<timestamp>_<id>.jsonl`; first line is the `session` header (id, timestamp, cwd); name = latest `session_info` entry; first message = first user text; activity = latest user/assistant message time.
- S5: Home is the isolated `C:\Users\josep\.gentle-shell\agent` (from the M1 brief).

## Tasks

- [x] T1 (S3, S4) core: pure `SessionScanner` (line-by-line accumulator) + ISO timestamp parsing — RED: first run 2 of 12 failed, both due to a wrong hand-computed epoch constant in my test (checked the implementation against node Date.parse, fixed the constant); 12 tests, clippy/fmt clean — commit: see git log (feat/sessions-m4)
- [x] T2 (S1, S2, S4) backend: `SessionIndex` with per-file cache (size + mtime, parse only appended bytes, rescan on rewrite, drop deleted) — RED not observed (implementation written first); instead 3 mutants killed (no cache reuse, append from byte 0, keep deleted files); 10 tests; on the 14 real sessions the listing is identical to pi's own SessionManager.listAll() (modified, message count, cwd, title): first refresh 1.8 MB in 37 ms, second 0 bytes in under 1 ms; `examples/list_sessions.rs` is the manual check — commit: see git log (feat/sessions-m4)
- [x] T3a (S3) core: `messages_from_history` rebuilds the thread from a `get_messages` response — RED observed (5 of 6 failed against a stub), 6 tests — commit: see git log (feat/sessions-m4)
- [x] T3b (S2, S3) app: sessions list state, refresh command, OpenSession (switch_session -> get_messages -> history), failed responses as error notices — RED observed (all 9 new tests failed against a stub, 20 old stayed green); 29 app tests. Also backend `home` (which pi home the flags select: --home / --link / --isolated / GENTLE_SHELL_HOME, default isolated; RED 7 of 10) — commit: see git log (feat/sessions-m4)
- [x] T4 (S2) ui: sidebar with the list, refreshed on start, after each turn and on New chat; click opens — `gentle-ui`: sidebar (title, message count, project; current highlighted), refresh on start, after each settled turn, every 5 s and on demand, open on click; identical notices are not repeated (RED 1 test) — commit: see git log (feat/sessions-m4)

## Log

- L1: "dale con el índice de sesiones"

## Verification (real gentle-shell 4.6.0, isolated home, Windows)

- The 14 existing sessions list identically to pi's own `SessionManager.listAll()` (modified time, message count, cwd, title).
- Cost: first refresh 1.8 MB read in 37 ms; the next one 0 bytes in under 1 ms (a `stat` per file).
- GUI: the sidebar lists the sessions; clicking one switches the session and shows its history (heading, list, code block, Copy); the current one is highlighted.
- Reference bug "list loaded once": a session created by `gentle-term` while the window was open appeared at the top within the 5 s refresh, without restarting.
- 157 tests green across the workspace, clippy -D warnings and fmt clean.
- Not verified: history of sessions with tool calls in the GUI (covered by unit tests of `messages_from_history`), very large homes (the refresh runs on the UI thread; fine at 14 files / 37 ms cold), non-isolated (`--link`) homes in the GUI (resolution is unit-tested only), Windows paths with unusual encodings.

## Log (continued)

- L2: Real RPC shapes captured: `switch_session` -> `{success, data:{cancelled}}`; `get_messages` -> `{data:{messages:[system, user, assistant(content blocks), toolResult...]}}`. Switching sessions makes extensions re-announce themselves (the same `notify` repeated), hence the notice de-duplication.
- L3: GUI automation lessons: PowerShell must be DPI-aware (`SetProcessDPIAware`) or window rects and screenshots disagree; screenshot only the window rect (an earlier full-desktop capture also caught unrelated private browser content, which I deleted without using); a misclick had maximized the window.
- L4: Not done (next): background refresh off the UI thread for big homes, persisting the cache across runs, session rename/delete, filtering by project, timestamps in the sidebar.
