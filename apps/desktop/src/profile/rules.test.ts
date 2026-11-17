// The Profile tab's two pure rules (profiles-editor T2c): which of the four
// states the page draws, and which profile the chat runs - the bound one, else the
// global default, else nothing at all. They are plain data so the components only
// draw a decision (the `chat/tabs.ts` pattern).
import { describe, expect, it } from 'vitest'
import { chatProfile } from '../test-support/chat-profile'
import { missingStores, panelState, usedName, usedProfile } from './rules'

describe('panelState', () => {
  it('is reading until the first answer lands, and nothing before that is a failure', () => {
    expect(panelState(null, false)).toBe('reading')
  })

  it("is unavailable once a read answered nothing, or when no store could be read", () => {
    expect(panelState(null, true)).toBe('unavailable')
    expect(
      panelState(chatProfile({ stores: { profiles: 'unavailable', fallbacks: 'skipped', pins: 'skipped' } }), true),
    ).toBe('unavailable')
  })

  it('is partial when one store was read and another was not', () => {
    const view = chatProfile({ stores: { profiles: 'read', fallbacks: 'read', pins: 'skipped' } })

    expect(panelState(view, true)).toBe('partial')
    expect(missingStores(view)).toEqual(['pins'])
  })

  it('is ready when all three stores answered', () => {
    expect(panelState(chatProfile(), true)).toBe('ready')
    expect(missingStores(chatProfile())).toEqual([])
  })
})

describe('the profile this chat uses', () => {
  it('is the default profile for a chat that bound nothing', () => {
    const view = chatProfile()

    expect(usedName(view)).toBe('team')
    expect(usedProfile(view)?.name).toBe('team')
  })

  it('is the bound profile, whatever the default is', () => {
    const view = chatProfile({ bound: 'fast', bound_from_session_file: true })

    expect(usedName(view)).toBe('fast')
    expect(usedProfile(view)?.name).toBe('fast')
  })

  it('is nothing at all when neither a binding nor a default is named', () => {
    const view = chatProfile({ bound: null, default_profile: null })

    expect(usedName(view)).toBeNull()
    expect(usedProfile(view)).toBeNull()
  })

  it('keeps the bound name even when the store no longer defines it', () => {
    const view = chatProfile({ bound: 'gone', bound_from_session_file: true })

    expect(usedName(view)).toBe('gone')
    expect(usedProfile(view)).toBeNull()
  })
})
