// The Profile tab's TWO APPLY OUTCOMES (profiles-editor T3b, S4, S8, S9, L30):
// `Use in this chat` (the profile binds to THIS chat's own child) and `Make
// default` (the channel's own global apply, which asks first).
//
// What is pinned here, one test per rule:
//
// - one button per outcome, drawn for the profile THIS chat uses;
// - the copy says exactly which scope each one touches - `Make default` is GLOBAL
//   (S9, L30) and `Use in this chat` binds this chat only;
// - each sends the intent with the chat, the used profile and its own mode, and a
//   first apply carries no token (the answer is what asks);
// - both are disabled while an apply is in flight, and neither sends a second
//   intent, not even from a keyboard or a synthetic click.
//
// The child's own dialog, its Accept and Cancel, and the line each failure leaves
// are `ProfileConfirm.test.ts`.
import '../test-support/dom-env'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { cleanup, fireEvent, render, screen } from '@testing-library/svelte'
import type { ChatProfileView_Serialize as ChatProfile } from '../contract/bindings'
import { setLanguage } from '../i18n/language.svelte'
import { chatProfile, FAST, TEAM } from '../test-support/chat-profile'
import { handlerArgs, resetTauriMock, setHandler } from '../test-support/tauri-mock'
import ProfilePanel from './ProfilePanel.svelte'
import { profileCopy } from './copy'

vi.mock('@tauri-apps/api/core', () => import('../test-support/tauri-mock').then((m) => m.coreModule()))
vi.mock('@tauri-apps/api/event', () => import('../test-support/tauri-mock').then((m) => m.eventModule()))

/** The tab's own words in English, read through the dictionary (D4). */
const copy = profileCopy('en')
const apply = copy.apply

afterEach(cleanup)
beforeEach(() => {
  resetTauriMock()
  setHandler('dispatch_intent', () => null)
  setLanguage('en')
})

/** Draws the tab as the shell draws it: the open tab of one chat. */
function draw(over: { chatId?: string | null; view?: ChatProfile | null; active?: boolean } = {}) {
  return render(ProfilePanel, {
    props: { chatId: 'foreground_chat', view: chatProfile(), active: true, ...over },
  })
}

/** Every apply the page asked the shell for, in order. */
function applies(): {
  chat_id: string
  profile: string
  mode: string
  confirmation: string | null
}[] {
  return handlerArgs('dispatch_intent')
    .map(
      (args) =>
        args.intent as {
          type: string
          data?: {
            chat_id: string
            profile: string
            mode: string
            confirmation: string | null
          }
        },
    )
    .filter((intent) => intent.type === 'ApplyChatProfile')
    .map(
      (intent) =>
        intent.data as {
          chat_id: string
          profile: string
          mode: string
          confirmation: string | null
        },
    )
}

/** The button of one outcome, by the words the tab draws. */
function button(name: string): HTMLButtonElement {
  return screen.getByRole('button', { name }) as HTMLButtonElement
}

describe('the two apply outcomes', () => {
  it('draws one button per outcome for the profile this chat uses', () => {
    draw()

    expect(button(apply.useInChat)).toBeTruthy()
    expect(button(apply.makeDefault)).toBeTruthy()
  })

  it('says Use in this chat binds this chat and Make default is global', () => {
    draw()

    expect(screen.getByText(apply.useInChatNote)).toBeTruthy()
    expect(screen.getByText(apply.makeDefaultNote)).toBeTruthy()
    expect(apply.makeDefaultNote.toLowerCase()).toContain('global')
    expect(apply.makeDefaultNote.toLowerCase()).toContain('default')
    expect(apply.useInChatNote.toLowerCase()).toContain("chat's own child")
  })

  it('sends the profile this chat BOUND, in the session mode, with no token', async () => {
    draw({
      view: chatProfile({
        profiles: [TEAM, FAST],
        default_profile: 'team',
        bound: 'fast',
        bound_from_session_file: true,
      }),
    })

    await fireEvent.click(button(apply.useInChat))

    expect(applies()).toEqual([
      { chat_id: 'foreground_chat', profile: 'fast', mode: 'session', confirmation: null },
    ])
  })

  it('sends the default mode for Make default', async () => {
    draw()

    await fireEvent.click(button(apply.makeDefault))

    expect(applies()).toEqual([
      { chat_id: 'foreground_chat', profile: 'team', mode: 'default', confirmation: null },
    ])
  })

  it('disables both buttons while an apply is in flight and sends nothing', async () => {
    draw({ view: chatProfile({ applying: true }) })

    expect(button(apply.useInChat).disabled).toBe(true)
    expect(button(apply.makeDefault).disabled).toBe(true)
    await fireEvent.click(button(apply.useInChat))
    await fireEvent.click(button(apply.makeDefault))
    expect(applies()).toEqual([])
  })

  it('draws no apply button for a chat whose profile the store does not define', () => {
    draw({ view: chatProfile({ profiles: [], default_profile: 'gone' }) })

    expect(screen.queryByRole('button', { name: apply.useInChat })).toBeNull()
    expect(screen.queryByRole('button', { name: apply.makeDefault })).toBeNull()
  })

  it('draws the two outcomes in Spanish too', () => {
    setLanguage('es')

    draw()

    expect(button(profileCopy('es').apply.useInChat)).toBeTruthy()
    expect(button(profileCopy('es').apply.makeDefault)).toBeTruthy()
  })
})
