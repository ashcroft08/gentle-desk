// A thread DRAWN from its dictionary (0.2.0, S2, S3): one local language write
// redraws the scroll name, the author lines, Copy, the live status, the
// model-error title and the activity line in the same tick - no push, no restart
// (S2) - while a message's own text and the provider's error keep arriving as
// they were written (D7).
//
// The English words are asserted literally: they are what the thread has drawn
// since the iced view. The two columns themselves are pinned in `copy.test.ts`.
import '../test-support/dom-env'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { cleanup, render, screen, within } from '@testing-library/svelte'
import { tick } from 'svelte'
import type { MessageView_Serialize as MessageView } from '../contract/bindings'
import { resetTauriMock, setHandler, viewState } from '../test-support/tauri-mock'
import { setLanguage } from '../i18n/language.svelte'
import Thread from './Thread.svelte'

vi.mock('@tauri-apps/api/core', () => import('../test-support/tauri-mock').then((m) => m.coreModule()))
vi.mock('@tauri-apps/api/event', () => import('../test-support/tauri-mock').then((m) => m.eventModule()))

/** The user's own words, which no language of ours rewrites (D7). */
const SAID = 'please rename the workspace'
/** The provider's own error text, shown exactly as it arrived (D7). */
const ERROR = '429 too many requests'

function user(id: string, text: string): MessageView {
  return { id, role: 'User', text, streaming: false, error: null, images: [] }
}

function assistant(id: string, text: string, over: Partial<MessageView> = {}): MessageView {
  return { id, role: 'Assistant', text, streaming: false, error: null, images: [], ...over }
}

/** The message's article, by the author line it carries. */
function article(name: string): HTMLElement {
  return screen.getByRole('article', { name })
}

afterEach(cleanup)

beforeEach(() => {
  resetTauriMock()
  setHandler('dispatch_intent', () => viewState())
  setHandler('open_link', () => null)
})

describe('a drawn thread', () => {
  it('flips the scroll name, the author lines and Copy in the same tick', async () => {
    setLanguage('en')
    render(Thread, {
      props: {
        messages: [user('u1', SAID), assistant('a1', 'renamed it')],
        foregroundChat: 'chat-1',
      },
    })

    expect(screen.getByRole('region', { name: 'Chat thread' })).toBeTruthy()
    expect(within(article('You said')).getByText(SAID)).toBeTruthy()
    expect(within(article('Assistant said')).getByRole('button', { name: 'Copy' })).toBeTruthy()

    setLanguage('es')
    await tick()

    expect(screen.getByRole('region', { name: 'Hilo del chat' })).toBeTruthy()
    expect(within(article('Tú dijiste')).getByText(SAID)).toBeTruthy()
    expect(within(article('El asistente dijo')).getByRole('button', { name: 'Copiar' })).toBeTruthy()

    setLanguage('en')
  })

  it('flips the live status and the activity line, leaving the provider error alone', async () => {
    setLanguage('en')
    render(Thread, {
      props: {
        messages: [assistant('a1', ERROR, { error: ERROR }), assistant('a2', '', { streaming: true })],
        foregroundChat: 'chat-1',
        mark: 'Working',
      },
    })

    expect(screen.getByText('Model error')).toBeTruthy()
    expect(screen.getByText('Assistant is replying')).toBeTruthy()
    expect(screen.getByText('Working')).toBeTruthy()
    // D7: the provider's text is not ours, so it reads the same afterwards.
    expect(screen.getByText(ERROR)).toBeTruthy()

    setLanguage('es')
    await tick()

    expect(screen.getByText('Error del modelo')).toBeTruthy()
    expect(screen.getByText('El asistente está respondiendo')).toBeTruthy()
    expect(screen.getByText('Trabajando')).toBeTruthy()
    expect(screen.getByText(ERROR)).toBeTruthy()

    cleanup()
    setLanguage('es')
    render(Thread, { props: { messages: [], foregroundChat: 'chat-1', mark: 'Opening' } })
    expect(screen.getByText('Abriendo el chat')).toBeTruthy()

    setLanguage('en')
    await tick()

    expect(screen.getByText('Opening chat')).toBeTruthy()
  })
})
