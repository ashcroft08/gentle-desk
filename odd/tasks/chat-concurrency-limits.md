# chat-concurrency-limits: never abort a working chat, and coherent caps for live/working children

Fixes the observed data loss where opening another chat while one was mid-turn killed the running
call (`"Operation aborted"`) and the chat lost its `working` mark. Root cause: the host owns the
protection rules (`ChatView::protected()` = foreground | working | pending) but production never
feeds them — `set_working` has no call site outside tests — so the LRU victim at the live cap can
be a chat in the middle of a model call. The two caps also live as two unconnected constants
(host `Limits::default()` 4/2 and the reducer's `DEFAULT_MAX_WORKING` 2) that agree only by
accident, because `Input::SetWorkingCap` is never dispatched.

Caps decided here: **6 live children, 3 simultaneously working, 15 min idle** (up from 4/2/10min).

## Specs

- S1: A chat that is running a turn is never stopped to make room when another chat opens or when the user switches chats. In-flight work is never aborted by navigation. The observed failure must not reproduce: the user reported «lo que habia pasado al cambiar de chat es que se habia abortado "Operation aborted"», and «regreso y algo habia pasado porque no aparecia working».
- S2: The host is told which chats are working, so `protected()` actually protects them: a chat marked working is never the LRU victim of `admit()`, never reaped by `reap_idle()`, and `TooManyLive` refuses only when every live child is foreground, working or still starting.
- S3: One source of truth for the caps, values **6 live children**, **3 simultaneously working chats**, **15 minutes** idle. The app reducer's cap and the host's `Limits` cannot drift apart: the reducer cap is dispatched from the host's limits at boot (`Input::SetWorkingCap`, `0` = no cap) instead of the hardcoded `DEFAULT_MAX_WORKING`.
- S4: Over the working cap a prompt is queued, not refused and not lost (existing behavior, kept).
- S5: Idle children are actively reaped after the idle time, by a driver that actually runs in production, so live children never accumulate to the cap unnoticed. A protected chat is never reaped.
- S6: Preserved guarantees, pinned by a regression test each: opening a chat stops no other child (`crates/app/src/chats/store.rs:6`), closing a working chat is refused (`crates/app/src/chats/intents.rs:152-156`), and a background chat that finishes while unfocused keeps its reply and is marked unread.

## Tasks

- [x] T1 (S3) backend/app: one `Limits` source of truth with 6 live / 3 working / 15 min, and boot dispatch of `Input::SetWorkingCap` from the host's `max_working` so the reducer stops using the `DEFAULT_MAX_WORKING` constant as its authority. Route: parent; commit: 7004b03
- [x] T2 (S1, S2) shell→host: feed the chat's state to the host when it starts and finishes working, through the existing command seam, so a mid-turn chat is protected from eviction and the working cap is real. Route: parent; commit: a424a98
- [x] T3 (S5) shell: drive `reap_idle` from production so idle children are actually released under the 15 min window. Route: parent; commit: bce7cbb
- [x] T4 (S1-S6) checks: policy/caps/reducer tests updated for 6/3/15, plus the regression tests of S1, S2 and S6, including the reachable path where opening a chat at the live cap previously chose a working chat as victim. Route: parent; commit: 5b54a49 (its navigation slice; the cap and mark tests ride with T1 and T2)
- [ ] T5 (S1-S6) parent verifies with `tools/fake-gentle-shell` through `GENTLE_SHELL_BIN` (never the real model): a chat mid-turn survives another chat opening and switching back, 3 chats work in parallel and a 4th queues, an idle child is reaped after the window, `cargo fmt --check` and `cargo clippy` clean. Route: parent; commit: none - verification has no source of its own; the mechanical half is recorded in L10 and the window half is still pending

## Log

- L1: «a ver te explico lo que me paso estaba trabajando en algo y mientrar la ia trabaja, abri otro chat para probar el otro chat era uno antoguo asi que emoro en abrir hasta que se cargo y luego volvi al chat donde se suponia que estaba trabajando pero regreso y algo habia pasado porque no aparecia working... asi que entre en la terminal y lo que habia pasado al cambiar de chat es que se habia abortado "Operation aborted". Entonces en la teoria dice que si se puede maximo dos o se refiere maximo 2 a la misma carpeta?»
- L2: «si y tambien extablescamos unos valores coherentes para hijos vivos y trabajando porque siento que es poco eso»
- L3 (evidence, diagnosis): the caps are global per app instance, not per folder: `DEFAULT_MAX_WORKING = 2` (`crates/app/src/chats/queue.rs:17`) and host `Limits::new(4, 2, 10min)` (`crates/backend/src/chat_host/policy.rs:45`). The protection rules exist and are tested at policy level, but `Slot.working` (`crates/backend/src/chat_host/state.rs:96`) is never set in production: `set_working`/`may_start_work`/`reap_idle`/`working_count` have no call site in `apps/` (`apps/desktop/src-tauri/src/effects/runtime.rs` executes only `ChatCommand`s). `ChatView::protected()` still covers `foreground`/`pending` (`open_in` sets them), which is why the abort needed the live cap at 4 reached, exactly as the user observed after opening an old chat and returning.
- L4 (decision): caps 6 live / 3 working / 15 min idle, chosen by the user from three options (moderate tier).
- L5 (decision): single source of truth, so `max_working` is dispatched from the host at boot rather than kept as a second constant.
- L6 (implementation): the caps now live in `crates/gentle-core/src/chat/limits.rs` (`MAX_LIVE`, `MAX_WORKING`, `IDLE_AFTER`), read by `chat_host::Limits::default()` and by `chats::queue::DEFAULT_MAX_WORKING`. `AppStateHolder::with_config` folds `Input::SetWorkingCap(Limits::default().max_working)` at boot.
- L7 (implementation): `ChatCommand::SetWorking { chat, working }` added, emitted by `Chats::apply` only on a real transition, executed by `ChatRuntime::execute_in_language` → `ChatRuntime::set_working`, answered as `Outcome::Working` (folded to no input: the reducer already owns the mark).
- L8 (implementation): the reaper is `apps/desktop/src-tauri/src/reaper.rs`, started by `Bridge::start_reaper` in the setup step, one tick a minute, ending by itself when the run is closed.
- L9 (checks): new `crates/app/tests/chat_host_mark.rs` (7 tests: the mark on start/end/settled, no mark on a no-op event, and the S6 guarantees that opening, switching back and closing never stop a working child), new `crates/runtime/src/chat_runtime/tests/mark.rs` (3 tests: at the live cap a chat marked through the command seam survives and the same chat becomes the victim once cleared), the app-side and backend-side guard tests that both caps read the shared constant, and `chat_working_cap` updated to 3 with a test that the third prompt is sent and not queued.
- L10 (checks, verified): `cargo test --workspace` green with zero failures (17m 14s, almost all of it compiling the Tauri test binaries), `cargo fmt --all` applied and `--check` clean, `cargo clippy --workspace --all-targets` clean with `PIPESTATUS[0]` = 0 (the earlier `exit 0` was the pipeline's `echo`, not clippy's: a grep in the pipeline masks the real status, so the status is read from `PIPESTATUS[0]` from now on).
- L11 (process correction, from the user): two background jobs were polled with `sleep` instead of waiting for the completion notice, so a finished job went unrecognized. The completion event is the signal; poll never.
- L12 (pending): T5's real-window check is not done. It needs the app open with `GENTLE_SHELL_BIN` pointing at `tools/fake-gentle-shell`: a chat mid-turn surviving another chat's open and a switch back, three chats working with a fourth queued, and an idle child reaped after the window. Committed in the `feat/chat-concurrency-limits` worktree (`C:/Users/josep/dev/gentle-desk-chat-concurrency`), five commits, each one checked to compile alone.
