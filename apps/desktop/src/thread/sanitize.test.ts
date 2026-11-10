// What the thread is allowed to put in the document. Every case goes through
// the real pipeline (marked, then DOMPurify with the thread's allowlist), so a
// hole in the allowlist is a failing test here, not a review comment.

import { afterEach, describe, expect, it } from 'vitest'
import { cleanup, render } from '@testing-library/svelte'
import { createMarkdownCache } from './markdown'
import Markdown from './Markdown.svelte'

afterEach(cleanup)

const cache = createMarkdownCache()

/** Draws one markdown text the way the thread draws a reply, and returns the document. */
function draw(text: string): HTMLElement {
  const { container } = render(Markdown, { props: { html: cache.htmlFor(`m-${text}`, text) } })
  return container
}

describe('sanitized markdown', () => {
  it('neutralizes a script tag and its content', () => {
    const container = draw('<script>alert("x")</script>')

    expect(container.querySelector('script')).toBeNull()
    expect(container.textContent).not.toContain('alert')
    expect(container.innerHTML).not.toContain('script')
  })

  it('drops an img with an onerror handler and every on* attribute', () => {
    const container = draw('<img src="x" onerror="alert(1)">')

    expect(container.querySelector('img')).toBeNull()
    expect(container.innerHTML).not.toContain('onerror')
    expect(container.innerHTML).not.toContain('alert')
  })

  it('strips a javascript: href from an anchor', () => {
    const container = draw('[click me](javascript:alert(1))')
    const anchors = Array.from(container.querySelectorAll('a'))

    expect(anchors).toHaveLength(1)
    expect(anchors[0].getAttribute('href')).toBeNull()
    expect(container.innerHTML).not.toContain('javascript:')
  })

  it('strips a data: href from an anchor', () => {
    const container = draw('[click me](data:text/html;base64,PHNjcmlwdD5hbGVydCgxKTwvc2NyaXB0Pg==)')
    const anchors = Array.from(container.querySelectorAll('a'))

    expect(anchors).toHaveLength(1)
    expect(anchors[0].getAttribute('href')).toBeNull()
    expect(container.innerHTML).not.toContain('data:')
  })

  it('drops an iframe', () => {
    const container = draw('<iframe src="https://example.com"></iframe>')

    expect(container.querySelector('iframe')).toBeNull()
    expect(container.innerHTML).not.toContain('iframe')
  })

  it('never loads a remote image, and keeps its alt text as plain text', () => {
    // Privacy: a reply must not be able to phone home while it is drawn.
    const container = draw('![a picture of a cat](https://tracker.example/x.png)')

    expect(container.querySelector('img')).toBeNull()
    expect(container.textContent).toContain('a picture of a cat')
    expect(container.innerHTML).not.toContain('tracker.example')
  })

  it('keeps the formatting the thread draws', () => {
    const container = draw('# Title\n\n**bold** and [a link](https://example.com/a)')

    expect(container.querySelector('h1')?.textContent).toBe('Title')
    expect(container.querySelector('strong')?.textContent).toBe('bold')
    expect(container.querySelector('a')?.getAttribute('href')).toBe('https://example.com/a')
  })
})
