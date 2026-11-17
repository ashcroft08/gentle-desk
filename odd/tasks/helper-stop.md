# helper-stop: stop one running helper from its thread bar

The roadmap's "0.4.x: stopping a helper" (agreed 2026-11-16, maintainer:
"agenda helper Stop como feature propio"), next in the session queue after the
0.4.0 release. First real consumer of the /gentle:desktop transport (rpc-settings-sync T2) using each chat's own child.

## Specs (verbatim quotes)

- S1 (roadmap): "stop ONE running helper (subagent) without touching the chat or
  its other helpers."
- S2 (roadmap, what it needs): "a new intent (`StopHelper`) through the contract,
  the app-side execution that stops that helper's work and marks it
  `Ended`/`Cancelled` while the chat keeps running, and the helper thread bar's
  red square (the artboard's own `i-stop`), enabled only while that helper is
  running."
- S3 (roadmap correction 2026-11-16, T1's real probe of gentle-shell 4.8.0):
  "the desktop RPC channel DOES offer the capability - `cancel_subagent` and
  `cancel_job` are listed in `capabilities` - so this feature is built on the
  channel's transport (rpc-settings-sync T2), not on a backend that does not
  exist; only the app's own contract lacked it."
- S4 (roadmap): "Bilingual copy (`Stop` / `Detener`). Capability jump: minor bump
  when it ships."
- S5 (session queue, ordering): "Artboard primero (el cuadrado rojo en la barra
  del hilo del helper)."
- S6 (rpc-settings-sync L5 decision): "per-chat features (subagents, jobs,
  helper Stop) use each chat's own child."
- S7 (standing rules, rpc-settings-sync S4): "the maintainer approves any
  real-child command before it runs"; new copy bilingual; degrade silently (never
  break) when the channel is absent or the child does not answer.
- S8 (rpc-settings-sync L3 advisory, carried): helper/desktop requests must use
  ONE requestId counter per chat (ids are only unique per client today).

## Tasks

- [x] T1 (S5) artboard: the red square (`i-stop`) in the helper thread bar of
      design/gentle-desk-v2/01b-tree-helpers.html, right side of `ht-bar`,
      enabled only while that helper is running (the data already carries
      `stop:true`/`stop:false`); edit that artboard only, then open the canvas.
- [ ] T2 (S2, S3) contract: `StopHelper` intent through the contract (Rust
      `contract.rs` + TS `bindings.ts`), mapped in `intent.rs`, reducer answers
      for a helper that is not running refused/ignored per the state's rules.
- [ ] T3 (S2, S3, S6, S7, S8) app side: execution sends `cancel_subagent` through
      the foreground chat's own child (the T2 `DesktopClient`/`DesktopHub`
      transport), marks the helper `Ended`/`Cancelled`, the chat keeps running;
      silent degradation when the channel is inactive or the child times out
      (today's behaviour: no stop possible). TDD where testable.
- [ ] T4 (S2, S4) web: the helper thread bar's stop button sends the intent,
      enabled only while that helper is running; bilingual copy Stop/Detener.
      TDD.
- [ ] T5 (close) battery + real-window check (stop a real helper, the chat keeps
      running) + roadmap note. Version per the maintainer.

## Log

- L1 (maintainer, session queue 2026-11-16): helper Stop is queue item 3; "el
  canal de gentle-shell ofrece cancel_subagent y cancel_job (confirmado en el
  probe real); usa el hijo de cada chat y es el primer consumidor real del
  transporte /gentle:desktop ya construido (gentle-core rpc::desktop + backend
  chat_host::desktop). Spec en el roadmap, sección "0.4.x: stopping a helper".
  Artboard primero (el cuadrado rojo en la barra del hilo del helper)."
- L2 (parent, 2026-11-17): T1 delivered — the stop button lives in the helper
  thread bar's right side (after the h2, `tip-r` so the tooltip clears the
  right edge): a 32×32 `--danger` icon button whose hover band fills red (the
  #175 hover style's rule: the hover is the fill), `#i-stop` at 14px, disabled
  (faint, no hover) whenever the helper's data says `stop:false`, with the tip
  reading "Stopped" and the aria-label naming the helper once stopped. Clicking
  it on the artboard flips that helper to `ended` in the list and the bar, so
  the direction shows the outcome: one helper stops, the chat keeps running.
  Canvas opened at the maintainer's URL; only 01b-tree-helpers.html edited.
