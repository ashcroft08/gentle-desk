// The gear's dot (0.1.14): whether the sidebar footer promises an update. The
// rule is plain data, so it is tested without a window: any row the app can only
// update from a terminal is behind. The app's own row is excluded on purpose -
// its more urgent banner already sits at the top of the window.

import { describe, expect, it } from 'vitest'
import type {
  PackagesView_Serialize as PackagesView,
  UpdatesView_Serialize as UpdatesView,
  VersionRowView_Serialize as VersionRow,
} from '../contract/bindings'
import { updateWaiting } from './dot'

/** The Extensions block's slice: unasked unless a test says otherwise. */
function packages(over: Partial<PackagesView> = {}): PackagesView {
  return { available: false, behind: [], unknown: false, truncated: false, unchecked: false, checked: null, ...over }
}

/** One row of the check, behind (`true`), level (`false`) or unknown (`null`). */
function row(update_available: boolean | null): VersionRow {
  return {
    current: update_available === true ? '1.5.2' : '2.0.5',
    latest: '2.0.5',
    update_available,
  }
}

/** The pushed check with the three rows a test names, and the skip beside them. */
function updates(
  rows: { desk?: VersionRow; pi?: VersionRow; shell?: VersionRow },
  status: UpdatesView['status'] = 'Ready',
  skipped: string | undefined = undefined,
  packagesView: PackagesView = packages(),
): UpdatesView {
  return {
    status,
    skipped,
    desk: rows.desk ?? row(false),
    pi: rows.pi ?? row(false),
    shell: rows.shell ?? row(false),
    packages: packagesView,
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

  it('stays dark for the version the user skipped', () => {
    // S2: a skipped version is not offered again by the check the app runs when
    // it opens, and the gear must not promise it either.
    expect(updateWaiting(updates({ pi: row(true) }, 'Ready', '2.0.5'))).toBe(false)
    expect(updateWaiting(updates({ shell: row(true) }, 'Ready', '2.0.5'))).toBe(false)
  })

  it('lights up again for a version newer than the skipped one', () => {
    expect(updateWaiting(updates({ pi: row(true) }, 'Ready', '2.0.4'))).toBe(true)
  })

  it('reads the skip as a version, and a skip it cannot read hides nothing', () => {
    // The same version spelled with a leading zero is still the skipped one.
    expect(updateWaiting(updates({ pi: row(true) }, 'Ready', '02.0.5'))).toBe(false)
    // A stored value that is not a version is no skip at all.
    expect(updateWaiting(updates({ pi: row(true) }, 'Ready', 'latest'))).toBe(true)
    // Nothing skipped at all: the field is absent from the push.
    expect(updateWaiting(updates({ pi: row(true) }))).toBe(true)
  })

  it('lights up when pi reports an extension as behind (packages-status T3)', () => {
    // A behind package is a promise exactly like a behind pi or shell row: the
    // terminal is where it is updated from.
    expect(
      updateWaiting(
        updates({}, 'Ready', undefined, packages({ available: true, behind: [{ name: 'pi-hud' }] })),
      ),
    ).toBe(true)
  })

  it('stays dark for a packages read that never ran or could not run', () => {
    // Unasked (no chat open, or a shell that does not advertise it) and unknown
    // (a read that could not answer) are never a promise.
    expect(updateWaiting(updates({}, 'Ready', undefined, packages()))).toBe(false)
    expect(
      updateWaiting(updates({}, 'Ready', undefined, packages({ unknown: true }))),
    ).toBe(false)
    // An answered listing with nothing behind promises nothing either.
    expect(updateWaiting(updates({}, 'Ready', undefined, packages({ available: true })))).toBe(
      false,
    )
  })

  it('stays dark for a listing pi has not checked yet (packages-status T6, S6)', () => {
    // The listing exists and every checkable item is still `unknown`: pi's own
    // check never ran in that child, so the app knows of nothing waiting - and the
    // dot is a promise, not a maybe.
    expect(
      updateWaiting(
        updates({}, 'Ready', undefined, packages({ available: true, unchecked: true })),
      ),
    ).toBe(false)
    // A listing with one package behind lights it whatever the rest is: that much
    // the check did learn.
    expect(
      updateWaiting(
        updates(
          {},
          'Ready',
          undefined,
          packages({ available: true, unchecked: true, behind: [{ name: 'pi-hud' }] }),
        ),
      ),
    ).toBe(true)
  })

  it('never lets the skip darken a package: a package name is not a version', () => {
    // The skip is a version rule for the rows (S2). A package the user never
    // skipped still lights the dot, whatever version number sits beside it.
    const behind = packages({
      available: true,
      behind: [{ name: 'pi-hud', current: '2.0.5' }],
    })
    expect(updateWaiting(updates({}, 'Ready', '2.0.5', behind))).toBe(true)
  })
})
