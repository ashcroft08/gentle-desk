// The alert's tone (notifications S2): a short two-note greeting synthesized on
// the spot, from ONE AudioContext per window, and silence - never an error -
// wherever the webview will not play it. The window's own WebAudio is the mock
// (`test-support/audio`), so these cases read the envelope the maintainer hears
// instead of hearing it. Every case boots a FRESH copy of the module, because
// the context belongs to the module's lifetime: that is what "one per window"
// means and what `vi.resetModules` gives a test.
import { afterEach, describe, expect, it, vi } from 'vitest'
import { audioWindow } from '../test-support/audio'

type Tone = typeof import('./tone')

/** A fresh module, as one window gets one. */
async function fresh(): Promise<Tone> {
  vi.resetModules()
  return await import('./tone')
}

afterEach(() => {
  vi.unstubAllGlobals()
})

describe('the alert tone', () => {
  it('plays two short notes on one synthesized voice', async () => {
    const window = audioWindow()
    const tone = await fresh()

    await tone.playTone()

    const notes = window.notes()
    expect(notes).toHaveLength(2)
    // A tone, not a noise: a pitched sine, and the two notes rise (a pleasant
    // interval, never a flat beep).
    expect(notes.map((note) => note.wave)).toEqual(['sine', 'sine'])
    expect(notes[1].frequency).toBeGreaterThan(notes[0].frequency)
    expect(notes.every((note) => note.frequency > 200)).toBe(true)
    // Both notes are audible: each one's gain reaches the destination.
    expect(notes.every((note) => note.audible)).toBe(true)
  })

  it('keeps the whole tone short and quiet', async () => {
    const window = audioWindow()
    const tone = await fresh()

    await tone.playTone()

    const notes = window.notes()
    const start = Math.min(...notes.map((note) => note.start))
    const stop = Math.max(...notes.map((note) => note.stop))
    // About 300 ms: long enough to be heard, short enough not to be an alarm.
    expect(stop - start).toBeGreaterThan(0.2)
    expect(stop - start).toBeLessThanOrEqual(0.4)
    // Low volume: a hint next to someone's ear, never a startle.
    const peaks = notes.flatMap((note) => note.envelope.map((point) => point.value))
    expect(Math.max(...peaks)).toBeGreaterThan(0)
    expect(Math.max(...peaks)).toBeLessThanOrEqual(0.15)
  })

  it('fades each note in and back to silence instead of clicking', async () => {
    const window = audioWindow()
    const tone = await fresh()

    await tone.playTone()

    for (const note of window.notes()) {
      expect(note.envelope.map((point) => point.kind)).toEqual(['set', 'linear', 'exponential'])
      const [first, , last] = note.envelope
      // Leaves from (almost) nothing, swells, and falls back to (almost) nothing.
      expect(first.value).toBeLessThan(0.001)
      expect(last.value).toBeLessThan(0.001)
      expect(last.at).toBeGreaterThan(first.at)
    }
  })

  it('creates one context for the whole window, however many tones play', async () => {
    const window = audioWindow()
    const tone = await fresh()

    await tone.playTone()
    await tone.playTone()

    expect(window.contexts).toHaveLength(1)
    expect(window.notes()).toHaveLength(4)
    // Nothing is torn down between tones: the next alert reuses the same voice.
    expect(window.contexts[0].closes).toBe(0)
  })

  it('resumes a suspended context before playing', async () => {
    // What autoplay policy does to a context created without a user gesture.
    const window = audioWindow({ state: 'suspended' })
    const tone = await fresh()

    await tone.playTone()

    expect(window.contexts[0].resumes).toBe(1)
    expect(window.contexts[0].state).toBe('running')
    expect(window.notes()).toHaveLength(2)
  })

  it('resolves quietly when the webview has no AudioContext', async () => {
    const window = audioWindow({ unsupported: true })
    const tone = await fresh()

    await expect(tone.playTone()).resolves.toBeUndefined()
    expect(window.contexts).toEqual([])
  })

  it('resolves quietly when the context cannot be created', async () => {
    const window = audioWindow({ refuses: true })
    const tone = await fresh()

    await expect(tone.playTone()).resolves.toBeUndefined()
    expect(window.contexts).toEqual([])
  })

  it('resolves quietly when the context is blocked from resuming', async () => {
    const window = audioWindow({ state: 'suspended', blocked: true })
    const tone = await fresh()

    await expect(tone.playTone()).resolves.toBeUndefined()
    expect(window.contexts[0].resumes).toBe(1)
    // A context that will not resume is not made to schedule a note it could not
    // play: silence, and not a queue of pending ones.
    expect(window.notes()).toEqual([])
  })
})
