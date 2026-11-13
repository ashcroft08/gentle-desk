# model-selector-0.1.11: choose the model and thinking level from the composer

Version 0.1.11 of the tauri-settings-roadmap (odd/tasks/tauri-settings-roadmap.md).
The GUI asks; the shell does the work: the composer gets a model chip like the
screenshot the user shared (model name, thinking level, dropdown arrows), fed by
existing pi RPC commands. No new RPC command types are invented; no gentle-pi file
is written. Backed by the verified facts in the roadmap (2026-11-12 section).

## Specs

- S1: The composer shows the current model and thinking level of the chat's session,
  as the user described: "se podria implementar algo asi para poder seleccionar un
  modelo para la sesion" (screenshot: "DeepSeek-V41-Flash High ▾").
- S2: The user wants no new commands anywhere: "la cosa es pasar los comandos para
  usarse mediante la interfaz, no agregar nada nuevo."
- S3: A dropdown lists the models pi reports (RPC `get_available_models`); choosing
  one sends RPC `set_model { provider, modelId }` and updates the chip from the
  response's Model object. Selection is session-scoped, per the roadmap.
- S4: A thinking-level control sends RPC `set_thinking_level { level }`; levels are
  off|minimal|low|medium|high|xhigh|max; the per-model supported list comes from
  `get_available_thinking_levels`.
- S5: The list only contains providers whose auth resolves (auth.json, runtime key,
  or env var). If the list is empty or a request fails, the chip shows that state
  instead of inventing entries; the composer's text entry keeps working.
- S7: Placement and dropdown (user decision 2026-11-13, after the working chip):
  "es mejor poner el selector abajo del cuadro donde se escribe" — the chip's
  pills live in their own row BELOW the composer form, not inside it. And
  "desplegable propio con el tema de la app (lista oscura, filtro de búsqueda)" —
  no native select popups: a custom themed listbox, with a filter input, keyboard
  navigation (arrows, Enter, Esc) and click-outside close. Same dispatch intents
  as before; the child's truth and drawn failure states do not change.

## Tasks

- [x] T1 (S3, S4) gentle-core: add `GetAvailableModels`, `SetModel{provider,modelId}`,
      `SetThinkingLevel{level}`, `GetAvailableThinkingLevels` to `RpcCommand`
      (crates/gentle-core/src/rpc/types.rs, codec.rs) + codec tests — RED observed
      (4 E0599); GREEN 23/23 in tests/codec.rs. Commit 6af6296.
- [x] T2 (S3, S4, S5) app layer: intents to request list/apply/set-level; parse
      responses in events.rs (models list, Model object, thinking success); keep the
      current model/level in state (crates/app/src/{intents,input,chats/intents,
      events,state}.rs) + tests — RED observed (missing types, Loading, Failed,
      notice on level failure, stale chip on chat open); GREEN: gentle-app 9 model
      tests + whole crate, gentle-runtime open_chat rewritten for the multi-request
      open (get_messages, get_state, get_available_models). Commit 4e99e83.
- [x] T3 (S1, S3, S4, S5) tauri + web: expose the chip state through the contract,
      a ModelChip with two dropdowns in the composer, copy, and a bridge test
      (apps/desktop/src-tauri/src/contract.rs, intent.rs, state_view.rs;
      apps/desktop/src/composer/ModelChip.svelte, bridge/dispatch) — bindings
      regenerated via GENTLE_DESK_REGEN_BINDINGS; 7 chip tests. Commit ed4b0e4.
- [x] T4 (S5) empty/failed-list states: the chip renders a "no models" state and the
      send box is untouched + test — covered by ModelChip tests (loading, empty,
      failed, no chip) and app tests (Empty/Failed/notice); the send box is a
      separate component and the chip adds no disabled rule to it.
- [x] T5 (S6) final verification: focused tests, full suite, clippy/fmt, and a note
      in the roadmap that 0.1.11 shipped — automated: full cargo workspace (91
      binaries, 0 failed), web vitest 400, svelte-check 0, clippy -D warnings 0,
      fmt applied, contract drift guard ok. Native review of the feature and of
      the design/layout slices approved and acknowledged. Version bumped to
      0.1.11; the roadmap notes it shipped.
- [x] T6 (S1, L9) chip inline: the two dropdowns as pill selects inside the
      composer form, right of the box, left of Send, with a `chevronDown` icon
      added to the icon set — web tests green (393), svelte-check 0. Commit 4719ea4.
- [x] T7 (L9) textarea shrink: the height effect drops the inline height to
      `auto` before measuring; a stubbed scrollHeight (content vs current
      height, as a browser reports) reproduces the stuck-expanded bug RED and
      the fix GREEN. Commit 4719ea4.
- [ ] T8 (S7) pill row below the composer form + custom themed listbox
      (filter, keyboard, click-outside) for the model and level pills + tests.

## Verification plan

- RED first for each RPC variant codec round-trip and each response parser.
- Suite: cargo test in the workspace plus the web bridge test; clippy -D warnings.
- Manual window check against a stand-in shell: chip shows current model, dropdown
  lists models, choosing one switches the session; the thinking level list follows
  the chosen model.

## Log

- L5: Slice boundary: T1 commit 6af6296 (~225 lines) and T2 commit 4e99e83 (~480
  lines, most of it tests). T3 stays its own commit so native review can take the
  contract and the component alone.
- L6: A node_modules cache file slipped into the T3 commit; it was untracked from
  the index and the commit amended (ed4b0e4), and node_modules/ went to .gitignore.
- L7: The native review (lineage review-4488bf5658032c09, medium tier, lenses
  reliability + resilience + refuter) returned five deterministic CRITICAL
  findings, all introduced by this candidate: nothing ever dispatches
  get_available_thinking_levels (the level dropdown could never render);
  OpenSession cleared the chip without refetching; returning to an
  already-running chat asked nothing, leaving the chip on a loading line
  forever. All three explain the user's window report. Bounded correction
  committed as 9976f29 (149/150 planned diff lines, budget 200): every open
  path and the switch_session reply now ask get_state, get_available_models
  and get_available_thinking_levels; full suite green again (91 binaries).
- L8: The provider stop corrected_candidate_unavailable offered a recovery to
  successor lineage review-99ac896bdebadaa0 (disposition scope_changed). The
  gentle_review recover invocation fails its input type check ("Review
  controller input must be a string") across eight submissions, including the
  exact six provider inputs plus the envelope-required actor/reason; the
  facade cannot currently accept the serialized input. Authority stays parked
  in correction_required with the recovery authorized; no further recover
  attempts without a user decision.
- L9 (user requests after the window test, verbatim): "se quedo en Loading
  models... o no se si demora"; "quiero el diseño de esta forma" (chip inline in
  the composer bar, right-aligned next to Send) — done in T6; "el textarea se
  queda expandido" — done in T7; "el thinking no sale para elegir" — L11.
- L10: The real window folds every event through chats::apply_event and drops
  shared Command::Send (commands.rs:630), so the chip never parsed replies and
  its selections went nowhere in the chats shell — the user's "Loading
  models..." report. Fixed by chip.rs shared parsing (foreground-only) and the
  three intents as ChatCommand::Send (commit 159e840).
- L11: pi answers get_available_thinking_levels as { levels: [...] }, not a bare
  array; the parser only took arrays, so the level pill never rendered. Fixed
  with an object parse plus a bare-array fallback (commit bc17f47).
- L12: Native review of the chats-fix candidate (lineage review-003af3193db3ff9f,
  medium tier, reliability + resilience + refuter + targeted validator) returned
  two deterministic CRITICAL findings — the Existing open never ran the chip asks
  (effects.rs) and a set_model reply left the levels owned by the old model —
  both fixed in commit dec5d68 (151 diff lines of the 100 planned, budget 200;
  the validator admitted the correction). The review was APPROVED and the
  acknowledge-approved burned (review-ai.review-acknowledged/v1); three
  informational follow-ups remain: ModelChip.svelte:44, chip.rs:96-99,
  chats/intents.rs:46-49. Delivery follows ordinary repository policy.
- L13 (design decision after the working chip): "antes de probar, modifica el
  diseño es mejor poner el selector abajo del cuadro donde se escribe y
  desplegable propio con el tema de la app (lista oscura, filtro de búsqueda)",
  then relaunch for review — T8.
  "quiero el diseño de esta forma" (screenshot: the chip inline in the composer
  bar, right-aligned next to the send button, model name bold, level faint,
  chevron) — NEW scope, pending task; the textarea stays expanded after sending
  long text (height never resets) — NEW scope, pending task.

- L1 (user request, verbatim): "Quiero hacer investigaciones, para ver si a futuro se
  puede hacer ciertas implementaciones en la app, Los comandos /login, /logout,
  /model pueden utilizarse o pueden transformarse para que el usuario haga eso
  mediante interfaz pero al enviar se lo hace con comandos?"
- L2: User accepted the version split: 0.1.10 is the Settings dialog already on main;
  this feature is 0.1.11; profiles/auth view is 0.1.12; profile editing 0.2.0
  optional. "si acepto el reparto".
- L3: Scope correction (S2): "me explique mal, 'hoy modela 7 de los 33 comandos'
  con esto que me dijiste ya tengo claro el panorama, la cosa es pasar los comandos
  para usarse mediante la interfaz, no agregar nada nuevo."
- L4: The chip in the screenshot belongs to the gentle-pi TUI routing table; the
  gentle-desk composer has no chip today (verified 2026-11-12).
- L14 (user, after the merge of #137; verbatim): "tambien quiero tratar Las seis
  advertencias antes de lanzar la release". The six informational findings of the
  approved native review (lineage review-552abe7c43fcc5bb) were R3-2, R3-3, R3-4,
  R4-1, R4-2 (WARNING) and R4-3 (SUGGESTION). Their original texts were lost: the
  review state file is deleted when the approved review is acknowledged and they
  were not saved first, so each disposition below comes from reading the code at
  the reported location, not from the reviewer's words (lesson: save the findings
  before acknowledging). R4-1 and R3-3: the highlighted row was an index into the
  visible list that a shrinking list or an empty filter left out of range; it is
  now clamped (test: the list shrinks while open). R3-2 and R4-2: the pill button
  handled Enter and Space as well as the native click it already produces; only
  ArrowDown is handled now (test: Enter and Space are not prevented). R3-4: the
  chip's CSS comment still said it had its own row, and a margin-top: 0 was left
  over; both removed. R4-3: NOT changed. SidebarToggle's default no-op handler is
  there so the control renders alone in tests; both production call sites pass the
  real handler, and requiring it there would only move the default up to Header.
