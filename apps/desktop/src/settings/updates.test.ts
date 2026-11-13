// The Updates pane's data rules (0.1.12, S2, S3, S4, S7), one test per rule:
// which card the pushed check calls for, which rows are behind, that an unknown
// row is never "up to date", and that only pi and the shell offer the terminal's
// Update. The pane's drawing is UpdatesPane.test.ts's; this file held the old
// About-and-updates check, which the Updates section took over.
import { describe, expect, it } from 'vitest'
import type { UpdatesView_Serialize as UpdatesView } from '../contract/bindings'
import { cardOf } from './updates'

function updates(over: Partial<UpdatesView> = {}): UpdatesView {
  return {
    status: 'Ready',
    desk: { current: '0.1.12', latest: '0.1.12', update_available: false },
    pi: { current: '1.5.2', latest: '2.0.5', update_available: true },
    shell: { current: '4.7.0', latest: null, update_available: null },
    ...over,
  }
}

describe('the Updates card', () => {
  it('draws nothing before the check the app runs at open has answered', () => {
    const nothing = cardOf(null)
    expect(nothing.checked).toBe(false)
    expect(nothing.versions).toEqual([])

    const unknown = cardOf(updates({ status: 'Unknown' }))
    expect(unknown.checked).toBe(false)
    expect(unknown.available).toBe(false)
  })

  it('keeps the versions it had while a check is on its way', () => {
    const card = cardOf(updates({ status: 'Checking' }))

    expect(card.checking).toBe(true)
    expect(card.checked).toBe(false)
    expect(card.versions.map((line) => line.version)).toEqual(['0.1.12', '1.5.2', '4.7.0'])
  })

  it('names every row that is behind, and only those', () => {
    const card = cardOf(
      updates({
        shell: { current: '4.7.0', latest: '4.8.0', update_available: true },
      }),
    )

    expect(card.available).toBe(true)
    expect(card.headlines).toEqual([
      { label: 'pi', current: '1.5.2', latest: '2.0.5' },
      { label: 'shell', current: '4.7.0', latest: '4.8.0' },
    ])
  })

  it('is up to date only when no row claims an update', () => {
    const card = cardOf(
      updates({ pi: { current: '2.0.5', latest: '2.0.5', update_available: false } }),
    )

    expect(card.checked).toBe(true)
    expect(card.available).toBe(false)
    expect(card.headlines).toEqual([])
    expect(card.terminal).toBe(false)
  })

  it('never reads a row it could not compare as up to date', () => {
    const card = cardOf(
      updates({ pi: { current: '1.5.2', latest: null, update_available: null } }),
    )

    expect(card.available).toBe(false)
    // The version it runs is still shown: only the comparison is unknown.
    expect(card.versions[1]).toEqual({ label: 'pi', version: '1.5.2' })
  })

  it('hands the terminal only what a terminal window updates', () => {
    const both = cardOf(updates())
    expect(both.terminal).toBe(true)
    expect(both.appOnly).toBe(false)

    const app = cardOf(
      updates({
        desk: { current: '0.1.11', latest: '0.1.12', update_available: true },
        pi: { current: '2.0.5', latest: '2.0.5', update_available: false },
      }),
    )
    expect(app.available).toBe(true)
    expect(app.terminal).toBe(false)
    expect(app.appOnly).toBe(true)
    expect(app.headlines).toEqual([{ label: 'This app', current: '0.1.11', latest: '0.1.12' }])
  })
})
