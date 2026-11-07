# visual-polish-m8: readability fixes in the window

Backlog item 2 (odd/tasks/backlog.md).

## Specs

- S1: Helper cards were lighter than the rest of the dark theme and the green `done` had low contrast (known limits recorded in helper-thread-m6).
- S2: The sidebar shows no time; add how long ago each chat was active ("timestamps in the sidebar", backlog item 2).
- S3: One responsibility per file; the age formatting is pure and tested (`time_ago.rs`).

## Tasks

- [x] T1 (S2, S3) `time_ago(now_ms, then_ms)` ("now", "5m", "3h", "2d", "3mo", "1y"; future = now) — RED observed (5 of 5 failed against a stub), 5 tests
- [x] T2 (S2) sidebar rows read "9h · 12 msgs · project"; the wall clock is passed into the view
- [x] T3 (S1) cards are transparent buttons inside the theme's own `rounded_box`; status colours are fixed light tones shared by the list and the thread view; the card's error line uses the same red
- [x] T4 `helpers` scenario for the stand-in gentle-shell (`tools/fake-gentle-shell/helpers-frame.mjs`: one helper per state) so this can be checked by eye without the model

## Verification

- 222 tests green, clippy -D warnings and fmt clean.
- By eye in the window with the stand-in: five cards (running, waiting, queued, failed, done) with uniform background and readable status colours; sidebar ages show ("9h", "10h", "11h").
- Not done: the card has no hover feedback any more (the transparent button has none); ages refresh on the 5 s tick or any event, not continuously; no unit test for colours (visual).
