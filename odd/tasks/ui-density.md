# ui-density: the interface stops overwhelming the eye

The maintainer, while asking for profile editing, added a second complaint in the
same breath: the interface is full of information that tires the view. A read-only
audit answered "how full" with numbers, and this document keeps that baseline, the
rules that follow from it, and the tasks that are cheap and independent now versus
the redesign that has to wait for the content to settle.

The diagnosis is not "too much information". It is **no hierarchy plus repeated
facts**: the chat profile panel repeats its own scope **7 times** and the used
profile's name **5 times**, is **61% read-only report against 39% editable**, shows
**35 controls at rest** with **no identifiable primary action**, and opens with
**26 chain-editor controls already live** inside the auto-opened group.

## Specs (verbatim)

- S1 (maintainer, 2026-11-18): "otra cosa tambien anota la revision de la interfaz
  porque yo la ciento llena de informaicon que agobia a la vista".
- S2 (maintainer, 2026-11-18): "con tu recomendacion la b" - the audit covers the
  whole app (read-only), the redesign executes on the profile panel and Settings
  in the version that follows this one.
- S3 (S4 of `chat-transcript.md`, same maintainer): "lo que funciona dejarlo asi".
  Nothing here rewrites a screen that is not named.

## Baseline the redesign must beat (read-only audit, 2026-11-18)

| Screen | Visible info blocks | Worst finding |
| --- | --- | --- |
| Chat profile panel | ≈36 | scope 7x, profile name 5x, 26 live editor controls, 61/39 report/editable, no primary action |
| Settings | 11 chrome + 12-19 per section | 7 rail entries + heading + a full-sentence hint on every section; the dialog has **no scroll container**, so a long pane overflows the window |
| Usage pane | ≈40 | 98% report, 2% editable (data-dependent, unverified live) |
| Chat chrome | ≈13 + messages | the busy state is drawn **4 times** (thread activity, tab badge, composer hint, sidebar mark) |
| Sidebar | 8-12 | a permanent "Preview" chip in production |

Three offenders, with the cheapest structural fix for each:

1. Scope restated 7x - single source of truth: keep it in the header tag only.
2. The used profile opens with its chain editor live - progressive disclosure: it
   opens read-only like every other group, and Save/Cancel/step controls appear
   only after an explicit Edit.
3. Settings draws a rail entry, a heading and a full-sentence hint on every
   section, and the dialog cannot scroll - move the hint behind one affordance and
   bound the body.

## Tasks

- [x] T1 (S1) bound the Settings body: `.dlg.settings` sets a width and nothing
  else (`apps/desktop/src/app.css:1425-1429`), unlike `.dlg.whatsnew` which bounds
  its body (`:2659-2662`). A long pane (Usage, Account, Updates with notes) must
  scroll inside the dialog instead of overflowing the window.
- [x] T2 (S1) remove the dead component `apps/desktop/src/components/StatusLine.svelte`
  after confirming that only its own test imports it. Hygiene, not design.
- [ ] T3 (S2, deferred to the version after 0.7.0) the profile panel redesign: one
  place per fact, the chain editor behind an Edit, an identifiable primary action,
  and read-only report content demoted.
- [ ] T4 (S2, deferred with T3) the Settings redesign: hint behind an affordance,
  rail and heading reconciled, Usage under progressive disclosure.
- [ ] T5 (S1, needs a maintainer decision) the "Preview" chip in the sidebar and
  the 4x busy state are **product and design decisions, not defects**: the app is
  genuinely a preview (`README.md` says so), and merging the busy signals means
  choosing which one survives. Recorded, not implemented here.

## Log

- L1 (2026-11-18, the maintainer's words, verbatim): "dejemos el boton "Edit in
  terminal" por si el usuario quiere usarlo que lo pueda hacer que creo que te
  refieres a escape eso es darle dos metodos al usuario por interfaz o terminal.
  otra cosa tambien anota la revision de la interfaz porque yo la ciento llena de
  informaicon que agobia a la vista".
- L2 (2026-11-18, audit): the numbers above were counted from the templates
  (path:line recorded per screen in the session that produced them); they are the
  baseline a redesign is measured against, not a target in themselves.
- L3 (2026-11-18, ruling): T1 and T2 are defects and ship first; T3-T4 wait for
  the profile content to settle (agent model pickers, duplicate, rename), because
  redesigning the form before the content is how a panel gets redesigned twice.
- L4 (2026-11-18, correction): an earlier pass of this work called the "Preview"
  chip a defect. It is not - it is an honest product label - so it moved to T5 as
  a decision instead of a fix.
- L5 (2026-11-18, T1 done): `.settings-body` is bounded with
  `max-height: calc(100vh - 200px)` and `.settings-pane` scrolls (`overflow-y:
  auto` plus the `min-height: 0` a flex child needs to shrink). Pinned by
  `apps/desktop/src/settings/layout.test.ts`, which reads `app.css?raw` the way
  `layout.test.ts` already does. RED seen first: both assertions failed on the old
  rules. The pane's padding grew to 8px on the right so a scrollbar does not sit on
  the text.
- L6 (2026-11-18, T2 done): `components/StatusLine.svelte` and its test are gone.
  Confirmed dead first - the only importer was `StatusLine.test.ts` - and the
  `.status` rule stays because `helpers/HelperThreadItem.svelte:47` uses it.
