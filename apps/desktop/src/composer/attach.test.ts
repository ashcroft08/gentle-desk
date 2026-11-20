// The composer's attachments as plain data (design/composer-dock T4, S3, R5,
// R6): which of the paths the dialog or a drop named still travel as images,
// which become chips, what the message carries when it is sent, and how many
// chips are drawn before the row collapses. Every rule is here so the component
// draws a decision instead of making one.

import { describe, expect, it } from 'vitest'
import type { AttachmentKindView, AttachmentView } from '../contract/bindings'
import {
  MAX_VISIBLE,
  collapsedOf,
  filesOf,
  firstRejection,
  imagesOf,
  merged,
  promptWithPaths,
  rejectionOf,
} from './attach'

/** One path as `describe_attachments` answers it. */
function view(path: string, kind: AttachmentKindView = { type: 'File' }, outside = false): AttachmentView {
  const name = path.split(/[\\/]/).pop() ?? path
  return { path, name, kind, outside }
}

const FILE = view('C:\\dev\\repo\\header.tsx')
const OTHER = view('C:\\dev\\repo\\routes.ts')
const IMAGE = view('C:\\dev\\repo\\shot.png', { type: 'Image' })

describe('what a described path becomes', () => {
  it('sends the images to the image command, with the original paths', () => {
    expect(imagesOf([IMAGE, FILE])).toEqual(['C:\\dev\\repo\\shot.png'])
  })

  it('keeps every other file as a chip', () => {
    expect(filesOf([IMAGE, FILE, OTHER])).toEqual([FILE, OTHER])
  })

  it('reads the reason of a refused path', () => {
    const refused = view('C:\\dev\\repo\\assets', { type: 'Rejected', data: 'IsDirectory' })

    expect(rejectionOf(refused)).toBe('IsDirectory')
    expect(rejectionOf(FILE)).toBeNull()
  })

  it('draws ONE line for a refused path: the first one named', () => {
    const missing = view('C:\\dev\\repo\\gone.md', { type: 'Rejected', data: 'NotFound' })

    expect(firstRejection([FILE, missing])).toEqual({ name: 'gone.md', reason: 'NotFound' })
    expect(firstRejection([FILE])).toBeNull()
  })

  it('attaches the same file once, the first one winning', () => {
    const again = view('C:\\dev\\repo\\header.tsx')

    expect(merged([FILE], [again, OTHER])).toEqual([FILE, OTHER])
  })
})

describe('what a send carries', () => {
  it('appends every path below the text, one per line', () => {
    expect(promptWithPaths('please review', [FILE.path, OTHER.path])).toBe(
      'please review\nC:\\dev\\repo\\header.tsx\nC:\\dev\\repo\\routes.ts',
    )
  })

  it('sends only the paths when nothing was typed', () => {
    expect(promptWithPaths('   ', [FILE.path, OTHER.path])).toBe(
      'C:\\dev\\repo\\header.tsx\nC:\\dev\\repo\\routes.ts',
    )
  })

  it('leaves a text with no attachment exactly as it was', () => {
    expect(promptWithPaths('  hello  ', [])).toBe('hello')
    expect(promptWithPaths('   ', [])).toBe('')
  })
})

describe('how many chips are drawn', () => {
  const four = [FILE, OTHER, view('C:\\dev\\repo\\a.ts'), view('C:\\dev\\repo\\b.ts')]

  it('draws the first three and counts the rest', () => {
    const collapsed = collapsedOf(four, false)

    expect(MAX_VISIBLE).toBe(3)
    expect(collapsed.shown).toHaveLength(3)
    expect(collapsed.hidden).toBe(1)
  })

  it('draws them all once the row is expanded, and the button keeps its count', () => {
    const expanded = collapsedOf(four, true)

    expect(expanded.shown).toEqual(four)
    expect(expanded.hidden).toBe(1)
  })

  it('draws a short row whole, with nothing to expand', () => {
    expect(collapsedOf([FILE, OTHER], false)).toEqual({ shown: [FILE, OTHER], hidden: 0 })
  })
})
