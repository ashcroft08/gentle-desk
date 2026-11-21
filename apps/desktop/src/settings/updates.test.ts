// The Updates pane's data rules (0.1.12, S2, S3, S4, S7; 0.8.1 settings-polish S6,
// S8), one test per rule: which card the pushed check calls for, what each of the
// three rows draws, that an unknown row is never "up to date" and says which read
// failed, and that only pi and Gentle Shell offer the terminal's action - the
// shell's being its own `upgrade` (S7). The pane's drawing is
// UpdatesPane.test.ts's and the rows' is UpdateRows.test.ts's.
import { describe, expect, it } from 'vitest'
import type { UpdatesView_Serialize as UpdatesView } from '../contract/bindings'
import { cardOf } from './updates'
import { updatesCopy } from './updates-copy'

/** The card's labels in English, read through the dictionary (0.2.0, D4). */
const drawn = (view: UpdatesView | null) => cardOf(view, updatesCopy('en'))

const copy = updatesCopy('en')

function updates(over: Partial<UpdatesView> = {}): UpdatesView {
  return {
    status: 'Ready',
    desk: { current: '0.1.12', latest: '0.1.12', update_available: false },
    pi: { current: '1.5.2', latest: '2.0.5', update_available: true },
    shell: { current: '4.8.0', latest: '4.9.2', update_available: true },
    checked_ms: 1_700_000_000_000,
    ...over,
  }
}

/** The one row a test asks about, by the key the pane draws it under. */
function row(view: UpdatesView, key: 'app' | 'pi' | 'shell') {
  const found = drawn(view).rows.find((each) => each.key === key)
  if (found === undefined) throw new Error(`the card draws no ${key} row`)
  return found
}

describe('the Updates card', () => {
  it('draws nothing before the check the app runs at open has answered', () => {
    const nothing = drawn(null)
    expect(nothing.checked).toBe(false)
    expect(nothing.rows).toEqual([])
    expect(nothing.checkedMs).toBeNull()

    const unknown = drawn(updates({ status: 'Unknown', checked_ms: undefined }))
    expect(unknown.checked).toBe(false)
    expect(unknown.available).toBe(false)
    expect(unknown.checkedMs).toBeNull()
  })

  it('keeps the last time it read the machine while a check is on its way', () => {
    // S8: what is being checked now does not erase what was last known, and the
    // rows of the last answer are not redrawn from a check that has not answered.
    const card = drawn(updates({ status: 'Checking' }))

    expect(card.checking).toBe(true)
    expect(card.checked).toBe(false)
    expect(card.checkedMs).toBe(1_700_000_000_000)
    expect(card.rows).toEqual([])
  })

  it('draws the three rows, each with its own version and its own action', () => {
    const card = drawn(updates())

    expect(card.rows.map((each) => each.label)).toEqual(['This app', 'pi', 'Gentle Shell'])
    expect(card.rows.map((each) => each.version)).toEqual([
      '0.1.12',
      '1.5.2 \u2192 2.0.5',
      '4.8.0 \u2192 4.9.2',
    ])
    // The app's own release is the banner's job: no button, and its row says so
    // while it is the one that is behind.
    expect(card.rows.map((each) => each.action)).toEqual([null, 'update', 'upgrade'])
    expect(row(updates(), 'app').note).toBeNull()
    const appBehind = updates({
      desk: { current: '0.1.11', latest: '0.1.12', update_available: true },
    })
    expect(row(appBehind, 'app').note).toBe(copy.appUpdatesItself)
  })

  it('names every row that is behind, and only those', () => {
    const card = drawn(
      updates({
        pi: { current: '2.0.5', latest: '2.0.5', update_available: false },
      }),
    )

    expect(card.available).toBe(true)
    expect(card.rows.filter((each) => each.behind).map((each) => each.key)).toEqual(['shell'])
  })

  it('is up to date only when every row was read and none is behind', () => {
    const card = drawn(
      updates({
        pi: { current: '2.0.5', latest: '2.0.5', update_available: false },
        shell: { current: '4.9.2', latest: '4.9.2', update_available: false },
      }),
    )

    expect(card.checked).toBe(true)
    expect(card.available).toBe(false)
    expect(card.upToDate).toBe(true)
    expect(card.partial).toBe(false)
    expect(card.terminal).toBe(false)
  })

  it('never reads a row it could not compare as up to date', () => {
    // S8: a registry that did not answer leaves the row unknown, and the card
    // says "not checked" instead of "everything is up to date".
    const card = drawn(
      updates({
        pi: { current: '1.5.2', latest: null, update_available: null },
        shell: { current: '4.9.2', latest: '4.9.2', update_available: false },
      }),
    )

    expect(card.available).toBe(false)
    expect(card.partial).toBe(true)
    expect(card.upToDate).toBe(false)
    // The version it runs is still shown: only the comparison is unknown.
    expect(row(updates({ pi: { current: '1.5.2', latest: null, update_available: null } }), 'pi')
      .version).toBe('1.5.2')
  })

  it('says which read failed on the row it failed on, and only there', () => {
    const card = drawn(
      updates({
        pi: { current: '2.0.5', latest: '2.0.5', update_available: false },
        shell: {
          current: '4.8.0',
          latest: null,
          update_available: null,
          reason: 'Offline',
        },
      }),
    )

    expect(card.rows.map((each) => each.note)).toEqual([
      null,
      null,
      copy.rowReason('shell', 'Offline'),
    ])
    expect(card.rows[2].note).toBe(
      'Could not check the latest gentle-shell version on npm: offline.',
    )
  })

  it('draws the unknown word and its sentence for a version nobody could read', () => {
    const view = updates({
      pi: { current: null, latest: '2.0.5', update_available: null, reason: 'InstalledUnreadable' },
    })
    const pi = row(view, 'pi')

    expect(pi.version).toBe(copy.unknownVersion)
    expect(pi.note).toBe(copy.rowReason('pi', 'InstalledUnreadable'))
    expect(pi.behind).toBe(false)
    expect(drawn(view).partial).toBe(true)
  })

  it('offers a terminal action only on a row that is behind', () => {
    const card = drawn(
      updates({
        pi: { current: '2.0.5', latest: '2.0.5', update_available: false },
      }),
    )

    expect(card.terminal).toBe(true)
    expect(card.rows.map((each) => each.action)).toEqual([null, null, 'upgrade'])
  })

  it('says nothing about a terminal when only this app is behind', () => {
    const card = drawn(
      updates({
        desk: { current: '0.1.11', latest: '0.1.12', update_available: true },
        pi: { current: '2.0.5', latest: '2.0.5', update_available: false },
        shell: { current: '4.9.2', latest: '4.9.2', update_available: false },
      }),
    )

    expect(card.available).toBe(true)
    expect(card.terminal).toBe(false)
    expect(card.rows.map((each) => each.action)).toEqual([null, null, null])
    expect(row(updates(), 'app').skip).toBeNull()
  })

  it('carries when the rows were last read (S8, D11)', () => {
    expect(drawn(updates()).checkedMs).toBe(1_700_000_000_000)
    expect(drawn(updates({ checked_ms: undefined })).checkedMs).toBeNull()
  })

  describe('the one skip (D9)', () => {
    it('lives on the row whose published version is the greatest', () => {
      // pi 2.0.5 and Gentle Shell 4.9.2 are both behind: 4.9.2 is the newest, and
      // one stored version can only name one of them.
      const card = drawn(updates())

      expect(card.rows.map((each) => each.skip)).toEqual([null, null, 'skip'])
    })

    it('moves to the only row that is behind', () => {
      const card = drawn(
        updates({ pi: { current: '2.0.5', latest: '2.0.5', update_available: false } }),
      )

      expect(card.rows.map((each) => each.skip)).toEqual([null, null, 'skip'])
    })

    it('offers nothing left to skip once that version is the skipped one', () => {
      // The check the user asks for reports the real state, so the row is still
      // behind and the pane keeps saying so - with the way to stop skipping.
      const card = drawn(
        updates({
          pi: { current: '2.0.5', latest: '2.0.5', update_available: false },
          skipped: '4.9.2',
        }),
      )
      const shell = card.rows[2]

      expect(card.available).toBe(true)
      expect(shell.skip).toBe('unskip')
      expect(shell.note).toBe(copy.skippedNote('4.9.2'))
      expect(card.rows.slice(0, 2).map((each) => each.skip)).toEqual([null, null])
    })

    it('keeps a second row offering the skip while the newest one is skipped', () => {
      // 4.9.2 is skipped and pi is behind with 2.0.5: the one version the
      // contract stores cannot be two, so the way back sits on the skipped row
      // and the offer moves to the row that is still waiting.
      const card = drawn(updates({ skipped: '4.9.2' }))

      expect(card.rows.map((each) => each.skip)).toEqual([null, 'skip', 'unskip'])
    })

    it('offers a version newer than the skipped one as usual', () => {
      const card = drawn(
        updates({
          shell: { current: '4.9.2', latest: '4.9.2', update_available: false },
          skipped: '2.0.4',
        }),
      )

      expect(card.rows[1].skip).toBe('skip')
      expect(card.rows[1].note).toBeNull()
    })

    it('reads the skip as a version, and a skip it cannot read hides nothing', () => {
      // The same version spelled with leading zeros is still the skipped one.
      expect(drawn(updates({ skipped: '04.09.2' })).rows[2].skip).toBe('unskip')

      const unreadable = drawn(updates({ skipped: 'latest' }))
      expect(unreadable.rows[2].skip).toBe('skip')
      expect(unreadable.rows[2].note).toBeNull()
    })
  })
})
