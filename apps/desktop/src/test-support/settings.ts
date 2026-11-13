// The Settings dialog's test harness: the dialog opened the way the user opens
// it (the footer's gear, through the real Sidebar), and the three readers the
// dialog tests share - the rail, the one pane, and a pick of a section. Kept in
// one place so the dialog's two test files stay about their own rules.
import { fireEvent, render, screen, within } from '@testing-library/svelte'
import type {
  ShellView_Serialize as ShellView,
  ViewState_Serialize as ViewState,
} from '../contract/bindings'
import Sidebar from '../sidebar/Sidebar.svelte'
import { PANEL_ID } from '../settings/sections'
import { serveSidebar } from './sidebar'
import { handlerArgs, viewState } from './tauri-mock'

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
 * The dialog, opened from the footer gear, over the pushed state a test gives:
 * the shell it reports, the version this run runs as, and any other slice the
 * dialog draws (0.1.12's profiles, auth and updates).
 */
export async function openSettings(
  over: Partial<ShellView> = {},
  appVersion: string | null = '0.1.9',
  view: Partial<ViewState> = {},
): Promise<HTMLElement> {
  serveSidebar({
    view: viewState({ shell: shell(over), total_sessions: 1, app_version: appVersion, ...view }),
  })
  render(Sidebar)
  await fireEvent.click(await screen.findByRole('button', { name: SETTINGS_LABEL }))
  return await screen.findByRole('dialog')
}

/** The rail's entries, in the order the dialog draws them. */
export function rail(dialog: HTMLElement): HTMLElement[] {
  return within(dialog).getAllByRole('tab')
}

/**
 * The dialog's own pane: only the selected section is there. Read by id, since a
 * section may hold sub-tabs of its own whose panel is drawn inside this one
 * (0.1.12, L17: the Profiles section's Profile | Agents pair).
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
