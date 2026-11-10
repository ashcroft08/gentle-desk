// The incremental render (T5, Design notes: the spike re-parsed the whole growing
// document per delta and burned 5.3 CPU s per 10 s). What these tests count is
// the COST: how many top-level blocks one streamed delta renders. A finished
// block must never be rendered again, so a streamed reply costs one render per
// delta plus one per block, never one per delta times blocks.

import { describe, expect, it } from 'vitest'
import { createMarkdownCache, type BlockRender } from './markdown'

const LONG_A = Array.from({ length: 40 }, (_, i) => `alpha${i}`).join(' ')
const LONG_B = Array.from({ length: 40 }, (_, i) => `beta${i}`).join(' ')

/** A ~730-character reply with every block kind the sanitizer allows. */
const FINAL = [
  '# Title',
  'First paragraph with **bold** and `inline code`.',
  '- one\n- two',
  '> quoted',
  '```ts\nconst x = 1\n```',
  '| a | b |\n| - | - |\n| 1 | 2 |',
  LONG_A,
  LONG_B,
  'Last paragraph.',
].join('\n\n')

/** Every block but the last: each of these must be rendered at most twice. */
const FINISHED_BLOCKS = [
  '# Title',
  'First paragraph with **bold** and `inline code`.',
  '- one\n- two',
  '> quoted',
  '```ts\nconst x = 1\n```',
  '| a | b |\n| - | - |\n| 1 | 2 |',
  LONG_A,
  LONG_B,
]

/** How many pushes the whole reply arrives in: the deltas the bound is about. */
const DELTAS = 200

type Cache = ReturnType<typeof createMarkdownCache>

/** Streams `text` into `cache` the way the bridge does: one growing prefix per push. */
function stream(cache: Cache, id: string, text: string): number {
  const step = Math.max(1, Math.floor(text.length / DELTAS))
  let pushes = 0
  for (let end = step; end < text.length; end += step) {
    cache.htmlFor(id, text.slice(0, end))
    pushes += 1
  }
  cache.htmlFor(id, text)
  return pushes + 1
}

function countingCache(): { cache: Cache; rendersOf: (raw: string) => number } {
  const counts = new Map<string, number>()
  const render: BlockRender = (raw) => {
    counts.set(raw, (counts.get(raw) ?? 0) + 1)
    return `<div>${raw.length}</div>`
  }
  return { cache: createMarkdownCache(render), rendersOf: (raw) => counts.get(raw) ?? 0 }
}

describe('incremental markdown', () => {
  it('never renders a finished block again while the reply keeps growing', () => {
    const { cache, rendersOf } = countingCache()
    // Far enough in that two blocks are finished and a third is still open.
    const prefix = '# Title\n\nfirst paragraph\n\nsecond paragraph grows'
    cache.htmlFor('m1', prefix)
    const title = rendersOf('# Title')
    const first = rendersOf('first paragraph')
    expect(title).toBe(1)
    expect(first).toBe(1)

    const rest = '\n\n' + Array.from({ length: 60 }, (_, i) => `more ${i}`).join('\n\n')
    for (let end = 3; end <= rest.length; end += 3) cache.htmlFor('m1', prefix + rest.slice(0, end))
    cache.htmlFor('m1', prefix + rest)

    // Sixty more blocks and ~400 more deltas later: the two finished blocks
    // have not been rendered a second time.
    expect(rendersOf('# Title')).toBe(title)
    expect(rendersOf('first paragraph')).toBe(first)
  })

  it('costs one render of the open block per delta, plus one per block', () => {
    const { cache, rendersOf } = countingCache()
    const pushes = stream(cache, 'm1', FINAL)

    // A block is rendered while it grows at the tail, and once more when it
    // finishes; never again. A whole-document render would cost
    // `pushes * blocks` instead of `pushes + blocks`.
    expect(pushes).toBeGreaterThanOrEqual(DELTAS)
    for (const raw of FINISHED_BLOCKS) expect(rendersOf(raw)).toBeLessThanOrEqual(2)
    expect(cache.stats().renders).toBeLessThanOrEqual(pushes + FINISHED_BLOCKS.length)
  })

  it('never re-renders the finished prefix, however long the reply grows', () => {
    const { cache } = countingCache()
    const text = Array.from({ length: 20 }, (_, i) => `paragraph ${i}`).join('\n\n')
    const pushes = stream(cache, 'm1', text)

    expect(pushes).toBeGreaterThan(100)
    expect(cache.stats().renders).toBeLessThanOrEqual(pushes + 19)
  })

  it('ends with the same html a fresh render of the whole text gives', () => {
    const streamed = createMarkdownCache()
    stream(streamed, 'm1', FINAL)
    const fresh = createMarkdownCache()

    const html = streamed.htmlFor('m1', FINAL)
    expect(html).toBe(fresh.htmlFor('m2', FINAL))
    expect(html).toContain('<h1>Title</h1>')
    expect(html).toContain('<ul>')
  })

  it('drops the cache of a message whose text is not an extension (message_end resync)', () => {
    const cache = createMarkdownCache()
    expect(cache.htmlFor('m1', '# Draft\n\nhalf a reply')).toContain('Draft')

    const resynced = cache.htmlFor('m1', '# Final\n\n## Second\n\nthe authoritative reply')
    expect(resynced).toContain('Final')
    expect(resynced).not.toContain('Draft')
    expect(resynced).not.toContain('half a reply')
    expect(cache.stats().resyncs).toBe(1)
  })

  it('forgets the messages that left the fifty-message window', () => {
    const { cache, rendersOf } = countingCache()
    cache.htmlFor('gone', 'a paragraph')
    expect(rendersOf('a paragraph')).toBe(1)

    cache.retain(['kept'])
    cache.htmlFor('gone', 'a paragraph')
    expect(rendersOf('a paragraph')).toBe(2)
  })

  it('keeps the html of a message that stayed in the window', () => {
    const { cache, rendersOf } = countingCache()
    const first = cache.htmlFor('kept', 'a paragraph')
    cache.retain(['kept'])
    expect(cache.htmlFor('kept', 'a paragraph')).toBe(first)
    expect(rendersOf('a paragraph')).toBe(1)
  })
})
