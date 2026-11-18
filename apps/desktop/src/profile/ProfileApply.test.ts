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
// are `ProfileConfirm.test.ts`. The child's own ANSWER to an apply that ran is the
// last block here (`applied` and `not_applied` both said what they did).
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

// The child's own answer to an apply that RAN, verbatim (profiles-editor T3b, L32):
// `Make default` on a profile that already is the default applied, said what it
// did and the tab drew NOTHING, so the user could not tell anything had happened.
const ANSWER = [
  'el Gentleman applied profile "team" — 0 agents updated.\nNew routing takes effect on the next subagent launch.\nOrchestrator set to anthropic/claude-sonnet-4 in settings.json.',
  'anthropic/claude-sonnet-4 is not in the model catalog; this session keeps its current model.',
]

/**
 * One note as the tab drew it, compared with the child's text UNTOUCHED: the
 * default matcher collapses whitespace, which would hide a lost line break.
 */
function drawnNote(text: string): HTMLElement {
  return screen.getByText(text, { normalizer: (drawn) => drawn })
}

describe("the child's own answer to an apply that ran", () => {
  it('draws every note of it, verbatim, with no word of our own around it', () => {
    draw({ view: chatProfile({ apply_answer: { notes: ANSWER } }) })

    for (const note of ANSWER) {
      const drawn = drawnNote(note)
      expect(drawn.textContent).toBe(note)
    }
  })

  it('keeps the lines the child wrote inside one note', () => {
    draw({ view: chatProfile({ apply_answer: { notes: [ANSWER[0]] } }) })

    expect(drawnNote(ANSWER[0]).textContent?.split('\n')).toHaveLength(3)
  })

  it("draws the child's own words in Spanish too: they are not the app's to translate", () => {
    setLanguage('es')

    draw({ view: chatProfile({ apply_answer: { notes: ANSWER } }) })

    expect(drawnNote(ANSWER[0])).toBeTruthy()
    expect(drawnNote(ANSWER[1])).toBeTruthy()
  })

  it('stops drawing it as soon as a view does not carry it', async () => {
    const drawn = draw({ view: chatProfile({ apply_answer: { notes: ANSWER } }) })
    expect(drawnNote(ANSWER[0])).toBeTruthy()

    await drawn.rerender({ chatId: 'foreground_chat', view: chatProfile(), active: true })

    expect(screen.queryByText(ANSWER[0], { normalizer: (drawn) => drawn })).toBeNull()
  })

  it('draws nothing at all when the answer carries no notes', () => {
    draw({ view: chatProfile({ apply_answer: null }) })

    expect(screen.queryByRole('status')).toBeNull()
  })
})
