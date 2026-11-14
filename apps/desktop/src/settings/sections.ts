// The Settings dialog's rail (settings-dialog L4; 0.1.12 S1-S7; 0.2.0 S2/S4): the
// eight sections as entries in a left rail, the selected entry highlighted, and
// that section's content in the pane on the right. The order here is the order
// the rail draws and the first one is the section the dialog opens on. Each
// entry's glyph carries no meaning on its own - the entry's word is its name -
// and the eight follow the house style the settings gear set: Lucide outline
// geometry on one 24x24 grid. That word is NOT here: the rail is drawn in the
// language the app holds (0.2.0, D4), so `settings/copy.ts` keeps the words and
// this module keeps the order, the glyphs and the ids.
//
// The keys are the chat header's tablist keys (chat/tabs.ts): arrows walk the
// rail and wrap, Home and End jump to the ends, and one entry is the tab stop.
import type { IconName } from '../icons/shapes'

/** The eight sections, as the dialog names them. */
export type Section =
  | 'appearance'
  | 'home'
  | 'updates'
  | 'profiles'
  | 'account'
  | 'usage'
  | 'language'
  | 'about'

/** One rail entry: the section it draws and its glyph. */
export interface SectionEntry {
  id: Section
  icon: IconName
}

/** The rail, in the order the dialog draws it: Appearance comes first. */
export const SECTIONS: SectionEntry[] = [
  { id: 'appearance', icon: 'theme' },
  { id: 'home', icon: 'house' },
  { id: 'updates', icon: 'restart' },
  { id: 'profiles', icon: 'users' },
  { id: 'account', icon: 'key' },
  { id: 'usage', icon: 'gauge' },
  // The language of the app's own words (0.2.0): beside Usage and before About,
  // which stays the last entry Home and End walk to. Its glyph is the speech
  // bubble the house already draws (`message-square`), since the rail's set has
  // no letterform of its own.
  { id: 'language', icon: 'message-square' },
  { id: 'about', icon: 'info' },
]

/** The section the dialog opens on: the first one in the rail. */
export const DEFAULT_SECTION: Section = SECTIONS[0].id

/** The rail's accessible name: the dialog's title already says Settings. */
export const RAIL_LABEL = 'Settings sections'

/** The id of each rail entry, so the pane can name it with `aria-labelledby`. */
export const TAB_IDS: Record<Section, string> = {
  appearance: 'settings-tab-appearance',
  home: 'settings-tab-home',
  updates: 'settings-tab-updates',
  profiles: 'settings-tab-profiles',
  account: 'settings-tab-account',
  usage: 'settings-tab-usage',
  language: 'settings-tab-language',
  about: 'settings-tab-about',
}

/** The pane's one id, since one section is drawn there at a time. */
export const PANEL_ID = 'settings-pane'

/**
 * The entry a vertical move asks for: the arrow keys walk the rail and wrap,
 * Home and End jump to the ends. `null` for every other key, which keeps its own
 * meaning - Escape still closes the dialog.
 */
export function sectionAfter(key: string, from: Section): Section | null {
  const index = SECTIONS.findIndex((entry) => entry.id === from)
  switch (key) {
    case 'ArrowDown':
    case 'ArrowRight':
      return SECTIONS[(index + 1) % SECTIONS.length].id
    case 'ArrowUp':
    case 'ArrowLeft':
      return SECTIONS[(index - 1 + SECTIONS.length) % SECTIONS.length].id
    case 'Home':
      return SECTIONS[0].id
    case 'End':
      return SECTIONS[SECTIONS.length - 1].id
    default:
      return null
  }
}
