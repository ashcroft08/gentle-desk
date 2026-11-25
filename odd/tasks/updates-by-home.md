# Updates by home: the update commands follow the shell's home choice

Found by the maintainer testing 0.8.2 with "Mi pi" selected: "Actualizar Gentle Shell" opened a terminal
with `gentle-shell` and sent a chat message with the word `upgrade`, because the app always runs the
`gentle-shell ...` commands, which are only right for the "Separado" home.

## Specs

- S1 The terminal command each Updates action runs depends on the Home choice (Settings > Home del
  shell: "Separado" = the shell's own pi; "Mi pi" = the user's own pi), exactly:

  | action               | Separado                      | Mi pi                              |
  |----------------------|-------------------------------|------------------------------------|
  | update extensions    | `gentle-shell update --extensions` | `pi update --extensions`       |
  | update pi            | `gentle-shell update`         | `pi update`                        |
  | update Gentle Shell  | `gentle-shell upgrade`        | `pi install npm:gentle-pi@<latest>` |

- S2 In "Mi pi", `<latest>` is the stable version the Gentle Shell row already measured against npm
  (`gentle-pi`, S6 of settings-polish: plain x.y.z only). The command is built in RUST from the view's
  own `shell.latest`, never from a string the web sends. If `latest` is not a plain stable version (or
  unknown), the action does not run and says why in one honest line; it never installs `@latest` blindly
  and never falls back to `gentle-shell upgrade`.
- S3 Nothing else changes: the zero-open-chats gate, the visible terminal, the window titles, the skip
  rules, the Separado commands (they are today's).
- S4 The pane tells the user which command it will run (one quiet mono line under the action, e.g.
  `pi install npm:gentle-pi@5.0.0`), in both modes, so the Home choice is never a surprise.
- S5 If `pi` is not on the PATH in "Mi pi", the terminal's own error is what the user sees (the window
  stays open); the app does not guess another program.

## Tasks

- T1 (S1-S3, S5) Rust: the command table keyed by the home choice in `terminal.rs`, the `<latest>`
  derivation and its refusal. Route: worker, test-first over a fake launcher.
- T2 (S4) Web: the quiet command line under each action and the refusal line. Route: worker after T1.

## Log

- L1 (2026-11-24, the maintainer): the table above, verbatim: "si aqui yo tengo escogido separado, los
  comandos para actualizar las extensiones son gentle-shell update --extensions, para actualizar pi es
  gentle-shell update y para actualizar gentle-shell es gentle-shell upgrade. y si el usuario elije pi o
  tiene seleccionado pi los comandos son para actualizar las extensiones pi update --extensions, para pi
  pi update y para el paquete de gentle-shell ... pi install npm:gentle-pi@version por ejemplo la ultima
  fue la 5.0.0 asi que el comando fue pi install npm:gentle-pi@5.0.0 ... al actualizar abrio la terminal
  con gentle-shell y mando un mensaje con la palabra upgrade".
