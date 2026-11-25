# Packages status: the app tells the user which extensions are behind

Part of 0.9.0. The idea came from the maintainer's question (L1): the terminal shows pi's own
"Package updates are available" notice, and the same facts are already reachable by the app.

## Specs

- S1 When a chat is open, the Updates pane's Extensions block names the packages that are behind
  (pi's own check, through `get_update_status` over the existing `/gentle:desktop` channel) and
  counts them ("N extensiones con actualización disponible"), with each name drawn. States:
  `update_available` is behind; `up_to_date`/`no_update_reported`/`skipped`/`unknown` never read as
  behind; an unreadable listing says the read could not run, never "up to date".
- S2 One read per explicit ask only (the pane's "Check again" path and once per dialog open, through
  the existing snapshot load when it can ask). No watcher, no timer, no read at launch.
- S3 With NO chat open the block keeps the approved honest sentence (this Gentle Shell cannot tell
  which extensions are behind) and names its condition: it can check when a session is open.
- S4 Nothing here runs an update: the only action stays the existing "Update extensions" terminal
  button. No secrets, no raw sources, no paths on the wire; bounded (the channel's own deadline).
- S5 A capability the child does not advertise (an older shell) degrades to exactly today's block,
  and the first live probe of the real method happens against the installed 4.9.x shell.

## Facts (verified in the installed shell, gentle-pi 4.9.0, 2026-11-21)

- The terminal notice is pi's `checkForAvailableUpdates`; the RPC `get_update_status` reads the SAME
  check for its packages slice (`lib/update-status.ts:234`, contract `docs/desktop-rpc.md` line 65
  lists the method; the extension wires it at `extensions/gentle-desktop-rpc.ts:106`).
- The answer carries `packages: {available, items[], truncated, command, checkedAt}`; an item is
  `{name, kind, scope, current, latest (always null today), pinned, state}`; states
  `update_available | no_update_reported | skipped | unknown`; 100-item bound with `truncated`.
- A plain read (no `refresh`) never reaches the network; `refresh: true` runs the checks once with
  a 15 s deadline, and a read is memory-only. The method is read-only: "running an update stays the
  launcher's job".
- The channel lives in the chat's child: with zero chats the app cannot ask (`desktop_client(chat)`
  is None), which is why the startup cannot carry this. The app's existing seam:
  `crates/backend/src/chat_host/desktop.rs` + `DesktopClient` (crates/gentle-core/rpc/desktop.rs),
  the same path profile.rs and stop.rs use, with `supports(method)` on the cached capabilities.

- S6 HONEST "CHECKED" (a defect found by the maintainer, 0.9.0): the shell reports a package as
  `update_available` ONLY after its own check ran (`refresh: true`); with no prior refresh in that
  child every checkable package is `unknown`. The app asked `refresh: false` and read only
  `available`, so an extension that was behind read as "Las extensiones están al día" - a false claim.
  Now: "up to date" requires every checkable item to be `no_update_reported` (or `skipped`);
  ANY item still `unknown` means "not checked yet", which is its own state with its own words
  (never "al día", never "no se pudo revisar" either - nothing failed, it just has not run).
- S7 AN EXPLICIT CHECK: a new command `check_packages` runs pi's own check once (`get_update_status`
  with `refresh: true`) with its own longer deadline (the shell's refresh has a 15 s bound; the
  channel's 5 s default is for plain reads), on a thread of its own and OFF the snapshot load (it must
  never hold Settings open), and the result is HELD by the app and carried in the pushed updates
  slice (`ViewState.updates.packages`) so the gear's dot and the pane light WITHOUT opening Settings.
  It runs (a) once per app run, in the background, when the first chat's child is ready (the only
  moment the channel exists - still never at launch and never starting a child), and (b) when the user
  presses "Comprobar de nuevo" in Updates. No timer, no watcher. No chat open: nothing runs, the
  pane keeps today's honest sentence.
- S8 THE RAIL DOT: when something is waiting (the same rule as the gear's dot, `updateWaiting`, with
  the dialog's live updates), the "Actualizaciones" entry in the Settings rail shows a small dot, so a
  user who opened Settings on General knows where to go. It follows the skip rule like the gear.

## Tasks

- T1 (S1, S2, S4, S5) Rust: a bounded packages read through the desktop channel for one open chat
  (reuse `profile.rs`'s pattern: the client, `supports`, the one-deadline bound, typed failures),
  surfaced into the Updates slice for the dialog load; no chat -> `unasked`, never a fake state.
  Route: worker.
- T2 (S1, S3) Web: the Extensions block draws the packages (name list + the count), its states and
  its three honest degenerate cases (no chat / not advertised / read failed); tests first; both
  languages with the copy table the design already approved (the 04-updates artboard's block, plus
  the new count line). Route: worker, after T1 (needs the contract).
- T3 The gear dot: the packages slice joins `update/dot.ts`'s waiting rule (behind packages light
  it like pi/shell do). Route: worker, with T2.

- T7 (S7 fix, maintainer-reported) Rust: the check must work for ANY open chat (fresh ones included), connect on the long deadline, and retry until one check answered. Route: worker.
- T4 (S6, S7) Rust: the derivation fix (an `unchecked` state in `PackagesView`), the `check_packages` command with its longer deadline, the held result in the pushed updates slice, the once-per-app-run trigger on the first chat ready. Route: worker.
- T5 (S8) Web: the rail dot. Route: worker, in parallel with T4 (disjoint files).
- T6 (S6, S7) Web: the "not checked yet" state in the Extensions block, "Comprobar de nuevo" also asking `check_packages`, the gear dot reading the pushed packages. Route: worker, after T4.

## Log

- L1 (2026-11-21, the maintainer): "cuando abro gentle-shell en la terminal y tengo que actualizar
  una extension me sale asi Package Updates Available ... el que use gentle-shell actualiza con
  gentle-shell update--extension y el que usa pi y el paquete de gentle-shell en pi usa pi update
  --extension. ese mensaje que sale no podriamos usarlo para avisarle al usuario en la app que debe
  actualizar una extension?" - answered yes with the explorer's facts above; the maintainer chose
  to add it to 0.9.0 ("si agregala ahi") right after merging v0.8.2.
- L2 (2026-11-24, the maintainer, with everything else working): "cuando hay una actualizacion de gentle-shell por ejemplo aparece un circulo rojo en el icono de la tuerca de configuracion pero al abrir configuracion deberia tambien aparecer algo en la seccion de actualiacion para que el usuario sepa que hay es donde debe ir, otra cosa en las extensiones no hay forma de notificar cuando hay que actualizar aunque aparezca este mensaje si uso en terminal: Package Updates Available / Package updates are available. Run pi update --extensions / Packages: - @estebanforge/pi-antigravity-bridge". Verified in the code: S6 (the false "al día") and S7 (the dot could never light without opening Settings).
- L3 (2026-11-24, the maintainer testing the 0.9.0 build with the antigravity extension behind): "abri no uno si no varios chats pero no aparece". Real probes against the installed shell (gentle-pi 4.9.0, 2026-11-24) proved: (a) the shell works - `get_update_status` with `refresh: true` answered in 5 s and named `@estebanforge/pi-antigravity-bridge` as `update_available`; a plain read returned every item `unknown`; (b) a freshly spawned child answers NO desktop request until ready (~14 s after spawn on this machine; every request sent earlier is answered at once at that moment). The cause on the app side (code read): `asking_chat` keeps only chats whose id names a session FILE (`ChatId::session_path_of`), so a fresh `new-N` chat is declined, and the once-per-run trigger is consumed by that decline and never retried.
