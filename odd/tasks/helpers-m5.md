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
- [ ] T3 (S3, S4) ui: `view/helpers.rs` panel, verified with a real delegation — commit: pending

## Log

- L1: "si" (go ahead with the helpers) after the repo preparation.
