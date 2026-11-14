# language-0.2.0: English and Spanish for gentle-desk's own words

Version 0.2.0 of the version plan in odd/tasks/tauri-settings-roadmap.md: the app's own
UI text plus the notices, errors and validations Rust generates become bilingual
(English / Spanish), with the system language as the first-run default and an instant
switch that needs no restart. Nothing that comes from the shell or pi is translated.
Measured, not guessed: 17 web copy modules (194 exports, 25 parametrized) in
apps/desktop/src, 44 hardcoded UI strings (28 in .svelte, 16 in .ts) and ~62 Rust
fragments in 12 files.

## Specs

- S1: Initial language — "Idioma inicial: el del sistema; si el locale es cualquier
  español → español. El español de la app es neutral (no rioplatense)."
- S2: Instant switch — "El cambio de idioma aplica al instante, sin reiniciar."
- S3: Scope — "Alcance: todo lo propio de gentle-desk (textos de la UI + avisos/errores/
  validaciones generados en Rust). Lo que viene del shell/pi se muestra como llega, sin
  traducir."
- S4: Storage and plumbing, the maintainer's own preference — "Preferencia: AppConfig en
  crates/runtime/src/app_config/config.rs:32-73, CONFIG_VERSION = 1 en :25; campo
  aditivo = serde default + espejo en DocumentRef con skip_serializing_if. Flujo del tema
  como molde: config.json → store::load → ShellParts.theme → ViewState.theme →
  ThemeView → theme/paint.ts:22."
- S5: Why a minor bump — "0.2.0 (next, when the maintainer says): **language** - English
  and Spanish for the app's own words. [...] Not a patch: it is a capability jump,
  which is what a minor bump marks."

## Design (parent decisions; writers do not re-open these)

- D1: `Language { En, Es }` lives in `crates/gentle-core/src/i18n/` so runtime, app and
  the tauri crate share one type: `detect(locale: &str) -> Language` (case-insensitive
  primary subtag `es` → Es, anything else → En), `as_str()` = "en"/"es", `parse` for
  the config and the view state.
- D2: The config field is additive and optional (`language: Option<Language>`, serde
  default, `skip_serializing_if = "Option::is_none"` in `DocumentRef`, CONFIG_VERSION
  stays 1), and a stored value that does not parse reads as `None` instead of breaking
  boot. The OS locale is read with no new dependency: while the stored value is absent
  the contract carries `language: null` and the web paints from `navigator.language`
  with the same rule, then reports it once (T3) and Rust persists it through the normal
  config write. A stored preference always wins over a later OS language change; that is
  what makes the choice stable, and it is why Rust never re-detects on every start.
- D3: Projection mirrors the theme exactly: config.json → `store::load` →
  `ShellParts.language` (apps/desktop/src-tauri/src/state_view.rs:34) →
  `ViewState.language` (contract.rs:930) → bindings. Instant apply reuses the same
  re-emit path that makes a theme change immediate: no restart, no webview reload.
- D4: Web copy modules keep their current name and exported shape and gain a Spanish
  sibling dictionary typed `typeof <english>`; the module exports a builder
  `xCopy(language)` returning that whole object, so parametrized entries stay functions
  and no string-keyed `t("...")` lookup is introduced. Consumers read
  `const copy = $derived(xCopy(language.current))` (currency: `apps/desktop/src/i18n/
  language.svelte.ts` exports the reactive `language` plus the set-language dispatch).
- D5: Hardcoded UI strings move into the copy module that already owns their area. No
  new dictionary file per one-off string.
- D6: Rust fragments become catalog messages with parameters, localized where they are
  produced (Rust knows the language), so the web renders them verbatim. No codes cross
  the contract, no second translation layer. Neutral Spanish: no voseo, no regionalisms.
- D7: Child output, session titles, model and job names, shell notices and every other
  passthrough string are shown exactly as they arrive (S3).

## Tasks

- [x] T1 (S1) gentle-core: `Language`, `detect_from_locale` and `as_str`/`parse` in
      crates/gentle-core/src/i18n/ + module wiring in lib.rs; no OS read and no new
      dependency here. Tests pin es-ES/es-419/es-MX/es-AR/es_AR/ES-ar → Es,
      en-US/de/fr/empty/whitespace/"e" → En, the as_str/parse round trip and the serde
      form "en"/"es". Route: writer. Commit 383c76e (RED E0432 unresolved import;
      GREEN 128 passed / 0 failed, clippy and fmt clean).
- [x] T2 (S1, S4) config and projection: `language` field in
      crates/runtime/src/app_config/{config.rs,store.rs} (serde default, DocumentRef
      mirror, an unparsable stored value reading as `None`) and `ShellParts.language` →
      `ViewState.language` (`Option<Language>` as `"en"`/`"es"`/`null`) in
      apps/desktop/src-tauri/src/{state_view.rs,contract.rs,bridge.rs}; bindings
      regenerated through GENTLE_DESK_REGEN_BINDINGS. Load never writes. Tests: config
      round-trip, missing and malformed field → None, the projection carrying
      null/"en"/"es", the wire key's PRESENCE (`.get`, not indexing — serde_json's
      Index answers Null for a missing key too), the literal variant per input (no
      tautological mapping), an unrelated save keeping the stored language, and the
      byte-identity of the document an unset preference writes. Two mutation rounds
      proved the pins bite (skip_serializing_if on ViewState.language → 2 failed;
      view_language variant swap → 3 failed; both reverted, grep-verified clean).
      Route: writer. Commit fcd18d3 (RED E0609/E0432; GREEN 254 runtime + 361 tauri,
      clippy/fmt/drift guard clean). Web side followed the bindings: the full-fixture
      test-support/view-state.ts gained `language: null` (500 vitest, svelte-check 0).
- [ ] T3 (S2, S4) walking skeleton, end to end: web i18n core
      (apps/desktop/src/i18n/language.svelte.ts: reactive language from the view state,
      `navigator.language` es* fallback while the state carries null, the one-shot boot
      report, set-language dispatch) + Settings → Language section (settings/sections.ts
      Section :30, SECTIONS :47-55, titles :58, hints :75, TAB_IDS :86, PANEL_ID :97;
      SettingsDialog.svelte:108,136-149; settings/copy.ts:18-33) + the Rust write path
      (intent → store save → re-emitted state). Section titles and hints of the dialog
      move to the catalog here, so the switch is visibly instant. T2 left off here: the
      write path needs `AppConfig::set_language` (mirroring `set_theme`'s
      changed-bool), the intent+command, `bridge/language.rs` beside `bridge/theme.rs`,
      the command registration, and the web dispatch. Tests: vitest flip → new labels
      in the same tick; Rust test that a write re-projects. Route: writer. Commit:
      pending.
- [ ] T4 (S3) Rust catalog: the ~62 fragments in crates/runtime/src/app_config/{status,
      store}.rs, helper_text.rs, chat_runtime/execute.rs, time_ago.rs,
      crates/app/src/{chats/*,chip.rs,events.rs} and
      apps/desktop/src-tauri/src/{folder_name,new_folder,terminal,workspace_add,
      workspace_create,sessions/view,chat_rename,images}.rs become catalog messages in
      the two languages. Tests: each message asserted in en and es, parametrized ones
      with their substitution. Route: writer. Commit: pending.
- [ ] T5 (S3) web catalog pattern + the modules that need no other work: notices/,
      dialogs/, helpers/ and the 16 hardcoded .ts strings (theme/themes.ts:30-37 and
      friends) get their en/es dictionaries, consumers migrated, tests updated. This
      slice sets the pattern T6-T8 follow. Route: writer. Commit: pending.
- [ ] T6 (S3) chrome: App.svelte, sidebar/{copy.ts,home-copy.ts,Sidebar.svelte,
      ChatRow.svelte} — the largest concentration of the 28 hardcoded .svelte strings.
      Route: writer. Commit: pending.
- [ ] T7 (S3) settings area: settings/{copy.ts,account-copy.ts,profiles-copy.ts,
      updates-copy.ts,usage-copy.ts}, SettingsDialog.svelte and its panes. Route:
      writer. Commit: pending.
- [ ] T8 (S3) the rest: composer/, thread/, chat/, install/, update/, welcome/,
      whats-new/ and components/. Route: writer. Commit: pending.
- [ ] T9 (S3) test sweep: the ~45 vitest files that assert English and the ~12 that
      hardcode copy literals read through the catalog instead of literals; Spanish
      coverage where a Rust-generated string reaches the UI. Route: writer. Commit:
      pending.
- [ ] T10 (S5) close: version 0.2.0 in apps/desktop/package.json,
      apps/desktop/src-tauri/Cargo.toml and apps/desktop/src-tauri/tauri.conf.json, the
      what's-new entry, the shipped note in odd/tasks/tauri-settings-roadmap.md, then
      the full verification plan. Route: writer. Commit: pending.

## Verification plan

- RED before GREEN on every slice: the new test fails for the stated reason first.
- Cargo workspace tests, web vitest, svelte-check, clippy -D warnings, fmt, and the
  contract drift guard after each bindings regeneration.
- Real-window check, once at T3 and once at T10: Settings → Language → Español flips the
  dialog and the shell chrome in the same tick; a restart keeps the choice; a machine
  with an es-* locale starts in Spanish on a config without the field, and a stored
  "en" survives an es-* locale.
- Passthrough check at T10: a chat whose session title, model name and shell notice are
  English keeps them untouched while the surrounding labels are Spanish.

## Delivery

- Strategy: `chain_strategy=feature-branch-chain`, chosen by the maintainer in L1
  ("commit por slice en una rama de feature"). Work-unit commits on the feature branch,
  tests and docs with their behavior, one commit per task; push and PR creation stay the
  maintainer's decision.
- Forecast: well over 400 authored changed lines (194 web exports plus their es
  dictionaries, ~44 hardcoded strings and their consumers, ~62 Rust fragments, ~45 test
  files), so the chain above is the delivery shape, not a single PR.

## Log

- L1 (maintainer, 2026-11-14, verbatim): "Continuamos gentle-desk en
  C:/Users/josep/dev/gentle-desk. Estado y contexto:

  Estado del repo
  - main en 642bc29 (release 0.1.14 publicada). Ramas remotas: solo main — borré las
    otras 30 con tu OK.
  - Único cambio sin commitear: odd/tasks/tauri-settings-roadmap.md (36+/7−). Contiene:
    panel de uso → RESUELTO terminal-only (/gentle:usage); edición de
    profiles/subagents → RESUELTA con tu paquete; paleta Ctrl+K → DROPPED (nunca se
    construyó); editor de profiles/pins → ahora PLANIFICADO; y la sección nueva
    ## 0.2.x: adopting gentle-shell's desktop RPC channel con el orden: idioma (0.2.0)
    → sync viva de Settings → editor de profiles/pins → subagents/jobs/rate_limit.
  - Backlog (odd/tasks/backlog.md): sin items abiertos.

  Lo que falta de la app: 0.2.0 — Idioma (EN/ES). Es el próximo y único pendiente real.

  Decisiones que ya tomé para 0.2.0 (van al spec literal): 1. Idioma inicial: el del
  sistema; si el locale es cualquier español → español. El español de la app es neutral
  (no rioplatense). 2. El cambio de idioma aplica al instante, sin reiniciar. 3.
  Alcance: todo lo propio de gentle-desk (textos de la UI + avisos/errores/validaciones
  generados en Rust). Lo que viene del shell/pi se muestra como llega, sin traducir.

  Dimensionamiento ya medido (no repetir la exploración)
  - 16 módulos de copy web en apps/desktop/src con 194 exports (25 parametrizados;
    ~169 strings literales).
  - 44 strings hardcodeados en UI (28 en .svelte, 16 en .ts): theme/themes.ts:30-37,
    sidebar/ChatRow.svelte, App.svelte, sidebar/Sidebar.svelte son los mayores.
  - ~62 fragmentos propios de Rust en 12 archivos: crates/runtime/src/app_config/
    {status,store}.rs, helper_text.rs, chat_runtime/execute.rs, time_ago.rs,
    crates/app/src/{chats/*,chip.rs,events.rs}, apps/desktop/src-tauri/src/
    {folder_name,new_folder,terminal,workspace_add,workspace_create,sessions/view,
    chat_rename,images}.rs.
  - Preferencia: AppConfig en crates/runtime/src/app_config/config.rs:32-73,
    CONFIG_VERSION = 1 en :25; campo aditivo = serde default + espejo en DocumentRef
    con skip_serializing_if. Flujo del tema como molde: config.json → store::load →
    ShellParts.theme → ViewState.theme → ThemeView → theme/paint.ts:22.
  - Secciones de Settings: apps/desktop/src/settings/sections.ts (Section :30, SECTIONS
    :47-55, títulos :58, hints :75, TAB_IDS :86, PANEL_ID :97) +
    SettingsDialog.svelte:108,136-149 + settings/copy.ts:18-33.
  - Tests afectados: ~45 archivos vitest tocan inglés y ~12 hardcodean copy literal; en
    Rust ~8 de integración + ~8 unitarios afirman inglés exacto.

  Próximo paso concreto: crear odd/tasks/language-0.2.0.md (specs S# citando las 3
  decisiones de arriba, tareas numeradas), espejarlo en Engram, armar el todo list y
  recién ahí pasar los slices a escritores con TDD, commit por slice en una rama de
  feature.

  Contexto del canal RPC de gentle-shell (para después de 0.2.0, no ahora): gentle-shell
  4.8.0 (tenés gentle-pi@4.8.0 instalado) documenta docs/desktop-rpc.md, contrato
  gentle-shell.desktop-rpc/v1, activo solo con GENTLE_SHELL_INTERACTIVE_HOST=1 pi
  --mode rpc; se manda /gentle:desktop {json} como prompt y la respuesta vuelve por un
  widget de Extension UI correlacionado por requestId. Métodos: get_snapshot,
  get_config/set_config (protegido por writeRevision), get/set_permission_mode,
  get_subscription_usage, refresh_subscription_usage, list_jobs/cancel_job,
  apply_profile, list_subagents/cancel_subagent; eventos config_changed, rate_limit,
  jobs_update, profile_applied, agents_update. Hoy gentle-desk no lo consume (0
  referencias). Dejé el clon de lectura en %TEMP%\gpLH8b9P\pi-github-repos\
  runtime-8Twc2k (elegiste no borrarlo).

  Recordatorios de método: ODD de punta a punta; vos coordinás y los escritores hacen
  los cambios; TDD donde hay comportamiento testeable; nada de commits salvo que lo
  pidas o los slices de una feature grande; reportar siempre Risk: item N o Risk: none."
- L2 (parent, 2026-11-14): reconciled the baseline — main at 642bc29, worktree clean
  except odd/tasks/tauri-settings-roadmap.md, backlog empty, 17 copy modules on disk
  (the maintainer's count of 16 predates the settings split), version carried by three
  files. Slice count and surfaces set from those measurements, not re-explored. Delivery
  strategy taken from L1 as `feature-branch-chain`; no separate oversized menu asked.
- L3 (parent, 2026-11-14, native RDD assessment after T2 landed): the accumulated
  branch 642bc29..ef7dbb2 assesses **medium** (executable change in bridge.rs; 17
  files, 1061 lines), `reviewDue: false`, reason `under_budget` — the native review
  boundary stays pending at 642bc29 and the slice proceeds without review until a
  later commit reaches the budget or the maintainer asks. Tier, outcome and reason
  recorded here per commit: 383c76e+97c958a (docs/small), b810658 (tests, covered by
  the mutation verification), fcd18d3/ef7dbb2 (medium/under_budget, not due).
