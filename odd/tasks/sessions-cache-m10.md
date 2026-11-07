# sessions-cache-m10: persist the session index cache across runs

Backlog item 3, second half. Reversed an earlier recommendation to drop it.

## Specs

- S1: "esta app quiero que tambien la ocupen otras personas aparte de mi" and "no mantiene un orden de no crear tantas sesiones como yo" — a home with hundreds of big sessions must still list fast at startup.
- S2: Measured (release build, synthetic homes): 500 x 130 KB cold 363 ms; 2000 x 130 KB 1.3 s; 500 x 1 MB 9.7 s; warm refresh 3-47 ms. Cost grows with total bytes, so only the cold first read needs the cache.
- S3: Safety rule: a cached file is reused only if its size and mtime match (the in-memory rule); a corrupt, unknown-version or other-directory cache file is ignored and the full scan runs. The cache file never changes what the list shows, only how fast it appears.
- S4: Layers: serializable scanner state in `gentle-core` (pure), persistence in `gentle-backend`, location and wiring in `gentle-ui`. One responsibility per file.

## Tasks

- [x] T1 core: serde on `SessionSummary` and `SessionScanner` state (3 round-trip tests; RED = compile errors)
- [x] T2 backend: snapshot/restore of `SessionIndex` (`session_snapshot.rs`) and `session_cache.rs` (versioned JSON, atomic write: temp file + rename); `refresh()` itself untouched; disk helpers moved to `session_index/disk.rs`; 14 tests, RED observed (8 failed against stubs)
- [x] T3 ui: `%LOCALAPPDATA%gentle-desksession-index-v1.json` (`cache_path.rs`), load+restore at start, save only after a refresh that changed something; 7 tests, RED observed (3 failed against stubs)

## Log

- L1: "pero si esto lo ocupa otra persona ... quiero que tambien la ocupen otras personas aparte de mi" (after the question "explicame mejor eso" about dropping the cache)

## Verification

- Delegated to `gentle-ai-worker` (context backstop); re-verified by the parent: fmt, clippy -D warnings, 249 tests green, `refresh()` body unchanged in the diff (only snapshot/restore were added).
- **Startup on a 491 MB home (500 sessions x 1 MB), release build:** cold 10.1 s -> with the cache 12 ms, same list, cache file 650 KB.
- **Window:** the cache file appears in `%LOCALAPPDATA%\gentle-desk\` after the first refresh (10 KB for the 14 real sessions), is not rewritten by later unchanged refreshes (checked across several ticks), and the app closes with no leftover process.

## Notes for whoever maintains this

- The cache stores each session's title text (first user message) and project path, so it is a small plaintext copy of that metadata in the user's local app data; deleting the file is always safe (the next start does a full scan).
- Two places must agree if the format changes: `CACHE_VERSION` in `gentle-backend` and the `-v1` in the file name in `ui/src/cache_path.rs`. Any other version is ignored, never migrated.
- Known edges, none of which can make the list wrong: non-UTF-8 paths are not cached; two app instances share one temp file name (worst case one logs a failed save); on Windows a save can fail if another process holds the file open (retried on the next change); mtime must match to the nanosecond (same rule as in memory).
- Not covered: a GUI check of a really big home (the 10 s -> 12 ms measurement is from the backend example, not the window); the sidebar still says "No saved chats yet" while the first cold refresh runs (a "loading" state is a possible follow-up).

## Log (continued)

- L2: Scratch directories left by the worker were removed by the parent.
