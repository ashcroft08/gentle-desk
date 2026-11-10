// The one gate between markdown and the document: DOMPurify with an allowlist
// that names every tag the thread may draw and nothing else.
//
// Decisions this file makes, all of them privacy or safety rules:
//
// - no script, style, iframe or form, and no img: a reply must not be able to
//   run code or phone home while it is drawn. `img` is stripped here as a
//   second line of defence; the markdown renderer already turns an image into
//   its alt text, so a remote image never becomes a request;
// - every `on*` attribute is gone, because no attribute but `href` survives;
// - a link keeps `href` only: no target, no rel, no class, no style.

import DOMPurify from 'dompurify'

/** The tags drawn markdown may use. */
export const ALLOWED_TAGS = [
  'a',
  'p',
  'br',
  'hr',
  'strong',
  'em',
  'del',
  'code',
  'pre',
  'blockquote',
  'ul',
  'ol',
  'li',
  'h1',
  'h2',
  'h3',
  'h4',
  'h5',
  'h6',
  'table',
  'thead',
  'tbody',
  'tr',
  'th',
  'td',
] as const

/** The only attribute kept: a link's href. */
export const ALLOWED_ATTR = ['href']

/** Tags DOMPurify must drop with their content, never unwrap. */
const FORBID_TAGS = ['script', 'style', 'iframe', 'form', 'img']

/** Sanitizes one rendered block of markdown; safe to call with an empty string. */
export function sanitizeHtml(html: string): string {
  return DOMPurify.sanitize(html, {
    ALLOWED_TAGS: [...ALLOWED_TAGS],
    ALLOWED_ATTR,
    FORBID_TAGS,
    // No `data-*`, no `aria-*`: the thread draws what the markdown says, and
    // nothing a model writes may pose as the app's own markup.
    ALLOW_DATA_ATTR: false,
    ALLOW_ARIA_ATTR: false,
    KEEP_CONTENT: true,
    RETURN_TRUSTED_TYPE: false,
  }) as string
}
