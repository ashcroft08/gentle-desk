# helpers-m5: helpers activity (`setWidget` gentle-agents.activity/v1)

Show the delegated helpers (subagents) of the open chat: counts and a task list, fed by the `gentle-agents` widget that gentle-shell publishes over RPC.

## Specs

- S1: "helpers (setWidget gentle-agents.activity/v1)" — decode the widget frames into typed state (helpers activity deferred since core-m0 T4).
- S2: Behaviour follows the reference app's `helpersActivity.ts` / `chatReducer.ts` (tolerant parsing; malformed frame keeps the previous state; a clear empties; only widgetKey `gentle-agents`; tasks dropped by gentle-pi are retained and promoted to done; order running, waiting, queued, finished).
- S3: "que no se cree un archivo largo con todo metido" — one responsibility per file (core `helpers/{types,parse,merge}.rs`, UI `view/helpers.rs`).
- S4: Layers (core-m0 S4): pure decoding/state in `gentle-core`; the UI only draws.

## Tasks

- [x] T1 (S1, S2, S3) core `helpers`: types + `parse_helpers_activity`, tests ported from the reference — RED observed (12 of 14 failed against a stub); 14 tests incl. the real fixture decoded in T2; timestamps are epoch ms (ISO or number accepted, non-ISO strings dropped; overflowing number literals make the whole frame unusable, unlike JS which drops the field); `parse_iso_ms` moved to `time.rs` (re-exported from `session`) — commit: see git log (feat/helpers-m5)
- [x] T2 (S2, S3) core: merge/retention/order + reducer folds `setWidget` into `ChatState.helpers` — RED observed (10 of 14 failed against a stub in core; 1 of 34 in app); core `helpers/{types,parse,merge,widget}.rs`, `reduce_chat_at(state, event, now_ms)` (the clock is supplied by the caller to stay pure; `reduce_chat` = no clock), app `Input::RpcAt(event, now_ms)`; `ChatState` lost `Eq` (it now holds JSON values); the recorded gentle-agents fixture folds as in the reference — commit: see git log (feat/helpers-m5)
- [x] T3 (S3, S4) ui: `view/helpers.rs` panel, verified with a real delegation — `view/helpers.rs` (counters + one card per task: title, status, agent, turns, tool calls, last step, error), shown only when there are tasks, plus `clock.rs` for the arrival time; 4 unit tests of the pure formatting; window widened to 1100x560 for the third pane — commit: see git log (feat/helpers-m5)

## Log

- L1: "si" (go ahead with the helpers) after the repo preparation.

## Verification (real gentle-shell 4.6.0, GUI)

- Asked the chat to delegate two helpers in parallel: the panel showed "2 running" with each task's agent, turns, tool calls and last step updating live; when they finished it showed "0 running · 2 finished", both `done`, newest first.
- The recorded gentle-agents fixture folds identically to the reference behaviour (test), including a malformed frame and another extension's widget being ignored.
- 193 tests green across the workspace, clippy -D warnings and fmt clean.
- Not done / known limits: a task's thread (text/tool items) is decoded but not shown yet (cards only); the `done` colour has low contrast on the dark theme; a finished helper that gentle-agents stops reporting is stamped with the arrival time, not its true end time; helpers are not shown for reopened sessions (gentle-agents only reports live ones).

## Log (continued)

- L2: My own mistakes caught on the way: a hand-computed epoch constant in a test was wrong (checked with node, fixed, commit amended so no red commit remains) and a first test summary that hid a failing suite behind `head` (re-ran with every result line).
