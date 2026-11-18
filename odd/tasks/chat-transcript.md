# chat-transcript: the tool calls and the thinking, drawn as they happen

The maintainer compared this app with `Gentleman-Programming/gentle-shell-gpui`
(Rust + GPUI + gpui-kit) and asked to bring its chat to our components. This
document is the first half of that: the *transcript*. It changes what a message
list can hold, not the stack, not the virtualisation and not the theme.

Two facts drive the slicing. First, the data already arrives and is thrown away:
`crates/gentle-core/src/chat/reducer.rs:44-49` folds `tool_execution_start`,
`tool_execution_update` and `tool_execution_end` into an activity counter, and
`:82-87` does the same for `thinking_start`/`thinking_delta`/`thinking_end` and the
toolcall deltas. Second, a test pins that behaviour as the contract:
`crates/gentle-core/tests/chat.rs:651` asserts the transcript "ends with only the
final text". So the core goes first (T1) and the web can only draw what the core
starts keeping (T2, T3).

What is NOT ported, on purpose: the GPUI stack (see S2), the whole-view re-render
that the other app documents as its own cost (`docs/design-system.md:232` there:
"a frame re-renders the whole view and its ancestors (the conversation, during a
turn)"), and any gpui-kit widget.

## Specs (verbatim)

- S1 (maintainer, 2026-11-18): "queria ver si podiamos mostrar el chat como ellos
  lo hacen y el diseño claro adaptado a nuestros componentes y tambien lo demas de
  los perfiles y otras cosas que nos sirvan pero sin perder la optimizacion y
  nuestro diseño propio".
- S2 (maintainer, same message): "han sacado una app de gentle shell desktop hecha
  en "Rust with GPUI and GPUI Kit" esto usa la gpu y mi app es para que cualquier
  persona y laptop pueda usarla". The stack does not move: this app stays
  Tauri + Svelte 5 + WebView2, which degrades without a GPU, while the other app
  requires a Vulkan-class driver and documents no software fallback.
- S3 (the target anatomy, read from the maintainer's screenshots of that app): a
  tool call is ONE card holding an icon, the tool's name, a language chip (for
  example "Bash" with a "bash" chip), the command in monospace, a collapsed output
  body whose first line reads "14 earlier lines", and a footer with a check and
  "Done 0.2s" or a spinner and "Running 4.0s". Reasoning is a SEPARATE collapsible
  row: "Thought 1.2s".
- S4 (maintainer, 2026-11-18): "Lo que me planteas arriba lo veo bien, es mejorar lo
  que tenemos y lo que funciona dejarlo asi". Additive only: the transcript list,
  its virtualisation, the theme tokens and every message shape that already works
  keep working unchanged.
- S5 (maintainer, 2026-11-18): "no te dejes llevar solo porque te lo pedi". Where
  the evidence contradicts the request, the evidence wins and the disagreement is
  recorded here: the other app's re-render cost is not copied, and the card is not
  a gpui-kit port but our own component under our tokens.

## What the shape must respect (from the existing code)

- The transcript is a windowed list of rows with fixed-height measurement; a row
  that grows when opened must not break the measurement of the rows below it.
- The theme tokens live in `apps/desktop/src/theme.css` and `theme/`; the card
  uses those, never a colour of its own.
- Nothing is drawn from a guess: a member the child leaves out is not displayed.

## Tasks

- [x] T1 (S1, S5) core: model transcript items and stop discarding. Add a typed
  item (tool call with name, args, result, `isError`, start/end time and duration;
  thinking with its text and duration) beside the existing text item, fold the
  `tool_execution_*` and `thinking_*` events into it instead of the activity
  counter, and replace `crates/gentle-core/tests/chat.rs:651` with the contract
  that now holds. The web changes in this task: none.
  WIDENED by L5: the decode comes first. `rpc/codec.rs:153-167` builds the three
  `ToolExecution*` events from `toolCallId`, `toolName` and `isError` only, so
  `args`, `partialResult` and `result` never reach the reducer; T1 therefore also
  carries them on `rpc/types.rs:211-222` and decodes them, and updates the three
  readers that slice `ChatState.messages` directly
  (`crates/term/src/render.rs:38`, `apps/desktop/src-tauri/src/view.rs:37-38`,
  `apps/desktop/src-tauri/src/bridge/images.rs:55`).
- [ ] T2 (S1, S3, S4) web: the tool-call card in our components — icon, title,
  language chip, monospace subject, collapsible body with a tail-clipped
  "N earlier lines" head, and the status/duration footer. Measured before it is
  called done: a chat with 400 tool calls must keep the list responsive.
- [ ] T3 (S3, S4) web: the reasoning row, open while streaming and folded when the
  block ends, with its elapsed time.
- [ ] T4 moved to `chat-open-latency.md`: painting the transcript before the child
  answers is its own surface (the open path, not the transcript model) and its own
  measurement.

## Log

- L1 (2026-11-18, the maintainer's request, verbatim): "han sacado una app de
  gentle shell desktop hecha en "Rust with GPUI and GPUI Kit" esto usa la gpu y mi
  app es para que cualquier persona y laptop pueda usarla gentle-desk queria ver si
  podiamos mostrar el chat como ellos lo hacen y el diseño claro adaptado a
  nuestros componentes y tambien lo demas de los perfiles y otras cosas que nos
  sirvan pero sin perder la optimizacion y nuestro diseño propio".
- L2 (2026-11-18, evidence read): the events already reach the reducer and are
  discarded at `reducer.rs:44-49` and `:82-87`; `rpc/types.rs:211-222` and
  `rpc/codec.rs:153-165` carry `tool_execution_start/update/end` with `args`,
  `result` and `isError`; `codec.rs` also decodes the thinking events; the web
  draws three shapes today (`apps/desktop/src/thread/Message.svelte`).
- L3 (2026-11-18, ruling): T1 ships without UI so the contract can be reviewed on
  its own; T2 and T3 are one row shape each, so a review never has to hold both.
- L4 (2026-11-18, T1 shape ruled before writing code): `ChatState.items: Vec<ChatItem>`
  becomes the ONE ordered transcript and the single source of truth, with
  `ChatItem::{Text(ChatMessage), Thinking{..}, Tool{..}}`; `ChatMessage` stays as
  the text variant so its ids (`msg-<n>`) and its tests keep their meaning, while
  thinking and tools get `think-<n>` and `tool-<toolCallId>`. The web is NOT
  touched in T1: `apps/desktop/src-tauri/src/contract.rs` keeps its
  `messages: Vec<MessageView>` field, but built by a DERIVED projection of `items`
  (text variants only) so there is never a second truth - T2 switches the web to
  `items` and deletes that projection. Durations: the reducer already takes
  `now_ms` (today only helper bookkeeping uses it), so a thinking block and a tool
  call record their start and end and expose `duration_ms`, which is `None` when
  the caller supplies no clock (the `prompt-turn.jsonl` fixture carries no
  timestamps; a test that wants the card's "Done 0.2s" passes `now_ms`). Per-item
  drawing notions (icon, language chip, collapsed body) never enter the core: they
  are T2's.
- L5 (2026-11-18, CORRECTION of L2, found by the implementing worker before any
  code was written): L2 is wrong about the codec. `crates/gentle-core/src/rpc/codec.rs:153-167`
  builds `ToolExecutionStart` from `toolCallId` and `toolName`, `ToolExecutionUpdate`
  from the same two, and `ToolExecutionEnd` from those plus `isError` - `args`,
  `partialResult` and `result` are parsed out of the line and DROPPED, and
  `rpc/types.rs:211-222` has no field for them. L2 described the wire JSON, not the
  Rust variants, which is exactly the mistake this document exists to prevent.
  Consequences, ruled here: (a) T1 owns the decode, so its surfaces include
  `crates/gentle-core/src/rpc/` - without it a tool item could carry a command but
  never an output, and the card T2 must draw would be empty where the maintainer's
  screenshot shows "14 earlier lines"; (b) `ChatState.messages` goes away as the
  single-source rule requires, and the three direct readers of that field are
  inside T1's scope (`crates/term/src/render.rs:38` plus its test double,
  `apps/desktop/src-tauri/src/view.rs:37-38`,
  `apps/desktop/src-tauri/src/bridge/images.rs:55`); (c) the web projection is a
  pure method on `ChatState` in `crates/gentle-core/src/chat/`, called from
  `view.rs`, so `MessageView` and `apps/desktop/src/contract/bindings.ts` stay
  byte-identical and the web needs no change and no regeneration.
- L6 (2026-11-18, T1 done): the slice landed as 23 files, 1271 insertions and 251
  deletions, in two commits: the RPC decode, then the one-transcript model with
  its readers. Receipts from `cargo test --workspace`: **126 targets, 1688 passed,
  0 failed**; `cargo check --workspace` clean; `cargo test -p gentle-core` 308
  passed; `cargo test -p gentle-app` 240 passed. The web was not touched and the
  bindings drift guard proves `bindings.ts` is byte-identical.
  REDs seen, in order: `E0559` (the three `ToolExecution*` variants had no
  `args`/`partial_result`/`result` field), then `E0432`/`E0560`/`E0599` (no
  `ChatItem`, no `items`, no `messages()`), then `E0615` at 28 sites when the app
  tests still read the removed field. GREEN after each.
  Decisions this document did not settle, now pinned by tests and doc comments:
  (1) `activity` counts only the events that land in NO row, so the prompt-turn
  fixture's activity is 1 rather than 9; (2) `message_start`'s empty bubble is
  never displaced while empty, so streaming text stays the last row; (3) a tool
  row's timings are the EXECUTION's (`tool_execution_start` -> `end`), and both
  `partial_result` and `result` are kept; (4) `message_end` closes the streaming
  text row wherever it sits, so text said before a tool call stops streaming;
  (5) rows are created only by `toolcall_start`/`tool_execution_start`, and an
  update or end naming an unknown call only counts; (6) history still loads as
  text, converted in one place by `replace_messages`.
  Known untested edge, recorded instead of hidden: a `toolcall_end` with no id
  falls back to the last tool row without args or result - pi always sends the id,
  so no fixture covers it.
- L7 (2026-11-18, native review of T1 and its follow-up): lineage
  `review-9b802d5d79d4e84f`, tier medium, 27 paths, 1637 original lines, correction
  budget 200, **four lenses** (risk, resilience, readability, reliability),
  **APPROVED and its acknowledgement burned**. Eleven non-blocking advisories, and
  four of them were real defects in this model, so all six fixable ones were fixed
  in a follow-up commit rather than left "for later":
  - **R1** (risk, WARNING): a tool row kept every call's full `args`,
    `partial_result` and `result` for the chat's whole life; the old reducer kept
    none. Now `ToolPayload` is the only way a payload enters a row, every string is
    cut at `TOOL_PAYLOAD_MAX_BYTES` = 8 KiB on a character boundary and says it was
    cut, and `partial_result` is dropped the moment `result` arrives. This
    **supersedes L6 decision (3)**: both are no longer kept.
  - **R2-001 + R4-002** (readability + resilience, WARNING): `append_text` only
    appended to the last row, so text arriving after a tool call was dropped
    silently and unaccounted. It now appends to the OPEN assistant row, the same
    lookup `message_end` uses.
  - **R4-001** (resilience, WARNING): the desktop projection cloned the whole
    transcript on every refresh before slicing the pushed window. It now cuts the
    window inside the borrowed `text_items()` iterator: O(pushed) instead of
    O(chat).
  - **R3-002** (reliability, WARNING): an assistant row that the authoritative
    `message_end` left empty survived unless it was trailing, leaking into
    `messages()`. It is now dropped wherever it sits.
  - **R2-002 / R2-003** (readability, SUGGESTION): six `else { return; }` arms that
    could not execute were replaced by one total match per accessor, and
    `start_tool_execution` now makes ONE lookup instead of the same query twice
    around a push.
  Receipts: `cargo test --workspace` **1695 passed, 0 failed** (7 new tests),
  `cargo check --workspace` clean. Two findings honestly had no observable RED and
  are reported as structural instead of dressed up as a failing test.
  OPEN, for T2 to settle: the 8 KiB bound is per retained string, not per row - a
  payload of many long strings is bounded per element but not in aggregate. Also
  accepted with its mitigation: `var(--settings-chrome)` only resolves inside
  `.dlg.settings`, and the declaration-level test pins both sides (R4-004).
