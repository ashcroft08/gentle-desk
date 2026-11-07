# bounded-title-m14: bounded titles in the session summary and cache

Finding 2 of virtual-sidebar-m13.md: the summary kept the whole first message.

## Specs

- S1: Keep at most `MAX_TITLE_CHARS` (200) characters of a session name or first message, cut on a character boundary; message count and activity are unaffected.
- S2: A cache written before this change still holds whole messages, so it must not be reused: `CACHE_VERSION` 1 -> 2. The cache file name no longer carries a version (`session-index.json`); the version lives only inside the document, one place instead of two.

## Tasks

- [x] T1 (S1) `bounded()` in `gentle-core::session` — RED observed (4 of 5 new tests failed against limit 0; the fifth passes by design), 5 tests
- [x] T2 (S2) `CACHE_VERSION = 2`, file name without version, test that a version-1 document is ignored

## Verification

- 287 tests green, clippy -D warnings and fmt clean.
- Synthetic 20 000-chat home: cache 11.7 MB = 585 B per chat (was about 1.3 KB per chat with ~900-character first messages; real first messages can be much longer, so the saving is larger in practice). Warm start 477 ms for 20 000 files, cold 33.8 s (debug-heavy synthetic parse of 44 MB; unchanged by this work).

## Notes

- The old `session-index-v1.json` in the maintainer's `%LOCALAPPDATA%\gentle-desk` (10 KB, written by my earlier verification runs) was deleted by hand; anyone else with an old file can delete it, it is ignored from now on.
- The warm start of 477 ms for 20 000 files is the same linear `stat` cost behind backlog item 10.
