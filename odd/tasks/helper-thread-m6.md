# helper-thread-m6: show a helper's thread

The helpers panel (helpers-m5) decodes each task's thread (text, thinking, notes, tool calls) but only shows cards. Clicking a card must open that helper's thread.

## Specs

- S1: "mostrar el hilo de los helpers" (the suggestion the user accepted: "dale con tu sugerencia").
- S2: Selection state lives in `gentle-app` (testable); the UI only draws and emits events (core-m0 S4).
- S3: "que no se cree un archivo largo con todo metido" — one file per responsibility: `view/helpers.rs` (list), `view/helper_thread.rs` (detail), `view/helper_thread/format.rs` (pure text shaping).
- S4: Work goes on a branch with a pull request so the CI checks it before `main` (agreed after the first push).

## Tasks

- [x] T1 (S2) app: `Intent::SelectHelper(id)` / `CloseHelper`, `AppState.selected_helper`, cleared by New chat and Open session, unknown ids ignored — RED observed (1 of 4 new tests failed against a stub; 3 passed vacuously until the behaviour existed); 38 app tests; clippy/fmt clean — commit: see git log (feat/helper-thread)
- [ ] T2 (S3) ui: pure formatting of tool args/output and truncation — commit: pending
- [ ] T3 (S1, S3) ui: clickable cards, thread detail component, wiring; verified with a real delegation — commit: pending

## Log

- L1: "dale con tu sugerencia es mostrar el hilo de los helpers"
