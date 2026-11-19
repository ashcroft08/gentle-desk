# roadmap 0.7.0: the chat, the air, and free profiles

The version plan agreed with the maintainer on 2026-11-18, after 0.6.0 closed the
profiles editor. It orders the work by **unblocking**, not by urgency: the slices
that need no product decision go first while the ones that need one are prepared.

Every slice is one branch and one pull request, about 400 authored lines, with its
tests and docs in the same commit, reviewed natively before `main` (which is
protected) and checked in a real window by the maintainer.

## 0.7.0 - the chat and the air

| # | Slice | Document | Why here |
| --- | --- | --- | --- |
| 0 | S0 density defects: bound the Settings body, delete the dead `StatusLine` | `ui-density.md` T1, T2 | real bugs, no product decision, unblock nothing else but cheap and independent |
| 1 | S1 transcript items in the core (tool calls, thinking) | `chat-transcript.md` T1 | the data is already on the wire and discarded; nothing else in the chat can be drawn before this |
| 2 | S2 the tool-call card, under the density rules | `chat-transcript.md` T2 | the visible half of the maintainer's first request; measured on a 400-tool-call chat |
| 3 | S3 the reasoning row | `chat-transcript.md` T3 | one row shape, its own review |
| 4 | S4 paint the transcript before the child answers | `chat-open-latency.md` T1, T2 | the perceived speed the maintainer asked for, without touching the one-child-per-chat decision |
| 5 | P1 `agent.model` + `agent.thinking` pickers and pi's `invalid_data` message verbatim | `profiles-editor.md` T4 (widened) | the largest remaining capability gap; the model catalog already exists, the write path already works |

Exit criteria: a chat with many tool calls reads like the other app's, a chat shows
content as it opens, and the profile panel can route an agent to another model,
with the owner's own rejection shown when it refuses.

## 0.8.0 - edit freely and let the app breathe

| # | Slice | Document | Why after 0.7.0 |
| --- | --- | --- | --- |
| 6 | P2 duplicate a profile and rename it | `profiles-editor.md` (new T) | duplicating an already-accepted document is how a new profile is created without inventing the owner's safe-key policy |
| 7 | RED the profile panel redesign | `ui-density.md` T3 | the form is redesigned once the content (pickers, duplicate) is final |
| 8 | RED the Settings redesign | `ui-density.md` T4 | same reason; the Usage pane is the worst offender and gets progressive disclosure |
| 9 | LOGIN a pi login helper driven in-app | (new document) | removes the last two terminal hand-offs a user meets on purpose; it spawns pi's login helper and drives `AuthUrl`/`DeviceCode`/`Prompt`, so it needs its own design |

## After that, with triggers

- **MED** warm standby child: only if S4's measurement still shows more than ~1.5 s
  to content. Cost: ~64 MB per idle child against a 6-live cap.
- **UPSTREAM** an issue to `Gentleman-Programming/gentle-shell` asking for update
  status and apply over the desktop channel, so "Update" and "Update extensions"
  stop opening a terminal. Nothing in this repo exposes them today.
- **PROBE** the owner's real safe-key policy, `thinking` vocabulary and `tier`
  meaning: optional. It would let the UI promise `tier` editing; until then `tier`
  is displayed and never written.

## Never, or not without new evidence

- Migrating to a GPU-only stack: the other app requires Vulkan-class drivers and
  documents no software fallback, and this app exists to run on any laptop.
- `switch_session`/`new_session`: removed in m18 after reproducing a gentle-pi
  death; it only returns if upstream fixes the stale extension context.
- A control child (~400 MB) while editing profiles: the chat's own child writes.
- An in-app installer for usage sources: the maintainer's rule is that an install
  is explicit and never silent.
- Editing `tier`: its meaning is not documented by the owner, and a field we do not
  understand does not get an editor.

## Log

- L1 (2026-11-18, maintainer): "con tu recomendacion la b, pero dime como mismo
  vamos a empezar porque no te olvides que tambien esta lo del chat. osea ahora si
  organicemos como mismo va a quedar todo el roadmap".
- L2 (2026-11-18, maintainer): "ya se lanzo la 0.6.0, podes empezar". Work starts
  from `origin/main` = 54ae6d6, tag v0.6.0.
- L3 (2026-11-18, maintainer rulings that pin this plan): "dejemos el boton "Edit
  in terminal" ... eso es darle dos metodos al usuario por interfaz o terminal";
  update and usage sources stay on the terminal ("si lo del update/usage sorce
  sigue como estan"); target version 0.7.0.

## Status (2026-11-19, from the commits on `feat/0.7.0-transcript`, PR 208)

- Done and CI-green: slices 0-3 (density, transcript items, tool card, reasoning row), 4
  (the session file paints while the child starts; T3 measured with the FAKE shell only,
  `chat-open-latency.md` L5), and the unplanned `composer-dock.md` T1-T4 (status row,
  folder and branch chips, the session mode as a changeable control, file attachments).
- NOT done: slice 5 (P1 pickers `agent.model` / `agent.thinking`), and the real-shell
  open measurement that decides the warm-standby question.
- No review receipt for the whole range: it exceeds the reviewer's context budget; the
  parts reviewed are recorded in the per-feature logs.
- The release itself (version 0.7.0, notes, tag) is the maintainer's gate.
