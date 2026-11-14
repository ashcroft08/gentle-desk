// The Settings dialog's rail (settings-dialog L4; 0.1.12 S1-S7): the seven sections
// as entries in a left rail, the selected entry highlighted, and that section's
// content in the pane on the right. The order here is the order the rail draws
// and the first one is the section the dialog opens on. Each entry's glyph
// carries no meaning on its own - the entry's word is its name - and the seven
// follow the house style the settings gear set: Lucide outline geometry on one
// 24x24 grid.
//
// The keys are the chat header's tablist keys (chat/tabs.ts): arrows walk the
// rail and wrap, Home and End jump to the ends, and one entry is the tab stop.
import type { IconName } from '../icons/shapes'
import {
  ABOUT_HINT,
  ABOUT_TITLE,
  ACCOUNT_HINT,
  ACCOUNT_TITLE,
  APPEARANCE_HINT,
  APPEARANCE_TITLE,
  HOME_HINT,
  HOME_TITLE,
  PROFILES_HINT,
  PROFILES_TITLE,
  UPDATES_HINT,
  UPDATES_TITLE,
  USAGE_HINT,
  USAGE_TITLE,
} from './copy'

/** The seven sections, as the dialog names them. */
export type Section =
  | 'appearance'
  | 'home'
  | 'updates'
  | 'profiles'
  | 'account'
  | 'usage'
  | 'about'

/** One rail entry: the section it draws, its word and its glyph. */
export interface SectionEntry {
  id: Section
  title: string
  icon: IconName
}

/** The rail, in the order the dialog draws it: Appearance comes first. */
export const SECTIONS: SectionEntry[] = [
  { id: 'appearance', title: APPEARANCE_TITLE, icon: 'theme' },
  { id: 'home', title: HOME_TITLE, icon: 'house' },
  { id: 'updates', title: UPDATES_TITLE, icon: 'restart' },
  { id: 'profiles', title: PROFILES_TITLE, icon: 'users' },
  { id: 'account', title: ACCOUNT_TITLE, icon: 'key' },
  { id: 'usage', title: USAGE_TITLE, icon: 'gauge' },
  { id: 'about', title: ABOUT_TITLE, icon: 'info' },
]

/** Each section's word, for the pane's heading (the rail entries carry it too). */
export const SECTION_TITLES: Record<Section, string> = {
  appearance: APPEARANCE_TITLE,
  home: HOME_TITLE,
  updates: UPDATES_TITLE,
  profiles: PROFILES_TITLE,
  account: ACCOUNT_TITLE,
  usage: USAGE_TITLE,
  about: ABOUT_TITLE,
}

/** The section the dialog opens on: the first one in the rail. */
export const DEFAULT_SECTION: Section = SECTIONS[0].id

/** The rail's accessible name: the dialog's title already says Settings. */
export const RAIL_LABEL = 'Settings sections'

/** Each section's one-line hint, drawn at the top of the pane. */
export const SECTION_HINTS: Record<Section, string> = {
  appearance: APPEARANCE_HINT,
  home: HOME_HINT,
  updates: UPDATES_HINT,
  profiles: PROFILES_HINT,
  account: ACCOUNT_HINT,
  usage: USAGE_HINT,
  about: ABOUT_HINT,
}

/** The id of each rail entry, so the pane can name it with `aria-labelledby`. */
export const TAB_IDS: Record<Section, string> = {
  appearance: 'settings-tab-appearance',
  home: 'settings-tab-home',
  updates: 'settings-tab-updates',
  profiles: 'settings-tab-profiles',
  account: 'settings-tab-account',
  usage: 'settings-tab-usage',
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
