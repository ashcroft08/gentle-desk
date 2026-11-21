# Settings redesign: a bigger, calmer Settings, with far fewer cards

Starts after 0.8.1 (published). The General page with the environment probe and the packages (0.9.0)
will be built INSIDE this new frame, so the frame is designed first.

## Specs

- S1 The Settings surface is bigger. Today the dialog card is 680px wide whatever the window is and its
  body is about 312px tall (fixed by the rail's height), which reads small. The new design uses the space
  the window really has.
- S2 Far fewer cards. Today almost every setting is a card inside a card. The new design groups with
  titles, spacing and thin rules, and keeps a card only where one thing is truly a unit (a provider's row,
  a theme preview).
- S3 Nothing is lost: every pane that exists today (Appearance, Shell home, Updates, Account, Usage,
  Language, Notifications, About), the search field, both languages and every behaviour stay. This is a
  redesign of form and grouping, not of function.
- S4 The maintainer gave full creative freedom ("puedes hacerlo desde cero osea tienes libertad
  creativa"): the dialog, the rail and the panes may all be rethought, including whether Settings is a
  dialog at all. Several genuinely different directions are shown, not three variations of one.
- S5 The frame must hold what is coming without a second redesign: a General page with an environment
  probe, and a Packages page with install and remove and a live output box.
- S6 App tokens only, both themes, six themes; responsive from the window's 800x560 minimum up.

## Tasks

- T1 (S1-S6) Sync `design/system.md`, then design: a set of genuinely different directions as artboards
  in `design/settings-redesign/`, opened on the canvas for the maintainer to choose. Route: design worker.
- T2 Wire the chosen direction in small slices (frame, then each pane). Route: workers, after approval.

## Log

- L1 (2026-11-21, the maintainer): "quiero que demos tiempo para rediseñar el modal de configuracion, ya
  que lo veo pequeño y que todo esta apretado tambien mucho uso de cards. Te parece bien".
- L2 (2026-11-21, the maintainer, asked what bothers them most and for references): "dale, puedes hacerlo
  desde cero osea tienes libertar creativa".
- L3 (2026-11-21, design delivered): `design/settings-redesign/` - `02-direction-a-large-modal.html`, `03-direction-b-page.html`, `04-direction-c-search-first-sheet.html` (three directions over the same three screens: Appearance, Updates, Account), `00-future.html` (General with an environment probe and Packages with a live output box, in A) and `01-audit-vs-redesign.html` (the 12 audit findings: 7 closed by 0.8.1, 5 open; the measured case 680x310 -> 1180x544, column 432 -> 760, cards 9 -> 6). `design/system.md` was re-synced from `theme.css` + `app.css`. The design worker recommends A. Awaiting the maintainer's choice on the canvas.
