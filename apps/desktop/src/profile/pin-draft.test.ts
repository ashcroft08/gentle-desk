// The pin editor's own rules (profiles-editor T6, S2, S3), as plain data: what a
// layer's picker holds, whether that is a change, which names it may offer, and
// whether a change may be sent at all. They are the channel's three rules, caught
// here so the app never sends what the owner would refuse: a name the profiles
// store defines (or the explicit clear), a layer this worktree offers a write
// target for, and nothing when the layer already holds the pin.
import { describe, expect, it } from 'vitest'
import { pinLayer } from '../test-support/chat-profile'
import {
  NO_PIN,
  canSavePin,
  choiceOf,
  dirty,
  pinOptions,
  pinProblem,
  profileOf,
} from './pin-draft'

/** The profiles the store defines, as the picker gets them. */
const DEFINED = ['team', 'fast']

describe('what one layer’s picker holds', () => {
  it('holds the pin the store reports', () => {
    expect(choiceOf(pinLayer({ source: 'repo', profile: 'fast' }))).toBe('fast')
  })

  it('holds the explicit clear for a layer with no pin', () => {
    expect(choiceOf(pinLayer({ source: 'local', status: 'missing' }))).toBe(NO_PIN)
  })

  it('holds nothing at all for a pin the store does not report', () => {
    expect(choiceOf(pinLayer({ source: 'local', outranked: true }))).toBeNull()
  })

  it('turns its own choice back into the store’s profile', () => {
    expect(profileOf('fast')).toBe('fast')
    expect(profileOf(NO_PIN)).toBeNull()
    expect(profileOf(null)).toBeNull()
  })
})

describe('whether a choice is a change', () => {
  it('is not a change when it is what the layer already holds', () => {
    const layer = pinLayer({ source: 'repo', profile: 'fast' })
    expect(dirty(layer, 'fast')).toBe(false)
    expect(dirty(pinLayer({ source: 'local' }), NO_PIN)).toBe(false)
    expect(dirty(pinLayer({ source: 'local', outranked: true }), null)).toBe(false)
  })

  it('is a change when it says something else, the clear included', () => {
    const layer = pinLayer({ source: 'repo', profile: 'fast' })
    expect(dirty(layer, 'team')).toBe(true)
    expect(dirty(layer, NO_PIN)).toBe(true)
    expect(dirty(pinLayer({ source: 'local' }), 'fast')).toBe(true)
  })
})

describe('which names a picker offers', () => {
  it('offers every profile the store defines', () => {
    expect(pinOptions(pinLayer({ source: 'repo', profile: 'fast' }), DEFINED)).toEqual(DEFINED)
  })

  it('keeps a stale pin as an option, so the layer’s own state stays visible', () => {
    expect(pinOptions(pinLayer({ source: 'repo', profile: 'gone', mark: 'stale' }), DEFINED)).toEqual(
      ['team', 'fast', 'gone'],
    )
  })
})

describe('whether a change may be sent', () => {
  it('may be sent when it is a change to a name the store defines', () => {
    const layer = pinLayer({ source: 'repo', profile: 'fast' })
    expect(canSavePin(layer, 'team', DEFINED, false)).toBe(true)
    expect(canSavePin(layer, NO_PIN, DEFINED, false)).toBe(true)
  })

  it('may not be sent while a write is in flight', () => {
    expect(canSavePin(pinLayer({ source: 'repo', profile: 'fast' }), 'team', DEFINED, true)).toBe(
      false,
    )
  })

  it('may not be sent for a layer this worktree offers no target for', () => {
    const layer = pinLayer({ source: 'repo', profile: 'fast', writable: false })
    expect(canSavePin(layer, 'team', DEFINED, false)).toBe(false)
  })

  it('may not be sent for the unreported starting choice', () => {
    expect(canSavePin(pinLayer({ source: 'local', outranked: true }), null, DEFINED, false)).toBe(
      false,
    )
  })

  it('refuses a name the profiles store does not define, and says which', () => {
    const layer = pinLayer({ source: 'repo', profile: 'fast' })
    expect(pinProblem('gone', DEFINED)).toEqual({ kind: 'unknown-profile', profile: 'gone' })
    expect(canSavePin(layer, 'gone', DEFINED, false)).toBe(false)
    expect(pinProblem('team', DEFINED)).toBeNull()
    expect(pinProblem(NO_PIN, DEFINED)).toBeNull()
    expect(pinProblem(null, DEFINED)).toBeNull()
  })
})
