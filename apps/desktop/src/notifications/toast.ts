// The OS toast (S2, S5): one system notification through the official Tauri
// plugin, and the one place the app asks Windows for permission to show them.
//
// Two refusals are the same refusal, because the user's answer is the same: no
// notification appears. The plugin can report that permission was never granted
// (the answer to `requestPermission` is then `denied`), and every call can throw
// - a build without the plugin, a capability that does not carry
// `notification:default`, an OS that says no. Both land on 'denied', so the
// Settings pane can say "the tone played, but Windows did not allow the toast"
// (S5) and the alerting path never has an error to handle.
import {
  isPermissionGranted,
  requestPermission,
  sendNotification,
} from '@tauri-apps/plugin-notification'

/** What happened to one toast: it was handed to the OS, or it was not allowed. */
export type ToastResult = 'sent' | 'denied'

/**
 * Shows `title` with `body` as a system notification, asking for permission once
 * when it is not granted yet. Never rejects: a refusal is 'denied'.
 */
export async function sendToast(title: string, body: string): Promise<ToastResult> {
  try {
    const granted = (await isPermissionGranted()) || (await requestPermission()) === 'granted'
    if (!granted) return 'denied'
    await sendNotification({ title, body })
    return 'sent'
  } catch {
    return 'denied'
  }
}
