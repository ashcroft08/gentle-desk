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
- [x] T1 (S7, S9) artboards: 01-03 (commit 310fb4c) frame the editor INSIDE SETTINGS
      and are SUPERSEDED by S9 (kept as alternatives, not to be built). 04-06 redo it
      inside the chat. CHOSEN: 06-profile-tab.html (maintainer, L7).: the editor's directions on the canvas (profile list,
      per-agent model + thinking, fallback chain, pin, the conflict and rejection
      states). Maintainer picks one before any code.
- [~] T2 (S2, S3, S9, S6) channel through the CHAT's own child, sliced:
      T2a PURE CORE (this slice first): typed builders and answer parsers for
      `get_config` (profiles, profile_fallbacks, pins), `set_config`
      (`expectedRevision` from `writeTargets[].writeRevision`) and `apply_profile`
      (`session` / `default`, `confirmation_required` with its token), pinned with
      the doc's own example frames; plus a reader of the chat's session file for the
      LAST `gentle-pi.session-profile/v1` bind entry (L9). T2b plumbing: a generic
      per-chat desktop request in the runtime (generalising helper-stop's
      `Stopper`), the app state and the contract. T2c the read-only Profile tab.
      (reuse helper-stop's
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
- L7 (maintainer, 2026-11-17): T1 PICK - "Elijo la F": 06-profile-tab.html, a third
  Profile tab beside Chat | Helpers. 04 and 05 become alternatives. The preview
  server (port 5190) was stopped. NOTE: `gentle_design implement` reads the main
  checkout's `design/`, so the canvas hand-off cannot record the choice for this
  worktree's slug; the choice lives here.
- L8 (parent, 2026-11-17): explorer read of gentle-pi on what a chat's profile IS
  (path:line in the explorer report; gentle-pi lib/ and extensions/). FINDINGS:
  (1) NO channel read exposes a session's profile binding. `get_config` has six
  stores, none session-scoped; `get_snapshot` is identity/revision only; there is no
  profiles analogue of the permission `mode`. `apply_profile` mode `session` answers
  `{profile, mode, status:"applied", notes}` and emits `profile_applied`, both
  ephemeral (`seq` resets per session), so a later connection sees nothing. The
  binding IS persisted in the chat's session JSONL as `customType:
  "gentle-pi.session-profile/v1"` `{kind:"bind", origin:"user", name,
  modelProfiles}` (lib/session-profile-append-controller.ts:361,
  docs/session-profile-format.md:9-27); gentle-pi replays it with a corroboration
  rule (`replaySessionProfileBranch`, :37-60). `list_subagents` `modelSource:
  "binding"` proves a session is bound but not to which profile.
  (2) `session` writes only the session file plus an in-process binding; it does not
  touch profiles.json, models.json, settings, pins or other sessions. Subagents
  ALREADY running keep their routing (resolved at task-request creation); only
  launches from then on use it. It survives a child restart (replayed from disk) but
  the live ORCHESTRATOR is not re-applied after a reopen. When the profile defines
  `orchestrator.model` it switches the live orchestrator at apply time.
  (3) `confirmation_required` exists ONLY for `default` (the screen's apply-global
  path, gentle-ai.ts:6238, 6305); `session` never asks (desktop-rpc.md:880), so its
  round trip has no token.
  (4) Mid-turn: gentle-pi has no turn guard; the gates are the single apply slot
  (`profile_apply_busy`) and `set_config` refused during an apply. Whether pi
  dispatches `/gentle:desktop` while a run streams is UNVERIFIED, and the host
  channel's 5 s deadline would surface a deferred request as a timeout.
  (5) `active` is only the GLOBAL layer of pin > profile pin > active; a valid pin
  outranks it, and in a pinned repo "Make default" re-pins instead of writing
  `active`. `set_config` with `active` moves only the marker (no routing). The
  first-run seed of a missing profiles.json sets `active: "current"`, a side effect
  of the first apply.
  (6) Today's pane shows `active` from a file read, nothing session-scoped.
  CONSEQUENCES FOR THE DESIGN: the tab's state 1/2 ("this chat uses X") cannot be
  read from the channel; copy for "Use in this chat" must say it applies to
  subagents launched FROM NOW ON; "default" copy must say it can be outranked by a
  repo pin; the artboard's 8th state needs no token for `session`.
- L9 (maintainer, 2026-11-17): ruling on how the Profile tab knows which profile a
  chat uses, given L8's finding that the channel exposes no read: "Leer el archivo de
  sesión (Recomendado)". The app reads the LAST `gentle-pi.session-profile/v1` bind
  entry of the chat's own session file, labelled in the UI as an approximation (the
  branch / corroboration replay of gentle-pi is NOT reimplemented), and the request
  for a real read goes into the upstream issue draft (queue item 5) alongside the
  external-config-events one. A chat with no such entry shows the default. T2 is
  sliced T2a core (pure, TDD) -> T2b plumbing -> T2c read-only tab; writes are T3.
- L10 (parent, 2026-11-17): T2a DELIVERED, commit db35bfc (writer, pure core, no
  process). gentle-core: `rpc::answer` (StoreRevision and WriteRevision as DISTINCT
  types, so a write can never be preconditioned on the store revision),
  `rpc::config_store` (get_config / set_config builders and typed projections of
  profiles, profile_fallbacks and pins; WritableStore gate; read_config and
  write_config over DesktopClient with helper-stop's degradation ladder; the 64 KiB
  cap), `rpc::profile_apply` (Session | Default, validated 1-128 name,
  ConfirmationToken, Applied / NotApplied / ConfirmationRequired / UnknownProfile /
  Busy / ProfilesUnavailable; `session` + confirmation_required is reported as an
  unexpected answer, per L8). gentle-runtime: `session_profile` (the L9 reader of the
  last `gentle-pi.session-profile/v1` bind; newest entry terminal; malformed lines
  and CRLF tolerated; no branch replay). Verified by the parent: gentle-core +
  gentle-runtime 541/0 (69 new: config_store 34, profile_apply 22, session_profile
  13), clippy -D warnings and fmt clean. RED observed by the writer for the three
  modules (unresolved import before each existed). Frames pinned from the doc with
  their line numbers; the pins `worktree_unresolved` frame and `not_applied` are
  assembled from the doc's prose, not from a printed frame. DECISIONS the writer
  took (to confirm at review): an unknown `orchestratorMode` or an unknown
  profiles/fallbacks `kind`/`version` is a typed parse error (degrade the pane,
  never misrepresent); a missing `notes` reads as an empty list. ENVIRONMENT: this
  session's TMP pointed at a vanished directory and made 31 unrelated runtime tests
  fail with NotFound until TMP was reset; not a code defect.
  DELIVERY STRATEGY (the slice boundary the commit hook asked for): this branch is
  past 400 lines only because T1's artboards are ~6k lines of HTML for the eye, not
  for native review. It ships as separate PRs cut from main in order, never stacked
  on each other (the 0.2.0 squash lesson): PR 1 docs + design (feature doc, T0
  system sync, the six artboards); then one PR per code slice of about 400 authored
  lines or less (T2a core now; T2b plumbing; T2c read-only tab; T3 writes; ...), each
  with its own native review when its tier is high.
- L11 (parent, 2026-11-17): T2b mapped by an explorer (hop-by-hop, helper-stop as the
  template; path:line in its report). DECISIONS. (1) The chat-profile VIEW is
  SHELL-OWNED, like `ProfilesView` (bridge.rs, bridge/settings.rs), not reducer state:
  `crates/app` depends only on gentle-core while `gentle-runtime` depends on
  `gentle-app`, so the reducer cannot name `gentle_runtime::session_profile`; keeping
  the view out of the reducer avoids pushing typed profile data through `Input`.
  (2) The chat's request REUSES the per-chat `DesktopClient` that helper-stop already
  caches (`Stoppers`, one client per chat). A SECOND client map would give one chat two
  independent requestId counters talking to the same child, and `desktop-1` would
  collide (S8, the L3 advisory of rpc-settings-sync): a stop and a profile read of the
  same chat must number from ONE counter. (3) The bound profile comes from the
  session file through the `FileRead` seam (L9), not from the channel; the channel adds
  what a session file cannot: the pins of THIS chat's worktree and the
  `writeRevision` tokens T3 needs. (4) A degraded read keeps the previous view and
  says nothing (helper-stop's rule, and the `get_session_stats` rule), and the view is
  reset when the chat closes so a stalled read cannot stick the way `ModelsStatus::
  Loading` can. (5) Refresh on chat open and on `AgentSettled`; the settings watcher
  does not see session files, so it only marks the DEFINITIONS half stale.
  SLICES (each its own PR from main, about 400 authored lines or less):
  T2b-1 runtime request (`ChatCommand::ReadChatProfile`, `Outcome::ChatProfile`, the
  runtime method over the shared client, minimal compile arms in the desktop effects);
  T2b-2 shell view and plumbing (`ChatProfileView` in `ViewState`, the
  `RefreshChatProfile` intent, windowless tests, bindings regenerated);
  T2b-3 staleness and copy (refresh triggers, i18n words).
- L12 (parent, 2026-11-17): T2b-2 DELIVERED on `feat/chat-profile-view`, commit 97277d1
  (writer, then the parent: the one-line `chat_profile: null` in
  `apps/desktop/src/test-support/view-state.ts` that the new required field needs,
  outside the writer's surfaces). The shell holds a shell-owned `ChatProfileView`
  (not reducer state, L11): the session-file binding flagged as an approximation, the
  default, every profile's agents and fallback chains, the pin layers of the chat's
  worktree, and a per-store availability (read / unavailable / skipped) so a child
  that never answered is a state. `RefreshChatProfile` runs T2b-1's read for exactly
  that chat; the view is projected only while that chat is foreground; a read that
  reached no store keeps the previous view and says nothing; `Input::ChatClosed`
  drops it. No copy, no UI, no revisions or write tokens (T2b-3, T2c, T3). VERIFIED
  by the parent: cargo workspace 1504/0 (with `-j 2`: a first full run died on
  Windows error 1455, paging file too small, while linking many test binaries at once;
  an environment limit, not a defect), clippy -D warnings, fmt, bindings drift guard,
  vitest 745/745, svelte-check 0/0, eslint. REVIEW: lineage review-41a4dc9df89df8cc
  (risk + resilience + reliability, 12 paths, 1623 lines, tier high), 3 lens runs plus
  one refuter, ~$0.051 in total, every run asked first. A CRITICAL candidate (R3-001,
  `note_chat_profile` overwrites the single slot with any completed read) was REFUTED;
  APPROVED, authority burned. FIVE ADVISORY findings, recorded as BACKLOG, none
  reopens the review:
  (1) R4-late-read-reinstalls-view (with R1-closed-chat-view-restored): a read that
  answers after its chat closed re-installs the view the close dropped, because
  `note_chat_profile` does not check the chat is still open; a reopened session-path
  chat reuses its id, so the stale view can reappear until the next refresh, and the
  keep-previous rule (L11-4) can preserve it. T2b-3's refresh triggers are the natural
  place to fix it (drop a read whose chat is gone) and it needs a RED test.
  (2) R1-chat-for-path-fallback (WARNING): `RefreshChatProfile` passes the web's
  `chat_id` through `chat_for`, whose fallback turns ANY string into a session path
  that the `FileRead` seam reads; only the parsed bind name and origin are echoed
  back, never file content, and the webview is the app's own, but the read is not
  confined to the sessions store. Worth confining before T3 widens what this view
  carries.
  (3) R4-ran-single-slot-drops-read: `Ran` carries one profile read, so two
  `ReadChatProfile` commands in one run lose all but the last answer.
  (4) R4-unreadable-session-silent: an unreadable session file reads as "uses the
  default" with no log; by design (L9) but invisible to an operator.
  KNOWN LIMIT: per-layer pin marks carry only what the `pins` value names (winner /
  stale / invalid); `layers[]` lives in the projection that T2b-1 drops, so T6 may
  need it.
- L13 (parent, 2026-11-17): T2b-3 RESCOPED and DELIVERED as hardening
  (`fix/chat-profile-hardening`). DECISION: the refresh trigger is the Profile TAB
  (T2c: on open, on window focus, after an apply), NOT every chat open and not every
  `AgentSettled`. A read costs a `capabilities` round trip plus three `get_config`,
  each under the channel's 5 s deadline, against a child that may be mid-turn; paying
  that on every chat open for a tab most users never visit is waste, and it is
  reversible (the intent already exists). T2b-3 therefore fixes what the T2b-2 review
  found: (1) a profile read that answers AFTER its chat closed no longer re-installs
  the view (`note_chat_profile` asks the app state whether the chat is still open;
  tested in the same run and in a later one, and reopening shows no stale view);
  (2) the session-file read is CONFINED, lexically, to `<agent dir>/sessions/`,
  ending in `.jsonl`, with no `..` component, no filesystem canonicalization, and the
  seam is never asked otherwise (the chat then uses the default); the sessions root
  is `SettingsPaths::sessions`, the same rule `sessions::startup` gives the index;
  (3) `Ran` carries every profile read (a Vec), so two reads in one run both land;
  (4) a session file that exists but cannot be read now leaves a stderr line, the
  result unchanged. VERIFIED by the parent: workspace 1513/0 (`-j 2`), clippy
  -D warnings, fmt. Known residuals, recorded not fixed: a close racing between the
  open check and the install can still install a view (closing it needs a lock-order
  change in `apply`; the shell forbids nested state locks); "open" is the app state's
  chat entry, so a chat whose child died unexpectedly (entry kept, ended) still
  accepts a read; the single view slot means the last noted read wins when two chats
  are read in one run.
- L14 (parent, 2026-11-17): T2b-3 REVIEWED AND APPROVED, lineage
  review-a09b1c5b66cabb43 (risk + resilience + reliability, 9 paths, 608 lines, tier
  high): 3 lens runs, no refuter needed, about $0.034, each asked first; authority
  burned. SIX advisory findings, none reopens it; four are the residuals L13 already
  named (the close racing between the open check and the install: R1-close-race-stale-
  view, R3-001, R4-late-close-race-window; an ended chat still counting as open:
  R4-ended-entry-still-read). TWO are new and worth keeping:
  (1) R3-002: the open check is keyed by ChatId, not by a chat GENERATION, so a late
  read from a closed incarnation can be accepted if the same id is reopened before the
  read is noted; the tests cover a late read before the reopen but not close, reopen,
  old read. A generation counter on the chat entry would close it together with the
  lock-order race.
  (2) R1-lexical-confinement-symlink: the confinement is lexical only, so a symlink
  planted inside the sessions store would still be followed by the real reader to a
  target outside it. Needs local write access to the sessions directory and matches
  the rule the session index applies; hardening, not a reachable new risk.
- L15 (parent, 2026-11-17): T2c DELIVERED on `feat/profile-tab` (writer, web only): the
  third tab `Profile` beside Chat | Helpers (roving tabindex and arrows as the others),
  read-only. It draws artboard 06's states 1 (uses the default), 2 (bound, "this chat
  only"), 5 (pin layers with winner / stale / invalid marks; `worktree_unresolved` as
  "Pins need a Git workspace") and 10 (calm "could not read this chat's profile
  configuration" with Try again); a partial read draws what was read and marks each
  missing store where its content would be. The L9 approximation is one honest line in
  both states. One RefreshChatProfile on open, one per window focus while open, one per
  Try again, coalesced (one in flight, one queued, cancelled on close), none without a
  foreground id; the panel is mounted but reads only while it is the open tab. NO
  edit control exists, not even disabled, and no "Edit in terminal" (T3+). Copy EN+ES in
  `profile/copy.ts`; every value is a design/system.md token. VERIFIED by the parent:
  vitest 788/788 (42 new), svelte-check 0/0, eslint. THE DRAWING WAS NOT SEEN
  RENDERED: no browser here, so contrast, the chevron, scrolling and the 880 px column
  are unverified by eye. DECISIONS where the artboard was silent: the four artboard
  groups become one group per profile (agents and chains inside) plus a Pin group,
  with the profile the chat RUNS open first; the artboard's "read at 14:18" meta line
  is dropped (no read time in the contract); Spanish invented where the app had none
  (`Perfil fijado`, `Cadena de respaldo`, `Razonamiento`, `Nivel`). FOLLOW-UP T2d, not
  built: Settings > Profiles still reads as global (S9's original complaint). It needs
  its scope labelled ("Default for new chats") and a pointer to the chat's tab; small
  and web-only, so its own slice.
- L16 (maintainer + parent, 2026-11-17): FIRST REAL-WINDOW LOOK AT THE PROFILE TAB, and
  it FAILED: "pongo en perfil y la app se traba, se pone lento todo, no puedo navegar en
  esa seccion". T2c passed 788 jsdom tests and was never seen rendered (L15 said so);
  this is the cost of that gap. DIAGNOSIS (an explorer read the code; nothing observed
  in a window): no read re-triggers another (RefreshChatProfile comes only from the
  web) and the blocking read runs on a worker thread, so navigating does not wait
  behind it. What the PR added was a per-push cost: (1) the panel was mounted
  permanently, hidden, so its subtree was re-patched on every state push even on the
  Chat tab; (2) `ProfileGroup` rendered every collapsed profile's agents, chains and
  pins, O(profiles x agents x chains) per render; (3) Rust clones `chat_profile` into
  EVERY push (one push per child event, a streaming reply is about one per event) and
  the store does not preserve identity, so the prop was new every time. FIXED, each RED
  first, vitest 793/793: the profile object keeps its identity when a push carries an
  equal value; the panel is mounted only while its tab is open; a group builds its body
  only when open. NOT FIXED, recorded: Rust still sends `chat_profile` in every push
  (only the first-read case needs it); and `chat/view.ts`'s identity guard is false for
  most slices (`notices`, `images`, `helpers`... are rebuilt per push), a PRE-EXISTING
  waste that the Profile tab merely made visible. NOT EXPLAINED: this accounts for
  slowness while a chat streams, but not obviously for a freeze with the chat idle. The
  maintainer's answer to "does it freeze with the chat idle, or only while it is
  working?" decides whether this fix was the cause or only a contributor; if it
  freezes idle, the next suspects are the focus-driven re-read and the whole-session-
  file read per refresh. LESSON: a UI slice is not closed by jsdom; the next UI slice
  (T2d, T3) is looked at in a real window BEFORE its PR is opened.
- L17 (maintainer, 2026-11-17): SECOND real-window look WITH the perf fix (43280ef):
  "igual, se congelo esta vez hasta dejo de responder la app y se congelo por completo".
  NOT slowness, a hard hang: my perf fix was not the cause (or not the whole cause). The
  dev window's log for that run ends with EIGHT repeats of `chat command thread
  unavailable: Acceso denegado. (os error 5)` (commands.rs:1036, the thread a
  `ChatCommand` runs on could not be spawned), which means the tab asked for the chat's
  profile about eight times and NO read ever ran. Two facts follow and are recorded
  before any further attempt: (a) an ask whose thread cannot be spawned leaves the tab
  in `reading` for ever (nothing tells it the read failed), so a user retries, each
  retry is another thread, and the app never answers; (b) why CreateThread answers
  "access denied" here is NOT established (insufficient handles, teardown, or a retry
  storm) and must be measured, not guessed. Decision taken: PR #192 is not to be merged
  while its tab can freeze the app; the read plumbing under it (T2b-1..3) is already in
  main and is unaffected. Next steps belong to a fresh session with a real window each
  round: instrument the asks and the pushes (count + timestamps), fix the liveness hole
  first (a failed spawn must answer the web, not vanish), THEN reopen the tab.
- L18 (parent, 2026-11-17): THE FREEZE MEASURED AND FIXED (eb82482). Local, uncommitted
  instrumentation (a 500 ms sampler of RSS / OS threads / counters, a line per
  `RefreshChatProfile` in, per chat-command spawn/run/end and per push with its bytes, and
  the web's own lines forwarded on the same clock). Run 1 (chat Curriculum): the web sent
  `RefreshChatProfile` every ~7 ms from the moment the tab opened; 4,570 asks in 25 s, each
  a `chat-command` thread serialised on the child's client (~100 ms per read), so 4,282
  were alive at +62 s, ~4,300 OS threads, 264 MiB, 22 s per read, 6,350 pushes of ~28 KB
  (180 MB) into the WebView. The `os error 5` of L17 is the END of that storm, not its
  cause. Run 2: ONE `ProfilePanel` instance; its read `$effect` was cleaned up and re-run
  on every push. Cause: the effect read the `chatId` prop, which App.svelte passes as
  `chat.chat` off `$derived($chrome)` - a new object per push - so the effect subscribed
  to the object, not to the id. Fix: `const chat = $derived(chatId)`. RED
  `ProfilePanel.push.test.ts` (mounted as App mounts it): 20 pushes naming the same chat
  -> 21 reads; GREEN 1. Run 3 (real window, maintainer: "ahora si funciona"): 3 reads in
  160 s (open + focus), RSS 51 MiB flat, no failed spawn. Still open: the liveness hole of
  L17(a) (a failed spawn must answer the web) and an optional Rust-side bound of one
  profile read in flight per chat - a separate change against main.
- L19 (maintainer, 2026-11-17, verbatim): "ahora si funciona, una cosa falta Fallback y
  Modo de orquestación, tiene que tenes lo mismo que lo de perfil en configuracion."
- L20 (maintainer, 2026-11-17, verbatim, answering whether the mode is this chat's):
  "Se supone que en la terminal al cambiar eso, cambia la forma en que se trabaja en ese
  chat pero en la app lo que vamos a tener es que solo en cada nuevo chat se aplicaria"
  then "una cosa en si eso cambia para todos los chats igual en la terminal no es algo que
  afecte solo al chat en que se trabaja". Reading: the orchestrator mode is GLOBAL (not
  per chat); the Profile tab carries the same switch and the same Fallback pair as
  Settings > Profiles.
- L21 (maintainer, 2026-11-17, verbatim, a proposal under discussion - not yet a ruling):
  "otra cosa, creo que tener en configuracion es algo que no tiene sentido, creo que
  eliminamos en configuracion y dejamos en cada chat, porque lo que afecta de forma
  global se mantedria ahi tambien. que te parece?"
