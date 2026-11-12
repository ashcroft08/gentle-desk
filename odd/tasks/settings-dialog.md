# settings-dialog: a Settings dialog for home, theme, about and updates

The maintainer's request (2026-11-12): move the footer's My pi|Separate switch and the
theme picker into a Settings dialog, improve the home explanation, add an About section
(app name + version) and an Updates section with a check button. Decisions taken: the
updater keeps its automatic startup check and Settings adds a manual check; the home
switch moves COMPLETELY (the footer keeps only the Preview chip and a gear button);
the theme moves too (Appearance section).

## Specs

- S1: The footer keeps exactly two things: the Preview chip (unchanged) and a gear
  button that opens the Settings dialog. HomeSetting and ThemePicker leave the footer.
- S2: The Settings dialog uses the house bits-ui dialog shape (like
  NewWorkspaceDialog/DeleteChatDialog), with three sections in order: Appearance, Shell
  home, About + Updates.
- S3: Appearance holds the existing ThemePicker, moved as-is (same component, same
  behavior, no redesign).
- S4: Shell home holds the existing HomeSetting, moved as-is, PLUS the full
  explanation the welcome screen states (My pi uses the pi installation already on the
  machine: sign-ins, local models and chats appear in the terminal; Separate creates a
  clean space at ~/.gentle-shell/agent and touches nothing else; a change waits for a
  restart).
- S5: About carries the app name and the running version (the bridge already holds
  app_version; a contract field carries it to the web - bindings regenerate, drift
  guard green).
- S6: Updates shows the current version and a Check now button that runs the same
  updater check the startup runs, with its result drawn in the section (checking /
  update available / up to date). The automatic startup check is unchanged. The exact
  wiring follows how the updater plugin is integrated today (T2 maps it first).
- S7: The what's-new modal is unchanged (still the only place release notes show).

## Tasks

- T1 (S1, S2, S3, S4) Footer slim-down + the Settings dialog shell with Appearance
  and Shell home moved and their explanations - route: delegated writer, commit:
  see git log (feat/settings-t1)
- T2 (S5, S6) About (app version to the web) + the Updates section with check-now
  over the existing updater wiring - route: delegated writer, commit: pending
- T3 (S4 copy audit) The welcome screen and the Settings dialog say the same thing
  about the two homes - route: delegated writer, commit: pending

## Log

- L1 (2026-11-12, user): "ya que vamos a tocar diseño queria ver si metiamos ese tipo
  switch de My pi|Separate y lo del tema al apartado de configuraciones hay podemos
  mejorar el diseño de My pi|Separate y darle una mejor explicacion al usuario,
  tambien agregar un apartado de info para ver la informacion de la app y version,
  tambien podemos meter el apartado de actualizacion para que ahi aparezca para
  actualizar pero no se si esta bien el planteamiento."
- L2 (2026-11-12, parent): the framing is sound - the standard desktop pattern, and
  moving the home switch out of the footer protects a restart-requiring decision from
  stray clicks. Decisions by the user: updates automatic + manual check in Settings;
  the home switch moves completely; the footer keeps only the Preview chip and the
  gear; the theme moves into Settings too.
