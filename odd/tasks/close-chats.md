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
- S3 If any open chat is WORKING (a turn in flight, queued input, or waiting for the user's answer),
  the button first asks for confirmation in the app's existing confirm dialog pattern, naming the
  count: "Hay N chats trabajando. Cerrarlos corta lo que están haciendo; las sesiones se guardan." /
  "N chats are working. Closing them stops what they are doing; the sessions are kept." with Cancel
  and "Cerrar los chats". With nothing working it closes at once, without a dialog.
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
