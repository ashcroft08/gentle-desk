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
import { chain, chatProfile, definition } from '../test-support/chat-profile'
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

/** Every chain save the page asked the shell for. */
function chainSaves(): { agent: string; steps: { model: string; thinking: string | null }[] }[] {
  return handlerArgs('dispatch_intent')
    .map((args) => args.intent as { type: string; data?: { agent: string; steps: never } })
    .filter((intent) => intent.type === 'SaveChatProfileChain')
    .map((intent) => intent.data as never)
}

/** The used profile's `worker` chain, as the editor draws it. */
const worked = definition('team', {
  agents: [{ agent: 'worker', model: 'openai/gpt-5-codex', thinking: 'high', tier: null }],
  chains: [chain('worker', [{ model: 'openai/gpt-fallback', thinking: 'low' }])],
})

it('does not re-dispatch a chain save on the pushes that answer it', async () => {
  setHandler('dispatch_intent', () => null)
  const source = writable({
    chat: 'foreground_chat' as string | null,
    chatProfile: chatProfile({ profiles: [worked], default_profile: 'team' }) as ReturnType<
      typeof chatProfile
    > | null,
  })
  render(PanelHost, { props: { source } })
  await waitFor(() => expect(reads()).toBe(1))

  // The editor writes once, as the user clicked Save.
  const field = screen.getByRole('textbox', { name: profileCopy('en').chainModel(1) })
  await fireEvent.input(field, { target: { value: 'openai/gpt-edited' } })
  expect(chainSaves()).toEqual([])
  await fireEvent.click(screen.getByRole('button', { name: profileCopy('en').chainSave }))
  await waitFor(() => expect(chainSaves()).toHaveLength(1))

  // The pushes that answer the write (a save in flight, the failure notice, the
  // re-read) are new views of the same chat: none of them is a reason to write.
  for (let push = 0; push < 20; push += 1) {
    source.set({
      chat: 'foreground_chat',
      chatProfile: chatProfile({
        profiles: [worked],
        default_profile: 'team',
        saving: push === 0,
        chain_failed: push === 1,
      }),
    })
    await tick()
    await new Promise((resolve) => setTimeout(resolve, 0))
  }

  expect(chainSaves()).toHaveLength(1)
  expect(reads()).toBe(1)
})

it('keeps the typed draft when a push carries the same chain', async () => {
  setHandler('dispatch_intent', () => null)
  const source = writable({
    chat: 'foreground_chat' as string | null,
    chatProfile: chatProfile({ profiles: [worked], default_profile: 'team' }) as ReturnType<
      typeof chatProfile
    > | null,
  })
  render(PanelHost, { props: { source } })
  await waitFor(() => expect(reads()).toBe(1))

  const field = screen.getByRole('textbox', { name: profileCopy('en').chainModel(1) })
  await fireEvent.input(field, { target: { value: 'openai/gpt-typed' } })

  // A chat that streams pushes a new view per event, with the SAME chain in it.
  for (let push = 0; push < 5; push += 1) {
    source.set({
      chat: 'foreground_chat',
      chatProfile: chatProfile({ profiles: [worked], default_profile: 'team' }),
    })
    await tick()
    await new Promise((resolve) => setTimeout(resolve, 0))
  }

  const kept = screen.getByRole('textbox', { name: profileCopy('en').chainModel(1) })
  expect((kept as HTMLInputElement).value).toBe('openai/gpt-typed')
  await fireEvent.click(screen.getByRole('button', { name: profileCopy('en').chainSave }))
  await waitFor(() => expect(chainSaves()).toHaveLength(1))
  expect(chainSaves()[0].steps).toEqual([{ model: 'openai/gpt-typed', thinking: 'low' }])
})

/** Every pin save the page asked the shell for. */
function pinSaves(): { chat_id: string; source: string; profile: string | null }[] {
  return handlerArgs('dispatch_intent')
    .map((args) => args.intent as { type: string; data?: { source: string; profile: string | null } })
    .filter((intent) => intent.type === 'SaveChatProfilePin')
    .map((intent) => intent.data as never)
}

/** One layer's picker, by the label the tab gives it. */
function pinPicker(name: string): HTMLSelectElement {
  return screen.getByRole('combobox', { name }) as HTMLSelectElement
}

it('does not re-dispatch a pin save on the pushes that answer it', async () => {
  setHandler('dispatch_intent', () => null)
  const source = writable({
    chat: 'foreground_chat' as string | null,
    chatProfile: chatProfile() as ReturnType<typeof chatProfile> | null,
  })
  render(PanelHost, { props: { source } })
  await waitFor(() => expect(reads()).toBe(1))

  // The editor writes once, as the user picked a profile and clicked Save.
  await fireEvent.click(screen.getByRole('button', { name: /^Pin/ }))
  await fireEvent.change(pinPicker(profileCopy('en').pinPick(profileCopy('en').pinRepo)), {
    target: { value: 'team' },
  })
  expect(pinSaves()).toEqual([])
  await fireEvent.click(
    screen.getByRole('button', { name: profileCopy('en').pinSaveLayer(profileCopy('en').pinRepo) }),
  )
  await waitFor(() => expect(pinSaves()).toHaveLength(1))

  // The pushes that answer the write (a save in flight, the failure notice, the
  // re-read) are new views of the same chat: none of them is a reason to write.
  for (let push = 0; push < 20; push += 1) {
    source.set({
      chat: 'foreground_chat',
      chatProfile: chatProfile({ saving: push === 0, pin_failed: push === 1 }),
    })
    await tick()
    await new Promise((resolve) => setTimeout(resolve, 0))
  }

  expect(pinSaves()).toHaveLength(1)
  expect(reads()).toBe(1)
})

it('keeps the chosen pin when a push carries the same pins', async () => {
  setHandler('dispatch_intent', () => null)
  const source = writable({
    chat: 'foreground_chat' as string | null,
    chatProfile: chatProfile() as ReturnType<typeof chatProfile> | null,
  })
  render(PanelHost, { props: { source } })
  await waitFor(() => expect(reads()).toBe(1))

  await fireEvent.click(screen.getByRole('button', { name: /^Pin/ }))
  const repo = profileCopy('en').pinRepo
  await fireEvent.change(pinPicker(profileCopy('en').pinPick(repo)), {
    target: { value: 'team' },
  })

  // A chat that streams pushes a new view per event, with the SAME pins in it.
  for (let push = 0; push < 5; push += 1) {
    source.set({ chat: 'foreground_chat', chatProfile: chatProfile() })
    await tick()
    await new Promise((resolve) => setTimeout(resolve, 0))
  }

  const kept = pinPicker(profileCopy('en').pinPick(repo))
  expect(kept.value).toBe('team')
  await fireEvent.click(
    screen.getByRole('button', { name: profileCopy('en').pinSaveLayer(repo) }),
  )
  await waitFor(() => expect(pinSaves()).toHaveLength(1))
  expect(pinSaves()[0]).toEqual({
    chat_id: 'foreground_chat',
    source: 'repo',
    profile: 'team',
  })
})
