# Shell choice: use the user's pi or a separate gentle-shell (m19 follow-up)

Feature: at first start, detect an existing pi and let the user choose between using it or keeping gentle-shell separate; when gentle-shell itself is missing, ask permission to install it. Reference design: the "Welcome to gentle shell" screen of gentle-shell-desktop (user's screenshot, `C:\Users\josep\OneDrive\Im*genes\Captura de pantalla 2026-11-06 205559.png`, read once with the user's OK).

## Specs

- S1 Detection. At start the app finds out, without reading file contents, (a) whether pi is on the machine (`~/.pi/agent`, or `PI_CODING_AGENT_DIR`) and (b) whether the `gentle-shell` launcher resolves. User: "Que gentle desk detecto si hay una instalacion y si hay preguntarle si usa esa que encontro".
- S2 Welcome screen, shown only when pi is found and no choice is stored yet. Drawn after the reference: title "Welcome to gentle shell", a card "We found pi on this machine" with the path, and two cards: "Use my pi setup" (RECOMMENDED; links to ~/.pi/agent; sign-ins, local models and chats appear here; chats started here also show in the terminal; pi settings are never edited) and "Keep it separate" (a clean space with its own sign-ins and chats; nothing on this machine is touched; creates ~/.gentle-shell/agent). Footer line: "No pi on the machine? This screen is skipped and gentle shell starts in its own space." User: "Primer arranque, y cambiarlo luego".
- S3 The choice is stored in the app config as an additive optional field (CONFIG_VERSION stays 1, absent = not chosen yet) and applied as the home flag of every spawn: link -> `--link`, separate -> `--isolated`. An explicit `--home`/GENTLE_SHELL_HOME of the developer keeps winning.
- S4 Missing gentle-shell. User: "Se le dice al usuario que hay que instalar gentle-shell y que de el permiso, porque sin eso no se puede usar se supone". A screen says gentle-shell is required and asks for explicit permission; only on a granted permission the app runs the install (`npm install -g gentle-pi`), shows its progress and outcome (success, or the failure with what to do, e.g. Node/npm missing), and offers "Check again". Nothing is installed without that permission. User also: "si no instala gentle-shell asi como se hacia en gentle-shell-desktop".
- S5 Change it later. User: "cambiarlo luego". A control in the sidebar footer next to the theme shows the current choice and lets the user switch; the change is saved at once and takes effect after restarting the app, said in words (chats already open keep the home they started with).
- S6 Never edit the user's pi files; the app only passes flags and reads existence.
- S7 Process rules: tests use the fake shell and a fake npm, never the real gentle-shell or a real global install; every claim checked in a real window except the native or real-install parts, which are the user's to test.

## Tasks

- [ ] T1 (S1, S3, S6) Detection and home application in Rust: `detect` (pi path exists? launcher resolves?), config field `shell_home`, `with_home_flag` fed from the config. Route: inline; commit: none yet.
- [ ] T2 (S2, S3, S5) Contract + intents + view state (detected, choice) and the web welcome screen and footer control. Route: inline; commit: none yet.
- [ ] T3 (S4) Missing gentle-shell: eager launcher check at startup, permission screen, install runner with progress events, Check again. Route: worker for the runner if it stays independent; commit: none yet.
- [ ] T4 (S5, S7) Footer change with the restart notice, docs (README, backlog), real-window check, installer rebuild by the user's request. Route: inline; commit: none yet.

## Log

- L1 (2026-11-10, user): "Podemos ponerle de nombre Gentle Shell Desktop a la app?" ... earlier: "el instalador puede identificar si tengo pi o gentle-shell para escoger uno de los dos o si tengo pi darme la opcion de escoger que use eso o una instalacion limpia de gentle-shell y que use esa?" and later "podemos hacer lo que te dije de que elija si usar pi o instalar gentle-shell".
- L2 (2026-11-10, user answers): "Que gentle desk detecto si hay una instalacion y si hay preguntarle si usa esa que encontro, si no instala gentle-shell asi como se hacia en gentle-shell-desktop"; "Primer arranque, y cambiarlo luego"; "Se le dice al usuario que hay que instalar gentle-shell y que de el permiso, porque sin eso no se puede usar se supone".
