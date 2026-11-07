# helper-thread-m6: show a helper's thread

The helpers panel (helpers-m5) decodes each task's thread (text, thinking, notes, tool calls) but only shows cards. Clicking a card must open that helper's thread.

## Specs

- S1: "mostrar el hilo de los helpers" (the suggestion the user accepted: "dale con tu sugerencia").
- S2: Selection state lives in `gentle-app` (testable); the UI only draws and emits events (core-m0 S4).
- S3: "que no se cree un archivo largo con todo metido" — one file per responsibility: `view/helpers.rs` (list), `view/helper_thread.rs` (detail), `view/helper_thread/format.rs` (pure text shaping).
- S4: Work goes on a branch with a pull request so the CI checks it before `main` (agreed after the first push).

## Tasks

- [x] T1 (S2) app: `Intent::SelectHelper(id)` / `CloseHelper`, `AppState.selected_helper`, cleared by New chat and Open session, unknown ids ignored — RED observed (1 of 4 new tests failed against a stub; 3 passed vacuously until the behaviour existed); 38 app tests; clippy/fmt clean — commit: see git log (feat/helper-thread)
- [x] T2 (S3) ui: pure formatting of tool args/output and truncation — RED observed (10 of 10 failed against a stub); one expectation changed after the fact: JSON object keys come out alphabetically (serde_json maps are ordered by key), so the test now says so — commit: see git log (feat/helper-thread)
- [x] T3 (S1, S3) ui: clickable cards, thread detail component, wiring; verified with a real delegation — cards are buttons (`Event::Select`), `helper_thread` shows prompt, text, thinking, notes and tool calls (name, ok/failed/running, args, output preview), "N earlier items dropped", back button; selected task replaces the list, falls back to the list if it vanishes — commit: see git log (feat/helper-thread)

## Log

- L1: "dale con tu sugerencia es mostrar el hilo de los helpers"

## Verification

- 207 tests green across the workspace (38 app, 10 new formatting tests), clippy -D warnings and fmt clean.
- Real gentle-shell, GUI: delegated two helpers, opened one card: its thread (prompt, text, thinking) showed; Back returned to the list.
- That run exposed a bug: long lines were clipped at the right edge instead of wrapping. Fixed (the items column now fills the width) and re-checked with a stand-in `gentle-shell` (a small script run through `GENTLE_SHELL_BIN`, kept outside the repo) that publishes a frame with long text, an ok/failed/running tool call, a dropped-items count and a failed task without a label: everything wraps, tool cards show args and a 12-line output preview, statuses are coloured, Back works.
- Stopped using the real model for further checks once gentle-shell reported "Claude rate limit warning: 91% used (five_hour)".
- Not done: scrolling to the start of a long finished thread is manual (the view is anchored to the bottom, good for live helpers); the card colour (`button::secondary`) is lighter than the rest of the dark theme and the green `done` on it has low contrast; thread items are plain text (no markdown).

## Log (continued)

- L2: Force-killing the GUI with `taskkill /F` once did not leave a `gentle-shell` orphan (the child exits when its stdin closes), but graceful close is what the app is designed for.
