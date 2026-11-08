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

- [x] T1 (S1) backend: `refresh_with_progress` on `SessionIndex`; `refresh()` delegates to it with a no-op callback, so its behaviour and the existing tests are untouched. Route: gentle-ai-worker; commit: 1571282
- [x] T2 (S4) app: `list_status: Loading{done,total} | Ready` in `AppState`, `Input::SessionsProgress` (replaces the list, updates the counters) and the final `SessionsPrepared` setting `Ready`; search and `visible_sessions` behaviour unchanged. Route: gentle-ai-worker; commit: 21a40ec
- [x] T3 (S2, S3, S5, S6) ui: partial `Progress` and the final `Refreshed` from the worker, throttled to one message per ~500 ms only on the first full pass, consumed with `Task::stream`; publish a prepared list from the restored cache's `entries()` at boot; the sidebar's list text as a pure function ("Reading chats… done / total" while Loading, "No saved chats yet" only in Ready with total 0, never "Chats (0 of 0)" as final); progress flow in `session_refresh/progress.rs` and message handling out of `app.rs` into its own module. Route: gentle-ai-worker; commit: d76e3ef and a03d0a8
- [x] T4 (S1-S6) parent verifies: real window with a 50 000-chat synthetic home — the loading text appears at once with partial rows, the counters advance, the final list is right and no "No saved chats yet" flashes at any point; a warm run seeds from the cache before the pass ends; `cargo fmt --check` and `cargo clippy` clean; commit and PR. Route: parent; commit: this PR's docs commit

## Design notes

- The throttle is time-based on purpose: files vary in size, so "every N files" would either flood the UI thread on a fast machine or stay silent on a slow one. One message per ~500 ms bounds the UI cost whatever the pass does.
- Partials carry the prepared list, so `update` still only swaps a list it did not fold; the folding stays on the worker thread (T3a of m16).
- Only the first full pass emits partials: a later pass is short (targeted) or already showing a complete list, and a `Loading` state there would flicker.
- With no cache file the first pass has nothing to seed and the list goes from Loading straight to the final list; the loading text still shows the counters.
- Workspaces (folders as groups, chats without a folder as their own group) are backlog item 13 and need their own design doc; they do not change this flow, which is per-list, not per-group.

## Window check (T4, real window)

Release build at `a03d0a8`, Windows 11, window 1393x747 (125 % DPI), stand-in
gentle-shell (`GENTLE_SHELL_BIN=tools/fake-gentle-shell/fake.mjs`, no model), a
synthetic home of 50 000 chats x 4 KB (416 MB) and a throw-away `LOCALAPPDATA`
per run. Every screenshot is the app window only, saved outside the repo; the
homes, screenshots and scripts were deleted afterwards. All checks pass.

- S2 (throttled partials, silent later passes): the cold run's header counter
  changed about once per 500 ms window, never per file; the final `Refreshed`
  settled it to plain "Chats". A later pass (one appended message) updated the
  row and moved it to the top — "chat 7 · 5 msgs" — with the header never leaving
  "Chats".
- S3 (seed from the restored cache): the warm run showed all 50 000 rows with
  "Chats (50000 of 50000) · reading…" 0.66 s after launch, before the pass ended;
  the final pass reconciled the list (the cold run's last partial had 49 531 and
  the settled list answers "Chats (0 of 50000)" to a query that matches nothing).
- S5 (never "No saved chats yet" while loading): 30 frames sampled from 0.6 s to
  the end of the pass in the cold and warm runs never showed it, nor
  "Chats (0 of 0)". The empty home showed "No saved chats yet" only after the
  pass ended; a home whose sessions directory does not exist also ends in
  "Chats" + "No saved chats yet" (the watcher logs the missing directory once).

| check | observed |
|---|---|
| A cold 50 000, empty LOCALAPPDATA | header "… · reading…" with rows from the first frame on: 5005 rows at 0.6 s, 17 032 at ~2 s, 30 430 at ~4-5 s, 43 518 at ~8-9 s, 49 531 at ~10 s, then plain "Chats" with 50 000 rows. The pass took ~10-11 s here (page cache), not 25 s |
| B second launch, same LOCALAPPDATA | seeded rows in the first frame (0.66 s) with "· reading…", plain "Chats" from ~10 s; the cache was not rewritten, so the pass was stat-only — and still as slow as the cold one on this machine |
| C empty home (0 chats) | "Chats" + "No saved chats yet" from the earliest frame (1 s); "Reading chats…" never seen |
| D query matching nothing while loading | "zzzznomatch" typed at ~6 s was drawn at once: "No chats match" with header "Chats (0 of 40438) · reading…"; after the pass, "Chats (0 of 50000)" + "No chats match". The window is not frozen while loading, only the worker thread is busy |

Idle CPU after loading: 0.109 s of CPU over 10.96 s of wall time (app + stand-in)
= ~1 % of a core, in line with m16.

Odds and ends: while Loading with rows on screen the header prints
"Chats (N of N)" — two equal counters that read like a settled total, and only
the "· reading…" suffix says otherwise; the "Reading chats… done / total" line
is drawn only while the list is empty, so here it was gone in under 0.6 s and
what is actually seen during the pass is rows plus the "· reading…" header; the
visible top rows kept their order through every partial (all synthetic mtimes are
equal), so no jumping was observed.

## Log

- L1: "Empezá por el punto 1: leé lo anterior, proponeme el diseño en pocas líneas y esperá mi OK antes de escribir código."
- L2: user decisions "1A, 2A" = time-based throttle ~500 ms, and include cache seeding in this batch.
- L3: user wants workspaces (folders as groups + independent chats) as a later item after this one; whether it goes before backlog 8 is the user's decision.
- L4: Window check passed (A-D plus later-pass); findings carried over: warm pass of 50 000 files still takes ~10 s here so the seeded list stays under "· reading…" that long; the header shows "Chats (N of N) · reading…" while loading which can look settled; literal "Reading chats…" body text only shows while the list is empty.
