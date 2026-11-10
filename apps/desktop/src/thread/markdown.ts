// Incremental markdown: render only what changed (T5, and the Design notes
// warning that the spike re-parsed the WHOLE growing document per delta and
// burned 5.3 CPU s per 10 s).
//
// How it works, per message id:
//
// - the text is lexed into top-level blocks by `marked`, and the cache stores
//   the raw prefix it was built from, the rendered html of every FINISHED
//   block, and the html of the one block still growing at the end;
// - a new text that extends the cached raw re-lexes only the tail after the
//   last finished block, so a delta renders the open block plus, once, the
//   blocks that just finished. Finished blocks are never rendered again;
// - a text that does not extend the cached raw (message_end resyncs a message
//   from its authoritative text) drops that message's cache and renders fresh;
// - `retain` drops the messages that left the window, so the cache stays the
//   size of what is on screen.
//
// Cost of the block granularity, accepted here and not hidden: each block is
// parsed on its own, so a link reference DEFINED in one block and USED in
// another does not resolve. Tables, lists, fences and setext headings - what a
// reply actually contains - are unaffected.

import { Marked } from 'marked'
import { sanitizeHtml } from './sanitize'

/** Turns one top-level block's raw markdown into html. Injected by the tests. */
export type BlockRender = (raw: string) => string

/** What the cache did: block renders, and fresh renders after a resync. */
export type MarkdownCacheStats = { renders: number; resyncs: number }

export type MarkdownCache = {
  /** The html of `text`, rendered incrementally for the message `id`. */
  htmlFor(id: string, text: string): string
  /** Drops every message whose id is not in `ids`. */
  retain(ids: readonly string[]): void
  /** The render counts, for the tests that bound the cost of a delta. */
  stats(): MarkdownCacheStats
}

/** One message: its raw text, the html of its finished blocks, the open tail. */
type Entry = {
  /** The whole text this entry was built from. */
  raw: string
  /** Where in `raw` the still-open block starts: everything before it is done. */
  cut: number
  /** The finished blocks, in order, already sanitized. */
  done: string[]
  /** The block that is still growing, already sanitized. */
  open: string
}

/** The lexer and parser of this cache: one instance, never the global singleton. */
const parser = new Marked({
  gfm: true,
  // Privacy: an image never reaches the document. Its alt text is drawn as
  // plain text instead, so a remote url is not even requested.
  renderer: { image: ({ text }) => text ?? '' },
})

/** The default renderer: one block of markdown, sanitized. */
function renderBlock(raw: string): string {
  return sanitizeHtml(parser.parse(raw, { async: false }))
}

/** Renders every block but the last as finished, and the last as still open. */
function firstRender(text: string, render: (raw: string) => string): Entry {
  const tokens = parser.lexer(text)
  const done: string[] = []
  let cut = 0
  let open = ''
  tokens.forEach((token, index) => {
    // A `space` token holds only the blank lines between blocks: it can never
    // grow into content, so it is always finished and never worth rendering.
    if (index < tokens.length - 1 || token.type === 'space') {
      if (token.type !== 'space') done.push(render(token.raw))
      cut += token.raw.length
    } else {
      open = render(token.raw)
    }
  })
  return { raw: text, cut, done, open }
}

/** Renders the tail after the last finished block, keeping the finished ones. */
function extend(entry: Entry, text: string, render: (raw: string) => string): Entry {
  const tokens = parser.lexer(text.slice(entry.cut))
  const done = entry.done.slice()
  let cut = entry.cut
  let open = ''
  tokens.forEach((token, index) => {
    if (index < tokens.length - 1 || token.type === 'space') {
      if (token.type !== 'space') done.push(render(token.raw))
      cut += token.raw.length
    } else {
      open = render(token.raw)
    }
  })
  return { raw: text, cut, done, open }
}

/**
 * Creates one cache. `render` defaults to marked plus the sanitizer; the tests
 * inject a counting renderer, which is how the cost of a delta is measured.
 */
export function createMarkdownCache(render: BlockRender = renderBlock): MarkdownCache {
  const entries = new Map<string, Entry>()
  const stats: MarkdownCacheStats = { renders: 0, resyncs: 0 }

  function counted(raw: string): string {
    stats.renders += 1
    return render(raw)
  }

  function htmlFor(id: string, text: string): string {
    const held = entries.get(id)
    let entry: Entry
    if (held !== undefined && text.startsWith(held.raw)) {
      entry = text === held.raw ? held : extend(held, text, counted)
    } else {
      if (held !== undefined) stats.resyncs += 1
      entry = firstRender(text, counted)
    }
    entries.set(id, entry)
    return entry.done.join('') + entry.open
  }

  function retain(ids: readonly string[]): void {
    const keep = new Set(ids)
    for (const id of [...entries.keys()]) {
      if (!keep.has(id)) entries.delete(id)
    }
  }

  return { htmlFor, retain, stats: () => ({ ...stats }) }
}
