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

## Tasks

- T1 (S1, S2, S3) Updates: config field, intent, snapshot/view carry, pane action, startup rule. Route:
  worker.
- T2 (S4) Remove the dead intents. Route: worker, AFTER T1 (same surfaces).
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
