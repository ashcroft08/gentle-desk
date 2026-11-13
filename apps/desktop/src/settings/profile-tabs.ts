// The Profiles pane's two sub-tabs (0.1.12, L17): the maintainer's ask was one
// pane too long, so the switch and the active profile took the first tab and the
// agent table took the second. The pair is the chat header's tablist pattern
// (chat/tabs.ts): one tab stop, the arrow keys walk the pair and wrap, Home and
// End jump to the ends, and each tab names the panel it controls.

/** The two views the Profiles pane switches between. */
export type ProfileTab = 'profile' | 'agents'

/** The tabs in the order the pane draws them: the Profile first (L17). */
export const PROFILE_TABS: ProfileTab[] = ['profile', 'agents']

/** The tablist's accessible name: the pane's own title already says Profiles. */
export const PROFILE_TABS_LABEL = 'Profile details'

/** Each tab's own word. */
export const PROFILE_TAB_COPY: Record<ProfileTab, string> = {
  profile: 'Profile',
  agents: 'Agents',
}

/** The id of each tab, so its panel can name it with `aria-labelledby`. */
export const PROFILE_TAB_IDS: Record<ProfileTab, string> = {
  profile: 'profiles-tab-profile',
  agents: 'profiles-tab-agents',
}

/** The id of each panel, so its tab can point at it with `aria-controls`. */
export const PROFILE_PANEL_IDS: Record<ProfileTab, string> = {
  profile: 'profiles-panel-profile',
  agents: 'profiles-panel-agents',
}

/**
 * The tab a horizontal or vertical move asks for: the arrow keys walk the pair
 * and wrap, Home and End jump to the ends. `null` for every other key, which
 * keeps its own meaning.
 */
export function profileTabAfter(key: string, from: ProfileTab): ProfileTab | null {
  const index = PROFILE_TABS.indexOf(from)
  switch (key) {
    case 'ArrowRight':
    case 'ArrowDown':
      return PROFILE_TABS[(index + 1) % PROFILE_TABS.length]
    case 'ArrowLeft':
    case 'ArrowUp':
      return PROFILE_TABS[(index - 1 + PROFILE_TABS.length) % PROFILE_TABS.length]
    case 'Home':
      return PROFILE_TABS[0]
    case 'End':
      return PROFILE_TABS[PROFILE_TABS.length - 1]
    default:
      return null
  }
}
