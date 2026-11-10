// What the web side is willing to hand to `open_link`. Rust validates the url
// again; this guard only keeps the two cases the slice names (an empty href and
// a `javascript:`/`data:` one) from ever leaving the view.

import { describe, expect, it } from 'vitest'
import { forwardableHref } from './links'

describe('link forwarding', () => {
  it('forwards an http and an https href unchanged', () => {
    expect(forwardableHref('https://example.com/a?b=c#d')).toBe('https://example.com/a?b=c#d')
    expect(forwardableHref('http://example.com')).toBe('http://example.com')
    // An href is padded by the author, not by the reader: the padding is not forwarded.
    expect(forwardableHref('  https://example.com/x  ')).toBe('https://example.com/x')
  })

  it('never forwards an empty or a relative href', () => {
    expect(forwardableHref('')).toBeNull()
    expect(forwardableHref('   ')).toBeNull()
    expect(forwardableHref(null)).toBeNull()
    expect(forwardableHref(undefined)).toBeNull()
    expect(forwardableHref('/docs/page')).toBeNull()
    expect(forwardableHref('#anchor')).toBeNull()
  })

  it('never forwards a javascript: or data: href, whatever its case or padding', () => {
    expect(forwardableHref('javascript:alert(1)')).toBeNull()
    expect(forwardableHref('JavaScript:alert(1)')).toBeNull()
    expect(forwardableHref(' javascript:alert(1)')).toBeNull()
    expect(forwardableHref('java\nscript:alert(1)')).toBeNull()
    expect(forwardableHref('data:text/html;base64,PHNjcmlwdD4=')).toBeNull()
  })

  it('never forwards a scheme the shell would refuse anyway', () => {
    expect(forwardableHref('file:///c:/secrets.txt')).toBeNull()
    expect(forwardableHref('mailto:someone@example.com')).toBeNull()
    expect(forwardableHref('https://')).toBeNull()
  })
})
