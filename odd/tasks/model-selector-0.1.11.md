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
- S6: Nothing is persisted by gentle-desk: no gentle-pi JSON is written (roadmap
  decision kept). The chat sends the commands through the existing runtime path
  (`runtime` execute.rs is generic; no change there).

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
- [ ] T3 (S1, S3, S4, S5) tauri + web: expose the chip state through the contract,
      a ModelChip with two dropdowns in the composer, copy, and a bridge test
      (apps/desktop/src-tauri/src/contract.rs, intent.rs, state_view.rs;
      apps/desktop/src/composer/ModelChip.svelte, bridge/dispatch).
- [ ] T4 (S5) empty/failed-list states: the chip renders a "no models" state and the
      send box is untouched + test.
- [ ] T5 (S6) final verification: focused tests, full suite, clippy/fmt, and a note
      in the roadmap that 0.1.11 shipped.

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
