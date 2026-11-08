# process-per-chat-m18: one gentle-shell process per chat, and a recoverable death notice

Implements backlog item 16: gentle-desk keeps one long-lived `gentle-shell --mode rpc` child and
changes chat inside it (`switch_session` / `new_session`), so a stale extension context from
gentle-pi's skill-registry timer kills the child and the window dead-ends until the app restarts.
This batch spawns one child per chat with `--session <path>` instead, keeps the children of chats
that are working alive in the background, and offers a Restart on death.

## Specs

- S1: A `ChatHost`-style manager keeps a map `session -> live child`: one `gentle-shell --mode rpc --session <path>` per chat, and a new chat is a child spawned without `--session`. Opening a chat makes it the foreground chat; it never stops the other children and never sends `switch_session` or `new_session` to a live child.
- S2: Spawns are serialized per session (rapid clicks on the same chat never create two children; a stale start result is discarded).
- S3: If a chat's child dies, the UI shows a recoverable notice with a Restart action, per chat child: Restart relaunches that chat's child and reopens that chat; no dead-end until app restart.
- S4: A chat that is working keeps running in the background. The sidebar shows a per-chat state: working / finished-unread / needs-attention (a pending extension dialog). Returning to it shows its full reply. Extension dialogs raised by background chats are queued, never lost.
- S5: Closing the app leaves no orphan process, for every child it spawned (existing guarantee, now multiplied by the number of live children).
- S6: One responsibility per file; process lifecycle logic pure and testable in `crates/app`, I/O in `crates/backend`; no file past ~200 lines.
- S7: Idle children are reaped after a configurable idle time, and the number of live children and of simultaneously working chats each has a configurable cap (default proposal: 4 live, 2 working — open decision, see Design notes), because parallel model calls consume the usage limit faster. A reaped chat reopens through `--session`.

## Tasks

- [ ] T1 (S1, S2, S5, S7) backend/app: manager lifecycle `session -> child` — one pure transition in `crates/app` for "start child for session S", "make S foreground" and "reap idle children under the caps", and the I/O in `crates/backend` (spawn with `--session` or without for a new chat, per-session spawn serialization, discard a stale start result, idle reap, stop every child on close). Route: gentle-ai-worker; commit:
- [ ] T2 (S1) ui: wire open/new chat to the manager as "make foreground" and remove the `switch_session` / `new_session` paths (`crates/app/src/intents.rs:32-34` and `:91-93`, the `switch_session` exchange at `crates/app/src/events.rs:33-41`), including their tests. Route: gentle-ai-worker; commit:
- [x] T3 (S3) ui: per-chat death notice with a Restart action that relaunches that chat's child and reopens it, replacing the terminal `"gentle-shell exited"` / `"cannot reach gentle-shell: …"` state (`crates/ui/src/app/process_update.rs:24-27`, `crates/ui/src/app.rs:66-68,155,161`). Route: gentle-ai-worker; commit:
- [ ] T4 (S4) ui: per-chat state in the sidebar (working / finished-unread / needs-attention) and queued extension dialogs from background chats; returning to a chat shows its full reply. Route: gentle-ai-worker; commit:
- [ ] T5 (S1-S7) parent verifies: real window with `tools/fake-gentle-shell` via `GENTLE_SHELL_BIN` (never the real model), including a stand-in that works for N seconds and one that dies after a session switch; background chats finishing and being read, dialog queueing, idle reap and caps; latency measured with the real gentle-shell using only `get_state`-style no-model calls; `cargo fmt --check` and `cargo clippy` clean; commit and PR. Route: parent; commit:

## Design notes

Open decisions for the user, each with a recommendation:

1. **Latency of starting a child is measured before anything is optimised.** Spawning a child per chat adds process start-up to the first open of a chat and to reopening a reaped one. T5 measures it with the real gentle-shell using only `get_state`-style calls (no model). Recommendation: measure first and optimise only if the number is felt in the window.
2. **A warm standby child is a later optimisation.** A pre-spawned child that receives the session could hide the start-up cost. Recommendation: not now; only if the measurement in 1 hurts. It doubles the live processes and the orphan-guard surface, so it deserves its own batch.
3. **Background chats stay alive (decided, L4/L5).** A chat that is working when another opens keeps working; opening a chat only changes the foreground. Consequence to keep in mind: the user can come back to a finished reply, and a pending extension dialog from a background chat is queued instead of being lost. Nothing in the UI may silently kill a working child.
4. **Memory and process cost of several live children.** Each child is a Node process running gentle-shell: tens of MB of RSS each plus its own extension load, so N live children cost roughly N times the current footprint. Recommendation: accept it because the alternative (one shared child) is what kills the window, and bound it with the caps of 5.
5. **Caps on live and working children (open decision).** Parallel model calls consume the usage limit faster: working chats are the expensive axis. Recommendation: default 4 live children and 2 simultaneously working chats, both configurable; over the working cap, queue the start instead of refusing it so no click is lost. Alternatives: no cap on live children (memory grows with the number of chats visited), or a cap of 1 working chat (loses the whole point of background work).
6. **Idle reap (open decision, S7).** Recommendation: reap a child after a configurable idle time (proposal ~10 min without activity and without a running model call); the chat reopens through `--session`, so the cost of a reap is one start-up. Alternative: reap only under memory pressure, which is harder to observe reliably.
7. **A chat whose stored cwd no longer exists.** `switch_session` fails in that case today; with `--session <path>` the child may fail to start for the same reason. Recommendation: treat a failed spawn as a spawn error on the death notice (S3) and let the user pick another chat, rather than leaving the window in the fatal state.
8. **Report the extension bug upstream to gentle-pi.** Separate user decision. It is reproduced (see Log), so the report is unblocked, but it is not part of this batch and the gentle-desk side must not depend on it being fixed.

Recorded uncertainties:

- The link between `switch_session` and the crash is inferred from code reading, not proven by an isolated test: RPC `switch_session` alone did not reproduce it in 3 trials because the skill-registry timer did not fire there. The fix is chosen because it removes the trigger class (session replacement inside a live child), not because the mechanism is fully proven.
- The reference behaviour is read from gentle-shell-desktop `ChatHost.ts:141-186` and `PiSession.ts:127-133,216-241`: one child per chat, `--session <path>`, no `switch_session`, and on child death an error whose next open/new spawns a fresh child. gentle-desk's S3 goes one step further by offering Restart.
- Background children multiply the orphan risk: the existing single-child close path does not cover N children, so S5 has to be verified per child in T5, not only for the foreground one.

## Follow-ups from the native review of T1

Advisory only: the review at lineage `review-6c50190c40c30c6e` was approved and acknowledged, and
none of these findings reopens it. Each one cites the code it was written against, one factual line
about what that code does and why it may matter, and the task that should address it.

- R3-existing-false-promise — `crates/backend/src/chat_host/host/lifecycle.rs:37-41`: a chat that
  already holds a slot is only marked active and answered `Ok(Opened::Existing)`, even while that
  slot is still `Status::Spawning`, so a caller that reads `Existing` as "a live child serves this
  chat" then gets `SendError::Starting` from `send`; it may matter because T2 makes `open` the path
  of every click, so a second click during a slow start looks like success. Task: T2.
- R3-send-holds-lock — `crates/backend/src/chat_host/host/routing.rs:14-30`: `send` holds the whole
  `State` mutex across `child.send(...)`, a blocking stdin pipe write, so a slow or blocked child
  stalls every other chat's send, reap and open until that write returns — a candidate fix belongs
  in T2 before the UI sends from many chats. Task: T2.
- R3-working-cap-race — `crates/backend/src/chat_host/host/routing.rs:53-55`: `may_start_work`
  evaluates `policy::working_full` under one lock acquisition and `set_working` takes the lock again
  later, so the pair is check-then-act and two chats starting together can both see room and both
  exceed `max_working`. Resolved by S7 (T4): the cap is enforced in the `gentle-app` reducer, in one
  step with the working mark, so no two prompts can take the same slot; `may_start_work` stays
  informational (design note 5).
- R4-002 — `crates/backend/src/chat_host/host/lifecycle.rs:70-71`: `self.stop_all(stops)` runs every
  reaped victim's graceful shutdown (up to `DEFAULT_GRACE`, 2 s each) synchronously before
  `spawner.spawn` is called, so the chat just clicked stays `Spawning` for the length of those
  shutdowns before its own child has even started. Task: T2.
- R4-003 — `crates/backend/src/chat_host/host.rs:150-167`: `forward` starts the per-chat event
  thread with `.spawn(..).ok()`, discarding the `Result`, so a thread that cannot be started
  forwards no event and never clears `alive`, leaving the slot looking live while the chat stays
  silent forever and no death is noticed. Task: T3.
- R4-001 (CRITICAL) — `crates/backend/src/chat_host/host/routing.rs:16-30`: recorded as **refuted by
  the provider refuter**; no action.

### Follow-ups as addressed by T2 (backend + app half, step 1 of 2)

- R3-send-holds-lock — **fixed**. `Child` now hands out a writer of its own
  (`chat_host::Writer`, `RpcWriter` for the real process) and `ChatHost::send`
  takes it under the lock, releases the lock, then writes. The regression test is
  `chat_host::tests::isolation::a_blocked_write_of_one_chat_stalls_no_other_chat`:
  while one chat's write is parked in its pipe, `open`, `close`, `set_working`,
  `working_count`, `foreground`, `live_keys` and a send to another chat all have to
  answer within 5 s (each runs on its own thread so a held lock fails the test
  instead of hanging it).
- R3-existing-false-promise — **half fixed, deliberately**. The dead-child reading
  is now pinned by
  `chat_host::tests::startup::a_chat_whose_child_died_starts_a_fresh_child_instead_of_answering_existing`
  (an exited child's slot is dropped, the chat is started anew, `Started` and not
  `Existing`). The `Spawning` reading is left as it is on purpose: a second click
  during a slow start must still not start a second child (S2), and
  `chat_host::tests::serialization::concurrent_opens_of_one_chat_start_exactly_one_child`
  pins `Opened::Existing` for exactly that case. A distinct `Opened::Starting`
  answer is a UI-visible distinction ("already coming up" vs "live") and belongs
  to T3/T4, together with the death notice that reads it; changing it now would
  rewrite the S2 test for a caller that does not exist yet.
- R4-002 — **real defect, fixed**. `open` now starts the clicked chat's child
  *before* it stops the children the reap or the cap took out, so the chat the user
  just clicked is no longer `Spawning` for the length of up to `DEFAULT_GRACE` (2 s)
  per victim. Pinned by
  `chat_host::tests::startup::the_clicked_chat_starts_before_the_reaped_victim_is_stopped`,
  which asserts the order of the fake spawner's journal (`spawn c` before
  `shutdown a`). The live cap is exceeded by one process for the length of a
  shutdown, which is the cheaper trade; the caller still blocks until the victims
  are gone, so the remaining latency (a frozen caller during shutdowns) is the
  async-command work of T1/T5, not of this fix.

### Follow-ups as addressed by T3

- R3-existing-false-promise — **now closed**. `ChatHost::open` on a chat whose
  child is still starting answers the new `Opened::Starting` instead of
  `Existing`, so a second click during a slow start no longer claims a live
  child. `chat_host::tests::serialization::concurrent_opens_of_one_chat_start_exactly_one_child`
  now pins `Opened::Starting` for that second click and still proves exactly one
  child (the assertion that changed; the count assertions did not), and
  `reopening_a_chat_only_makes_it_foreground_again` keeps pinning `Existing` for
  a chat whose child is up. The shell asks the child for nothing after a
  `Starting` answer, because the start already on its way is the one that will
  answer (`chat_runtime::tests::open_chat`).
- R4-003 — **fixed**. `ChatHost::forward` no longer discards the thread's
  `Result`: when the forwarder cannot be started, the child is marked gone and one
  unexpected `Closed` is published at once, so the chat gets the death notice and
  its Restart instead of looking live and silent forever. Not pinned by a test: a
  thread that cannot be started cannot be simulated without injecting a thread
  factory into the host, which would test the injection instead of the failure.
  The rest of the close path is pinned by `chat_host::tests::closes`.

### What T3 changed (S3, plus the S2 distinction above)

- App layer, pure and UI-free: a chat that loses its child is marked *ended* next
  to its activity (`Chats::is_ended`, `Chats::ended`, `ChatActivity::Ended`).
  `Input::ChatClosed { chat, expected }` is the only way in. `expected` is true
  when the host stopped the child itself, and then the chat is just `Idle`, keeps
  its thread and reopens silently. An unexpected close of **any** chat, foreground
  or background, ends that chat and leaves **one** error notice naming it, with
  `NoticeAction::RestartChat` as the notice's action (notices grew an optional
  action; extension notices still carry none). `Intent::RestartChat(chat)` clears
  the mark, makes the chat foreground and answers `ChatCommand::OpenProcess`, which
  reopens it through `--session`, so the history comes back through the existing
  open follow-up. Opening a chat clears the same mark, so clicking a dead chat in
  the sidebar is already a restart.
- Decision — **a prompt to an ended chat is a notice, not an implicit restart.**
  Restarting under the prompt would need the prompt queued until the child is up,
  and a send to a spawning chat is refused by the host; the simpler, honest
  behaviour is to keep the text in the composer, send nothing and re-show the same
  notice with its Restart button. Chosen explicitly, not by omission.
- Decision � **the death mark is cleared when the reopen is asked for, not when the
  child is confirmed up.** There is no input for "this chat's child is up" yet, so
  a restart whose spawn fails (a session whose cwd is gone, design note 7) reports
  itself through the existing `ChatFailed` error notice and the chat is no longer
  marked dead; clicking it again tries once more. The alternative � staying disabled
  after a failed spawn � would need that extra input and a second notice shape.
- Host side: `TaggedEvent.expected` is true only for a `Closed` of a child the
  host took out itself (idle reap, cap victim, `close`, `shutdown`). The slot holds
  a stop flag that is set before the stop begins and is read by the forwarder when
  it stamps the event, so the UI can tell a stop from a death.
- UI: a chat's death is no longer `self.fatal`. `fatal` stays for the one failure
  that ends everything (the launcher cannot be resolved): the banner is gone for
  child deaths, the notice carries the `Restart` button (component event mapped in
  `app`), and the composer stays usable for every other chat. For the ended chat
  it is disabled, and the notice beside it says why. New handling lives in
  `crates/ui/src/app/chat_closed.rs`, `send_prompt.rs`, `visible.rs` and
  `frame.rs`.
- Tests: `crates/app/tests/chat_death.rs` (seven cases: one notice per death with
  its action, a background death touching nothing else, an expected stop silent,
  restart, the refused prompt, reopening a dead chat) and
  `crates/backend/src/chat_host/tests/closes.rs` (a death is not expected; reap,
  cap victim, close and shutdown are).

## Window check (T5, real window)
Release build of `aaf073e` (`cargo build --release -p gentle-ui`), Windows 11,
window 1393x747 (125 % DPI), the stand-in gentle-shell
(`GENTLE_SHELL_BIN=tools/fake-gentle-shell/fake.mjs`, no model, no network), a
synthetic home of 20 chats (`node tools/perf/gen-home.mjs`) and a throw-away
`LOCALAPPDATA`, both in a `mktemp -d` outside the repo. Every screenshot is the
app window only (window rect, never the desktop); clicks are fractions of the
client rect, and typing happens only while the exact app window is the verified
foreground window. The home, the screenshots and the driver scripts were deleted
afterwards. All checks pass.

Stand-in extension (only `tools/fake-gentle-shell/**`, old behaviour kept as the
default): the `stream N` prompt streams `tick ` every 100 ms for N seconds and
then ends the message with the whole text, so a chat can be left while it works
and still finish on its own; `get_state` reports the `--session` path, or opens
and reports `<--home>/sessions/--fake--/<stamp>_<pid>.jsonl` when the child has
no session yet (the new chat needs that to become a stored chat);
`FAKE_SHELL_LOG` appends one line per lifecycle step and per received command,
which is what proves F. `error`, `dialog`, `slow`, `helpers`, `flood`, the echo
default and the `{}` reply to every other command are unchanged.

| check | observed |
|---|---|
| A two chats, two children | opening "chat 0" then "chat 12" left exactly two stand-in children: pid 23172 with `--session …/--C--proj0--/2026-01-01T00-00-00-000Z_0.jsonl` and pid 26736 with `…_12.jsonl`. Chat 0's child (23172) stayed alive the whole run (age 28 s when chat 12 opened, still `pid=23172` at the end) and received no `abort`; only chat 12's row was highlighted after the second click |
| B a working chat finishes in the background | prompt `stream 6` in chat 0: 2 s later ~20 `tick` tokens were on screen and the composer showed **Stop** (working). Chat 12 was opened at t+7 s while the stream ran; no `abort` reached pid 23172 and no new child was spawned. Back on chat 0 at t+24 s the window showed the complete reply — three wrapped lines of `tick` (6 s at 100 ms) plus its `Copy` action — and the composer was back to **Send** (the turn had settled while it was in the background) |
| C new chat becomes the stored chat | "New chat" spawned pid 14324 with `--home <home>` and **no** `--session`; it received one `get_state`, whose `sessionFile` reply turned the chat into the stored one: the sidebar gained "(no messages) · now · 0 msgs · tmp.L5Z9AMROvC", highlighted as the open chat, and the home gained `sessions/--fake--/…_14324.jsonl`. Clicking that same row again started no child (3 children before, 3 after, zero `start` lines in the log after the click) and pid 14324 was unchanged |
| D a child killed while it is a background chat | killed pid 26736 (chat 12) with `Stop-Process -Force` while the foreground chat was the new one: the app logged `[…_12.jsonl] child exited` on stderr and nothing else; no banner, no notice, the window kept answering (a click on chat 0 was drawn at once and showed its complete reply intact). Opening the killed chat spawned a fresh child for the same `--session` (pid 33488); the window then showed an **empty thread** for it (the stand-in answers `get_messages` with `{}`, so there is no stored history to draw), no error, and a prompt sent there answered `echo: hola` |
| E close leaves nothing behind | `WM_CLOSE` on the window; at **2.0 s** (the first sample) 0 `gentle-desk` processes and 0 stand-in `node` processes remained, well inside the 5 s bound. The log has 3 `exit` lines for the 3 children that were alive (stdin EOF, exit 0); the killed one has none, as expected |
| F no session switch was ever sent | every command the whole run delivered to every child: 3 `get_messages`, 1 `get_state`, 2 `prompt`, and **0 `switch_session`, 0 `new_session`**. The 4 child argv lines carry `--home …` and `--session …` only for stored chats; the new chat's argv has no `--session` |

Odds and ends:

- Stored history is not rendered in this run: `get_messages` is answered `{}` by
  the stand-in and `messages_from_history` reads `{"messages": [...]}`, so an
  opened stored chat shows an empty thread and only a live reply is visible.
  The check says nothing about history drawing, only about which chat is open.
- The generated session headers keep `cwd: C:/dev/projN`, which does not exist.
  The stand-in never looks at it; real pi would refuse such a session, so this
  run does not cover design note 7 (a stored chat whose cwd is gone).
- Start-up latency per open (design note 1) was not measured: it needs the real
  gentle-shell, which this check is not allowed to run.
- `stream N` ends with the complete text, unlike `slow`, which ends with
  `ABORTED`; `slow` itself was left alone.
- The per-chat death notice (S3/T3) does not exist yet: a killed background chat
  shows nothing in the window until it is opened, and opening it silently starts
  a fresh child. D only asserts what this batch promises — the window stays
  responsive and the other chats are untouched — not the T3 notice.

## Log

- L7 (S7, T4 working cap, user decision "Ponerlo en cola"): the app enforces the working
  cap now. `Input::SetWorkingCap` passes the host's `Limits::max_working` at boot (`0` means
  no cap), and a prompt to an idle foreground chat with no slot free is **queued**, never lost
  and never refused: its user message is appended as today, the chat is marked Queued in the
  sidebar, and the prompt goes out FIFO, one per freed slot, the moment a chat stops working
  (`agent_end` / `agent_settled`) or its child goes away (expected stop or death), still under
  the cap. The released prompt marks its chat Working in the same reducer step, so a slot is
  never handed out twice. An Abort on a Queued chat cancels its prompt, and a chat that dies
  (S3) drops its queued prompt with no further notice. Tests: `crates/app/tests/chat_working_cap.rs`.

- L1: "la 3 explicame mejor, hace algo parecido a gentle-shell-desktop?"
- L2: "si dale con eso, es lo mejor por ahora o tienes algo mejor y robusto?"
- L3: parent answer — option 3 (two layers: one child per chat, plus a recoverable death notice) is the best now; warm standby is a later optimisation to be measured first.
- L4: "con lo de s4 hay algo, si yo como usuario abro otro chat para trabajar algo nuevo alla pero quiero que el chat viejo siga trabajando y regresar a ver su respuesta cuando termine, eso es un proceso organico"
- L5: parent answer — the stop-on-switch idea is replaced by a process manager that maps session -> child process: opening a chat makes it the foreground and leaves every other child running, so a chat that is working finishes in the background and its reply is there on return. S1, S2, S3, S4 and S5 are rewritten to that model, S7 (idle reap + caps) is added, and the tasks are re-cut accordingly. The upstream report stays a separate user decision.
- L6: "native review of T1 approved (lenses resilience+reliability, reviewers ran on
  commandcode/deepseek-v4.1-flash, total cost ~US$0.05); advisory findings recorded as
  follow-ups".

### Reproduction evidence (kept here as text, not as a path)

Real window, gentle-shell 4.6.0 and 4.7.0, twice; observed lines:

```text
# changing chat in the window (OpenSession -> RPC switch_session)
Notify: error: This extension ctx is stale after session replacement or reload
UnhandledPromiseRejection  (gentle-pi/extensions/skill-registry.ts: timer/poll callback
                            calls notify with a ctx captured at session_start)
-> the child process exits
UI: "gentle-shell exited"
UI on every further send: "cannot reach gentle-shell: ... (os error 232)"
```

The same change over RPC `switch_session` alone was tried 3 times and did not reproduce: the
skill-registry timer did not fire in those runs. Unhandled rejection ends Node, so a single stale
`notify` is enough; the UI had no recovery path except restarting the app.
