# chat-event-commands: chat commands asked for by a child event are actually run

The context chip shows a stale number while a chat works and only refreshes after the chat is
closed and reopened. The cause is structural, not in the chip: a child event is folded by the
event pump, and the pump discards the `effects.chat_commands` that fold returns
(`apps/desktop/src-tauri/src/events.rs:224` → `bridge.rs:189 fold_input` → `bridge.rs:189,292`).
Only the intent path runs chat commands (`commands.rs:551,579-600`). Reopening refreshes because
opening is a different fold (`dispatch_commands`) whose commands *are* run.

The same drop silently defeats two other behaviours shipped in `chat-concurrency-limits` and one
latent one:

- `ChatCommand::SetWorking` (the working mark that keeps a mid-turn chat from being evicted) never
  reaches the host, because it is emitted from the same child-event fold.
- `queue::release`'s prompt sends (a queued prompt going out when a slot frees) are dropped too.
- `RpcCommand::GetSessionStats` on `agent_settled`, which is this report.

## Specs

- S1: A chat command the reducer asks for while folding a child event is executed. The user's
  report: «esta barra no se actualizo mientras trabajo, debo cerrar y volvr abrir el chat para que
  se actualice». Concretely: the context chip updates from the settled turn's own
  `get_session_stats` reply, with no close and reopen.
- S2: The same path carries the other commands of that fold: `SetWorking` reaches the host, and a
  queued prompt released by a freed slot is sent.
- S3: The command runs off the fold thread. The pump thread must never wait on a child's pipe: it is
  the thread that keeps delivering every other chat's events.
- S4: Nothing runs twice. The intent path keeps executing its own chat commands exactly as today.
- S5: The drop is pinned by a test: a child event folded through the pump seam with a recording
  executor proves the command was run, so it cannot regress silently again.

## Tasks

- [x] T1 (S1-S4) shell: the pump hands the fold's chat commands to the same runner the intent path already uses, off the fold thread. Route: gentle-ai-worker; commit: 0a0da36
- [x] T2 (S5) shell: a test through the pump seam with a recording executor proving a settled turn's `get_session_stats` is run, plus the full suite, `fmt` and `clippy`. Route: gentle-ai-worker; commit: cd6417b
- [x] T3 (S1) parent verifies with `tools/fake-gentle-shell` through `GENTLE_SHELL_BIN`: the chip moves after a turn without reopening. Route: parent; commit: none - verified by the user in the real app (see L12)

## Log

- L1: «esta barra no se actualizo mientras trabajo, debo cerrar y volvr abrir el chat para que se actualice» (with a screenshot of the context chip reading 4%).
- L2 (diagnosis, evidence): main worktree `C:/Users/josep/dev/gentle-desk`. `fold_input` = `apply(input).0` (`bridge.rs:188-190`) and `apply` returns `chat_commands` without running them (`bridge.rs:292`); the pump loop is `push(holder.fold_input(input))` (`events.rs:224`) on the thread named `chat-events` (`events.rs:215`). The intent path runs them on a spawned `chat-command` thread (`commands.rs:579-600`). `run_shared` runs only `effects.commands` (`bridge.rs:317-324`), which is why the session list refreshes from the same fold while the chip does not.
- L3 (diagnosis, scope): the drop also defeats `ChatCommand::SetWorking` from commit a424a98 of `chat-concurrency-limits`, so that fix is inert in production despite its unit tests passing: they pin the reducer and the runtime seam, not the delivery.
- L4 (decision, from the evidence): fix the delivery once. It is one seam and three symptoms, and the concurrency fix is incomplete without it.
- L5 (constraint): `ChatHost::send` writes to the child's stdin with no timeout (`crates/backend/src/chat_host/host/routing.rs:21-50`, `crates/backend/src/process/writer.rs:29-37`), so running it on the fold thread could stall every chat's events. The run must go off that thread (S3), exactly as the intent path already does.

- L6 (implementation, T1): delivery is a property of the fold entry point, not of the command list. `AppStateHolder::fold_event` answers the slice plus the commands, `deliver_chat_commands` hands them to a `ChatCommandSink` armed in `arm_events`; `fold_input` returns nothing and delivers nothing, and `dispatch_commands` still returns its own commands to the intent path. No single fold can both answer and deliver, so double execution would need one input folded by two entry points. The sink funnels the one runner `commands::run_chat_commands`, which spawns a `chat-command` thread per command: the pump thread only starts threads and never touches a child's pipe (S3).
- L7 (RED, verified by the parent): with the pump reverted to `push(holder.fold_input(input_of(converted)))`, `a_settled_child_event_asks_its_own_child_for_the_context_reading` fails with `left: []` / `right: [Send { chat, rpc: GetSessionStats }]`; restored, both pump-seam tests pass. The regression is pinned, not just described.
- L8 (known gap, out of this feature): `fold_and_push` folds an executor's outcome through `fold_input`, so a command produced by an OUTCOME fold is still dropped - concretely a `queue::release` prompt carried by the `ChatClosed` outcome of a close, a workspace removal or `close_first`. Fixing it changes intent-path behaviour, so it needs its own decision rather than riding this one.
- L9 (scope note): the pump-seam tests assert the delivered command list with a recording sink rather than running the real executor, because the real runner needs a live child pipe that a windowless seam must not create. The recording sink is exactly what `arm_chat_commands` installs in production.

- L10 (checks, verified by the parent): `cargo fmt --all --check` clean, `cargo clippy --workspace --all-targets` exit 0, `cargo test --workspace` exit 0 with 111 test binaries ok, run with the worktree's own `target/`. The first suite run failed with `LNK1318` PDB errors because it shared the sibling session's `target/`; that was a method error, not a code failure, and the shared `CARGO_TARGET_DIR` is no longer used. The first `fmt` run also caught `chat_no_abort.rs`, created after the earlier `cargo fmt --all`; fixed in c088407.
- L11 (pending): T3 is the user's real-window check, not done: the chip must move after a turn with no close and reopen, and a queued prompt must go out when a slot frees, both against `tools/fake-gentle-shell` through `GENTLE_SHELL_BIN`.

- L12 (window check, reported by the user): the app was run from this branch with `npm run tauri dev` and the real `gentle-shell` (the fake shell does not answer `get_session_stats`, so it cannot exercise the chip). The user's words: «Ahora si funciono sin ningun problema». Evidence from the run log: a 5 minute session, three chats opened, three children `stopped by the host` at close, no panic, no `Operation aborted`, no leftover `gentle-desk-app` process. The only error line is WebView2's harmless `Failed to unregister class Chrome_WidgetWin_0` at exit. Not exercised in that session and so not verified: the 15 minute idle reap, and a fourth prompt queuing behind three working chats.
