// The Profile tab's render cost (profiles-editor T2c, the performance defect): the
// panel is redrawn on every state push of its chat, and a streaming reply is about
// one push per event. A redraw must cost what the DRAWING needs - the heads of the
// groups, and the body of the ONE group that is open - never the agents and chains
// of every profile in the store.
//
// The counter is `chainName`, the one pure helper the chain rows go through
// (`ProfileChains`): it is called once per chain of a group the render actually
// builds, so its call count is the per-profile label work a redraw paid for.
//
// What this does NOT measure: no frame time, no paint, no layout (jsdom does none
// of them) and no real push rate - only how much drawing work a push sets off.
import '../test-support/dom-env'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { cleanup, render, screen } from '@testing-library/svelte'
import { setLanguage } from '../i18n/language.svelte'
import { chatProfile } from '../test-support/chat-profile'
import { resetTauriMock, setHandler } from '../test-support/tauri-mock'
import ProfilePanel from './ProfilePanel.svelte'
import { chainName, profileCopy } from './copy'

vi.mock('@tauri-apps/api/core', () => import('../test-support/tauri-mock').then((m) => m.coreModule()))
vi.mock('@tauri-apps/api/event', () => import('../test-support/tauri-mock').then((m) => m.eventModule()))

// The real helper, counted: the labels drawn are the app's own words (D4), and the
// count is what the assertion reads.
vi.mock('./copy', async (importOriginal) => {
  const actual = await importOriginal<typeof import('./copy')>()
  return { ...actual, chainName: vi.fn(actual.chainName) }
})

/** How many times the chain label helper has run. */
function labels(): number {
  return vi.mocked(chainName).mock.calls.length
}

afterEach(cleanup)
beforeEach(() => {
  resetTauriMock()
  setHandler('dispatch_intent', () => null)
  setLanguage('en')
  vi.mocked(chainName).mockClear()
})

describe('a redraw of the open Profile tab', () => {
  it('labels the chains of the open group only, not of every profile per push', async () => {
    // The fixture: two profiles (four agents, two chains), the chat on `team`, so
    // `team` is the open group and its one chain is the only body in the drawing.
    const drawn = render(ProfilePanel, {
      props: { chatId: 'foreground_chat', view: chatProfile(), active: true },
    })
    expect(labels()).toBe(1)

    // Twenty pushes, each a deep-equal-but-NEW view object, as Rust sends them.
    const passes = 20
    for (let pass = 1; pass <= passes; pass += 1) {
      await drawn.rerender({ chatId: 'foreground_chat', view: chatProfile(), active: true })
    }

    // The bound is the drawing's, never per profile: at most one label per redraw
    // (the open group's single chain), where building every group's body costs two
    // - one of them for a profile that is not on screen. Observed with the fix: 1
    // in total, because Svelte 5 does not re-run a label whose value did not move.
    expect(labels()).toBeLessThanOrEqual(passes + 1)
    // And the open group's chain is still drawn: the count is bounded because the
    // work is the drawing's, not because the drawing stopped.
    expect(screen.getByText(profileCopy('en').profileWide)).toBeTruthy()
  })
})
