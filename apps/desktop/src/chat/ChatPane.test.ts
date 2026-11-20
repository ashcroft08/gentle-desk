// View B of the welcome screen (welcome-screen T3, S1, S3, R1): a chat with no
// rows yet holds ONE tree - the hero, the thread and the composer dock - and the
// class on the pane is the whole difference. The composer is the same element
// before and after the first row, so the draft, the focus and the chips survive
// the move; the move itself is the FLIP `dock-flip`'s numbers describe.
//
// What a test can prove here: which chat draws the hero (a fresh id with no rows,
// never a stored chat, never one that already has a row), that the composer is the
// same node across the change, and the FLIP's arithmetic against a mocked rect.
// What it CANNOT prove: that the column lands above the middle of a real window or
// that 240 ms ease-out feels like one move - jsdom lays nothing out. The geometry
// is therefore read as TEXT out of the stylesheet, the way Home.layout.test.ts and
// Composer.dock.test.ts read theirs, and the feel is reported as unverified.
import '../test-support/dom-env'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { cleanup, fireEvent, render, screen, waitFor } from '@testing-library/svelte'
import type { ViewState_Serialize } from '../contract/bindings'
import APP_CSS from '../app.css?raw'
import { setLanguage } from '../i18n/language.svelte'
import { mediaWindow } from '../test-support/media'
import { resetTauriMock, setHandler, viewState } from '../test-support/tauri-mock'
import { threadOf } from '../thread/view'
import ChatPane from './ChatPane.svelte'
import { chatCopy } from './copy'
import { DOCK_MOVE_MS } from './new-session'
import { chatOf } from './view'

vi.mock('@tauri-apps/api/core', () => import('../test-support/tauri-mock').then((m) => m.coreModule()))
vi.mock('@tauri-apps/api/event', () => import('../test-support/tauri-mock').then((m) => m.eventModule()))
vi.mock('@tauri-apps/api/webview', () => import('../test-support/webview').then((m) => m.webviewModule()))

const FOLDER = { name: 'gentle-desk', path: 'C:\\Users\\josep\\dev\\gentle-desk' }
const STORED = 'session:C:\\sessions\\alpha.jsonl'
const MESSAGE = {
  id: 'msg-1',
  role: 'User' as const,
  text: 'hola',
  streaming: false,
  error: null,
  images: [],
}
const ITEMS = [{ kind: 'Message' as const, data: MESSAGE }]

/** The two slices the pane draws, from one pushed state, as App hands them over. */
function slices(over: Partial<ViewState_Serialize> = {}) {
  const state = viewState(over)
  return { chat: chatOf(state), thread: threadOf(state) }
}

/** The dock's distance from the top of the window, as jsdom cannot compute it. */
function rect(top: number): DOMRect {
  return new DOMRect(0, top, 480, 120)
}

/** One animation frame, so the move has taken its first step. */
async function frame(): Promise<void> {
  await new Promise<void>((resolve) => requestAnimationFrame(() => resolve()))
}

/** The body of the first rule whose selector is exactly `selector`. */
function rule(selector: string): string {
  const escaped = selector.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')
  const match = APP_CSS.match(new RegExp(`(?:^|[}\\n])\\s*${escaped}\\s*\\{([^}]*)\\}`))
  return match?.[1] ?? ''
}

/** The rules inside `@media (query) { ... }`, with their braces balanced. */
function media(query: string): string {
  const at = APP_CSS.indexOf(`@media (${query})`)
  if (at < 0) return ''
  const open = APP_CSS.indexOf('{', at)
  let depth = 0
  for (let index = open; index < APP_CSS.length; index += 1) {
    if (APP_CSS[index] === '{') depth += 1
    if (APP_CSS[index] === '}') {
      depth -= 1
      if (depth === 0) return APP_CSS.slice(open + 1, index)
    }
  }
  return ''
}

/** One rule's body INSIDE a media query, read the way `rule` reads a loose one. */
function scoped(query: string, selector: string): string {
  const escaped = selector.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')
  const match = media(query).match(new RegExp(`(?:^|[}\\n])\\s*${escaped}\\s*\\{([^}]*)\\}`))
  return match?.[1] ?? ''
}

beforeEach(() => {
  resetTauriMock()
  setHandler('dispatch_intent', () => viewState())
})

afterEach(() => {
  cleanup()
  vi.unstubAllGlobals()
  setLanguage('en')
})

describe('the hero of a new session', () => {
  it('draws the rose, the title and the folder a fresh chat has no rows in', () => {
    const { container } = render(ChatPane, {
      props: slices({ foreground_chat: 'new-1', foreground_folder: FOLDER }),
    })

    const copy = chatCopy('en')
    expect(screen.getByRole('heading', { level: 2, name: copy.newSessionTitle })).toBeTruthy()
    expect(screen.getByRole('img', { name: copy.roseLabel })).toBeTruthy()
    expect(container.querySelector('.chat-pane')?.className).toBe('chat-pane is-fresh')
  })

  it('draws no hero for a stored chat that has no rows yet', () => {
    // A stored chat opens on its file: until those rows land it has none, and it
    // is NOT a new session - the history must never flash a welcome first.
    const { container } = render(ChatPane, {
      props: slices({ foreground_chat: STORED }),
    })

    expect(screen.queryByRole('heading', { level: 2 })).toBeNull()
    expect(screen.queryByRole('img')).toBeNull()
    expect(container.querySelector('.chat-pane')?.className).toBe('chat-pane')
    // The thread is drawn instead, and the composer is in its usual place.
    expect(screen.getByRole('region', { name: 'Chat thread' })).toBeTruthy()
    expect(screen.getByRole('textbox', { name: 'Message' })).toBeTruthy()
  })

  it('draws no hero for a fresh chat that already has a row', () => {
    const { container } = render(ChatPane, {
      props: slices({ foreground_chat: 'new-1', messages: [MESSAGE], items: ITEMS }),
    })

    expect(screen.queryByRole('heading', { level: 2 })).toBeNull()
    expect(container.querySelector('.chat-pane')?.className).toBe('chat-pane')
  })

  it('draws no folder line when the chat has no folder to name', () => {
    const { container } = render(ChatPane, {
      props: slices({ foreground_chat: 'new-1' }),
    })

    expect(screen.getByRole('heading', { level: 2, name: 'New session' })).toBeTruthy()
    expect(container.querySelector('.new-session-where')).toBeNull()
  })

  it('draws the rose at the size its window affords, capped for the hero', async () => {
    // A maximised window: view A would draw 136 px, but the hero shares its pane
    // with the composer, so it stops at 116 (plus the canvas's own 8 px padding).
    mediaWindow(1600, 1000)
    const { container } = render(ChatPane, { props: slices({ foreground_chat: 'new-1' }) })
    const canvas = container.querySelector('canvas') as HTMLCanvasElement

    await waitFor(() => expect(canvas.style.height).toBe('132px'))
  })

  it('names the hero in the language of the moment', async () => {
    const { container } = render(ChatPane, {
      props: slices({ foreground_chat: 'new-1', foreground_folder: FOLDER }),
    })

    setLanguage('es')
    await waitFor(() =>
      expect(screen.getByRole('heading', { level: 2, name: 'Nueva sesión' })).toBeTruthy(),
    )
    // The folder line is the composer row's; the hero does not repeat it.
    expect(screen.queryByText('en gentle-desk')).toBeNull()
    expect(screen.getByRole('img', { name: 'La rosa de gentle shell' })).toBeTruthy()
    expect(container.querySelector('.chat-pane')?.className).toBe('chat-pane is-fresh')
  })
})

describe('the composer that moves', () => {
  it('is the same element before and after the first row, with its draft', async () => {
    const first = slices({ foreground_chat: 'new-1', foreground_folder: FOLDER })
    const { container, rerender } = render(ChatPane, { props: first })
    const box = screen.getByRole('textbox', { name: 'Message' }) as HTMLTextAreaElement
    const form = container.querySelector('.composer')
    await fireEvent.input(box, { target: { value: 'una idea a medio escribir' } })

    await rerender(
      slices({
        foreground_chat: 'new-1',
        foreground_folder: FOLDER,
        messages: [MESSAGE],
        items: ITEMS,
      }),
    )

    // The very node, not a second composer: the draft came with it.
    expect(screen.getByRole('textbox', { name: 'Message' })).toBe(box)
    expect(box.value).toBe('una idea a medio escribir')
    expect(container.querySelector('.composer')).toBe(form)
    expect(container.querySelector('.chat-pane')?.className).toBe('chat-pane')
  })
})

describe('the move to the bottom', () => {
  it('pushes the dock back to where it was and lets it travel in 240 ms', async () => {
    // No reduced-motion preference: this window animates.
    mediaWindow(1200, 900)
    const first = slices({ foreground_chat: 'new-1' })
    const { container, rerender } = render(ChatPane, { props: first })
    const pane = container.querySelector('.chat-pane') as HTMLElement
    const dock = container.querySelector('.chat-pane-dock') as HTMLElement
    // jsdom lays nothing out, so the dock's top follows the state the pane is in -
    // which also pins the ORDER: measured while the class is still there, and
    // again once it is gone.
    dock.getBoundingClientRect = () => rect(pane.classList.contains('is-fresh') ? 200 : 440)

    await rerender(slices({ foreground_chat: 'new-1', messages: [MESSAGE], items: ITEMS }))

    // The inverse of the distance it just fell: it starts at the old place.
    expect(dock.style.transform).toBe('translateY(-240px)')
    // One frame later it is on its way, carried by the transition.
    await frame()
    expect(dock.style.transform).toBe('')
    expect(dock.style.transition).toBe(`transform ${DOCK_MOVE_MS}ms ease-out`)
    // And nothing is left behind once it arrives.
    dock.dispatchEvent(new Event('transitionend'))
    expect(dock.style.transition).toBe('')
  })

  it('moves nothing when the window asks for no motion', async () => {
    mediaWindow(1200, 900, { reducedMotion: true })
    const { container, rerender } = render(ChatPane, {
      props: slices({ foreground_chat: 'new-1' }),
    })
    const pane = container.querySelector('.chat-pane') as HTMLElement
    const dock = container.querySelector('.chat-pane-dock') as HTMLElement
    dock.getBoundingClientRect = () => rect(pane.classList.contains('is-fresh') ? 200 : 440)

    await rerender(slices({ foreground_chat: 'new-1', messages: [MESSAGE], items: ITEMS }))
    await frame()

    // Instant: the composer is simply where it belongs.
    expect(dock.style.transform).toBe('')
    expect(dock.style.transition).toBe('')
  })

  it('moves nothing on the first paint, when there is no place to come from', async () => {
    mediaWindow(1200, 900)
    const { container } = render(ChatPane, { props: slices({ foreground_chat: 'new-1' }) })
    const dock = container.querySelector('.chat-pane-dock') as HTMLElement
    dock.getBoundingClientRect = () => rect(200)

    await frame()

    expect(dock.style.transform).toBe('')
    expect(dock.style.transition).toBe('')
  })
})

describe('the pane’s geometry', () => {
  it('centres the column a little above the middle, and scrolls its overflow', () => {
    const centred = rule('.chat-pane.is-fresh')

    expect(centred).toContain('align-items: center')
    // `safe` keeps the centre for the ordinary case and falls back to the start
    // when the content is taller than the pane, so the top is never unreachable.
    expect(centred).toContain('justify-content: safe center')
    expect(centred).toMatch(/padding:[^;]*clamp\(24px, 8vh, 96px\)/)
    expect(centred).toContain('overflow-y: auto')
  })

  it('caps the composer’s column at 560 px', () => {
    expect(rule('.chat-pane.is-fresh .composer-dock')).toContain('width: min(560px, 100%)')
  })

  it('takes the thread’s place while the chat has no rows, keeping it mounted', () => {
    expect(rule('.chat-pane.is-fresh .chat-pane-body')).toContain('display: none')
  })

  it('starts a short window at the top, with tighter paddings, still scrolling', () => {
    const short = scoped('max-height: 600px', '.chat-pane.is-fresh')

    expect(short).toContain('justify-content: flex-start')
    expect(short).toMatch(/(?:^|[;{\s])padding\s*:/)
    expect(short).toContain('overflow-y: auto')
  })

  it('gives a narrow window 16 px at the sides', () => {
    expect(scoped('max-width: 640px', '.chat-pane.is-fresh')).toContain('padding-inline: 16px')
  })

  it('keeps the composer’s dock inside the pane at any size', () => {
    // The dock is the pane's own child now: it spans the column, and the box
    // inside it decides its own width (560 px here, the thread's 880 px after).
    expect(rule('.chat-pane-dock')).toContain('width: 100%')
  })

  it('leaves the dock’s own width alone once the chat has rows', () => {
    // PRESERVE: the 560 px column is view B's. With rows the box keeps the rule it
    // always had, so it still lines up with the thread's messages.
    expect(rule('.composer-dock')).toContain('width: min(var(--chat-max), calc(100% - 48px))')
  })

  it('names the pane’s state with a class no other rule of the app owns (L: the sidebar’s `.new`)', () => {
    // The pane's first class for "a new session" was `new`, which the sidebar's New
    // session BUTTON already owns (`.new`: a 100%-wide box with a surface background,
    // a border and an accent border on hover). The whole pane inherited it and drew
    // as one giant card. A state class of this pane must appear in the stylesheet
    // only as `.chat-pane.<state>`, never alone.
    const alone = APP_CSS.match(/(^|[^\w.-])\.is-fresh\b/g) ?? []
    const scopedToPane = APP_CSS.match(/\.chat-pane\.is-fresh\b/g) ?? []
    expect(alone.length).toBe(0)
    expect(scopedToPane.length).toBeGreaterThan(0)
    expect(rule('.new')).toContain('height: 38px')
  })

  it('gives the pane’s containers class names of their own, so the Settings dialog’s `.pane-body` is untouched', () => {
    // `SettingsDialog` styles its own `.pane-body` (`.settings-pane .pane-body`). The
    // chat pane's body and dock first took that generic name, which put the chat's
    // `flex: 1 1 0` and `min-height: 0` on the Settings body. No rule of this
    // stylesheet may style a bare `.pane-body` or `.pane-dock`.
    expect(APP_CSS).not.toMatch(/(^|\n)\.pane-body\b/)
    expect(APP_CSS).not.toMatch(/(^|\n)\.pane-dock\b/)
    expect(APP_CSS).toMatch(/\.settings-pane \.pane-body \{/)
    expect(rule('.chat-pane-body')).toContain('flex: 1 1 0')
  })
})
