// The gear's dot (0.1.14): whether the sidebar footer promises an update. The
// rule is plain data, so it is tested without a window: any row the app can only
// update from a terminal is behind. The app's own row is excluded on purpose -
// its more urgent banner already sits at the top of the window.

import { describe, expect, it } from 'vitest'
import type { UpdatesView_Serialize as UpdatesView, VersionRowView_Serialize as VersionRow } from '../contract/bindings'
import { updateWaiting } from './dot'

/** One row of the check, behind (`true`), level (`false`) or unknown (`null`). */
function row(update_available: boolean | null): VersionRow {
  return {
    current: update_available === true ? '1.5.2' : '2.0.5',
    latest: '2.0.5',
    update_available,
  }
}

/** The pushed check with the three rows a test names. */
function updates(
  rows: { desk?: VersionRow; pi?: VersionRow; shell?: VersionRow },
  status: UpdatesView['status'] = 'Ready',
): UpdatesView {
  return {
    status,
    desk: rows.desk ?? row(false),
    pi: rows.pi ?? row(false),
    shell: rows.shell ?? row(false),
  }
}

describe('The gear dot', () => {
  it('lights up when pi is behind', () => {
    expect(updateWaiting(updates({ pi: row(true) }))).toBe(true)
  })

  it('lights up when the shell is behind', () => {
    expect(updateWaiting(updates({ shell: row(true) }))).toBe(true)
  })

  it("stays out of the way when only this app is behind", () => {
    // The app's own release has the update banner at the top of the window, and
    // it installs itself: a second promise on the gear would say nothing new.
    expect(updateWaiting(updates({ desk: row(true) }))).toBe(false)
  })

  it('stays dark when nothing is behind', () => {
    expect(updateWaiting(updates({}))).toBe(false)
  })

  it('stays dark for a row the check could not read', () => {
    // `null` is unknown, never "up to date": it is not a promise either.
    expect(updateWaiting(updates({ pi: row(null), shell: row(null) }))).toBe(false)
  })

  it('stays dark before the first answer, and keeps the promise while a check runs again', () => {
    // A check on its way keeps the rows it already read, so the dot does not
    // blink off while "Check again" runs.
    expect(updateWaiting(null)).toBe(false)
    expect(updateWaiting(updates({ pi: row(true) }, 'Checking'))).toBe(true)
  })
})
