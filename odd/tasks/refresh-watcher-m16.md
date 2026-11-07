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
- [ ] T2 (S2, S3, S4, S5) backend: pure `Debouncer` (quiet window 250 ms, max wait 1 s, time passed in), pure schedule (`next_delay`, moved from the measurement example), `SessionWatcher` over `notify` producing `Batch { paths, rescan }` and an `Err` when it cannot start — delegated to gentle-ai-worker
- [ ] T3 (S1-S4) ui: watcher events -> targeted refresh on the worker thread (queued, never dropped while one runs), full pass cadence from the last pass duration (adaptive timer replaces the fixed 5 s tick), watcher retry — delegated to gentle-ai-worker
- [ ] T4 (S1) Verify with the real window and synthetic homes of 20 000 / 50 000 chats: idle CPU, append-to-visible latency, a burst, watcher unavailable (fallback), then PR + CI

## Open / unverified (to check, not assume)

Network drives and OneDrive folders; bursts far larger than 50 files overflowing the OS buffer (the rescan signal and the safety-net pass are the answer, to be tested); sleep/resume; sessions directory deleted while watched; a deleted project directory (may not report its files).

## Log

- L1: "1. Observador + refresco por archivo + sondeo lento (recomendada)" (user's choice among A keep, B bounded polling, C watcher). Context backstop reached (60 %): work is delegated in units and re-verified by the parent.
