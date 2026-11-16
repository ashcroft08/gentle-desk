// Which tab of the Chat | Helpers tablist a key moves to. The rule is plain data so
// the component only draws a decision, and jsdom can test the decision itself.
//
// The tablist's own words are NOT here (0.2.0, S3, D5): the header is drawn in the
// language the app holds, so `chat/copy.ts` - the module that owns the area - keeps
// the tablist's name, the two tab words and the working badge, and this module keeps
// the order, the ids and the walking.

/** The two views the chat header switches between. */
export type ViewTab = 'chat' | 'helpers'

/** The tabs in the order the tablist draws them. */
export const TABS: ViewTab[] = ['chat', 'helpers']

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
