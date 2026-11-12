// The one place that talks to the updater plugin. Everything else works on the
// plain data of `state.ts`, so the rules stay testable without a window.
//
// The check is skipped until the app carries a real public key: with the
// placeholder still in the config the app could offer an update it cannot verify,
// and an unverifiable update is exactly what S1 forbids.

import { check, type Update } from '@tauri-apps/plugin-updater'

/**
 * What one check of the release server answered, as plain data: the release it
 * found by version, nothing newer, or the reason it could not ask.
 */
export type CheckResult =
  | { kind: 'available'; version: string }
  | { kind: 'current' }
  | { kind: 'failed'; message: string }

/** What a check answers inside this module, where the install needs the plugin's
 * own handle on the release. */
type Checked =
  | { kind: 'available'; update: Update }
  | { kind: 'current' }
  | { kind: 'failed'; message: string }

/** Why a page that is not the app cannot ask: it has no updater at all. */
const NOT_THE_APP = 'this window is not the app'

/**
 * The one check of the release server: the plugin's own, asked once and mapped to
 * plain data, never thrown. The window's startup check and the Settings dialog's
 * Check now button both come through here, so the app keeps ONE updater path.
 *
 * While the config still holds the placeholder key the plugin refuses, and that
 * refusal lands here like any other: a failure, never an update the app cannot
 * verify (S1).
 */
async function checked(): Promise<Checked> {
  if (typeof window === 'undefined' || !('__TAURI_INTERNALS__' in window)) {
    return { kind: 'failed', message: NOT_THE_APP }
  }
  try {
    const update = await check()
    return update === null ? { kind: 'current' } : { kind: 'available', update }
  } catch (error) {
    return { kind: 'failed', message: error instanceof Error ? error.message : String(error) }
  }
}

/**
 * The newer release, or `null` when there is none, when this page is not the app,
 * or when the build cannot be trusted to verify one. While the config still holds
 * the placeholder key the plugin refuses, and that refusal lands here like any
 * other: nothing is offered, because an update the app cannot verify is worse
 * than none (S1). No network, no release, no endpoint, no usable key: every
 * failure lands here as `null` too, so the window says nothing (S2).
 */
export async function findUpdate(): Promise<Update | null> {
  const found = await checked()
  return found.kind === 'available' ? found.update : null
}

/**
 * The same check, as what a drawn line says (settings-dialog S6): the release it
 * found, nothing newer, or why it could not ask. This is the Settings dialog's
 * Check now button; it reports and installs nothing, because the install stays
 * the update notice's own flow.
 */
export async function checkForUpdate(): Promise<CheckResult> {
  const found = await checked()
  return found.kind === 'available' ? { kind: 'available', version: found.update.version } : found
}

/** Reads what the plugin reports while it downloads and installs. */
export async function downloadAndInstall(
  update: Update,
  onProgress: (done: number, total: number | null) => void,
): Promise<void> {
  let done = 0
  let total: number | null = null
  await update.downloadAndInstall((event) => {
    if (event.event === 'Started') total = event.data.contentLength ?? null
    if (event.event === 'Progress') done += event.data.chunkLength
    if (event.event === 'Finished') done = total ?? done
    onProgress(done, total)
  })
}
