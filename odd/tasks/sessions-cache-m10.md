# sessions-cache-m10: persist the session index cache across runs

Backlog item 3, second half. Reversed an earlier recommendation to drop it.

## Specs

- S1: "esta app quiero que tambien la ocupen otras personas aparte de mi" and "no mantiene un orden de no crear tantas sesiones como yo" — a home with hundreds of big sessions must still list fast at startup.
- S2: Measured (release build, synthetic homes): 500 x 130 KB cold 363 ms; 2000 x 130 KB 1.3 s; 500 x 1 MB 9.7 s; warm refresh 3-47 ms. Cost grows with total bytes, so only the cold first read needs the cache.
- S3: Safety rule: a cached file is reused only if its size and mtime match (the in-memory rule); a corrupt, unknown-version or other-directory cache file is ignored and the full scan runs. The cache file never changes what the list shows, only how fast it appears.
- S4: Layers: serializable scanner state in `gentle-core` (pure), persistence in `gentle-backend`, location and wiring in `gentle-ui`. One responsibility per file.

## Tasks

- [ ] T1 core: serde on `SessionSummary` and `SessionScanner` state
- [ ] T2 backend: snapshot/restore of `SessionIndex` and `session_cache.rs` (versioned JSON, atomic write)
- [ ] T3 ui: cache file under the user's local app data, load at start, save after a refresh that changed something

## Log

- L1: "pero si esto lo ocupa otra persona ... quiero que tambien la ocupen otras personas aparte de mi" (after the question "explicame mejor eso" about dropping the cache)
