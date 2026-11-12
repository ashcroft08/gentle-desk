// The chat row's actions menu, as the user reads it (design fix, T3): the button
// and the timestamp never print over each other, the menu opens inside the
// sidebar instead of being cut by its edge, and every label lands on one whole
// line of a real card.
//
// The geometry itself is the browser's, so the two things a test CAN pin down are
// pinned: where the floating layer puts the menu (outside the scrolling tree, so
// no scroll box can clip it, right-aligned below its button), and the stylesheet
// rules the card, the hit areas and the timestamp's step-aside are drawn with.
// The stylesheet comes in as `?raw` text, which Vitest turns into an EMPTY module
// unless `test.css.include` lists it (vitest.config.ts does): the first stylesheet
// test below fails on an empty one, so an unlisted file cannot pass silently.
import '../test-support/dom-env'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { cleanup, fireEvent, render, screen, within } from '@testing-library/svelte'
import APP_CSS from '../app.css?raw'
import { resetTauriMock, viewState, workspaceRow, chatRow } from '../test-support/tauri-mock'
import { serveSidebar } from '../test-support/sidebar'
import Sidebar from './Sidebar.svelte'

vi.mock('@tauri-apps/api/core', () => import('../test-support/tauri-mock').then((m) => m.coreModule()))
vi.mock('@tauri-apps/api/event', () => import('../test-support/tauri-mock').then((m) => m.eventModule()))

const TITLE = 'First chat'
const SESSION = 'C:\\pi\\sessions\\--dev-gentle-desk--\\2026_s1.jsonl'

/** One rule's text, by the first declaration block that follows `selector` at the
    start of a line (so a mention inside a comment never matches). */
function rule(selector: string): string {
  const at = APP_CSS.indexOf(`\n${selector}`)
  if (at < 0) throw new Error(`no rule mentions ${selector}`)
  const end = APP_CSS.indexOf('}', at)
  return APP_CSS.slice(at, end)
}

/** A `padding-right: 44px`-shaped declaration as its number of pixels. */
function pixels(body: string, property: string): number {
  const found = new RegExp(`${property}:\\s*(\\d+)px`).exec(body)
  if (found === null) throw new Error(`no ${property} in ${body}`)
  return Number(found[1])
}

afterEach(cleanup)

beforeEach(() => {
  resetTauriMock()
})

/** One workspace with one chat, its actions menu opened the way the user opens it. */
async function openRowMenu(): Promise<HTMLElement> {
  serveSidebar({
    view: viewState({ total_sessions: 1 }),
    rows: [workspaceRow('gentle-desk'), chatRow(SESSION, { title: TITLE })],
    total: 2,
  })
  render(Sidebar)
  const trigger = await screen.findByRole('button', { name: `Chat actions for ${TITLE}` })
  await fireEvent.click(trigger)
  return await screen.findByRole('menu', { name: `Chat actions for ${TITLE}` })
}

describe("The chat row's actions menu", () => {
  it('renders in the floating layer, outside the scrolling tree', async () => {
    const menu = await openRowMenu()

    // A scroll box that held the menu would cut it at its own edge.
    expect(menu.closest('.tree')).toBeNull()
    expect(menu.closest('nav')).toBeNull()
  })

  it('opens below its button and right-aligned to it, so it stays inside the sidebar', async () => {
    const menu = await openRowMenu()

    expect(menu.getAttribute('data-side')).toBe('bottom')
    expect(menu.getAttribute('data-align')).toBe('end')
    expect(menu.classList.contains('chat-menu')).toBe(true)
  })

  it('draws every label whole, one menu item per action', async () => {
    const menu = await openRowMenu()

    const items = within(menu).getAllByRole('menuitem')
    expect(items.map((item) => item.textContent?.trim())).toEqual(['Rename', 'Delete'])
  })
})

describe('The stylesheet rules that keep the row and its menu readable', () => {
  it('reads the real stylesheet, never an empty module', () => {
    expect(APP_CSS.length).toBeGreaterThan(500)
  })

  it('reserves the actions slot, and fades the timestamp out of it', () => {
    // The button is 32px wide and sits 6px from the row's edge: the row's padding
    // must leave at least its whole footprint, so the two cannot print together.
    expect(pixels(rule('.frow .row.chat'), 'padding-right')).toBeGreaterThanOrEqual(38)
    expect(rule('.row .ago')).toContain('transition: opacity')
    expect(rule('.frow.chat-row:hover .ago')).toContain('opacity: 0')
    expect(rule('.frow.chat-row:focus-within .ago')).toContain('opacity: 0')
    expect(rule('.frow.chat-row.menu-open .ago')).toContain('opacity: 0')
    // The icon keeps its own slot while the menu it opened is still up.
    expect(rule('.frow.chat-row.menu-open .nc')).toContain('opacity: 1')
  })

  it('draws the menu as a card with a hit area per item', () => {
    const card = rule('.menu')
    expect(card).toContain('background: var(--surface)')
    expect(card).toContain('border-radius: 10px')
    expect(card).toContain('box-shadow: var(--shadow)')
    expect(pixels(card, 'padding')).toBeGreaterThanOrEqual(6)

    const item = rule(".menu [role='menuitem']")
    expect(item).toContain('width: 100%')
    expect(item).toContain('white-space: nowrap')
    expect(pixels(item, 'padding')).toBeGreaterThanOrEqual(6)
  })

  it('keeps a chat row menu inside the sidebar it opens in', () => {
    // The sidebar is 280px wide (`grid-template-columns: 280px 1fr`), and the card
    // leaves a margin on both sides of it, so no label can be cut by its edge.
    expect(rule('.app')).toContain('grid-template-columns: 280px 1fr')
    expect(rule('.chat-menu')).toContain('max-width: calc(280px - 24px)')
  })
})
