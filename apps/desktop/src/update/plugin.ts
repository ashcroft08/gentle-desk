// The one place that talks to the updater plugin. Everything else works on the
// plain data of `state.ts`, so the rules stay testable without a window.
//
// The check is skipped until the app carries a real public key: with the
// placeholder still in the config the app could offer an update it cannot verify,
// and an unverifiable update is exactly what S1 forbids.

import { check, type Update } from '@tauri-apps/plugin-updater'

/**
 * The newer release, or `null` when there is none, when this page is not the app,
 * or when the build cannot be trusted to verify one. While the config still holds
 * the placeholder key the plugin refuses, and that refusal lands here like any
 * other: nothing is offered, because an update the app cannot verify is worse
 * than none (S1).
 */
export async function findUpdate(): Promise<Update | null> {
  if (typeof window === 'undefined' || !('__TAURI_INTERNALS__' in window)) return null
  try {
    return await check()
  } catch {
    // No network, no release, no endpoint, no usable key: the window says nothing (S2).
    return null
  }
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
