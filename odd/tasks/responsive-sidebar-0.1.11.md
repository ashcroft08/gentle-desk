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

## Tasks

- [ ] T1 (S2, S3) `sidebar/collapse.ts`: the pure rule (manual wins, else the
      width decides) + tests.
- [ ] T2 (S1) `SidebarToggle.svelte` (icon, aria-expanded, label) + the toggle in
      ChatHeader and the no-chat header + tests.
- [ ] T3 (S4) App.svelte wiring (resize listener, collapsed class) + the CSS
      rules, pinned in layout.test.ts.
- [ ] T4 Full suites (vitest, cargo, clippy, svelte-check) and commit.

## Verification plan

- RED first for the rule and the header toggle; the CSS rules are pinned in
  layout.test.ts (the file already reads the stylesheet as raw text).
- Window check against the dev build at narrow and wide widths.

## Log

- L1 (user request, verbatim): "abrir pr y luego merge,en la 0.1.11 podemos meter
  tambien que la app sea responsive ademas que el sidebar tenga lo de plegar y
  desplegar porque me percate que no es responsive" (two screenshots at ~690px
  and ~700px where the fixed sidebar squeezes the conversation).
