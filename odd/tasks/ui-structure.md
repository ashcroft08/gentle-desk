# ui-structure: split the UI crate into components

`crates/ui/src/main.rs` had grown to 451 lines with state, wiring, subscriptions and every view in one `impl App`.

## Specs

- S1: "la ui se maneja por componentes osea separando todo y no teniendo todo en un solo archivo?" — it was not; it must be.
- S2: "hazlo primero, y que asi en adelante se maneje, que no se cree un archivo largo con todo metido" — refactor first, and keep working this way from now on (rule recorded in README "Code organization" and in memory).
- S3: Behaviour does not change; the existing tests stay green.

## Tasks

- [x] T1 (S1, S3) `view/` with one file per component (header, sidebar, thread, notices, dialog, composer, banner), each with its own `Event`; `message`, `subscription`, `backend` (startup), `app` (state and wiring), thin `main` — largest file now 174 lines (was 451)
- [x] T2 (S2) Convention written in README ("Code organization") and `view/mod.rs`

## Verification

- 157 tests green (same as before), clippy -D warnings and fmt clean.
- By hand against the real gentle-shell: typed a prompt in the window (streamed reply, Copy, notices, the new session appears in the sidebar), New chat clears the thread, clicking a session opens it highlighted; closing leaves no process behind.
- No unit tests for the components themselves (visual); their logic lives in `gentle-app`.

## Log

- L1: "si hazlo primero, y que asi en adelante se maneje, que no se cree un archivo largo con todo metido"
