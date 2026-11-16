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
  ASYMMETRY, adopted in T3 (deviation accepted by the parent): only a DETECTED SPANISH
  is reported and stored. An English detection stores nothing, because English is
  exactly what an absent preference already means — storing it would freeze a choice
  the user never made, and with nothing stored the app keeps following the machine
  (S1's "el idioma inicial es el del sistema") until a human chooses.
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
- [x] T3 (S2, S4) walking skeleton, end to end: runtime `AppConfig::set_language`
      (changed-bool, repeat saves nothing, +3 tests in language_config.rs); Tauri
      `UiIntent::SetLanguage` answered in `config_intent` through the new
      bridge/language.rs (the theme's ONE-step update_config mold, re-emitted slice,
      no command registration needed — dispatch_intent is the only command); bindings
      regenerated (+10 lines); web `i18n/language.svelte.ts` (self-subscribing,
      adopts every push, navigator fallback, one-shot Spanish report per D2's
      asymmetry) + Settings → Language rail entry between Usage and About, the
      LanguagePane radiogroup (English / Español each in its own language), and the
      dialog's titles/hints through `settingsCopy(language)` dictionaries. Web tests:
      8 i18n + 6 settings copy, instant flip in the same tick. Route: writer (T3
      started with an interaction_required stop: the two rail-count pinning test files
      joined the surfaces by maintainer approval, placement A). Commit: pending — see
      Log L4 for the five deviations accepted.
- [x] T4 (S3) Rust catalog: 57 catalog messages in crates/gentle-core/src/i18n/messages.rs
      (one function per message, Language first, English byte-identical, neutral
      Spanish) and a one-way language channel — `AppState.language` fed by
      `bridge::speak` from the config snapshot (None == English, D2), preserved across
      folds (`Intent::NewChat` rebuilds state, `fold` re-applies), read by every
      producer through the holder. 61 OWN fragments localized (the maintainer's ~62;
      the writer's inventory found 61 + 8 passthrough); two stay English BY RULING:
      the damaged-config notice (no readable preference exists at that moment) and
      images.rs:348 `Attached image:` (prompt content, D7 by spirit). First attempt
      stopped: 9 of 12 producer files could not read the language — maintainer
      approved the 18-path widening plus the Usage countdown's 2 paths
      (settings/usage.rs; bridge/settings.rs turned out unused — refresh_usage already
      holds the holder). English pinned by the pre-existing exact-English suites with
      only mechanical Language::En args; deliberate speak-no-op mutation proved the
      channel carries (RED then reverted). Route: writer. Commit a75c77c (GREEN:
      gentle-core 143, gentle-runtime 260, gentle-app 215, gentle-desk-app 369,
      0 failed; clippy/fmt clean).
- [ ] T5 (S3) web catalog pattern + the modules that need no other work: notices/,
      dialogs/ and helpers/ migrated on the D4 shape (first attempt delivered 18+2 files:
      levels words out of level.ts into noticesCopy, dialogs and helpers builders, 532
      tests green); the theme labels (theme/themes.ts THEME_LABELS) finish in T5 with
      their consumer, sidebar/ThemePicker.svelte + its test, approved by the parent as
      completing the maintainer's named item (its only consumer is the picker). The
      recipe for T6-T8 is in the T5 report (5 steps, SAME_IN_BOTH rule, forced
      setLanguage('en') flip test — vi.resetModules is unsafe across module
      generations). icons/shapes.ts MARK_LABELS is UNASSIGNED in T6-T8's lists →
      assigned to T8. Deferred: chat/tabs.ts → T8, settings/profile-tabs.ts and
      sections.ts 'Settings sections' → T7. Route: writer. Commit: pending.
- [x] T6 (S3) chrome: App.svelte (new minimal src/app-copy.ts), sidebar/{copy.ts,
      home-copy.ts} converted to the D4 builder shape, the 23 hardcoded chrome words
      and the six mark names (icons/shapes.ts markLabelsCopy, English MARK_LABELS kept
      as the bare Mark default) migrated; titles/labels/timestamps/the delete dialog's
      path stay passthrough, proven by the flip tests. Two seams recorded for their
      owners: Sidebar.svelte:75 still passes the flat SETTINGS word into
      copy.updateWaiting() (one line, now inside T7's surfaces) and the bare <Mark>
      draws in notices/Notice.svelte (T5-owned) and chat/ViewTabs.svelte (T8) until
      those slices pass markLabelsCopy(language.current)[kind]. Route: writer. Commit
      802864c (RED 7; GREEN 558/0, svelte-check 0, eslint 0).
- [x] T7 (S3) settings area: the five copy modules (copy/account/profiles/updates/usage)
      become D4 builders with both columns (~68 entries, 7 builders), every pane and the
      dialog read through them, profile-tabs and the rail label join their owners, and
      the T6 gear seam is CLOSED (Sidebar.svelte:78 derives the dialog's word; mutation
      proof attached). settings/** has zero hardcoded UI words left. The orchestrator
      mode words are translated (Orgánico / Delegación total) while Fallback/pi/shell
      and the pi document's mode values stay — flagged for the maintainer's T10 native
      read. Route: writer. Commit de5322d (RED 45; GREEN 594/0, svelte-check 0,
      eslint 0).
- [x] T8 (S3) the rest: composer (20 entries), thread (8), chat (new copy.ts; the words
      left the rules module tabs.ts with a pointer comment), install (14 + elapsed),
      update (7), welcome (10 + the two 3-point lists), whats-new (4; the release body
      from latest.json stays released content, D7), and the bare <Mark> draws closed:
      Notice.svelte passes markLabelsCopy[mark]; ViewTabs keeps its count sentence via
      chatCopy().workingBadge (translating the sentence, not the mark — passing
      markLabelsCopy would silently drop the count). Sentence helpers take the
      dictionary as a parameter (composerStateOf, chipOf, sinceLabel). Route: writer.
      Commit 04f613c (RED 1 suite + 37 tests; GREEN 633/0, svelte-check 0, eslint 0).
- [ ] T9 (S3) bilingual completion audit (RE-SCOPED, Log L6): DONE — the web side had
      exactly one drawn app-owned English literal (App.svelte's update-retry reason →
      update/copy.ts noNewerVersion) plus two latent ones reported (update/plugin.ts
      NOT_THE_APP: carrier has no consumer; the retry branch itself is dead code today,
      left in place); the app-level flip test exists (App.language.test.ts: stored es →
      Ajustes/Enviar/Hilo del chat/Instalar actualización in one tick, and back; bit
      under a composerCopy mutation, reverted). Audit tables in the T9 report. Route:
      writer. Commit: pending.
- [x] T11 (S3, from the T9 audit; Log L7) T4b — DONE. The audit's 12 Rust producer
      groups localized: 38 new catalog messages (95 total in messages.rs) and the
      threading reaching every draw site the audit traced (config save, install ×6,
      the three Settings probes/views, both document readers as whole sentences, the
      close gates, the chat reducer's unknown-model-error). Language is read at call
      time (a switch applies to the next read, S2). Two channel-level Spanish pins,
      both mutation-proven and reverted (config save → config_notice in Spanish;
      reduce_chat(Es) → "error desconocido del modelo"). The 7+2 call-site paths the
      survey needed beyond the 13 were approved by the parent as mechanical-only
      (Log L8). Route: writer. Commit df6740e (RED 78×E0425; GREEN 152/260/215/370/15,
      0 failed; clippy -D warnings across five crates; fmt clean).
- [x] T10 (S5) close: version 0.2.0 in apps/desktop/package.json,
      apps/desktop/src-tauri/Cargo.toml and apps/desktop/src-tauri/tauri.conf.json (+ the
      repo-root Cargo.lock; the handoff's src-tauri lock path does not exist), the
      shipped-on-branch note in odd/tasks/tauri-settings-roadmap.md, then the full
      verification plan. Route: writer. Commit 37c8918 (writer's verification:
      cargo test --workspace 1243/0 — core 152, backend 231, term 15, app 215, runtime
      260, desk-app 370; clippy -D warnings clean; fmt clean; drift guard green with
      byte-identical bindings; vitest 634/0; svelte-check 0; eslint 0). The what's-new
      body is written at publish (latest.json embeds the release body). The real-window
      checks ran on the maintainer's machine 2026-11-14 (tauri dev, v0.2.0; the
      maintainer reviewed and approved the launch).
      Delivered (writer, 2026-11-14): 0.2.0 in exactly the three carriers, plus the one locked
      `gentle-desk-app` version in the workspace lock — the lock lives at the REPO ROOT
      (`Cargo.lock`); the handoff's `apps/desktop/src-tauri/Cargo.lock` does not exist, and
      `cargo check --workspace` rewrote the single locked line. The roadmap's 0.2.0 entry is
      now SHIPPED-ON-BRANCH (shipped on feat/language-0.2.0 pending release). The what's-new
      notes are NOT a repo file: latest.json embeds the release body at publish, so the 0.2.0
      body is written then, per the 0.1.12/0.1.14 flow. No test asserts the version string
      (the six `0.1.14` literals in the suites are fixtures, not the app's version).
      Verification, all green: cargo test --workspace 1243 passed / 0 failed / 1 ignored
      (gentle-core 152, gentle-backend 231, gentle-term 15, gentle-app 215, gentle-runtime
      260, gentle-desk-app 370 = 69 lib + 301 across 82 integration targets); clippy
      --workspace --all-targets -D warnings clean; fmt --check clean; contract_bindings green
      WITH GENTLE_DESK_REGEN_BINDINGS=1 (bindings byte-identical — git hash-object matches
      HEAD) and green again without the switch; vitest 634/634 in 106 files; svelte-check 0
      errors 0 warnings; eslint clean. Commit: still pending, the writer does not commit —
      release, tag and publish stay the maintainer's decision. The T10 real-window and
      passthrough checks (Verification plan) stay PENDING: they need the app running.

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
- L4 (parent, 2026-11-14): T3 complete, five suites green (runtime 257, tauri 364,
  vitest 514, svelte-check 0, clippy/fmt/bindings clean). Deviations adjudicated:
  (1) ACCEPTED and folded into D2 — only a detected Spanish is stored; an English
  detection stores nothing. (2) ACCEPTED — Spanish copy carries accents; the ASCII
  note in settings/copy.ts now covers the English column only. (3) ACCEPTED — the
  rail glyph is the existing message-square; a dedicated language glyph is optional
  polish for T7, not a gap. (4) RECORDED for T7 — SECTIONS entries no longer carry
  `title` and SECTION_TITLES/SECTION_HINTS are gone; T7 must not re-add flat
  constants, the words live in settingsCopy(language). (5) RECORDED for T7 — the
  dialog's own "Settings" word stays English in this slice. The stale "seven
  entries" comment in SettingsDialog.test.ts:26 was fixed by the parent (one word).
  The real-window check the verification plan asks at T3 stays PENDING: it needs the
  app running on the maintainer's machine — parent to schedule it with the T10 check
  or ask for a window when the maintainer is at the desk. Mutation risk for the
  verification of this slice: dispatchIntent not called on choice (silent no-op),
  adopt() inverted (stored preference ignored), report() firing twice, and the
  fallback rule drifting from Rust's.
- L6 (parent, 2026-11-14): T9 re-scoped. The original "test sweep" description was
  written before the catalog pattern emerged; routing every English-asserting test
  through the builder would be churn without value, because the byte-identical English
  pins ARE the preservation evidence and every area now owns copy + flip tests. T9's
  remaining value is the completion audit (grep for drawn app words outside the
  dictionaries, one app-level flip, the Rust-producer equivalent). T8's ViewTabs
  deviation accepted: the badge's accessible name carries the count, so the count
  sentence moved into chatCopy() rather than the generic mark label.
- L7 (parent, 2026-11-14): T9 delivered the audit: web clean after one fix
  (noNewerVersion), two latent hits reported, app-level flip test in place. The Rust
  audit found 12 drawn producer groups T4's inventory missed — none were in its 12-file
  list; the audit's draw-site tracing is what caught them. Parent approved T4b (task
  T11) with approve-all (13 paths) rather than deferring: S3's verbatim scope covers
  these texts, and leaving Spanish users half-English reason lines would ship the
  feature incomplete. New rulings: NO_PATH_NOTICE joins ruling 1 (the missing folder
  is the one that would name the language); the config save-failure line is cataloged
  (language in hand); the un-drawn usage-sources line joins anyway (cheap, same file).
- L9 (parent, 2026-11-14): the accumulated branch reached reviewDue=true, nativeRisk
  high (hot_path: auth in settings/auth.rs). The offered review.start failed
  preflight THREE times with lens_context_budget_exceeded — full branch (216 paths),
  then the smallest natural candidate containing the hot path (81d9e0f..7381e70: 36
  paths, 1739 lines) with three lenses, with requestContextPath omitted, and with a
  single lens. No authority was created (retry-safe, nothing to abandon); the
  provider offers no continuation beyond splitting, and no smaller natural candidate
  exists. Outcome: Needs your decision — the maintainer investigates the review
  authority, disables the review switch for this clone, or defers review to a smaller
  PR chain. Preserved verbatim: the three failure envelopes name lineage
  review-93f4c464349be318 and next_action stop.
- L8 (parent, 2026-11-14): T11's two widening rounds approved by the parent as
  mechanical-only (7 call-site paths, then the 2 fixture files), within S3's mandate —
  the sets carried zero design ambiguity and the alternative was shipping half-English
  draw sites the spec excludes. Accepted deviations: the catalog shape for the usage
  refusals (one whole sentence each — nesting translated fragments is ungrammatical);
  close::CLOSING became closing(language) -> String (a const cannot be localized);
  Installer::install(language) is the trait's carrier; Settings views take the language
  LAST matching T4's usage::view shape (catalog functions keep Language first); the two
  Spanish channel pins live beside their English siblings. Final by-design-English
  statement is in the T11 report: ruling-1 family (NO_PATH_NOTICE, damaged-config
  notice), ruling 2 (prompt line), shell/pi passthrough, and the OS's/JSON's own words
  interpolated into localized sentences.
- L10 (parent, 2026-11-14, the chain that answered L9): reviewing each slice FROM ITS
  OWN BOUNDARY (detached HEAD at the boundary commit, candidate = previous boundary..that
  commit) is the shape that works — the maintainer's question "no se puede dividir en
  revisiones pequeñas?" was right, and the earlier failures were my ranges always ending
  at the full tip. Results, in order: T1 (6 paths/447 lines) reviewed natively with
  risk+reliability, APPROVED and acknowledged (lineage review-f675569aab400891, authority
  burned, ordinary-repository-policy delivery); T2 (656), T3 (1056), T5+T6 (2061), T4
  (2354), T7+T8 (4816), T9 sliver (121) all assess `not due` — the controller's own
  budget logic covers them. The ONE due unit is T11 (31 paths/~1.7k, high via the auth
  hot path), and it exceeds the provider's lens-context budget at every derivable size
  (two more preflight refusals: 38a724d..7381e70 and df6740e alone, lineages
  review-54ab2737045733bc and review-9dd90c7e677a7cae; no authority created). Chain
  verdict: everything is reviewed except T11's commit, which is atomic and over budget —
  upstream defect data: 447 lines pass, 1.7k fail, so the budget sits between and a
  31-file Rust slice with a 433-line catalog file should fit an intended budget. T11's
  compensating proof stands: catalog EN/ES pairs, channel-level Spanish pins
  (mutation-proven), draw-site tracing, and the five green suites.
