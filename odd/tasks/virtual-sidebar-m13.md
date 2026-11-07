# virtual-sidebar-m13: virtualized chat list (step 2 of session-list-design.md)

## Specs

- S1: Step 2 of the decision record: fixed-height rows, only the rows near the viewport exist as widgets, windowed from the scroll offset with blank spacers above and below ("vamos con la C").
- S2: Fixed row height accepted by the user: one-line title with an ellipsis, then age, message count and project.
- S3: Pure arithmetic separated from drawing (`windowing.rs`, tested), the sidebar component draws what it returns; one file per responsibility.
- S4: Prototype first: unknowns were scroll restoration on refresh, window resize and smoothness.

## Tasks

- [x] T1 (S3) `windowing::window(total, row_height, offset, viewport, overscan)` -> row range + spacer heights; clamps the offset, tolerates NaN/zero/negative/infinite inputs, covers the full list height exactly — RED observed (10 of 12 failed against a stub), 12 tests
- [x] T2 `text::truncate` moved out of the helper-thread formatter so the sidebar shares it (tests moved with it)
- [x] T3 (S1, S2, S4) sidebar: `responsive` supplies the pane height (follows a window resize), `scrollable::on_scroll` reports the offset, kept in `App` (window geometry, not app state); a new search sends the list back to the top (`scroll_to`); titles never wrap

## Measurements (Windows 11, release build, stand-in gentle-shell, synthetic homes, throw-away LOCALAPPDATA)

| 5000 chats | idle CPU | scrolling | working set |
|---|---|---|---|
| before (every row built) | 5.3 % | 11.7 % | 389 MB |
| virtualized | 3.5 % | 18.8 % (190 wheel ticks in 6 s; the earlier run sent 93, so about the same cost per tick) | **233 MB** (the empty-app baseline is ~230 MB) |

- Rows are correct and gap-free while scrolling (screenshots at the top and ~chat 3500); the search over 50 000 chats is instant and returns to the top of the list.
- Answers to the unknowns: resize is covered by `responsive` (not exercised by hand); scroll position survives refreshes because the list keeps the same scrollable; smoothness judged by eye only, no frame times.

## Findings that this change exposed (NOT fixed here)

1. **Periodic refresh cost grows linearly with the number of files.** With 50 000 chats the app idled at 30 % of one core (40 % while scrolling) with working set 341 MB: every 5 s the worker `stat`s every session file (about 25 us each, ~1.2 s per refresh at 50 000). Drawing is no longer the bottleneck; polling is. Candidates: a file-system watcher (push-based, cost independent of N, new dependency) or cost-bounded polling (never refresh more often than a multiple of the last refresh's duration).
2. **The session summary keeps the whole first message** (title source) in memory and in the persisted cache: the 50 000-chat cache file was 65 MB (about 1.3 KB per chat with ~900-character first messages). Real first messages can be much longer. The summary should store a bounded title (the UI shows at most 32 characters).

## Not done / limits

- Resize and keyboard navigation not exercised; no frame-time or input-latency measurement.
- With a query that matches thousands of chats the list is windowed the same way (verified by reasoning and unit tests, not separately timed).
- Row height 50 px is tuned for the current fonts; larger system font scaling is untested.

## Log

- L1: "mergea y vamos con la C": PR #7 merged, then this step. Clippy rejected my negated float comparison (`!(row_height > 0.0)`); replaced by `!(finite && > 0)` and added infinity to the test.
