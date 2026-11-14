// The Usage pane's rules (settings-0.1.12 S6's deferred half): the four pane
// states and what one provider's card draws. A null is not a zero anywhere - a
// provider that answered "no data" stays "no data" all the way to the drawn row.
import { describe, expect, it } from 'vitest'
import type { UsageView_Serialize as UsageView } from '../contract/bindings'
import { paneOf } from './usage'

/** One provider as Rust pushes it, overridable field by field. */
function provider(over: Partial<UsageView['providers'][number]> = {}): UsageView['providers'][number] {
  return {
    provider: 'commandcode',
    status: 'Ok',
    plan: 'individual goat',
    error: null,
    limits: [
      {
        name: 'command code',
        limit_reached: false,
        windows: [
          { label: '5h', used_percent: 4.677, resets_in: 'resets in 2h' },
          { label: 'week', used_percent: 88.27, resets_in: 'resets in 6d' },
        ],
      },
    ],
    ...over,
  }
}

function usage(over: Partial<UsageView> = {}): UsageView {
  return { status: { type: 'Installed' }, providers: [provider()], ...over }
}

describe('the Usage pane state', () => {
  it('is reading before the first answer and while the read runs', () => {
    expect(paneOf(null).kind).toBe('reading')
    expect(paneOf(usage({ status: { type: 'Unknown' } })).kind).toBe('reading')
    expect(paneOf(usage({ status: { type: 'Reading' } })).kind).toBe('reading')
  })

  it('draws one card per provider the sources named', () => {
    const state = paneOf(
      usage({
        providers: [
          provider(),
          provider({ provider: 'claude-bridge', plan: 'pro' }),
        ],
      }),
    )

    expect(state.kind).toBe('installed')
    if (state.kind !== 'installed') return
    // Both cards report the same peak here, and the tie breaks by provider name.
    expect(state.providers.map((row) => row.provider)).toEqual([
      'claude-bridge',
      'commandcode',
    ])
    expect(state.providers[0].plan).toBe('pro')
  })

  it('rounds a share and gives the gauge the house tone and a clamped width', () => {
    const state = paneOf(usage())
    if (state.kind !== 'installed') throw new Error('installed expected')

    const [near, far] = state.providers[0].limits[0].windows
    expect(near.percent).toBe(5)
    expect(near.width).toBe('5%')
    expect(near.tone).toBe('normal')
    expect(near.resets).toBe('resets in 2h')

    // 88% is past the warning step of the reader's own scale.
    expect(far.percent).toBe(88)
    expect(far.tone).toBe('warning')
    expect(far.resets).toBe('resets in 6d')
  })

  it('never lets a reading past its own ends overflow the gauge', () => {
    const state = paneOf(
      usage({
        providers: [
          provider({
            limits: [
              {
                name: 'claude',
                limit_reached: true,
                windows: [{ label: 'week', used_percent: 140, resets_in: null }],
              },
            ],
          }),
        ],
      }),
    )
    if (state.kind !== 'installed') throw new Error('installed expected')

    const window = state.providers[0].limits[0].windows[0]
    expect(window.percent).toBe(140)
    expect(window.width).toBe('100%')
    expect(window.tone).toBe('danger')
    expect(state.providers[0].limits[0].reached).toBe(true)
  })

  it('leaves out a window the contract could not carry a share for, never drawing it as 0%', () => {
    const state = paneOf(
      usage({
        providers: [
          provider({
            limits: [
              {
                name: 'command code',
                limit_reached: false,
                // The generated type marks every float nullable; a window with
                // no reading is left out rather than shown as 0%.
                windows: [
                  { label: '5h', used_percent: null, resets_in: null },
                  { label: 'week', used_percent: 12, resets_in: 'resets in 6d' },
                ],
              },
            ],
          }),
        ],
      }),
    )
    if (state.kind !== 'installed') throw new Error('installed expected')

    expect(state.providers[0].limits[0].windows.map((window) => window.label)).toEqual(['week'])
  })

  // The shell's own panel never lists a provider whose source answered "no
  // data"; the maintainer saw this pane name providers he has never signed in
  // to. Only a reading is drawn.
  it('does not draw a provider that answered no data', () => {
    const state = paneOf(
      usage({
        providers: [
          provider({ provider: 'opencode-go', status: 'NoData', plan: null, limits: [] }),
          provider({ provider: 'commandcode' }),
        ],
      }),
    )
    if (state.kind !== 'installed') throw new Error('installed expected')

    expect(state.providers.map((row) => row.provider)).toEqual(['commandcode'])
  })

  // What is actually being spent comes first: a card at 88% matters more than
  // one at 0%, and a failure goes last because it is a note, not a reading.
  it('sorts by peak usage, quiet ones after, failures last', () => {
    const state = paneOf(
      usage({
        providers: [
          provider({ provider: 'quiet', limits: [{ name: 'l', limit_reached: false, windows: [{ label: 'week', used_percent: 0, resets_in: null }] }] }),
          provider({ provider: 'busy', limits: [{ name: 'l', limit_reached: false, windows: [{ label: 'week', used_percent: 91, resets_in: null }] }] }),
          provider({ provider: 'broken', status: 'Failed', error: 'boom', limits: [] }),
        ],
      }),
    )
    if (state.kind !== 'installed') throw new Error('installed expected')

    expect(state.providers.map((row) => row.provider)).toEqual(['busy', 'quiet', 'broken'])
  })

  it('keeps a failed provider and its own reason', () => {
    const state = paneOf(
      usage({
        providers: [
          provider({ status: 'Failed', plan: null, error: 'timed out after 10000ms', limits: [] }),
        ],
      }),
    )
    if (state.kind !== 'installed') throw new Error('installed expected')

    expect(state.providers[0].status).toBe('Failed')
    expect(state.providers[0].error).toBe('timed out after 10000ms')
  })

  it('names the three states that are not a card', () => {
    expect(paneOf(usage({ status: { type: 'Missing', data: 'not installed' } }))).toEqual({
      kind: 'missing',
    })
    expect(paneOf(usage({ status: { type: 'Unreadable', data: 'permission denied' } }))).toEqual({
      kind: 'unreadable',
      reason: 'permission denied',
    })
    expect(paneOf(usage({ status: { type: 'Failed', data: 'wrong schema' } }))).toEqual({
      kind: 'failed',
      reason: 'wrong schema',
    })
  })
})
