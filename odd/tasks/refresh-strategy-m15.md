# refresh-strategy-m15: how the session list should notice changes

Backlog item 10. Measured, not decided: the numbers below come from `crates/backend/examples/refresh_strategies.rs` (Windows 11, NTFS, release build, synthetic homes of 2 KB sessions, warm page cache). No shipped code path was changed.

## Problem

The UI refreshes every 5 s with a full `SessionIndex::refresh()`, which `stat`s every session file (24-25 us per file). The cost is linear in the number of chats and the app is meant for people who may never delete any.

## Measurements

| Candidate | 20 000 chats | 50 000 chats |
|---|---|---|
| **A. polling every 5 s (shipped)**: pass time | 499 ms | 1 201 ms |
| A: CPU of one core | 10.0 % | 24.0 % |
| A: new activity visible after | 5.6 s | 6.5 s |
| **B. bounded polling** (`max(5 s, 20 x last pass)`): delay | 10.0 s | 24.0 s |
| B: CPU of one core | 5.0 % | 5.0 % |
| B: new activity visible after | 10.6 s | 25.6 s |
| **C. `notify` watcher**: idle CPU | 0.08 % | 0.00 % |
| C: event arrives after (median) | 0.4-0.7 ms | 0.2-0.4 ms |
| C: new activity visible after, running the full refresh per event | 586 ms | 1 500 ms |
| C: burst of 50 new files | 0 missed, 2 events per file (create + modify) | same |

## What the numbers say

- A degrades with scale in CPU; B trades that CPU for staleness that also grows with scale and has no ceiling (a 25 s old list at 50 000 chats). Neither removes the linear cost.
- C's cost does not depend on the number of chats, but as measured it still ran the *full* refresh on each event, so the list took as long as one polling pass (1.5 s at 50 000). The event itself is not the slow part: a refresh limited to the changed path was **not implemented, so its latency is unmeasured**.

## Unverified (must be checked before relying on a watcher)

- Network drives / OneDrive folders as the pi home; event-buffer overflow on bursts much larger than 50 files; sleep and resume; the sessions directory being deleted while watched; a new project directory appearing; Linux and macOS backends. Debouncing is needed (two events per created file).
- The 32-78 s *cold* pass on a fresh home (first read + antivirus) is unrelated to this choice; with the persisted cache the first pass is the warm one (1.2 s at 50 000).

## Options

1. **Watcher + targeted refresh + slow safety-net poll (recommended on scalability and architecture).** `notify` watches the sessions directory; each (debounced) event refreshes only that file; a low-frequency full pass (bounded polling, B) reconciles anything the watcher missed, and polling alone is the fallback when the watcher cannot start. Cost independent of chat count, near-instant updates; costs a new runtime dependency and more moving parts.
2. **Bounded polling only (B).** No dependency, small change; staleness grows with the number of chats.
3. **Leave as is (A).**

## Log

- L1: "lanzalo asi" (worker measured; parent re-verified: 293 tests green, clippy and fmt clean, shipped sources unchanged vs main, scratch homes gone). Decision pending.
