# refresh-watcher-m16: watcher + targeted refresh + slow safety-net poll

Implements option 1 chosen by the user in refresh-strategy-m15.md ("1. Observador + refresco por archivo + sondeo lento (recomendada)").

## Specs

- S1: The list notices a new or changed chat without enumerating every file: cost independent of the number of chats; with 50 000 chats a new activity must show in well under a second (m15 measured 1.5 s per full pass, event arrival 0.2-2.5 ms).
- S2: A watcher (`notify`) on the sessions directory feeds a debounced stream of changed paths (a created file produces two events); each batch refreshes only those paths.
- S3: A watcher can miss events (buffer overflow, sleep/resume, network or cloud-synced folders). A low-frequency full pass reconciles: `max(60 s, 20 x last full pass)` while the watcher runs, `max(5 s, 20 x last full pass)` when it does not. An overflow/rescan signal from the watcher triggers a full pass at once.
- S4: If the watcher cannot start (directory missing, unsupported filesystem) the app falls back to the bounded polling and retries starting the watcher on each full pass. The list is never wrong, only slower to update.
- S5: Layers: pure logic (debouncer, scheduling) tested without a window or file system; file-system code in `gentle-backend`; wiring in `gentle-ui`. One file per responsibility; shipped behaviour of `SessionIndex::refresh` unchanged (existing tests untouched).
- S6: `notify` becomes a regular dependency of `gentle-backend` (it is a dev-dependency of the measurement example today).

## Tasks

- [x] T1 (S1, S5) `SessionIndex::refresh_paths(&[PathBuf])`: same size/mtime/append/rescan/remove rules as `refresh` for just those paths (a missing file is dropped; paths outside `<sessions>/<project>/*.jsonl` ignored; no directory enumeration). `refresh` and `refresh_paths` share one per-file update — delegated to gentle-ai-worker; verified by the parent: 315 tests green, existing session_index tests untouched, entries identical to main on the real home (18 chats); RED observed (18 of 39 failed against a stub); shared per-file rule in `session_index/store.rs`, path validation in `paths.rs`; pre-existing quirk noticed, not fixed: `session_files` filters by extension only, so a directory named `x.jsonl` would be counted
- [x] T2 (S2, S3, S4, S5) backend: pure `Debouncer` (quiet window 250 ms, max wait 1 s, time passed in), pure schedule (`next_delay`, moved from the measurement example), `SessionWatcher` over `notify` producing `Batch { paths, rescan }` and an `Err` when it cannot start — delegated to gentle-ai-worker; verified by the parent: 366 tests green, watcher tests green 5 runs in a row, only the planned files changed; RED observed (45 failed against stubs); pure `session_schedule`, `session_watch/{debounce,classify,paths}`, real `SessionWatcher`; `notify` is now a regular dependency of gentle-backend
- [x] T3a (S1) app: `prepare_sessions(items) -> PreparedSessions` (pure; builds the folded search keys) and `Input::SessionsPrepared`, so the O(N) key folding (measured ~50 ms at 50 000 chats) runs on the worker thread, not in `update`; `SessionsLoaded(items)` stays and behaves exactly as `SessionsPrepared(prepare_sessions(items))` — delegated to gentle-ai-worker; verified by the parent: 374 tests green, existing tests untouched, RED observed (4 of 8 failed against a stub); UI-thread cost of a list change at 100 000 chats 107 ms -> 15 ms (folding, ~92 ms, moved to the worker); the remaining O(N) on the UI thread is a list comparison (3.7 ms) plus dropping the worker copy (11.5 ms), about 7 ms at 50 000
- [x] T3b (S1-S4) ui: pure `RefreshQueue` (paths accumulate while a pass runs and are never dropped; a full pass supersedes pending paths; more than a cap of paths becomes a full pass) and pure `SaveThrottle` (the persisted cache is rewritten at most every 30 s, flushed on close and by the next full pass; a 50 000-chat snapshot must not be written per event); worker functions for a targeted refresh and a full pass returning the prepared list and the pass duration; watcher batches bridged into the app (`Message::SessionsChanged`, `Message::WatcherLost`); adaptive full-pass timer from `full_pass_delay`; start the watcher at boot, retry on each full pass while not watching — delegated to gentle-ai-worker; verified by the parent: 407 tests green, shipped session_refresh tests untouched, no other crate touched; new modules refresh_queue, save_throttle, refresh_flow, watch_source; debt: two allow(dead_code) (Refreshed::stats, RefreshQueue::is_busy) used only by tests
- [x] T4 (S1) Verify with the real window and synthetic homes of 20 000 / 50 000 chats: idle CPU, append-to-visible latency, a burst, watcher unavailable (fallback), then PR + CI — see Verification below

## Design notes added while planning T3

- A targeted refresh still has to hand the UI a full list (`Input::SessionsLoaded` replaces it), so its cost in `update` was O(N): equality check plus re-folding every title (about 50 ms at 50 000 chats, from the m12 measurement of 99 ms at 100 000). T3a moves the folding to the worker. What stays O(N) on the UI thread is a comparison and an index list, to be measured in T4; if it is above one frame (16 ms) at 50 000 chats the next step is an incremental update of the list.
- Writing the persisted cache on every event would cost O(N) disk per change (about 29 MB at 50 000 chats); hence `SaveThrottle`.

## Open / unverified (to check, not assume)

Network drives and OneDrive folders; bursts far larger than 50 files overflowing the OS buffer (the rescan signal and the safety-net pass are the answer, to be tested); sleep/resume; sessions directory deleted while watched; a deleted project directory (may not report its files).

## Verification (real window, release build, stand-in gentle-shell, synthetic homes, throw-away LOCALAPPDATA)

| | before (5 s polling) | now (watcher + targeted refresh) |
|---|---|---|
| idle CPU, 50 000 chats (list loaded) | 24-30 % of a core | **1.7 %** |
| idle CPU, 20 000 chats | 10 % | 4.4 % (includes the app baseline of about 2-4 %) |
| a chat modified on disk shows at the top | 6.5 s | within 2 s (screenshot taken 2-2.5 s after the write) |
| burst of 200 new chats at 50 000 | n/a | all 200 visible within 4 s (search shows 200 of 50200) |
| cache file after a change and after the burst | rewritten on each change | not rewritten within 30 s (SaveThrottle) |
| watcher cannot start (no sessions dir) | n/a | reason logged once; the list still finds a chat created later; a later change shows in 2 s, so the retry started the watcher |

The 2 s figures are upper bounds from screenshots, not measured latencies.

## Findings, not fixed here

1. **The first full pass of a big home is long and silent.** With 50 000 chats and no cache it took about 25 s here (files in the page cache; 33-78 s uncached in the m15 measurements), and until it ends the sidebar says "No saved chats yet" (and "Chats (0 of 0)" while searching), which is misleading. Changes that happen during it queue behind it. A loading state and showing partial results are the follow-ups.
2. The first run of this check was invalid for that reason (the list was still empty); it was repeated waiting for the cache file.
3. Still O(N) on the UI thread per list change: a list comparison plus dropping the old copy, about 7 ms at 50 000 and 15 ms at 100 000 (T3a); not re-measured in the window.
4. Unverified, unchanged from m15: network drives and OneDrive folders, bursts far beyond 200 files overflowing the OS buffer (a rescan signal and the 60 s pass are the answer, not exercised end to end), sleep/resume, a deleted project directory.

## Log

- L1: "1. Observador + refresco por archivo + sondeo lento (recomendada)" (user's choice among A keep, B bounded polling, C watcher). Context backstop reached (60 %): work is delegated in units and re-verified by the parent.

## CI failure and fix (PR #10)

- The first CI run failed in `session_watch::tests::batches::an_append_to_a_session_file_is_reported` (it had passed 5 of 5 locally). Cause, confirmed with a probe (not committed): on Windows/NTFS a write through a handle that stays open and is never flushed is **not reported for at least 15 s** (0 of 5 runs); closing or `sync_all` gives the event in about 265 ms. The test kept its handle open, and locally it could pass by accident on the late "modify" event of the file creation.
- pi appends with `appendFileSync` (open, write, close) and writes headers with `openSync`/`closeSync` (checked in its `session-manager.js`), so its changes are reported promptly. A different writer that holds the file open is only noticed by the 60 s reconciliation pass; this is documented in `session_watch.rs`.
- Fix: the test now closes the handle like pi does and lets the create settle before appending; the limit is documented. 8 consecutive local runs green.
