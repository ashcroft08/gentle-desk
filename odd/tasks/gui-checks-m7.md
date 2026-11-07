# gui-checks-m7: check the shipped-but-unverified paths

The model-error box, the extension dialogs and the Stop button were shipped with unit tests of the app layer only. This batch checks them for real and fixes what breaks, without spending model quota.

## Specs

- S1: "dale" on the ordered backlog, item 1: GUI checks of the error box, dialogs (confirm/select/input/editor) and Stop (odd/tasks/backlog.md).
- S2: No real model: a stand-in `gentle-shell` (`tools/fake-gentle-shell`) driven through `GENTLE_SHELL_BIN` (gentle-shell reported 91% of the 5-hour Claude limit).
- S3: One responsibility per file (README "Code organization").

## Tasks

- [x] T1 (S2) Stand-in gentle-shell with prompt-keyed scenarios (error, dialog, slow/abort, echo) — delegated to `gentle-ai-worker` (context backstop), verified by the parent (fmt, clippy, 217 tests, no leftover node process)
- [x] T2 (S2) `gentle-term` end-to-end tests against the stand-in (echo, flags, provider failure, cancelled dialogs, missing launcher) and `split_args` moved to its own tested module — same delegation
- [x] T3 (S1) GUI checks: error box, dialogs, Stop — all three verified in the window against the stand-in; two usability defects found and fixed (below) — commit: see git log (feat/gui-checks)

## Log

- L1: "ordena lo que toca hacer y dale"

## Verification (window + stand-in gentle-shell, no model)

- **Model error:** prompt `error` -> red box "✗ Model error / quota exceeded (fake)"; the provider's own text ("Provider says no.") is not shown as a reply; the `notify` warning appears as a yellow notice.
- **Dialogs:** prompt `dialog` -> confirm (Yes/No/Cancel), then select (a/b), then input; the clicked answers reached the child exactly: `confirmed:true`, `value:"b"`, `value:"Ana"` (echoed by the stand-in in the reply).
- **Stop:** prompt `slow` -> streaming "tick tick ..." with the button showing Stop; clicking it sent `abort`; the message ended "ABORTED" and the button went back to Send.
- **Defects found and fixed:** (1) the text box of an input/editor dialog was not focused, so typing needed a click first; (2) after answering a dialog the prompt box lost the keyboard, so the next prompt typed nowhere. Now a new text dialog takes the focus and answering returns it to the prompt box; re-checked by typing `Ana{ENTER}` and then `hola{ENTER}` with no clicks in between.
- 217 tests green (the stand-in's end-to-end tests included), clippy -D warnings and fmt clean.
- Not covered: the `editor` dialog variant (same code path as `input`, not exercised by the stand-in), no unit test for the focus behaviour (it is iced runtime behaviour, checked by hand).

## Log (continued)

- L2: The stand-in and the terminal end-to-end tests were delegated (context backstop) and re-verified by the parent before committing.
