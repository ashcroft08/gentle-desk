# session-list-design: how the chat list and the thread scale

Decision record, made on scalability and architecture, not on speed of implementation and not on the maintainer's own usage (the app is meant for other people, who may never delete chats).

## Specs

- S1: "analicemos la mejor opcion, no la mas rapida de implementar, ni para mi caso, la mejor en escalabilidad y arquitectura".
- S2: Decided by the user: search by title and project only (not inside message text); sidebar rows of fixed height (one-line title with an ellipsis; below it age, message count and project).
- S3: Facts that shaped it (perf-check-m11): every sidebar row is built and drawn today (~30 KB per row, 389 MB at 5000 chats); opening a history parses all of its markdown at once (300 messages 46 ms, 3000 messages 192 ms, release); iced 0.14 has no virtualized list, but `scrollable::on_scroll` reports the viewport (absolute offset, bounds, content bounds), which is enough to window rows by hand; no maintained virtual-list crate for iced was found.

## Problem split

The list fuses three things; the design separates them: (1) the query (which chats, in what order), (2) the store (where summaries live and how they are queried), (3) the drawing window (which rows are really painted).

## Options considered

- A: cap at 100 rows + "show more" + search. Bounded memory, poor UX, no scalable abstraction; search is needed anyway and is part of C.
- B: only virtualize drawing. Better, but no query model: finding a chat stays unsolved.
- C (chosen): pure query model in `app` + virtualized drawing; summaries in memory (~300 B each: 100 000 chats ~ 30 MB).
- D: C plus SQLite as the store with paging (and FTS5 for message-text search). Scales further and enables content search; costs a native dependency, a schema and migrations, and nothing visible needs it yet.

## Decision

C, with a boundary prepared for D:

- `app` (pure, tested without a window): `SessionQuery` (text now; project filter and order later) and a function returning the visible chats; selection stays by id (path), never by position, so a refresh cannot move what is open.
- `ui`: virtualized sidebar with fixed row height, windowed from `on_scroll` with spacers above and below.
- `backend`: `SessionIndex` stays; put behind a `SessionStore` trait only when D is triggered.
- Thread: windowing at the data level (show the last N messages, "load earlier", parse markdown only for the window), not pixel virtualization: message heights vary and the bottom anchor works today.
- Triggers for D: wanting to search inside message text, or more than ~100 000 chats.

## Order of work (one PR each)

1. Pure query model in `app` + search box in the sidebar (preceded by a behaviour-preserving split of `crates/app/src/lib.rs`, 283 lines, into one file per responsibility).
2. Virtualized sidebar (prototype first: scroll restoration on refresh, window resize, smoothness are not yet known).
3. Data-level window for the thread.
4. `SessionStore` / SQLite only if a trigger fires.

## Known unknowns

- Virtualization with `on_scroll` is not prototyped: `on_scroll` fires on scroll, not on resize; scroll restoration after a refresh is untested.
- Measurements so far are CPU and memory, not frame time or input latency.

## Log

- L1: User asked whether the list and the thread are lazy (they are not) and then asked for the best option by scalability and architecture; chose, when asked, search by title/project only and fixed-height rows. PR #6 (measurements) was merged first.
