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

## Log

- L1 (2026-11-21, the maintainer): "cuando abro gentle-shell en la terminal y tengo que actualizar
  una extension me sale asi Package Updates Available ... el que use gentle-shell actualiza con
  gentle-shell update--extension y el que usa pi y el paquete de gentle-shell en pi usa pi update
  --extension. ese mensaje que sale no podriamos usarlo para avisarle al usuario en la app que debe
  actualizar una extension?" - answered yes with the explorer's facts above; the maintainer chose
  to add it to 0.9.0 ("si agregala ahi") right after merging v0.8.2.
