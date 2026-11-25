// The Updates pane's command lines (updates-by-home S4): the ONE mirror on the web
// side of the table Rust runs (`apps/desktop/src-tauri/src/terminal.rs`,
// `command_for_home`). The pane draws these lines so the home choice is never a
// surprise; Rust still builds the command it runs, and these literals are the same
// ones `terminal.rs`'s tests pin, so a drift on either side is a failing test.
//
// The rule for `<latest>` is Rust's, not a second one: only a plain stable
// `x.y.z` (`gentle_runtime::updates::parse`, trimmed) is a version to install.
// Anything else draws the honest clause the pane words, never a guess.
import { describe, expect, it } from 'vitest'
import type { HomeChoiceView } from '../contract/bindings'
import { commandAction, commandLine, drawnLine } from './update-commands'

/** The line a command draws, or the word a refusal draws. */
function drawn(
  action: Parameters<typeof commandLine>[0],
  home: HomeChoiceView | null,
  latest: string | null = '5.0.0',
): string {
  const answer = commandLine(action, home, latest)
  return answer.kind === 'command' ? answer.line : '<refused>'
}

describe('the update commands', () => {
  it('names the six lines of the Rust table, byte for byte', () => {
    // Separado is the shell's own home, so every line is the shell's own command;
    // Mi pi is the user's own pi, named directly, and its "Update Gentle Shell"
    // installs the version the pane measured. These are `terminal.rs`'s literals.
    const table = [
      ['UpdateExtensions', 'gentle-shell update --extensions', 'pi update --extensions'],
      ['Update', 'gentle-shell update', 'pi update'],
      ['UpgradeShell', 'gentle-shell upgrade', 'pi install npm:gentle-pi@5.0.0'],
    ] as const

    for (const [action, separado, miPi] of table) {
      expect(drawn(action, 'isolated'), `${action} on Separado`).toBe(separado)
      expect(drawn(action, 'link'), `${action} on Mi pi`).toBe(miPi)
    }
  })

  it('reads no home as Separado, the way Rust unwraps it', () => {
    // The Rust command answers `choice.unwrap_or(HomeChoiceView::Isolated)`, so a
    // run that never chose says the Separado line and nothing else.
    for (const action of ['Update', 'UpdateExtensions', 'UpgradeShell'] as const) {
      expect(drawn(action, null, null)).toBe(drawn(action, 'isolated', null))
      expect(drawn(action, null, null)).not.toContain('pi ')
    }
  })

  it('refuses the upgrade when Mi pi has no measured stable version', () => {
    // The same values Rust refuses: nothing measured, an empty or blank read, a
    // `v` prefix, a channel build, a prerelease, a two- or four-part version, a
    // dist-tag. `@latest` is never installed and `gentle-shell upgrade` is never
    // the other home's fallback.
    for (const latest of [
      null,
      undefined,
      '',
      '   ',
      'v5.0.0',
      '4.9.0-main.abc123',
      '5.0.0-beta.1',
      '5.0',
      '5.0.0.0',
      'latest',
    ]) {
      expect(commandLine('UpgradeShell', 'link', latest), String(latest)).toEqual({
        kind: 'noStableLatest',
      })
    }
    // And nothing guesses one: no line of the pane ever names `@latest`.
    for (const action of ['Update', 'UpdateExtensions'] as const) {
      expect(drawn(action, 'link', null)).not.toContain('@')
    }
  })

  it('installs the measured version as it was measured, trimmed', () => {
    expect(drawn('UpgradeShell', 'link', ' 5.0.0 ')).toBe(
      'pi install npm:gentle-pi@5.0.0',
    )
    // The Separado home never needs a latest: its line is the shell's own upgrade.
    expect(drawn('UpgradeShell', 'isolated', null)).toBe('gentle-shell upgrade')
  })

  it("maps a row's own action onto the table", () => {
    expect(commandAction('update')).toBe('Update')
    expect(commandAction('upgrade')).toBe('UpgradeShell')
  })

  it('draws the line of a command, and the pane\'s clause for a refusal', () => {
    // The wording of a refusal belongs to the pane's dictionary (both languages);
    // this module only says which of the two it is.
    expect(drawnLine({ kind: 'command', line: 'pi update' }, 'clause')).toBe('pi update')
    expect(drawnLine({ kind: 'noStableLatest' }, 'clause')).toBe('clause')
  })
})
