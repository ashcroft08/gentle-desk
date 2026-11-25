// The Updates pane's command lines (updates-by-home S4): the ONE mirror on the web
// side of the table Rust runs. Rust owns the command it actually starts
// (`apps/desktop/src-tauri/src/terminal.rs`, `command_for_home`); this module only
// answers what that command WILL be, so the pane can draw it under the button and
// the home choice is never a surprise.
//
// The table, byte for byte (`terminal.rs`'s own literals):
//
//   action              Separado                           Mi pi
//   update extensions   `gentle-shell update --extensions` `pi update --extensions`
//   update pi           `gentle-shell update`              `pi update`
//   update Gentle Shell `gentle-shell upgrade`             `pi install npm:gentle-pi@<latest>`
//
// "<latest>" is the version the pane's Gentle Shell row measured against npm, and
// the rule is Rust's, never a second one: only a plain stable `x.y.z`
// (`gentle_runtime::updates::parse`, trimmed - the same shape `parseVersion`
// reads) is a version to install. Anything else draws the pane's honest clause
// instead of a guessed `@latest`, and never the other home's `upgrade`.
import type { HomeChoiceView } from '../contract/bindings'
import { parseVersion } from '../update/version'
import type { RowAction } from './updates'

/**
 * The three Updates actions of the table. They are named as the contract's
 * `TerminalActionView` names them, so the pane and Rust cannot drift on the word.
 */
export type UpdateCommandAction = 'Update' | 'UpdateExtensions' | 'UpgradeShell'

/** What the pane draws under one action's button. */
export type DrawnCommand =
  /** The exact line the button runs. */
  | { kind: 'command'; line: string }
  /** Mi pi has no measured stable version, so that button installs nothing. */
  | { kind: 'noStableLatest' }

/** The shell's own home: every action is the shell's own command, unchanged. */
const SEPARADO: Record<UpdateCommandAction, string> = {
  UpdateExtensions: 'gentle-shell update --extensions',
  Update: 'gentle-shell update',
  UpgradeShell: 'gentle-shell upgrade',
}

/** The user's own pi: pi's commands, with no launcher and no home flag of the shell's. */
const MI_PI: Record<'Update' | 'UpdateExtensions', string> = {
  UpdateExtensions: 'pi update --extensions',
  Update: 'pi update',
}

/**
 * The line `action` runs on `home`, or the refusal of the one action that needs a
 * measured version.
 *
 * A run that never chose a home is the shell's own, exactly as Rust unwraps it
 * (`view.shell.choice.unwrap_or(HomeChoiceView::Isolated)`).
 */
export function commandLine(
  action: UpdateCommandAction,
  home: HomeChoiceView | null,
  shellLatest: string | null | undefined,
): DrawnCommand {
  if (home !== 'link') return { kind: 'command', line: SEPARADO[action] }
  if (action !== 'UpgradeShell') return { kind: 'command', line: MI_PI[action] }
  // The package itself, as npm's stable release names it: the MEASURED version,
  // never `@latest`, and never the other home's `upgrade` as a fallback.
  const latest = stableLatest(shellLatest)
  if (latest === null) return { kind: 'noStableLatest' }
  return { kind: 'command', line: `pi install npm:gentle-pi@${latest}` }
}

/** The text one drawn command is: its line, or the pane's own clause for a refusal. */
export function drawnLine(command: DrawnCommand, noStableLatest: string): string {
  return command.kind === 'command' ? command.line : noStableLatest
}

/** The table's action for a row's own action, so the pane names it once. */
export function commandAction(action: RowAction): UpdateCommandAction {
  return action === 'upgrade' ? 'UpgradeShell' : 'Update'
}

/**
 * The version `latest` names, when it is a plain stable `x.y.z` by the very rule
 * Rust applies, trimmed: no `v`, no channel build, no prerelease, nothing.
 */
function stableLatest(latest: string | null | undefined): string | null {
  const text = (latest ?? '').trim()
  return parseVersion(text) === null ? null : text
}
