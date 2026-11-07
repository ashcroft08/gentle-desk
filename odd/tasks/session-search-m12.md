# session-search-m12: search the chat list (step 1 of session-list-design.md)

## Specs

- S1: Step 1 of the decision record `session-list-design.md`: a pure query model in `app` and a search box in the sidebar.
- S2: "Solo titulo y proyecto" (the user's choice): words are looked for in the title and the project name only, never in the rest of the path or in message text.
- S3: The search covers every chat, not a visible window, and selection stays by path so filtering never closes or moves the open chat.
- S4: One file per responsibility (`app/src` was split first: `lib.rs` of 283 lines mixed state, inputs, events, intents and notices).

## Tasks

- [x] T1 (S4) Behaviour-preserving split of `crates/app/src/lib.rs` into `state`, `input`, `intents`, `events`, `notices`; the 250 existing tests unchanged and green
- [x] T2 (S1-S3) `session_query`: `SessionItem::project()`, folded search keys built once per loaded list, `visible` indices recomputed on query change, `AppState::visible_sessions()`; `Intent::SetSessionQuery`; an identical reload changes nothing; the query survives refresh and New chat — RED observed (13 of 16 failed against a stub; the 3 that passed did so vacuously), 19 tests
- [x] T3 (S1) sidebar search box, "Chats (3 of 18)" header while filtering, "No chats match" state; the duplicated project-name code in the sidebar now uses `SessionItem::project()`

## Measurements (release, synthetic chats)

First design (filter inside `view`, folding on every call): 3-word query over 10 000 chats 8.3 ms, over 100 000 chats 88 ms, paid on **every redraw**. Reworked to derived state computed in `update` with keys prepared when the list changes:

| | 10 000 chats | 100 000 chats |
|---|---|---|
| load a new list (keys built, on the UI thread) | 9.6 ms | 99 ms |
| one keystroke (filter) | 0.26 ms | 3.6 ms |
| read the result in `view` | 0.5 us | 4.7 us |
| periodic reload with an identical list | 0.9 ms | 11.8 ms |

## Verification

- 269 tests green, clippy -D warnings and fmt clean.
- Window, stand-in gentle-shell, the real home: typing `DELEGA` (capitals, no accent) shows the three chats titled "Delegá ..." ("Chats (3 of 18)"); `zzz` shows "No chats match".

## Not done / limits

- The sidebar still draws one row per visible chat (step 2 virtualizes it); with an empty query and thousands of chats the drawing cost is the one measured in perf-check-m11.
- Loading a new 100 000-chat list costs ~99 ms on the UI thread (only when the list changed; moving the key-building to the worker thread is possible if it ever matters).
- Accent folding covers the common Latin letters (a e i o u n c and combining marks); other scripts match as typed, without case folding beyond Unicode lowercase.
- No project filter or sort order yet (the query type is where they would go).
- The search box has no clear button and no keyboard shortcut.

## Log

- L1: Two things went wrong on the way and were fixed: a long shell command wrote a truncated test file (rewritten whole), and the first query design recomputed on every redraw (found by timing it, reworked).
