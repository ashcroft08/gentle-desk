# app-m2: the `app` crate (state, intents, commands)

Middle layer between `ui` and `backend` (core-m0 S4): pure and testable, no I/O, no UI framework. Continues ui-spike (iced chosen).

## Specs

- S1: "app (estado, reducers, comandos)" — pure state + reducer + commands, no processes, no UI.
- S2: The reducer does not emit user messages; "the app layer must append them" (ui-spike L2).
- S3: "los `notify` dentro de `ChatState`" (promised after M1) — notices live in app state; core's reference-validated reducer stays untouched.
- S4: "de las opciones que me diste vamos con la 1, el boton de copiar para no complicarnos" — a per-message copy intent that yields the raw markdown.
- S5: Bugs of the reference app not to repeat: "New chat" was a no-op by reference equality; the chat list loaded once.
- S6: Dependency direction: app -> gentle-core only (backend and ui depend on app/core, never the reverse).

## Tasks

- [x] T1 (S1, S2, S5) `gentle-app`: AppState, Intent/Input/Command, `update`; SendPrompt, Abort, NewChat — RED observed (stub: 12 of 20 failed); 20 tests green, clippy/fmt clean — commit: see git log (feat/app-m2)
- [x] T2 (S3) notices from `notify` events (level, id, cap, dismiss) — RED observed (stub: 12 of 20 failed); 20 tests green, clippy/fmt clean — commit: see git log (feat/app-m2)
- [x] T3 (S4) CopyMessage intent + dialog answers removing pending dialogs — RED observed (stub: 12 of 20 failed); 20 tests green, clippy/fmt clean — commit: see git log (feat/app-m2)
- [ ] T4 Wire `gentle-term` to `update` instead of its own loop logic (proves the layer end to end) — commit: pending

## Log

- L1: "de las opciones que me diste vamos con la 1, el boton de copiar para no complicarnos. continua"
