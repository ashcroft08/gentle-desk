// A stand-in for the official notification plugin (notifications S2, S5): the
// app talks to it through `notifications/toast.ts` alone, so a test mocks that
// one module and then reads what Windows was asked to show, how often the
// permission was asked for, and what the plugin refused.
//
// Usage in a test file (the factory runs before the test file's imports):
//
//   vi.mock('@tauri-apps/plugin-notification', () =>
//     import('../test-support/notification').then((m) => m.notificationModule()))
//
// The defaults are the maintainer's machine after the one time Windows asked:
// notifications allowed, so no test sees a permission prompt it did not ask for.

/** One notification as the plugin received it. */
export type SentToast = { title: string; body?: string }

let granted = true
let answer: 'granted' | 'denied' = 'granted'
let asks = 0
let sent: SentToast[] = []
let broken: 'check' | 'send' | null = null

/** Notifications already allowed: the OS is asked nothing (the default). */
export function grantPermission(): void {
  granted = true
  answer = 'granted'
}

/**
 * Notifications unknown or refused, so the app has to ask: `answer` is what the
 * user then answers in the dialog the plugin opens.
 */
export function denyPermission(answerWith: 'granted' | 'denied' = 'denied'): void {
  granted = false
  answer = answerWith
}

/** `isPermissionGranted` throws, as a plugin that is not installed does. */
export function breakChecking(): void {
  broken = 'check'
}

/** `sendNotification` throws, as a plugin the OS refused does. */
export function breakSending(): void {
  broken = 'send'
}

/** Every notification the plugin was asked to show, oldest first. */
export function sentToasts(): SentToast[] {
  return sent
}

/** How many times the app asked the user for permission. */
export function permissionAsks(): number {
  return asks
}

/** Back to the defaults, and no notification sent. */
export function resetNotificationMock(): void {
  granted = true
  answer = 'granted'
  asks = 0
  sent = []
  broken = null
}

/** The `@tauri-apps/plugin-notification` module shape a test mocks in. */
export function notificationModule(): {
  isPermissionGranted: () => Promise<boolean>
  requestPermission: () => Promise<'granted' | 'denied' | 'prompt'>
  sendNotification: (options: SentToast) => void
} {
  return {
    isPermissionGranted: async () => {
      if (broken === 'check') throw new Error('the notification plugin is not installed')
      return granted
    },
    requestPermission: async () => {
      asks += 1
      return answer
    },
    sendNotification: (options: SentToast) => {
      if (broken === 'send') throw new Error('the OS refused the notification')
      sent.push(options)
    },
  }
}
