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
- R5: the rose of gentle-shell-gpui is NOT reproduced: its asset is not on this machine
  (not under `C:\Users\josep\dev`). The artboards use the app's own rose icon with a
  dot veil as a stand-in. Needs the repo path to read how it is animated.
- R6: any animation stops under `prefers-reduced-motion`, and (build rule) pauses when
  the window is unfocused and after the first send, because low-end machines are a
  standing concern (open-latency and stream measurements in `tools/perf`).

## What is already in place

- A new chat in a workspace names that workspace's folder from the start
  (`composer-dock.md` L12); the branch is re-asked on every open.
- One composer, one dock: B must MOVE the existing composer, never add a second.
- The session list already carries `cwd`, `modified_ms`, `message_count`.

## Tasks

- [ ] T1 (S1, S3, R3, R4) Rust: derive the last workspace and its recent chats for the
  view from the session list; handle no sessions, a missing folder, and a workspace
  with no chats. A new `ViewState` field, bindings regenerated.
- [ ] T2 (S2, S3, R1) web: view A replaces "Ningún chat abierto" (rose, title, the last
  workspace, one primary action, recents, other workspaces as quiet links).
- [ ] T3 (S1, S3, R1) web: view B, the centred layout for a chat with no messages and
  the move of the existing composer to the bottom on the first send (about 240 ms,
  instant under reduced motion).
- [ ] T4 (R2) runtime: start the chat's process on focus or first keystroke instead of
  on open. The riskiest task: it touches the open path of every new chat.
- [ ] T5 (S4, R5, R6) the rose animation. BLOCKED on the gentle-shell-gpui asset.

## Open questions

- Where is the gentle-shell-gpui repo (for the rose)?
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
