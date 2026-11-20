# Notifications: a tone and a system toast when the app is not in front

Feature for 0.8.x. The agent finishes a reply, needs an answer, or fails while the maintainer is in
another window: the app says so with a short tone and an OS toast. Idea read from gentle-shell-gpui
(MIT, `crates/app/src/features/alerts`), re-done on our stack: WebAudio tone + the official Tauri
notification plugin, detector in the web store (every trigger already arrives in `ViewState`).

## Specs

- S1 Fires ONLY when the window is not focused (the maintainer's choice). With the window in front:
  silence, always.
- S2 Both a short synthesized tone and an OS toast (the maintainer's choice). No audio file, no audio
  crate.
- S3 Three triggers: (a) a chat's reply finished, (b) a chat needs the user (a pending dialog /
  `NeedsAttention`), (c) a new Warning/Error notice. Nothing else (no Info, no stored/idle chats).
- S4 One setting that can be turned off in Settings. Both languages. Its default (on, "when the window
  is not in front") is the parent's proposal, NOT yet confirmed by the maintainer.
- S5 A "Try it" control in the setting plays the tone and sends one toast, so the maintainer can hear it
  and learn whether Windows allows it, without waiting for a real event.
- S6 A burst never spams: one alert per trigger kind per short window, and nothing on the first
  `ViewState` after launch or after reopening a stored chat (only real transitions).

## Tasks

- T1 (S4) Rust: `notifications` in `AppConfig` (additive, lenient like `theme`), `UiIntent::SetNotifications`,
  `ViewState.notifications`, in the settings snapshot's config slice. Route: worker.
- T2 (S1, S3, S6) Web: pure detector over prev/next `ViewState` (`src/notifications/detect.ts`) + the
  window-focus store, tests first. Route: worker.
- T3 (S2, S5) Web: tone (WebAudio) + toast (`@tauri-apps/plugin-notification`, capability
  `notification:default`, `.plugin(...)`, permission gate), wired to the detector. Route: worker.
- T4 (S4, S5) Web: Settings pane "Notifications" (design approved first, `design/notifications/`).
  Route: worker after design.

## Facts (explorer, with evidence)

- Push is one event `state-changed` carrying `ViewState` (`contract.rs:1963`, web `sessions/store.ts:74`).
- Reply finished = chat `Working` -> gone: `activity` omits idle chats and the foreground chat never shows
  `NewReply` (`crates/app/src/chats/activity.rs:42-64`, `contract.rs:24-31`).
- Needs the user = `ViewState.dialog` null->Some or `activity` gains `NeedsAttention`
  (`crates/gentle-core/src/chat/reducer.rs:546`, `activity.rs:52`).
- Failure = `ViewState.notices` gains a new `id` with level != Info; ids are monotonic
  (`crates/app/src/state.rs:78`).
- Window focus is known nowhere today (only `settings/AccountPane.svelte:51`'s own listener).
- Nothing installed: no notification plugin, no audio. Capabilities today: `core:default`, `updater:default`.
- Windows: the toast works only for INSTALLED apps; in dev it shows PowerShell's name and icon
  (https://v2.tauri.app/plugin/notification/). Whether the NSIS `currentUser` build needs the AUMID
  registered is unverified: check on an installed build. Whether WebView2 blocks `AudioContext` without a
  user gesture is unverified.

## Log

- L1 (2026-11-20, the maintainer): "otra cosa para agregar ellos tienen lo de las notificaciones que
  suenan eso es algo pesado de implementar en nuestra app?"
- L2 (2026-11-20, the maintainer, asked which): when = "Sólo si la ventana no está en foco"; kind =
  "Sonido y aviso del sistema".
