// The parity rule of the composer, one test per state: working turns Send into
// Stop (a queued chat shows the same Stop, as iced does: it cancels the queued
// prompt), opening and ended disable Send and each say why, and a finished reply
// or a pending dialog change nothing.

import { describe, expect, it } from 'vitest'
import { composerCopy } from './copy'
import { composerStateOf } from './state'

/** The dictionary a run that has not been told otherwise draws its words from. */
const EN = composerCopy('en')

// The two reasons read through the dictionary, never as literals of their own.
const { endedReason: ENDED_REASON, openingReason: OPENING_REASON } = EN

describe('composerStateOf', () => {
  it('offers Send on an idle chat', () => {
    expect(composerStateOf(null, EN)).toEqual({ mode: 'send', disabled: false, reason: null })
  })

  it('turns Send into Stop while the chat is working', () => {
    expect(composerStateOf('Working', EN)).toEqual({ mode: 'stop', disabled: false, reason: null })
  })

  it('disables Send while the chat is opening, and says so', () => {
    expect(composerStateOf('Opening', EN)).toEqual({
      mode: 'send',
      disabled: true,
      reason: OPENING_REASON,
    })
  })

  it('shows the same Stop while the chat waits for a working slot', () => {
    expect(composerStateOf('Queued', EN)).toEqual({ mode: 'stop', disabled: false, reason: null })
  })

  it('disables Send once the chat process is gone, and points at the restart', () => {
    expect(composerStateOf('Ended', EN)).toEqual({
      mode: 'send',
      disabled: true,
      reason: ENDED_REASON,
    })
  })

  it('leaves the composer ready after a reply nobody has read yet', () => {
    expect(composerStateOf('NewReply', EN)).toEqual({ mode: 'send', disabled: false, reason: null })
    expect(composerStateOf('NeedsAttention', EN)).toEqual({
      mode: 'send',
      disabled: false,
      reason: null,
    })
  })
})
