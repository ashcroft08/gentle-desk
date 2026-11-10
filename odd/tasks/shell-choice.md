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

- [x] T1 (S1, S3, S6) Detection and home application in Rust: `detect_pi`, config field `shell_home` (lenient), `HomeChoice` flag in `with_home_flag`. Route: inline; commit: d9587bd.
- [x] T2a (S2, S3) Contract: `ShellView`, `HomeChoiceView`, `SetShellHome`, pi found at start. Route: inline; commit: b5580d7.
- [x] T2b (S2) Welcome screen and the shell slice of the web view. Route: inline; commit: ea44dcd.
- [x] T2c (S2, S3) The app starts again after the welcome choice so it applies at once (`RestartApp`, `close::exit_step`). Route: inline; commit: 5d50d4d.
- [x] T3a-b (S4) Launcher probe, install state machine, `CommandInstaller`, `InstallShell` and `CheckShellAgain`. Route: inline; commit: 4810caf.
- [x] T3c-d (S4) Permission screen, checked in a real window with a failing and a succeeding fake npm. Route: inline; commit: 18ddd6e.
- [x] T4a (S5) Footer control "My pi / Separate", `restart_needed`, Restart now, npm failure hint, `npm()` argv test. Route: inline; commit: a100752.
- [x] T4b (S7) Docs: README section, this document, backlog 30-33. Route: inline; commit: the one that adds this line.
- [x] V Independent verification (gentle-ai-verify, read-only): no blockers; S1, S2, S3, S4, S6 met, S5 then implemented; its should-fix items 2 and 3 were fixed in a100752; items 1, 4, 5, 8 are known limits below.

### Known limits (from the independent verification)

- Closing the app during an install leaves `npm install -g` running to its end: it is not killed on purpose, because a half-killed global install can leave a broken package (verifier finding 1, decision L4).
- On Windows the 20-minute timeout kills `npm.cmd` (a `cmd.exe`) and may leave its `node` child (finding 5).
- Install progress is one static line; the output is shown only if it fails (finding 8).
- The wiring of the startup launcher probe, `RestartApp` and `ask` reaching the web has no windowless test (they need a window handle); they were checked by hand in a real window (findings 4 and 10).

### Slice boundary

The branch holds eight work-unit commits (about 2 300 changed lines), too many for one review. It goes out as four chained PRs, each green on its own (a contract change and the web code that reads it travel together, because the web type check fails between them): (1) T1, the home choice in Rust, nothing visible; (2) T2a-c, the contract, the welcome and the relaunch; (3) T3, the missing gentle-shell and the install; (4) T4, the footer control and the docs. Merged in that order, each retargeted to main after the one below it.

## Log

- L1 (2026-11-10, user): "Podemos ponerle de nombre Gentle Shell Desktop a la app?" ... earlier: "el instalador puede identificar si tengo pi o gentle-shell para escoger uno de los dos o si tengo pi darme la opcion de escoger que use eso o una instalacion limpia de gentle-shell y que use esa?" and later "podemos hacer lo que te dije de que elija si usar pi o instalar gentle-shell".
- L2 (2026-11-10, user answers): "Que gentle desk detecto si hay una instalacion y si hay preguntarle si usa esa que encontro, si no instala gentle-shell asi como se hacia en gentle-shell-desktop"; "Primer arranque, y cambiarlo luego"; "Se le dice al usuario que hay que instalar gentle-shell y que de el permiso, porque sin eso no se puede usar se supone".
- L3 (2026-11-10, found in a real window): after choosing at the welcome the run kept the separate home until a restart, which made "Use my pi setup" look broken; decision: the app saves the choice and starts again (T2c).
- L4 (2026-11-10, agent decision): an install in progress is not killed when the app closes (see Known limits); to revisit with the user if they prefer otherwise.
- L5 (2026-11-10, verifier): no blockers; see V above.
