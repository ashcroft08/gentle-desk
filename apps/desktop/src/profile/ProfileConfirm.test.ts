// The channel's OWN confirmation in the Profile tab (profiles-editor T3b, L30
// ruling 2): `Make default` is the channel's global apply, and gentle-pi asks
// before it writes.
//
// What is pinned here, one test per rule:
//
// - the dialog is drawn AS IT ARRIVED - the child's own title and message, in the
//   child's own language, never translated, never re-worded;
// - Accept resends the SAME profile and mode WITH the token of what was shown;
// - Cancel sends NOTHING at all and the dialog goes away;
// - a state push is never a reason to send an apply or to reopen a cancelled
//   dialog, and a NEW token is what asks again;
// - every answer that wrote nothing has its own short line, in both languages.
//
// The two buttons that start the flow are `ProfileApply.test.ts`.
import '../test-support/dom-env'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { cleanup, fireEvent, render, screen } from '@testing-library/svelte'
import { tick } from 'svelte'
import { writable } from 'svelte/store'
import type { ChatProfileView_Serialize as ChatProfile } from '../contract/bindings'
import { setLanguage } from '../i18n/language.svelte'
import { chatProfile } from '../test-support/chat-profile'
import { handlerArgs, resetTauriMock, setHandler } from '../test-support/tauri-mock'
import PanelHost from './PanelHost.test.svelte'
import ProfilePanel from './ProfilePanel.svelte'
import { applyCopy } from './apply-copy'
import { profileCopy } from './copy'

vi.mock('@tauri-apps/api/core', () => import('../test-support/tauri-mock').then((m) => m.coreModule()))
vi.mock('@tauri-apps/api/event', () => import('../test-support/tauri-mock').then((m) => m.eventModule()))

/** The tab's apply words in English, read through the dictionary (D4). */
const copy = applyCopy('en')

afterEach(cleanup)
beforeEach(() => {
  resetTauriMock()
  setHandler('dispatch_intent', () => null)
  setLanguage('en')
})

/** Draws the tab as the shell draws it, with the view the child's answer made. */
function draw(view: ChatProfile = chatProfile()) {
  return render(ProfilePanel, {
    props: { chatId: 'foreground_chat', view, active: true },
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
          data?: { chat_id: string; profile: string; mode: string; confirmation: string | null }
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

/** The child's own dialog, as the channel answered it. */
function asking(over: Partial<NonNullable<ChatProfile['apply_confirmation']>> = {}) {
  return {
    profile: 'team',
    mode: 'default' as const,
    title: 'Apply profile "team"?',
    message: 'New chats start on team. Chats already open keep their routing.',
    token: '3f1c0a',
    ...over,
  }
}

describe("the child's own confirmation", () => {
  it('is drawn exactly as it arrived, with Accept and Cancel', () => {
    const shown = asking()
    draw(chatProfile({ apply_confirmation: shown }))

    const dialog = screen.getByRole('alertdialog')
    expect(dialog.textContent).toContain(shown.title)
    expect(dialog.textContent).toContain(shown.message)
    expect(screen.getByRole('button', { name: copy.confirmAccept })).toBeTruthy()
    expect(screen.getByRole('button', { name: copy.confirmCancel })).toBeTruthy()
  })

  it("never translates the child's text: a Spanish answer stays Spanish", () => {
    const spanish = asking({
      title: '¿Aplicar el perfil "equipo"?',
      message: 'Los chats nuevos empiezan con equipo. Los chats abiertos conservan su ruta.',
    })
    draw(chatProfile({ apply_confirmation: spanish }))

    const dialog = screen.getByRole('alertdialog')
    expect(dialog.textContent).toContain(spanish.title)
    expect(dialog.textContent).toContain(spanish.message)
  })

  it('accepting resends the SAME profile and mode with the token of what was shown', async () => {
    const shown = asking()
    draw(chatProfile({ apply_confirmation: shown }))

    await fireEvent.click(screen.getByRole('button', { name: copy.confirmAccept }))

    expect(applies()).toEqual([
      { chat_id: 'foreground_chat', profile: 'team', mode: 'default', confirmation: shown.token },
    ])
  })

  it('cancelling sends nothing at all and the dialog goes away', async () => {
    draw(chatProfile({ apply_confirmation: asking() }))

    await fireEvent.click(screen.getByRole('button', { name: copy.confirmCancel }))
    await tick()

    expect(applies()).toEqual([])
    expect(screen.queryByRole('alertdialog')).toBeNull()
  })

  it('an apply that ended leaves no dialog behind', () => {
    draw(chatProfile({ apply_notice: 'unknown_profile' }))

    expect(screen.queryByRole('alertdialog')).toBeNull()
    expect(screen.getByText(copy.notices.unknown_profile)).toBeTruthy()
  })
})

describe('the one line an answer that wrote nothing leaves', () => {
  const cases = [
    'unknown_profile',
    'busy',
    'revision_conflict',
    'invalid_data',
    'no_answer',
    'unsupported',
    'not_applied',
    'refused',
  ] as const

  it('draws the line of each case, in English', () => {
    for (const notice of cases) {
      cleanup()
      draw(chatProfile({ apply_notice: notice }))
      expect(screen.getByText(copy.notices[notice]), notice).toBeTruthy()
    }
  })

  it('draws them in Spanish too, and every case has its own words', () => {
    setLanguage('es')
    const spanish = applyCopy('es')

    for (const notice of cases) {
      cleanup()
      draw(chatProfile({ apply_notice: notice }))
      expect(screen.getByText(spanish.notices[notice]), notice).toBeTruthy()
    }
    expect(new Set(cases.map((notice) => copy.notices[notice])).size).toBe(cases.length)
    expect(profileCopy('es').apply).toEqual(spanish)
  })
})

describe('a state push', () => {
  /** The panel mounted as App mounts it, over a store every push replaces. */
  function host(view: ChatProfile) {
    const source = writable({ chat: 'foreground_chat' as string | null, chatProfile: view })
    render(PanelHost, { props: { source } })
    return source
  }

  it('never sends an apply, and never reopens a dialog that was cancelled', async () => {
    const shown = asking()
    const source = host(chatProfile({ apply_confirmation: shown }))
    await tick()
    expect(screen.getByRole('alertdialog')).toBeTruthy()

    await fireEvent.click(screen.getByRole('button', { name: copy.confirmCancel }))
    await tick()
    expect(screen.queryByRole('alertdialog')).toBeNull()

    // Twenty pushes carrying the SAME answer are new views of the same chat: none
    // of them is a reason to send an apply or to ask again.
    for (let push = 0; push < 20; push += 1) {
      source.set({
        chat: 'foreground_chat',
        chatProfile: chatProfile({ apply_confirmation: shown }),
      })
      await tick()
      await new Promise((resolve) => setTimeout(resolve, 0))
    }

    expect(applies()).toEqual([])
    expect(screen.queryByRole('alertdialog')).toBeNull()
  })

  it('asks again when the child answers with a NEW token', async () => {
    const source = host(chatProfile({ apply_confirmation: asking({ token: 'first' }) }))
    await fireEvent.click(await screen.findByRole('button', { name: copy.confirmCancel }))
    await tick()
    expect(screen.queryByRole('alertdialog')).toBeNull()

    source.set({
      chat: 'foreground_chat',
      chatProfile: chatProfile({ apply_confirmation: asking({ token: 'second' }) }),
    })
    await tick()

    expect(screen.getByRole('alertdialog')).toBeTruthy()
  })
})
