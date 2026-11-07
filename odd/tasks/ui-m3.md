# ui-m3: the iced UI crate (`gentle-desk` binary)

First real UI on top of `gentle-app` (app-m2) with the framework chosen in ui-spike.

## Specs

- S1: "UI 100% nativa en Rust" with iced (ui-spike L4: "elijo iced").
- S2: "ui (pinta estado, emite intenciones, sin I/O)" — the view renders `AppState` and emits `Intent`s; decisions stay in `gentle-app`.
- S3: "de las opciones que me diste vamos con la 1, el boton de copiar" — a Copy button per finished assistant message copies the raw markdown.
- S4: Model errors must be "visible, no como un mensaje normal" (core-m1 S5): shown as a red error box in the thread.
- S5: Closing never leaves orphans (core-m1 S3): the window close request shuts the child down before exiting.
- S6: Reference-app bugs not to repeat: chat list loaded once / "New chat" no-op (a New chat button emits `NewChat`, always effective).

## Tasks

- [x] T1 (S5) `RpcProcess::take_events` so a subscription can own the output stream — RED not observed (test and edit ran in parallel); 19 backend tests green
- [x] T2 (S1) `ThreadCache`: incremental markdown per message (push only the new tail, reparse on non-extension text, drop on shorter thread) — 5 tests
- [x] T3 (S1-S6) `gentle-ui` binary: thread, markdown, Copy, notices with dismiss, dialogs (confirm/select/input/editor), error box, composer with Send/Stop, New chat, autofocus, graceful close — verified by hand against the real gentle-shell

## Verification (real gentle-shell 4.6.0, Windows)

- Typed a prompt in the window: user bubble, streamed markdown reply with heading, list and highlighted code block, Copy button, notice from `notify` with dismiss.
- Copy put the raw markdown on the clipboard (checked with Get-Clipboard).
- Closing the window via `CloseMainWindow` exited the app and left no `gentle-shell` node process.
- 105 tests green across the workspace, clippy -D warnings and fmt clean.
- Not verified: model-error box and dialogs in the GUI (covered by unit tests of the app layer only), Stop button mid-stream, long-thread performance, selection (known iced limit).

## Log

- L1: "antes de que continues, explicame que significa 'es pura (sin I/O ni UI)'" then "entendido, ahora si continua" (continue with the iced UI crate).
- L2: Incident: my first GUI automation used `AppActivate("gentle-desk")`, which can match the terminal tab titled "π - gentle-desk" instead of the app; keystrokes may have gone to the user's terminal. Replaced by a script that raises the exact window handle and aborts without typing unless that window is the foreground one.
