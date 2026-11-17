# profiles-editor: edit profiles, fallbacks and pins from the app

The roadmap's step 3 of the desktop RPC adoption ("Profiles and pins editor: the
largest one, sliced as read+state, model and effort per agent, fallback-chain
editing, then the per-repo pin. It is safe now because `set_config` writes through
gentle-pi's own writers and `writeRevision` rejects the write when the file changed
underneath. Model pickers reuse the 0.1.11 model catalog."). Maintainer, 2026-11-16:
"No quiero la opcion mas facil de implementar, quiero implementar la mejor aunque
sea larga" and "en una app de escritorio mover trabajo de la terminal a la interfaz
es valor de producto".

## Specs (verbatim)

- S1 (roadmap): "a GUI Profiles/pins editor is now planned" - the pane stops being
  read-only: profiles (agent -> `{model?, thinking?, tier?}`), fallback chains and
  the per-repo pin are edited from the app.
- S2 (channel spec, gentle-pi docs/desktop-rpc.md:299): `set_config` params are
  exactly `store`, `data`, `expectedRevision`; "`data` is the COMPLETE new content,
  not a patch". Writable stores: `profiles`, `profile_fallbacks`, `pins`;
  `models`, `subagents`, `managed_assets` answer `read_only`.
- S3 (channel spec, :265-273): `expectedRevision` is compared against
  `get_config`'s `writeTargets[].writeRevision` - "never against `revision`" -
  and is re-read per store per write. A stale one answers `revision_conflict` and
  nothing is written.
- S4 (channel spec, :1143-1150): "a profile write STORES the definition and does
  not apply it", and `set_config` answers `profile_apply_busy` while an
  `apply_profile` runs.
- S5 (L5 of rpc-settings-sync, maintainer-accepted): "a control child while the
  user edits is justified" - a TEMPORARY control child alive only while the editor
  is open (measured: ~400 MB, 5-10 s to answer). Per-chat features keep each chat's
  own child.
- S6 (rpc-settings-sync S4, standing): new copy bilingual EN/ES; the maintainer
  approves any real-child command before it runs; a child that does not answer
  degrades to today's read-only pane, never breaks.
- S8 (channel spec :839-860 + maintainer, 2026-11-17): saving does not make a profile
  usable. `apply_profile` has `mode: "default"` ("Make default for new sessions",
  what new chat children read) and `mode: "session"` (bound to the temporary
  control child's own RPC session: useless for the app). It reuses
  `/gentle:profiles`' own actions - diff confirmation (`confirmation_required`
  with a token), the pinned-repository rule and the rollback. Maintainer: "solo
  edito y guardo pero no lo puedo usar o que mismo?" - so the editor saves AND
  offers Make default for new sessions; live chats keep the routing they started
  with.
- S9 (maintainer, 2026-11-17, RULING - supersedes the Settings framing of S1/S5):
  "en la terminal nosotros tenemos abierta una sesion y editamos/creamos [perfiles]
  desde una sesion y ahi podemos elegir guardar, usarla en esa sesion o usarla
  como default, en cambio en la app lo tengo en configuracion algo que lo veo
  confuso ya que hace parecer como si fuera global". The editor lives INSIDE THE
  CHAT, with the terminal's three outcomes: Save, Use in this chat
  (`apply_profile` mode `session`, sent to THAT chat's own child), Make default
  (`mode: default`, global, labelled as such). Evidence for the scope: the repo pin
  needs a session with a working directory in a resolvable worktree (spec :263,
  :381 `config_unavailable`), which a workspace-less control child cannot have; and
  every real chat child already runs with the channel on (helper-stop). Settings >
  Profiles stays the GLOBAL view (the default and the definitions, scope labelled)
  and points at the chat for editing. The temporary control child of S5 is
  expected to disappear (to be confirmed by T2's approved probe).
- S7 (session queue, 2026-11-16): "Primero leer la spec de set_config y la forma de
  un perfil NUEVO (no verificada), artboards en el canvas, hijo de control
  temporal mientras se edita, set_config con writeRevision." Artboards first.

## What the spec read established (explorer, path:line in the Log, L2)

- The writer's accepted shape is documented only for a small part: `kind`
  `gentle-pi.agent_model_profiles`, `version` 1, `active?`, `profiles` (name ->
  agent -> `{model?, thinking?, tier?}`), `workflow?.orchestratorMode`
  (`organic`|`full-delegation`, non-default only). Fallbacks: `kind`
  `gentle-pi.profile_model_fallbacks`, chains of at most 8 steps `{model,
  thinking?}`, a model at most once per chain, `*` = profile-wide default. Pins:
  exactly `{source: "local"|"repo", profile: string|null}` (null clears).
- NOT documented (the risky half, never guess): the "safe-key policy" (profile
  name / agent key charset and length, model-id syntax), the allowed `thinking`
  values for profiles, what `tier` means, a minimal valid profile, bounds on
  counts. gentle-desk's own READER already encodes the policies it needs
  (`crates/runtime/src/profiles.rs`: thinking off/minimal/low/medium/high/xhigh/
  max, chain length 8, `*` key, <=64-char slug names, `__proto__|constructor|
  prototype` refused) but it is not the writer's normalizer.
- Consequence for the design: the editor validates only what the reader already
  enforces, and treats the owner's `invalid_data` answer as the authority - it
  shows the rejection and writes nothing. A NEW profile is built from a profile
  the user already has (duplicate, then edit) or from a minimal one the first
  real probe confirms; that probe is the maintainer's to approve.
- The app does not watch the pin files (`<git-common-dir>/gentle-ai/
  profile-pin.json`, `<worktree>/.pi/gentle-ai/profile.json`), so a pin edited in
  the terminal is invisible until a refresh; the editor must read pins on open.
- Nothing in the app reaches the channel yet: no `set_config` builder, no control
  child, no per-store param types. `DesktopHub`/`HostDesktopChannel` and the
  generic `DesktopClient` exist (rpc-settings-sync T2) but are wired only to chat
  children (helper-stop).

## Tasks (sliced as the roadmap says; each ends with its own check)

- [x] T0 (S7) design system sync: `design/system.md` is older than
      `apps/desktop/src/app.css`; regenerate it (read-only over the sources) so the
      artboards use current tokens. Route: writer, design files only.
- [~] T1 (S7, S9) artboards: 01-03 (commit 310fb4c) frame the editor INSIDE SETTINGS
      and are SUPERSEDED by S9 (kept as alternatives, not to be built). 04-06 redo it
      inside the chat. The maintainer's pick is pending.: the editor's directions on the canvas (profile list,
      per-agent model + thinking, fallback chain, pin, the conflict and rejection
      states). Maintainer picks one before any code.
- [ ] T2 (S2, S3, S9, S6) channel through the CHAT's own child (reuse helper-stop's
      per-chat `DesktopClient`; no control child unless the probe shows a need),
      behind the swappable channel seam;
      `get_config` read + typed answers; silent degradation to read-only. TDD on
      the fake channel; the real-child proof waits for the maintainer's approved
      script (home temporal, `GENTLE_SHELL_NO_AUTO_SETUP=1`, cleanup).
- [ ] T3 (S2, S3, S4) write path: `set_config` with `expectedRevision` from
      `get_config`, `revision_conflict` -> re-read and ask, `invalid_data` ->
      show and keep the draft, `profile_apply_busy` -> wait/retry copy, whole
      document replacement.
- [ ] T3b (S4, S8, S9) apply: `apply_profile` mode `session` (this chat, the
      terminal's "Use in this session") and `default` with the
      `confirmation_required` round trip, the `unknown_profile` and
      `profile_apply_busy` answers, and the copy that says live chats keep their
      routing.
- [ ] T4 model + thinking per agent (reuses the 0.1.11 model catalog).
- [ ] T5 fallback-chain editing (<=8 steps, no repeated model, `*` default).
- [ ] T6 the pin (local/repo, null clears; read on open since it is not watched).
- [ ] T7 (close) battery, real-window check, roadmap note, version.

## Log

- L1 (maintainer, session queue 2026-11-16, repeated): editor of profiles/pins
  "la mejor versión aunque sea larga"; first read the `set_config` spec and the
  shape of a NEW profile (unverified), artboards on the canvas, temporary control
  child while editing (spike decision: ~400 MB and 5-10 s startup), `set_config`
  with `writeRevision`. Today the panel is read-only and updates itself when the
  terminal writes (native watcher).
- L2 (parent, 2026-11-17): the spec read (explorer, gentle-pi docs/desktop-rpc.md
  :246-262 write targets, :265-273 writeRevision, :275-302 set_config, :322-372
  the three payload shapes, :375-402 check order and the missing cross-process
  lock, :428-441 `revision_conflict`, :934-960 error codes, :996-1020
  `config_changed` with `source: "desktop"`) and gentle-desk's own side
  (`ProfilesPane.svelte:61-118` read-only, `commands.rs:739-751 refresh_profiles`,
  `settings/profiles.rs:57-100`, `contract.rs:326-371` the views,
  `settings/watch.rs:44-96` two booleans only). Open spec risks listed above.
- L3 (maintainer + parent, 2026-11-17): rulings at pickup. (a) Sync `design/system.md`
  before the artboards ("Si, sincronizar antes"). (b) Scope: the maintainer asked
  "Osea solo edito y guardo pero no lo puedo usar o que mismo?" - right: a save
  alone is not usable. The parent had recommended save-only; the spec read
  (`apply_profile` :839-860) shows that would force the terminal for the last
  step, against the feature's purpose, so S8 and T3b put Make default for new
  sessions inside this feature.
- L4 (parent, 2026-11-17): T0 delivered, commit e30f8a3 (writer, design files only):
  `design/system.md` re-synced against `theme.css` + `app.css` (87 token rows: 14
  colors x 6 theme blocks, `--chat-max`, `--swatch`, `--d`; a Python diff against
  theme.css found 0 mismatches; Cute and Sexy split into their own tables because
  the merged one broke the one-value-per-row rule; `system` got its own block),
  new Layout / Component-scoped / Motion sections and the component rules app.css
  gained (menu hover band `--tint`, the Working pulse, the helper stop square).
  `design/system.html` rewritten (1440 px, no external URLs, 16/16 tokens
  documented). Not exercised in a browser. NOTE: `gentle_design list` reads the
  main checkout, so it keeps reporting the old system.md until this branch lands.
- L5 (parent, 2026-11-17): T1 artboards drawn by a writer, commit 310fb4c, in
  `design/profiles-editor/` (linked to this doc): 01-edit-in-place.html (the ledger
  rows become the fields, sticky save bar), 02-detail-panel.html (the ledger stays
  read-only, a profile opens in a 322 px panel), 03-focused-mode.html (an explicit
  Edit mode with a Cancel / Save / Save and make default footer). Each shows 11
  states through an in-artboard switcher: idle, connecting, dirty draft, model
  picker, fallback chain (8-step limit, repeated model), pin, revision_conflict,
  invalid_data, apply confirmation, saved-but-not-applied, degraded read-only.
  Self-check against design/system.md passed (every hex maps to a dark token; no
  external URLs; reduced motion respected). Not rendered in a browser by the
  writer. NOT drawn: profile_apply_busy, unknown_profile, the two other
  confirmation sentences (empty / orchestrator-only profile), creating a new
  profile (its shape is the unverified half of the spec), read_only refusals.
  The other artboards are alternatives until the maintainer picks one. Open
  questions the writer raised, to settle before T3: (1) does the whole-document
  write carry `workflow.orchestratorMode`; (2) `active` lives in the profiles
  document while the pin is a separate file - what does the pane call "the
  default" and can the editor write `active` at all; (3) the channel's own
  confirmation text is English - translate the app's copy or the channel's; (4)
  per-profile fallback chains (T5 scope) or only `*`; (5) a native Spanish pass.
  PREVIEW: `gentle_design` reads the main checkout's `design/`, which lacks this
  slug, so the artboards were served from the worktree by a static loopback
  server (port 5190) instead of the canvas.
- L6 (parent, 2026-11-17): the maintainer's ruling S9 (in-chat editor) came from his own
  comparison with the terminal ("hace parecer como si fuera global"); the parent had
  framed the editor as a Settings screen and had discarded `apply_profile` mode
  `session` as useless because it bound to a control child. Both were wrong for the
  same reason: every chat child already speaks the channel. Spec evidence for the
  scope: pin writes need a session with a working directory in a resolvable worktree
  (desktop-rpc.md :263, :381, :947), which a workspace-less control child cannot have.
  T1 redone: `design/profiles-editor/` 04-chip-and-popover.html (profile chip in the
  composer row beside the model chip, popover, editor sheet), 05-header-and-sheet.html
  (chip in the header beside the pencil, a 470 px slide-over), 06-profile-tab.html (a
  third Profile tab beside Chat | Helpers). 01-03 stay as superseded alternatives
  (titles marked in design.json). Each shows 11 states, including the three outcomes
  (Save / Use in this chat / Make default with the channel's confirmation and "chats
  already open keep their routing"), the no-worktree pin case, revision_conflict,
  invalid_data, a working/ended chat that cannot apply, and Settings > Profiles as the
  GLOBAL view. NOT drawn: new-profile creation, profile_apply_busy, unknown_profile,
  read_only, the empty/orchestrator-only confirmations. Not rendered in a browser by
  the writer. OPEN, to verify LIVE with a probe the maintainer approves before T3b:
  (a) whether `apply_profile` mode `session` on a chat's own child re-routes model AND
  thinking of subagents already running, survives a restart of that chat's child, and
  leaves other chats untouched; (b) whether `session` mode also answers
  `confirmation_required`; (c) whether a chat that is mid-turn may apply at all;
  (d) whether the writer refuses the whole `pins` store or only `repo` without a
  worktree; (e) `active`: writable by the editor, or only via `apply_profile`.
