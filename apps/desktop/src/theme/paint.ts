// The page's theme: what Rust says it is, painted on `<html data-theme>`.
//
// Rust owns the choice (it lives in the app config, so it survives a restart),
// and this module decides nothing: it copies the state onto the root element,
// which is what selects one of the six blocks of `theme.css`. The web side keeps
// no copy of its own - no `localStorage` entry - because a second copy could
// disagree with the config and would outlive a config that was replaced.
//
// The FIRST paint is why this is not a component: `index.html` names the default
// theme, and `main.ts` awaits `startTheme()` before it mounts the app, so a
// window whose config says `light` never draws the dark default first.

import { commands, events, type ThemeView } from '../contract/bindings'

/** The theme of a page whose state has not been read: the `index.html` value. */
const DEFAULT_THEME: ThemeView = 'system'

/** Puts `theme` on the root element, which is what the stylesheet selects on. */
export function paintTheme(theme: ThemeView): void {
  document.documentElement.dataset.theme = theme
}

/**
 * Paints the theme of the state as it arrives: the one first read, which a
 * caller awaits before mounting anything, and then every push.
 *
 * A push that lands while the first read is in flight is NEWER than it, so the
 * read only paints when nothing has been pushed yet. A read or a listen that
 * fails leaves the default theme drawn, which is what a window with no config
 * file shows anyway.
 */
export async function startTheme(): Promise<void> {
  let pushed = false
  try {
    await events.stateChanged.listen((event) => {
      pushed = true
      paintTheme(event.payload.theme)
    })
  } catch {
    // Without the pushes the first read below still paints this frame's theme.
  }
  try {
    const view = await commands.getViewState()
    if (!pushed) paintTheme(view.theme)
  } catch {
    if (!pushed) paintTheme(DEFAULT_THEME)
  }
}
