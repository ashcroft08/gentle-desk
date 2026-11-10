// What the web side hands to `open_link`, and what it refuses to hand over.
//
// Rust validates the url again (T5a: only http/https with a host and no
// credentials, no control characters); this guard exists so an empty href or a
// `javascript:`/`data:` one never even leaves the view, and so a click inside
// the markdown never lets the WebView navigate itself.

import { openLink } from '../bridge'

/** The two schemes the shell opens. */
const FORWARDABLE = new Set(['http:', 'https:'])

/**
 * The href to forward, or `null` when the view must not forward anything: an
 * empty href, a relative one, or any scheme other than http and https.
 * Relative links are refused on purpose - the thread has no base url to resolve
 * them against, and Rust would refuse one anyway.
 */
export function forwardableHref(href: string | null | undefined): string | null {
  const raw = (href ?? '').trim()
  if (raw === '') return null
  let url: URL
  try {
    url = new URL(raw)
  } catch {
    return null
  }
  if (!FORWARDABLE.has(url.protocol) || url.host === '') return null
  return raw
}

/**
 * The one click handler of drawn markdown: an anchor inside the reply is
 * cancelled (so the WebView never navigates) and a forwardable href is opened
 * by the shell. Middle click and ctrl+click are the same case: WebView2 would
 * otherwise open a second window or a new tab, so `auxclick` runs the same
 * handler and cancels too.
 *
 * Keyboard Enter on a focused link is the browser's own activation of that
 * link, which dispatches a click event: it reaches this handler without a key
 * listener of its own, and a key listener would forward the link twice.
 */
export function linkClickHandler(event: MouseEvent): void {
  const target = event.target
  const anchor = target instanceof Element ? target.closest('a') : null
  if (anchor === null) return
  event.preventDefault()
  const href = forwardableHref(anchor.getAttribute('href'))
  if (href !== null) void openLink(href)
}
