// The bridge answers a mounting sidebar needs, filled from one fixture, so a
// component test runs with no Tauri at all. The handlers are the real command
// names the generated bindings call; only their bodies are stand-ins.

import { screen } from '@testing-library/svelte'
import type {
  SessionTreeRow_Serialize,
  UiIntent,
  ViewState_Serialize,
} from '../contract/bindings'
import { handlerArgs, pushEvent, sessionsWindow, setHandler, viewState } from './tauri-mock'

export type SidebarFixture = {
  /** The state every command answers with; change it and push it to move the view. */
  view: ViewState_Serialize
  /** The rows every window page answers with. */
  rows: SessionTreeRow_Serialize[]
  /** How many rows the tree says it has in total. */
  total: number
  /** The intents the components sent, oldest first. */
  intents: () => UiIntent[]
  /** How many window pages the view asked Rust for. */
  pages: () => number
  /** One pushed state, as the `state-changed` event carries it. */
  push: () => void
}

export function serveSidebar(
  options: {
    view?: ViewState_Serialize
    rows?: SessionTreeRow_Serialize[]
    total?: number
  } = {},
): SidebarFixture {
  const fixture: SidebarFixture = {
    view: options.view ?? viewState(),
    rows: options.rows ?? [],
    total: options.total ?? (options.rows ?? []).length,
    intents: () => handlerArgs('dispatch_intent').map((args) => args.intent as UiIntent),
    pages: () => handlerArgs('sessions_window').length,
    push: () => pushEvent('state-changed', fixture.view),
  }
  setHandler('get_view_state', () => fixture.view)
  setHandler('sessions_window', () => sessionsWindow(fixture.total, fixture.rows))
  setHandler('dispatch_intent', () => fixture.view)
  setHandler('add_workspace', () => fixture.view)
  return fixture
}

/** Every row the tree drew, in document order. */
export function treeItems(): HTMLElement[] {
  return screen.getAllByRole('treeitem')
}

/** The row element that holds `text`: the tree item a click lands on. */
export function rowOf(text: string): HTMLElement {
  const item = screen.getByText(text).closest('[role="treeitem"]')
  if (!(item instanceof HTMLElement)) throw new Error(`no tree row holds ${text}`)
  return item
}
