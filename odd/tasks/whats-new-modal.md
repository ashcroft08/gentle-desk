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
- S4: The modal. House dialog shape (bits-ui, like `NewWorkspaceDialog`/`DeleteChatDialog`), titled
  with the version, the notes as plain text paragraphs. Dismiss closes the dialog and the run then
  stores the current version in `last_run_version`. No other interaction.
- S5: The version stored must be the app's own version, never the manifest's, so a stale manifest
  cannot keep the modal coming back.

## Tasks

- T1 (S1, S3, S5) Detection + fetch + push: the config field, the update-detected state, the
  injectable fetcher, the view-state push, and the store-on-dismiss/store-on-fail seam -
  route: delegated writer, commit: see git log (feat/whats-new-t1)
- T2 (S4) The modal, once per version, wired to the pushed slice - route: delegated writer,
  commit: see git log (feat/whats-new-t2)
- T3 (S2) The release workflow embeds the release body into `latest.json` - route: delegated
  writer, commit: pending

## Log

- L1 (2026-11-12, user): "Quiero agregar un modal que luego de actualizar a lo que se vuelve abrir
  la app informe sobre la nueva actualizacion que es lo que trae."
- L2 (2026-11-12, user decisions): content from the pattern desktop apps use - the notes travel
  with the update in the updater's own manifest (the `notes` field `latest.json` already has, now
  carrying the release body); the modal shows once per version. Detection via `last_run_version`
  in the app config is the parent's call.
