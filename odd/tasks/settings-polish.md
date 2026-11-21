# Settings polish for 0.8.0: design pass, "skip this version", dead intents

Part of 0.8.0 (the maintainer's decision, L2). The General page with the environment probe and the live
package install are NOT in this release: they go to 0.9.0 as their own feature.

## Specs

- S1 The Updates pane gets a "Skip this version" action next to the offered version. The skipped version
  is persisted in the app config (additive, lenient, like `theme` and `notifications`) and survives a
  restart.
- S2 A skipped version is not offered again at startup. A NEWER version than the skipped one is offered
  as usual. The user's own "Check again" always shows the real state, including a skipped version, with
  a way to un-skip it; a skip never hides information from an explicit check.
- S3 A failure of the startup check (no network, a bad answer) is silent: no notice, no error pane. A
  failure of the user's own "Check again" is shown, as today.
- S4 The three intents the snapshot made unused (`RefreshAuth`, `RefreshUsage`, `CheckUpdates`) are
  removed from the contract and the Rust dispatch path, with their tests, only if nothing reads them any
  more (grep first); `CheckUpdates` stays if "skip" or the startup check still uses it.
- S5 A design pass over every Settings pane in the app's own tokens: the search field's icon spacing,
  empty space, alignment and hierarchy, judged from the real panes and reproduced as artboards before any
  code changes.

- S6 The shell row checks the latest `gentle-pi` version on npm (one GET through the existing `WebGet`, like pi), so it can be "behind".
- S7 When the shell is behind, the pane offers `gentle-shell upgrade` in a terminal (a new terminal action next to `gentle-shell update`), under the same zero-open-chats gate.
- S8 The pane shows when it last checked and, when a read failed, which one and why, instead of blank versions. An unknown stays unknown, never "up to date".

## Tasks

- T1 (S1, S2, S3) Updates: config field, intent, snapshot/view carry, pane action, startup rule. Route:
  worker.
- T2 (S4) Remove the dead intents. Route: worker, AFTER T1 (same surfaces).
- T4 (S6, S7, S8) Updates: the shell row, the upgrade action, last-checked and failure reasons. Route: worker, AFTER T1 (same surfaces); design first (the `03-updates` artboard gains these rows).
- T3 (S5) Design pass: artboards in `design/settings-polish/`, approved on the canvas, then wired. Route:
  design worker first.

## Slice note

`feat/0.8.1-notifications` carried 3,673 authored changed lines in 8 commits over `origin/main` when this
feature started (notifications T1-T4, the capability narrowing, the 0.8.0 version and notes, the
theme-card columns). That is past the ~400-line slice: the delivery shape is one PR for the 0.8.0
release, each commit a reviewable work unit.

## Log

- L1 (2026-11-20, the maintainer): "que mas falta hacer en configuracion?" then "quiero lanzar en la
  0.8.0 todas las mejoras de configuraciones y mejorar el diseño de configuraciones, que piensas".
- L2 (2026-11-20, the maintainer, asked which scope): "Mi propuesta: diseño + omitir versión + limpieza"
  - the design pass, "skip this version" and the dead-intent cleanup go in 0.8.0; the General page and
  the live install go to 0.9.0.
- L3 (2026-11-20, the maintainer): "nuestro actualizaciones no sirve la verdad, no avisa si hay una nueva version de pi o hay que actualizar extensiones. aparte no dice si hay actualizaciones de gentle-shell o pi install@gentle-shell ... y aparte estos manes manejan de mejor forma como se actualiza eso y los paquetes a eso me refiero que quiero poder implementar". Explorer check (verified, not assumed): pi's newer version IS checked (npm registry `@earendil-works/pi-coding-agent/latest`, gear dot, headline); the shell row's `latest` is always None (`apps/desktop/src-tauri/src/settings/updates.rs:66-68`); extensions/packages are not checked at all and "Update extensions" only opens a terminal on `gentle-shell update --extensions`; no last-checked time, no failure reason, no skip button (the Rust skip path exists, no pane dispatches it). The other app gets package state from the Desktop RPC `get_update_status`, which the installed Shell 4.8.0 does not advertise (needs Shell main), so a per-package "behind" state is not possible today.
- L4 (2026-11-20, the maintainer, asked which scope): "Lo que se puede ya (recomendado)" - 0.8.0 gets: S6 the shell row checks `gentle-pi` on npm (one GET through the existing `WebGet`, like pi); S7 a "gentle-shell upgrade" action in a terminal when it is behind (a new terminal action, like `gentle-shell update`); S8 the pane shows when it last checked and WHY a read failed instead of blank versions; S1-S3 (skip) as already specified. Package management (install, remove, which package is behind, live output) moves to 0.9.0 with the General page, because it mutates the machine, needs its own design and an isolated test setup, and depends on the Shell exposing package state.
