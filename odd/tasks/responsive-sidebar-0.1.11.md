# responsive-sidebar-0.1.11: collapse the sidebar when the window is narrow

The user's request (2026-11-13, with two screenshots): "en la 0.1.11 podemos meter
tambien que la app sea responsive ademas que el sidebar tenga lo de plegar y
desplegar porque me percate que no es responsive". At ~690px the fixed 280px
sidebar squeezes the conversation; there is no way to hide it.

## Specs

- S1: A toggle control collapses and expands the workspace sidebar. It lives in
  the chat header, before the title, and in the no-chat header too, so a
  collapsed sidebar can always be brought back.
- S2: Below a width threshold the sidebar starts collapsed automatically
  (responsive); above it, expanded. The threshold is a named constant, chosen so
  the 280px sidebar plus a readable conversation both fit (800px window width).
- S3: A manual toggle wins for the whole run: if the user expanded the sidebar on
  a narrow window, a resize does not re-collapse it (and vice versa), until the
  app restarts. The state is view-local (like the composer draft); nothing is
  persisted.
- S4: While collapsed the conversation takes the whole width and the sidebar is
  removed from the layout, not merely squeezed.
- S5 (added after the user's report): the app is responsive at EVERY width, not
  only collapsible. No horizontal overflow: the conversation column can shrink
  below its content, long titles truncate, long code scrolls inside its block.
  On a narrow window an open sidebar floats over the chat with a scrim (click
  outside closes it) instead of squeezing it; the bottom row wraps.

## Tasks

- [ ] T1 (S2, S3) `sidebar/collapse.ts`: the pure rule (manual wins, else the
      width decides) + tests.
- [ ] T2 (S1) `SidebarToggle.svelte` (icon, aria-expanded, label) + the toggle in
      ChatHeader and the no-chat header + tests.
- [ ] T3 (S4) App.svelte wiring (resize listener, collapsed class) + the CSS
      rules, pinned in layout.test.ts.
- [x] T4 Full suites (vitest, cargo, clippy, svelte-check) and commit.
- [x] T5 (S5) Fix after the user's report: single-column collapsed grid,
      `minmax(0, 1fr)` + `min-width: 0` on the main area, `overlayOf` rule +
      `.soverlay`/scrim, wrapping bottom row; verified with real Chrome renders.

## Verification plan

- RED first for the rule and the header toggle; the CSS rules are pinned in
  layout.test.ts (the file already reads the stylesheet as raw text).
- Window check against the dev build at narrow and wide widths.

## Log

- L2 (user, after trying the app; verbatim): "mira los horrores que hiciste, solo
  agregaste el boton para plegar/desplegar pero no hiciste responsive de forma
  correcta la app" (three screenshots: the collapsed chat crushed into a ~50px
  column, and the chat overflowing and cut off at the right edge on a narrow
  window with the sidebar open).
- L3 (what went wrong, so it is not repeated): the first slice shipped a toggle
  and a collapse rule, verified only by unit tests and by stylesheet tests that
  PINNED THE BUG (`grid-template-columns: 0 1fr`). Nobody rendered the layout at
  a narrow width. Two root causes, both reproduced with real Chrome renders of
  the real CSS before touching it: (1) with the sidebar `display: none` the main
  area became the grid's first item and landed in the leftover 0px column;
  (2) a bare `1fr` column cannot shrink below its content's min-content, so the
  nowrap title pushed the whole chat past the window edge. Fixes: one full-width
  column when collapsed, `minmax(0, 1fr)` and `min-width: 0` on the main area,
  an overlay drawer with a scrim below the threshold. Verified with renders at
  500/700/920/1280px (harness: real CSS and class names, static fake rows,
  state classes set by hand, so the resize listener in App.svelte is covered by
  rule tests only).
- L1 (user request, verbatim): "abrir pr y luego merge,en la 0.1.11 podemos meter
  tambien que la app sea responsive ademas que el sidebar tenga lo de plegar y
  desplegar porque me percate que no es responsive" (two screenshots at ~690px
  and ~700px where the fixed sidebar squeezes the conversation).
