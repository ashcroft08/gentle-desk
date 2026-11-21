// The OS toast (notifications S2, S5): one notification through the official
// Tauri plugin, sent only once Windows allows it, and 'denied' - never a throw -
// whenever it does not. The plugin is the mock (`test-support/notification`), so
// these cases read exactly what Windows was asked to show.
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import {
  breakChecking,
  breakSending,
  denyPermission,
  permissionAsks,
  resetNotificationMock,
  sentToasts,
} from '../test-support/notification'
import { sendToast } from './toast'

vi.mock('@tauri-apps/plugin-notification', () =>
  import('../test-support/notification').then((m) => m.notificationModule()),
)

beforeEach(() => {
  resetNotificationMock()
})

afterEach(() => {
  resetNotificationMock()
})

describe('the system toast', () => {
  it('sends the notification when Windows already allows it', async () => {
    await expect(sendToast('Reply ready', 'The chat finished its reply.')).resolves.toBe('sent')

    expect(sentToasts()).toEqual([{ title: 'Reply ready', body: 'The chat finished its reply.' }])
    // Allowed already: the user is not asked again.
    expect(permissionAsks()).toBe(0)
  })

  it('asks once when it is not allowed yet, and sends when the answer allows it', async () => {
    denyPermission('granted')

    await expect(sendToast('Needs you', 'A chat is waiting.')).resolves.toBe('sent')

    expect(permissionAsks()).toBe(1)
    expect(sentToasts()).toEqual([{ title: 'Needs you', body: 'A chat is waiting.' }])
  })

  it('answers denied and sends nothing when the answer refuses', async () => {
    denyPermission('denied')

    await expect(sendToast('Needs you', 'A chat is waiting.')).resolves.toBe('denied')

    expect(permissionAsks()).toBe(1)
    expect(sentToasts()).toEqual([])
  })

  it('answers denied when the plugin refuses to send', async () => {
    breakSending()

    await expect(sendToast('Something failed', 'Disk is full')).resolves.toBe('denied')
    expect(sentToasts()).toEqual([])
  })

  it('answers denied when the plugin cannot even be asked', async () => {
    breakChecking()

    await expect(sendToast('Reply ready', 'The chat finished its reply.')).resolves.toBe('denied')
    expect(sentToasts()).toEqual([])
  })
})
