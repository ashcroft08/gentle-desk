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

- [ ] T0 (S7) design system sync: `design/system.md` is older than
      `apps/desktop/src/app.css`; regenerate it (read-only over the sources) so the
      artboards use current tokens. Route: writer, design files only.
- [ ] T1 (S7) artboards: the editor's directions on the canvas (profile list,
      per-agent model + thinking, fallback chain, pin, the conflict and rejection
      states). Maintainer picks one before any code.
- [ ] T2 (S2, S3, S5, S6) control channel: a temporary control child opened when
      the editor opens and closed with it, behind the swappable channel seam;
      `get_config` read + typed answers; silent degradation to read-only. TDD on
      the fake channel; the real-child proof waits for the maintainer's approved
      script (home temporal, `GENTLE_SHELL_NO_AUTO_SETUP=1`, cleanup).
- [ ] T3 (S2, S3, S4) write path: `set_config` with `expectedRevision` from
      `get_config`, `revision_conflict` -> re-read and ask, `invalid_data` ->
      show and keep the draft, `profile_apply_busy` -> wait/retry copy, whole
      document replacement.
- [ ] T3b (S4, S8) apply: `apply_profile` mode `default` with the
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
