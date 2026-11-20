// The Settings dialog's test harness: the dialog opened the way the user opens
// it (the footer's gear, through the real Sidebar), and the three readers the
// dialog tests share - the rail, the one pane, and a pick of a section. Kept in
// one place so the dialog's own test files stay about their own rules.
import { fireEvent, render, screen, within } from '@testing-library/svelte'
import type {
  SettingsSnapshot_Serialize as SettingsSnapshot,
  ShellView_Serialize as ShellView,
  ViewState_Serialize as ViewState,
} from '../contract/bindings'
import Sidebar from '../sidebar/Sidebar.svelte'
import { PANEL_ID } from '../settings/sections'
import { serveSidebar } from './sidebar'
import { handlerArgs, setHandler, viewState } from './tauri-mock'

/** The folder the user's pi sits in, as every dialog test spells it. */
export const PI = 'C:/Users/someone/.pi/agent'

/** The gear's name and the dialog's title. */
export const SETTINGS_LABEL = 'Settings'

/** A shell view with what a dialog test needs, overridable field by field. */
export function shell(over: Partial<ShellView> = {}): ShellView {
  return {
    pi_path: PI,
    choice: 'isolated',
    ask: false,
    launcher_found: true,
    install: { type: 'Idle' },
    restart_needed: false,
    ...over,
  }
}

/**
 * The Settings dialog, opened from the footer gear, over the pushed state a test
 * gives: the shell it reports, the version this run runs as, and any other slice
 * the dialog draws (0.1.12's auth, updates and usage).
 *
 * `snapshot` is what the dialog's ONE read answers (settings-snapshot): the
 * plain object gathered from the same view unless the test says otherwise, and a
 * promise when the test wants to hold the answer and watch the dialog meanwhile.
 */
export async function openSettings(
  over: Partial<ShellView> = {},
  appVersion: string | null = '0.1.9',
  view: Partial<ViewState> = {},
  snapshot?: SettingsSnapshot | Promise<SettingsSnapshot>,
): Promise<HTMLElement> {
  const pushed = viewState({ shell: shell(over), total_sessions: 1, app_version: appVersion, ...view })
  serveSidebar({ view: pushed })
  setHandler('settings_snapshot', () => snapshot ?? snapshotOf(pushed))
  render(Sidebar)
  await fireEvent.click(await screen.findByRole('button', { name: SETTINGS_LABEL }))
  return await screen.findByRole('dialog')
}

/**
 * The snapshot `gather` would produce from one pushed slice: the same fields,
 * read the same way, so a test that only wants the dialog to keep drawing what
 * the push carried gets exactly that.
 */
export function snapshotOf(view: ViewState): SettingsSnapshot {
  return {
    theme: view.theme,
    language: view.language,
    notifications: view.notifications,
    shell: view.shell,
    app_version: view.app_version,
    open_chats: view.open_chats,
    config_notice: view.config_notice,
    auth: view.auth,
    updates: view.updates,
    usage: view.usage,
  }
}

/** The rail's entries, in the order the dialog draws them. */
export function rail(dialog: HTMLElement): HTMLElement[] {
  return within(dialog).getAllByRole('tab')
}

/**
 * The dialog's own pane: only the selected section is there. Read by id, so a
 * section that draws a panel of its own inside this one cannot be mistaken for
 * it.
 */
export function pane(dialog: HTMLElement): HTMLElement {
  const found = within(dialog)
    .getAllByRole('tabpanel')
    .find((panel) => panel.id === PANEL_ID)
  if (found === undefined) throw new Error('the dialog drew no pane')
  return found
}

/** Picks a section in the rail, the way the user does. */
export async function pick(dialog: HTMLElement, name: string): Promise<void> {
  await fireEvent.click(within(dialog).getByRole('tab', { name }))
}

/** The intents the bridge received, oldest first. */
export function intents(): unknown[] {
  return handlerArgs('dispatch_intent').map((args) => args.intent)
}
