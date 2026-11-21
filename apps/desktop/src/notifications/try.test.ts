// The Settings pane's Try it (notifications S5): the tone first, then one toast,
// and the toast's own answer back to the pane - which is what tells the
// maintainer whether Windows allows notifications without waiting for a real
// event. This is the API the pane imports, so its name and signature are pinned
// here as much as its order is.
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import type { LanguageView } from '../contract/bindings'

const { played, answer, playTone, sendToast } = vi.hoisted(() => {
  const order: string[] = []
  const reply: { value: 'sent' | 'denied' } = { value: 'sent' }
  return {
    played: order,
    answer: reply,
    playTone: vi.fn(async () => {
      order.push('tone')
    }),
    sendToast: vi.fn<(title: string, body: string) => Promise<'sent' | 'denied'>>(async () => {
      order.push('toast')
      return reply.value
    }),
  }
})

vi.mock('./tone', () => ({ playTone }))
vi.mock('./toast', () => ({ sendToast }))

import { notifyCopy } from './copy'
import { tryNotification } from './try'

beforeEach(() => {
  played.length = 0
  answer.value = 'sent'
  playTone.mockClear()
  sendToast.mockClear()
})

afterEach(() => {
  played.length = 0
})

describe('the Try it the Settings pane calls', () => {
  it('plays the tone before it sends the toast', async () => {
    await tryNotification('en')

    expect(played).toEqual(['tone', 'toast'])
    expect(sendToast).toHaveBeenCalledTimes(1)
  })

  it('answers what the toast answered, denial included', async () => {
    answer.value = 'denied'

    await expect(tryNotification('en')).resolves.toBe('denied')
    await expect(tryNotification('en')).resolves.toBe('denied')
    // The tone still plays: hearing it is half of what the button is for.
    expect(playTone).toHaveBeenCalledTimes(2)
  })

  it('writes the one toast in the language the app speaks', async () => {
    const english = await tryNotification('en')
    expect(english).toBe('sent')
    expect(sendToast).toHaveBeenLastCalledWith(
      notifyCopy('en').testTitle,
      notifyCopy('en').testBody,
    )

    sendToast.mockClear()
    await tryNotification('es')
    expect(sendToast).toHaveBeenLastCalledWith(
      notifyCopy('es').testTitle,
      notifyCopy('es').testBody,
    )
    expect(notifyCopy('es').testBody).not.toBe(notifyCopy('en').testBody)
  })

  it('keeps a dictionary for every language the app knows', () => {
    const languages: LanguageView[] = ['en', 'es']

    for (const language of languages) {
      const copy = notifyCopy(language)
      for (const words of Object.values(copy)) {
        expect(words.trim()).not.toBe('')
      }
    }
  })
})
