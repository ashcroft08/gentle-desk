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
- [ ] T3b (S2, S3) app: sessions list state, refresh command, OpenSession (switch_session -> get_messages -> history), failed responses as error notices — commit: pending
- [ ] T4 (S2) ui: sidebar with the list, refreshed on start, after each turn and on New chat; click opens — commit: pending

## Log

- L1: "dale con el índice de sesiones"
