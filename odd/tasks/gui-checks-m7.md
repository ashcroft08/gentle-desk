# gui-checks-m7: check the shipped-but-unverified paths

The model-error box, the extension dialogs and the Stop button were shipped with unit tests of the app layer only. This batch checks them for real and fixes what breaks, without spending model quota.

## Specs

- S1: "dale" on the ordered backlog, item 1: GUI checks of the error box, dialogs (confirm/select/input/editor) and Stop (odd/tasks/backlog.md).
- S2: No real model: a stand-in `gentle-shell` (`tools/fake-gentle-shell`) driven through `GENTLE_SHELL_BIN` (gentle-shell reported 91% of the 5-hour Claude limit).
- S3: One responsibility per file (README "Code organization").

## Tasks

- [x] T1 (S2) Stand-in gentle-shell with prompt-keyed scenarios (error, dialog, slow/abort, echo) — delegated to `gentle-ai-worker` (context backstop), verified by the parent (fmt, clippy, 217 tests, no leftover node process)
- [x] T2 (S2) `gentle-term` end-to-end tests against the stand-in (echo, flags, provider failure, cancelled dialogs, missing launcher) and `split_args` moved to its own tested module — same delegation
- [ ] T3 (S1) GUI checks: error box, dialogs, Stop — commit: pending

## Log

- L1: "ordena lo que toca hacer y dale"
