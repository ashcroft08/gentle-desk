// Which tab of the Chat | Helpers tablist a key moves to, and the name the
// working badge on the Helpers tab carries (01b). Both are plain rules so the
// component only draws a decision, and jsdom can test the decision itself.

/** The two views the chat header switches between. */
export type ViewTab = 'chat' | 'helpers'

/** The tabs in the order the tablist draws them. */
export const TABS: ViewTab[] = ['chat', 'helpers']

/** The tablist's accessible name, as 01b writes it. */
export const TABS_LABEL = 'View'

/** The tab's own word. */
export const TAB_COPY: Record<ViewTab, string> = { chat: 'Chat', helpers: 'Helpers' }

/** The id of each tab, so its panel can name it with `aria-labelledby`. */
export const TAB_IDS: Record<ViewTab, string> = { chat: 't-chat', helpers: 't-help' }

/** The id of each panel, so its tab can point at it with `aria-controls`. */
export const PANEL_IDS: Record<ViewTab, string> = { chat: 'p-chat', helpers: 'p-helpers' }

/**
 * The tab a horizontal or vertical move asks for: the arrow keys walk the list
 * and wrap, Home and End jump to the ends. `null` for every other key, which
 * keeps its own meaning.
 */
export function tabAfter(key: string, from: ViewTab): ViewTab | null {
  const index = TABS.indexOf(from)
  switch (key) {
    case 'ArrowRight':
    case 'ArrowDown':
      return TABS[(index + 1) % TABS.length]
    case 'ArrowLeft':
    case 'ArrowUp':
      return TABS[(index - 1 + TABS.length) % TABS.length]
    case 'Home':
      return TABS[0]
    case 'End':
      return TABS[TABS.length - 1]
    default:
      return null
  }
}

/**
 * The working badge's accessible name: one helper is singular, every other
 * count plural. The badge is drawn only while something runs, so this is never
 * asked for zero.
 */
export function workingBadgeName(running: number): string {
  return running === 1 ? '1 helper working' : `${running} helpers working`
}
