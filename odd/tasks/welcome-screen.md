# welcome-screen: what the app shows when no chat is open, and what a new chat looks like

The empty state ("Ningún chat abierto") feels bare, and the maintainer wants a
welcome screen at launch plus a centred "new session" view for every new chat.
Designed in `design/welcome-screen/` (artboards A and B approved; C not chosen).
NOTHING IS IMPLEMENTED YET: this document is the record of what was decided and
what is left, to be taken after the 0.7.0 release.

## Specs (verbatim)

- S1: "quiero que cuando se abra la app se mustre un chat nuevo pero que sea con el
  ultimo workspace que el usuario estuvo trabajando quiero que el chat se muestre en el
  centro y cuando se envie el mensaje pase a la posicion de abajo"
- S2: "si al abrir la app y que se muestre un chat nos cuesta rendimiento o es
  complicado, quiero que trabajemos en un diseño para esa pantalla que sea de
  bienvenida o algo bonito porque se ve simplona y vacia cuando se abre"
- S3: "para cuando se abra la app me gusta A · Sigue donde lo dejaste. y si tengo la
  app abierta estoy trabajando en un chat y abro un chat nuevo me gusta el diseño de B
  · El composer al centro, como una sesión nueva para los chat nuevos o sesiones nuevas"
- S4: "estaba pensando en este logo con una animacio que se muestra esa animacion usa
  gentle shell desktop gpui" (a dot-matrix rose that moves)

## Rulings taken from the conversation

- R1: A is the view whenever NO chat is open (app start, nothing in the foreground).
  B is the view of ANY new chat or session, i.e. a foreground chat with no messages.
  A's primary action ("Nueva sesión en <workspace>") lands in B.
- R2: opening the app starts NO child process. A is pure view. B's composer is a draft
  and the chat's process starts when the box is focused or the first key is typed,
  never on open. (Today a new chat spawns a `gentle-shell` at once - ~64 MB, several
  seconds - and a prompt is refused while a chat is Opening, `crates/app/src/chats/
  intents.rs:230-233`, so lazy start needs either spawn-on-focus or a queued prompt.)
- R3: "the last workspace" is the workspace of the most recent session in the list
  (`SessionItem.cwd` + `modified_ms`): no new config field. A workspace with no chats
  says so and keeps only the primary action; a folder that no longer exists disables
  the primary action and shows the reason the sidebar already uses ("Esta carpeta ya no
  existe."), no other action is invented (artboard A2).
- R4: no invented data. Recents and counts come from the session list only.
- R5: the rose IS available and it is not an image: gentle-shell-gpui draws it from
  braille art (`ROSE`, 17 rows, each character is 2x4 dots) plus a 0-9 shade map
  (`ROSE_SHADE`), in `crates/app/src/shared/banner.rs` (read remotely, the repo was NOT
  cloned). 1,779 dots. Its animation is procedural: it blooms from its heart outward
  over 2.6 s, floats and sways while it does, a glint crosses the petals, and it comes
  to REST (a finished rose asks for no more frames). The workspace declares MIT
  (`license = "MIT"` in its Cargo.toml; no top-level LICENSE file was seen), the same
  licence as this app, so it is ported with an attribution comment. The welcome
  artboards already render it with a faithful canvas port.
- R6: any animation stops under `prefers-reduced-motion`, and (build rule) pauses when
  the window is unfocused and after the first send, because low-end machines are a
  standing concern (open-latency and stream measurements in `tools/perf`).

## What is already in place

- A new chat in a workspace names that workspace's folder from the start
  (`composer-dock.md` L12); the branch is re-asked on every open.
- One composer, one dock: B must MOVE the existing composer, never add a second.
- The session list already carries `cwd`, `modified_ms`, `message_count`.

## Tasks

- [x] T1 (S1, S3, R3, R4) Rust: derive the last workspace and its recent chats for the
  view from the session list; handle no sessions, a missing folder, and a workspace
  with no chats. A new `ViewState` field, bindings regenerated.
- [x] T2 (S2, S3, R1) web: view A replaces "Ningún chat abierto" (rose, title, the last
  workspace, one primary action, recents, other workspaces as quiet links).
- [x] T3 (S1, S3, R1) web: view B, the centred layout for a chat with no messages and
  the move of the existing composer to the bottom on the first send (about 240 ms,
  instant under reduced motion).
- [ ] T4 (R2) runtime: start the chat's process on focus or first keystroke instead of
  on open. The riskiest task: it touches the open path of every new chat.
- [x] T5 (S4, R5, R6) the rose: port `ROSE`/`ROSE_SHADE` and the bloom, float, sway and glint
  to a canvas component (the artboard script is the reference), still once it has
  bloomed, drawn open under reduced motion. Unblocked. Optional later: the same banner
  also writes the "Gentle Shell" wordmark with a pen (`pen_times`); the sidebar draws that
  wordmark static today.

## Open questions

- Does B's first version show a workspace picker (a popover, like the reference) or just
  the folder as text? Default proposed: text only, picker later.
- In B, are the context rail and the mode chip drawn before the first message? Default
  proposed: the mode chip yes (it is state), the rail no (0% says nothing).

## Log

- L1 (2026-11-19, the maintainer's words, S1-S4 above in full). He then asked to be told
  whether opening a chat at launch costs performance; answer: yes, one child process per
  chat, so launch gets view A and only a NEW session starts one (R2).
- L2 (2026-11-19, design): three directions in `design/welcome-screen/` (A resume, B
  composer in the centre, C start panel). The maintainer chose A for launch and B for
  new sessions (S3). Release order agreed earlier: 0.7.0 first, this after it.
- L3 (2026-11-19, the rose found): the maintainer gave the repo URL and asked NOT to
  download it. Read remotely, file by file: `features/welcome/{mod,finale,view}.rs` is
  gentle-shell-gpui's FIRST-RUN onboarding (the banner writes itself in, the rose blows
  kisses) - a different screen from ours - and the rose lives in `shared/banner.rs`. The
  three artboards now use a canvas port of it (verified: the script parses, the data
  decodes to 1,779 dots; NOT seen rendered by me - the canvas is for the maintainer to look at).

- L4 (2026-11-20, T1-T5 landed, all on `feat/0.8.0-welcome`): T1/T1b/T1c (`96380e5`,
  `7c184e0`, `50992cf`), T2/T2b (`3e22794`, `3ba3fa4`), T3 (`052853f`, plus the giant-card fix
  `4a831d4` and the redundant folder line `a888afe`), T5 (`f915997`), the sidebar fallback
  (`526db7b`), the `Inicio` header (`3f1acb3`), the window minimum 800x560 (`b36526c`) and T4
  (`736cfc4`). T4 landed as option 1 (the maintainer's choice): `OpenNewChat` allocates the chat
  and marks Opening but spawns nothing; `EnsureStarted` (dispatched from the composer's focus and
  its first keystroke, once per fresh chat) runs the same `OpenProcess` path, so the opening net,
  request_after_open and the folder hints are unchanged. While the process has not answered the
  composer stays disabled with today's reason - a first Enter sends nothing and the draft stays.
  Verified: `gentle-app` 246, the seven shell binaries green, clippy 0 warnings, fmt clean, web
  chat/App 127. Behaviour NOT yet seen in a running window: the process really waiting for the
  focus, and the view B composer travelling to the bottom on the first send.
- L5 (2026-11-20, Configuration work started from gentle-shell-gpui's dialog, MIT, read from a
  clone at `C:/Users/josep/dev/gentle-shell-gpui`): the search field over the panes and the theme
  library as preview cards (no app instance per theme) landed in `47864f3`, retiring the old
  `sidebar/ThemePicker`. Its own way of working that we still owe: ONE snapshot loaded off-thread
  per dialog open (their `settings/services/snapshot.rs`), which is the real performance fix, and
  the setup probe behind their General page. Notifications with sound: their `features/alerts`
  synthesises WAVs and plays them with the OS player; ours would be a WebAudio tone plus the
  official Tauri notification plugin, and the real work is the policy.
