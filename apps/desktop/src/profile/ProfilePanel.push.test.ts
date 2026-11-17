// A state push is not a reason to read (profiles-editor T2c, L13): the tab reads
// once per open, and the pushes that keep arriving while it is open - one per
// child event of a streaming chat, and one for the read's own answer - must not
// start another read. Mounted as App.svelte mounts it, where every push hands the
// panel a new chat object that names the SAME chat.
import '../test-support/dom-env'
import { afterEach, beforeEach, expect, it, vi } from 'vitest'
import { cleanup, fireEvent, render, screen, waitFor } from '@testing-library/svelte'
import { tick } from 'svelte'
import { writable } from 'svelte/store'
import { chatProfile } from '../test-support/chat-profile'
import { handlerArgs, resetTauriMock, setHandler } from '../test-support/tauri-mock'
import PanelHost from './PanelHost.test.svelte'
import { profileCopy } from './copy'

vi.mock('@tauri-apps/api/core', () => import('../test-support/tauri-mock').then((m) => m.coreModule()))
vi.mock('@tauri-apps/api/event', () => import('../test-support/tauri-mock').then((m) => m.eventModule()))

function reads(): number {
  return handlerArgs('dispatch_intent').filter(
    (args) => (args.intent as { type: string }).type === 'RefreshChatProfile',
  ).length
}

afterEach(cleanup)
beforeEach(resetTauriMock)

it('reads once however many pushes name the same chat', async () => {
  setHandler('dispatch_intent', () => null)
  const source = writable({ chat: 'foreground_chat' as string | null, chatProfile: null as ReturnType<typeof chatProfile> | null })
  render(PanelHost, { props: { source } })
  await waitFor(() => expect(reads()).toBe(1))

  for (let push = 0; push < 20; push += 1) {
    source.set({ chat: 'foreground_chat', chatProfile: push % 2 === 0 ? null : chatProfile() })
    await tick()
    await new Promise((resolve) => setTimeout(resolve, 0))
  }

  expect(reads()).toBe(1)
})

/** Every mode save the page asked the shell for. */
function saves(): number {
  return handlerArgs('dispatch_intent').filter(
    (args) => (args.intent as { type: string }).type === 'SaveChatProfileMode',
  ).length
}

it('does not re-dispatch a mode save on the pushes that answer it', async () => {
  setHandler('dispatch_intent', () => null)
  const source = writable({
    chat: 'foreground_chat' as string | null,
    chatProfile: chatProfile() as ReturnType<typeof chatProfile> | null,
  })
  render(PanelHost, { props: { source } })
  await waitFor(() => expect(reads()).toBe(1))

  // The switch writes once, as the user clicked it.
  const switchOf = async () =>
    screen.getByRole('radio', { name: profileCopy('en').modeFullDelegation })
  await fireEvent.click(await switchOf())
  await waitFor(() => expect(saves()).toBe(1))

  // The pushes that answer the write (a save in flight, the failure notice, the
  // re-read) are new views of the same chat: none of them is a reason to write.
  for (let push = 0; push < 20; push += 1) {
    source.set({
      chat: 'foreground_chat',
      chatProfile: chatProfile({ saving: push === 0, save_failed: push === 1 }),
    })
    await tick()
    await new Promise((resolve) => setTimeout(resolve, 0))
  }

  expect(saves()).toBe(1)
  expect(reads()).toBe(1)
})
