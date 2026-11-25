# General: the environment check, the first Settings page

Part of 0.9.0 (with packages-status, which is already built on `feat/packages-status`). The future
board `design/settings-redesign/00-future.html` drew this page inside the approved dialog; the
maintainer approved that direction of it (the frame is the 0.8.2 one).

## Specs

- S1 A new rail entry "General" FIRST in the list, and it is the section Settings opens on.
- S2 The page is an environment check: one row per tool the app and the shell depend on
  (Gentle Shell, pi, node, git), each with: installed/missing, the installed version, and a short
  fix hint when it is missing or unreadable ("instalalo con ...", the exact command the app names).
- S3 Every fact comes from a source the app already has or a bounded read it already knows how to
  run: the shell and pi versions are the Updates rows' (`get_update_status`-free, from the
  `gentle-shell --version` probe the checks already run); node and git are two bounded probes
  (`node --version`, `git --version`) run like `auth check` runs (off the window thread, deadline,
  no console window). NOTHING new is invented and nothing asks the network.
- S4 One read per dialog open, riding the snapshot (the same rule packages-status uses: no watcher,
  no timer; "Check again" on this page re-runs the probes once).
- S5 Both languages; missing vs unreadable are different words; no secret and no path on the wire.
- S6 The row for git exists only where the shell depends on it (it is used for the folder chips):
  if a probe fails because git is absent, the row says so; the app never blocks on a missing git.

## Facts

- The version probe (`gentle-shell --version`) already yields shell+pi; the snapshot carries both
  (`UpdatesView.desk/pi/shell` and the shell row) - General reuses the SAME read's answers.
- `node --version` / `git --version` are two spawns of the shape `settings.rs::run` already has
  (launcher-independent, `without_console`, 8 s budget), run on the snapshot's blocking worker.
- The rail: `sections.ts` `Section` union + `SECTIONS` (icon, keywords ES/EN for the search) +
  `TAB_IDS`; the dialog opens on `language` today (the keys test pins it).

## Tasks

- T1 (S2-S6) Design: ONE proposal of the General pane inside the 0.8.2 frame (extend
  `design/settings-redesign/` with `08-general.html`), using the rules R1-R4 the 07 board factors
  (sections + hairline, full column, quiet sub-headings, bottom air/fade). Route: design worker.
- T2 (S2-S6) Rust: the two probes + the `GeneralView` (tools rows) into the snapshot. Route: worker,
  after T1 is approved.
- T3 (S1, S2) Web: the pane, the rail entry first, open-on-General, keywords, both languages.
  Route: worker, after T2.

## Log

- L1 (2026-11-21, the maintainer, confirming the extensions feature): "si funciona bien, continua."
  with the app updated to 0.8.2 and everything fine. The next planned feature of 0.9.0 is General.
- L2 (2026-11-22, design delivered): two worker attempts for T1 died on provider errors before writing anything, so the parent authored `design/settings-redesign/08-general.html` INLINE (the budget rule: never repeat an attempt on the same cause). One proposal: the tool rows (Gentle Shell / pi / node / git) with Instalado/Falta/No se pudo leer + version + fix hint (missing and unreadable worded apart, S5), the never-blocks-on-git sentence (S6), the last-checked + Check again row (S4), General first in the rail (S1), the 800x560 sheet, and the ES/EN table. Awaiting the maintainer's approval on the canvas.
- L3 (2026-11-24, the maintainer, on the canvas review of `08-general.html`): "apruebo" - the General pane as drawn (four tool rows, Instalado / Falta / No se pudo leer worded apart, the fix hints only when missing or unreadable, the never-blocks-on-git sentence, last-checked + Comprobar de nuevo, General first in the rail and the section Settings opens on). The maintainer also pushed `feat/packages-status`.
