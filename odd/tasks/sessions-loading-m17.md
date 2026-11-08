# sessions-loading-m17: loading state and partial results for the first full pass

Implements backlog item 12 (finding 1 of refresh-watcher-m16.md): on a big home the first pass is long and silent, so until it ends the sidebar claims there are no chats. The first pass now reports progress, shows what the restored cache already knows, and the final pass reconciles it.

## Specs

- S1: `SessionIndex::refresh_with_progress(on_progress)` calls the callback from the per-file loop with `(done, total)`; `refresh()` becomes a wrapper with a no-op callback and its behaviour and existing tests stay unchanged.
- S2: The refresh worker emits partial `Progress { sessions, done, total }` messages at most once every ~500 ms (time-based throttle, not per N files) and a final `Refreshed`; the UI consumes them with `Task::stream`. Only the first full pass emits partials; later passes stay silent.
- S3: At startup, before the first pass ends, the UI publishes a prepared list from the restored cache's `entries()`; the final pass reconciles it (new chats appear, deleted ones go).
- S4: `AppState` gains `list_status: Loading{done,total} | Ready`; `Input::SessionsProgress` replaces the list and updates counters; the final `SessionsPrepared` sets `Ready`.
- S5: While Loading the sidebar never says "No saved chats yet" (nor "Chats (0 of 0)" as final): it shows "Reading chats… done / total"; "No saved chats yet" only in Ready with total 0. The text comes from a pure function so it can be unit-tested.
- S6: One responsibility per file: progress flow in its own file (session_refresh/progress.rs), message handling out of app.rs (249 lines) into a module; no file past ~200 lines.

## Tasks

- [x] T1 (S1) backend: `refresh_with_progress` on `SessionIndex`; `refresh()` delegates to it with a no-op callback, so its behaviour and the existing tests are untouched. Route: gentle-ai-worker; commit: 
- [ ] T2 (S4) app: `list_status: Loading{done,total} | Ready` in `AppState`, `Input::SessionsProgress` (replaces the list, updates the counters) and the final `SessionsPrepared` setting `Ready`; search and `visible_sessions` behaviour unchanged. Route: gentle-ai-worker; commit: 
- [ ] T3 (S2, S3, S5, S6) ui: partial `Progress` and the final `Refreshed` from the worker, throttled to one message per ~500 ms only on the first full pass, consumed with `Task::stream`; publish a prepared list from the restored cache's `entries()` at boot; the sidebar's list text as a pure function ("Reading chats… done / total" while Loading, "No saved chats yet" only in Ready with total 0, never "Chats (0 of 0)" as final); progress flow in `session_refresh/progress.rs` and message handling out of `app.rs` into its own module. Route: gentle-ai-worker; commit: 
- [ ] T4 (S1-S6) parent verifies: real window with a 50 000-chat synthetic home — the loading text appears at once with partial rows, the counters advance, the final list is right and no "No saved chats yet" flashes at any point; a warm run seeds from the cache before the pass ends; `cargo fmt --check` and `cargo clippy` clean; commit and PR. Route: parent; commit: 

## Design notes

- The throttle is time-based on purpose: files vary in size, so "every N files" would either flood the UI thread on a fast machine or stay silent on a slow one. One message per ~500 ms bounds the UI cost whatever the pass does.
- Partials carry the prepared list, so `update` still only swaps a list it did not fold; the folding stays on the worker thread (T3a of m16).
- Only the first full pass emits partials: a later pass is short (targeted) or already showing a complete list, and a `Loading` state there would flicker.
- With no cache file the first pass has nothing to seed and the list goes from Loading straight to the final list; the loading text still shows the counters.
- Workspaces (folders as groups, chats without a folder as their own group) are backlog item 13 and need their own design doc; they do not change this flow, which is per-list, not per-group.

## Log

- L1: "Empezá por el punto 1: leé lo anterior, proponeme el diseño en pocas líneas y esperá mi OK antes de escribir código."
- L2: user decisions "1A, 2A" = time-based throttle ~500 ms, and include cache seeding in this batch.
- L3: user wants workspaces (folders as groups + independent chats) as a later item after this one; whether it goes before backlog 8 is the user's decision.
