# whats-new-modal: what the update brings, shown once per version

A modal after the app updates and reopens, telling what the new version brings. The maintainer's
request (2026-11-12), with the decisions taken: the notes travel in the updater's own `latest.json`
(the `notes` field the release workflow already writes, now carrying the release body), and the
modal shows once per version.

## Specs

- S1: Detection. `app_config` gains `last_run_version` (additive, like the window state; the config
  version does not move). At startup the run compares it with the app's own version: absent means a
  FIRST run (no modal); different means this run follows an update. The stored version is NOT
  written at startup: it is written when the modal is dismissed (or when the notes cannot be
  fetched), which is what makes "once per version" true.
- S2: Content. The release workflow embeds the release body into `latest.json`'s existing `notes`
  field (today it writes a generic "Gentle Shell Desktop x.y.z"). The maintainer writes the body at
  release time as always.
- S3: The fetch. On a detected update the app fetches `latest.json` ONCE through an injectable
  fetcher: if its `version` equals the app's own version, it pushes a what's-new view state with
  those notes; if it does not match, or the fetch fails, nothing is pushed and the run records the
  stored version anyway (the modal is best-effort, the version bookkeeping is not).
- S4: The modal. House dialog shape (bits-ui, like `NewWorkspaceDialog`/`DeleteChatDialog`), with the version
  shown as a pill inside the accessible title, and "since X" only when `previous_version` is strictly older
  (numeric comparison; never on a downgrade). The notes are drawn as sanitized markdown through the thread's
  pipeline (`createMarkdownCache().htmlFor('whats-new', notes)` inside `Markdown.svelte`), keeping a leading
  heading as a small section label. Dismiss closes the dialog and the run then stores the current version in
  `last_run_version`. Escape and a backdrop click dismiss too.
- S5: The version stored must be the app's own version, never the manifest's, so a stale manifest
  cannot keep the modal coming back.

## Tasks

- T1 (S1, S3, S5) Detection + fetch + push: the config field, the update-detected state, the
  injectable fetcher, the view-state push, and the store-on-dismiss/store-on-fail seam -
  route: delegated writer, commit: see git log (feat/whats-new-t1)
- T2 (S4) The modal, once per version, wired to the pushed slice; redone as themed markdown (L10) in
  0c297b1 on feat/whats-new-modal-design - route: delegated writer, commit: see git log
- T3 (S2) The release workflow embeds the release body into `latest.json` - route: delegated
  writer, commit: see git log (chore/whats-new-workflow)

## Log

- L1 (2026-11-12, user): "Quiero agregar un modal que luego de actualizar a lo que se vuelve abrir
  la app informe sobre la nueva actualizacion que es lo que trae."
- L2 (2026-11-12, user decisions): content from the pattern desktop apps use - the notes travel
  with the update in the updater's own manifest (the `notes` field `latest.json` already has, now
  carrying the release body); the modal shows once per version. Detection via `last_run_version`
  in the app config is the parent's call.

- L10 (2026-11-15, user): after seeing the plain-text modal, the user chose design artboard A
  (design/whats-new-modal/01-document.html) and asked: "el modal se debe adaptar al tema que tengo la app"
  — render the release body as markdown and follow the active theme with CSS variables only. This rewrote
  S4; T2 reopened and redone in feat/whats-new-modal-design (previous_version from the config, Markdown
  pipeline in the dialog, .dlg.whatsnew styles, theme-render verification in Chrome).
- L9 (2026-11-12): T1-T3 all implemented and verified. T3 embeds the release body into latest.json's notes via gh release view at manifest-write time, with the generic line as fallback and the ordering documented (edit the body, then re-run the workflow on the same tag to refresh the notes). Proven by a dry-run harness with a fake gh: 19 assertions green, 8 red against the pre-change snippet. Note for the first tag run: the first run embeds the generic text unless the release body pre-exists; the re-run path delivers the maintainer's notes.
