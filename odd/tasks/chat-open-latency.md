# chat-open-latency: the chat reads before the child answers

The maintainer noticed that the other desktop app opens chats fast and asked for
the same without losing performance. The measurement says the cost is not the
renderer: this app spawns one `gentle-shell --mode rpc` per chat and waits for it
before anything is drawn, and that wait is the perceived slowness.

Measured in `process-per-chat-m18.md:238-248`: first stdout 3222 ms cold and
2231-2889 ms warm; `get_state` warm median **3036 ms**, max 3557 ms, cold
3824 ms; a new chat 2641 ms; child RSS ~64 MB. By comparison the session list is
already solved by its cache: 500x130 KB cold in 363 ms and a warm refresh of
3-47 ms (`sessions-cache-m10.md:8`).

What the other app does NOT do: it also spawns per tab, lazily
(`start_now=false` plus an idempotent `ensure_started`), and its "instant" comes
from switching conversations INSIDE a live tab over an RPC `switch_session`, not
from a cheaper spawn. That mechanism was removed here on purpose - see S3.

## Specs (verbatim)

- S1 (maintainer, 2026-11-18): "aparte los chats abren rapido" - the perceived
  target is a chat that shows its content when it opens, not 3 s later.
- S2 (maintainer, 2026-11-18): "sin perder la optimizacion", and S4 of
  `chat-transcript.md`: "lo que funciona dejarlo asi". No architecture rewrite, no
  second source of truth that can disagree with the child for long.
- S3 (recorded decision, `process-per-chat-m18.md`): `switch_session`/`new_session`
  are never sent again - reproduced twice in the real window, and gentle-pi dies
  with "This extension ctx is stale after session replacement or reload". This is
  the reason the other app's trick is not available to us, and no task here may
  reintroduce it.
- S4 (channel spec, `crates/gentle-core/src/chat/history.rs:17`): the transcript
  currently comes from `get_messages` on the child, so any pre-paint must be
  reconciled with that answer and lose to it when they differ.

## Tasks

- [x] T1 (S1, S4) paint the session file first: the scanner and the reparsers that
  already read a session file for the tree feed the thread's first frame, and the
  `get_messages` answer replaces it when the child answers. The child's answer is
  authoritative; a partial or stale file must never win.
- [x] T2 (S1, S2) hold the "opening" state until the first event instead of
  drawing an empty thread and then repainting it.
- [ ] T3 (S1, S2) MEASURE, and only then decide: if a chat still takes more than
  ~1.5 s to show content, evaluate a warm standby child. That costs ~64 MB per
  idle child against a 6-live-child cap, so it is a maintainer decision with a
  measurement in hand, never a default.

## Log

- L1 (2026-11-18, the maintainer's words, verbatim): "si ya la estoy cerrando,
  solo que viendo que hicieron una app de escritorio tan bien, quisiera usarla para
  poder copiar a la mia sin perder el rendimiento y dejar de seguir usando la
  terminal porque en la app que crearon del repo que te envie yo la probe y no se
  ve que se abre una terminal, aparte los chats abren rapido".
- L2 (2026-11-18, evidence read): the numbers above come from the recorded
  measurements, and the other app's lazy spawn plus `switch_session` behaviour was
  read from its source, not assumed.
- L3 (2026-11-18, ruling): T1 and T2 are separable and neither needs the other's
  review; T3 is explicitly "measure first" because its cost lands on the low-end
  machines this app exists for.
- L4 (2026-11-19, T1+T2 landed): a stored chat paints its rows from its own session file
  while the child starts (`messages_from_session_file`, shell-owned provisional rows keyed
  by an incarnation epoch compared under the install lock, 8 MiB cap, partial trailing line
  ignored). The child's `get_messages` answer wins: the reducer now records
  `history_loaded` and the draw rule needs foreground + non-empty + no reducer row + history
  not loaded. File rows carry no Copy; the composer still refuses to send while Opening.
  Verified: `npm run check` 0 errors, web 354, gentle-core 340, gentle-app 241,
  gentle-desk-app 597. NOT measured: whether a real chat now shows content in under ~1.5 s
  (T3 is still open and needs a harness against the real shell). Known: the file is read
  whole into memory before the cap refuses it (the shared FileRead seam returns a String).
