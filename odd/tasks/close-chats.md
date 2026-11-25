# Close the chats from Updates: one button where the gate is

The update terminals need every chat closed (the shell's own files are replaced under a running child),
and the pane says so with a sentence that sends the user to each chat row's menu. Meanwhile the new
extensions notice needs an OPEN chat to exist at all, so a user who sees "1 extension has an update"
has to hunt for the way to close what made the notice possible.

## Specs

- S1 The gate sentence ("Las actualizaciones necesitan todos los chats cerrados. N chats abiertos...")
  gets its own button "Cerrar los chats" / "Close the chats", in the pane, next to that sentence. It
  appears only while at least one chat is open.
- S2 It closes EVERY open chat in one step (a new `CloseAllChats` intent handled where `CloseChat` is
  handled today, so the count, the stopped children and the foreground follow the existing close
  rules - including the phantom/never-started chats, and a closed chat stays closed). Stored sessions
  are untouched: closing a chat never deletes its session.
- S3 (REVISED, L2) A chat that is WORKING (a turn in flight, queued input, or waiting for the user's
  answer) is NOT closed: `CloseChat` already refuses it (`close_chat` -> `close_working`, the user must stop
  the turn first) and `CloseAllChats` keeps that guard instead of bypassing it. The button closes every
  chat that is NOT working, silently skipping the working ones (no per-chat notices), and the pane then
  says how many are still working and why they stay: "N chats siguen trabajando: detenlos y vuelve a
  intentarlo." / "N chats are still working: stop them and try again." No confirmation dialog is needed
  because nothing that is running is ever stopped by this button.
- S4 After the chats close, the sentence and the button disappear and the three update buttons are
  enabled by the existing rule (open chats == 0): nothing else about the gate changes.
- S5 One button, no timer, no auto-close, and nothing here runs an update.

## Tasks

- T1 (S2) Rust: the `CloseAllChats` intent through `crates/app` (reducer emits one close per open chat
  through the same path `CloseChat` uses) and the contract/bindings. Route: worker.
- T2 (S1, S3, S4) Web: the button, the working-chats confirmation, both languages. Route: worker, after T1.

## Log

- L1 (2026-11-25, the maintainer, with the extensions notice working): "ahora si funciona, solo hay que
  mejorar lo de cerrar los chats: ... Las actualizaciones necesitan todos los chats cerrados. 1 chat está
  abierto - Cerrar chat en el menú de su fila y los botones de actualización se activan." Asked which
  improvement; chose: "Un botón 'Cerrar los chats' en ese aviso (recomendado)" with confirmation when a
  chat is working.
- L2 (2026-11-25, parent, from the worker's map): the app's existing rule (`close_chat` refuses a working chat) conflicts with the first S3 (confirm and close it). Decision: keep the guard; the button closes the idle chats only and the pane names the working ones. This is MORE conservative than the maintainer's choice, not less; a 'stop and close' action would be its own feature. Also approved: `apps/desktop/src-tauri/src/bridge.rs`, limited to `AppStateHolder::apply`'s closing bookkeeping (the post-fold `forget_transcript` for every chat that left the open set), so a chat closed in bulk reopens exactly like one closed alone.
