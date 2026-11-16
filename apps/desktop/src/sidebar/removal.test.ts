// The rules a workspace removal draws on (S1, S2, S3): the count and the size the
// dialog names, the path with its middle elided, the split of its one consequence
// line into a bold scope, and the chat titles its disclosure lists. The offer
// itself is Rust's, so the two reads below also pin what the view does with an
// answer - including a refused one.
import '../test-support/dom-env'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import type { SessionTreeRow_Serialize } from '../contract/bindings'
import { handlerArgs, resetTauriMock, setHandler } from '../test-support/tauri-mock'
import {
  chatCount,
  chatScope,
  chatTitles,
  consequenceText,
  elideMiddle,
  megabytes,
  removalOffer,
  removeWorkspace,
} from './removal'

vi.mock('@tauri-apps/api/core', () => import('../test-support/tauri-mock').then((m) => m.coreModule()))

beforeEach(() => {
  resetTauriMock()
})

/** One workspace header, as `sessions_window` answers it. */
function header(label: string): SessionTreeRow_Serialize {
  return {
    kind: 'Workspace',
    key: label,
    label,
    path: `C:\\dev\\${label}`,
    name: label,
    collapsed: false,
    chat_count: 1,
    available: true,
    reason: null,
  }
}

/** One chat row under the workspace above it. */
function chat(title: string): SessionTreeRow_Serialize {
  return {
    kind: 'Chat',
    id: `C:\\pi\\sessions\\${title}.jsonl`,
    title,
    project: 'gentle-desk',
    message_count: 1,
    modified_ago: 'now',
  }
}

describe('the size a removal names', () => {
  it('writes megabytes with one decimal, the spec`s own measurements included', () => {
    expect(megabytes(1_800_000)).toBe('1.7 MB')
    expect(megabytes(36 * 1024 * 1024)).toBe('36.0 MB')
    expect(megabytes(0)).toBe('0.0 MB')
  })

  it('draws no size at all when none arrived', () => {
    expect(megabytes(null)).toBeNull()
    expect(chatScope(2, null)).toBe('2 chats')
  })

  it('counts chats in one word both languages spell alike', () => {
    expect(chatCount(1)).toBe('1 chat')
    expect(chatCount(2)).toBe('2 chats')
    expect(chatCount(0)).toBe('0 chats')
    expect(chatScope(2, 1_800_000)).toBe('2 chats (1.7 MB)')
  })
})

describe('the path a dialog shows', () => {
  it('elides the middle and keeps both ends', () => {
    expect(elideMiddle('C:\\dev\\gentle-desk')).toBe('C:\\…\\gentle-desk')
    expect(elideMiddle('C:\\Users\\josep\\dev\\gentle-desk')).toBe('C:\\…\\gentle-desk')
    expect(elideMiddle('/home/josep/gentle-desk')).toBe('/…/gentle-desk')
    expect(elideMiddle('C:\\dev\\a-workspace-with-a-long-name')).toBe(
      'C:\\…\\a-workspace-with-a-long-name',
    )
  })

  it('leaves a path of one or two parts whole, since it is all ends already', () => {
    expect(elideMiddle('C:\\dev')).toBe('C:\\dev')
    expect(elideMiddle('C:\\pi\\sessions\\2026_s1.jsonl')).toBe('C:\\…\\2026_s1.jsonl')
    expect(elideMiddle('a')).toBe('a')
    // The spaces a name arrives with are trimmed; the middle still goes.
    expect(elideMiddle('  C:\\dev\\one  ')).toBe('C:\\…\\one')
  })
})

describe('the consequence line', () => {
  it('reads as one sentence with the scope in the middle', () => {
    expect(
      consequenceText({ before: 'Removing deletes its ', scope: '2 chats', after: '.' }),
    ).toBe('Removing deletes its 2 chats.')
  })
})

describe('the chats a removal lists', () => {
  it('answers the titles under that workspace alone, and never their file names', () => {
    const rows = [header('one'), chat('Alpha'), chat('Bravo'), header('two'), chat('Charlie')]

    expect(chatTitles(rows, 'C:\\dev\\one')).toEqual(['Alpha', 'Bravo'])
    expect(chatTitles(rows, 'C:\\dev\\two')).toEqual(['Charlie'])
    expect(chatTitles(rows, 'C:\\dev\\gone')).toEqual([])
    expect(chatTitles(rows, 'C:\\dev\\one').join(' ')).not.toMatch(/\.jsonl/)
  })

  it('answers fewer titles when the page starts inside the group', () => {
    expect(chatTitles([chat('Alpha')], 'C:\\dev\\one')).toEqual([])
  })
})

describe('the two commands a removal sends', () => {
  it('reads the offer out of the shell answer', async () => {
    const offer = { key: 'one', path: 'C:\\dev\\one', label: 'one', chats: 2, bytes: 1_800_000 }
    setHandler('workspace_removal_offer', () => offer)

    expect(await removalOffer('C:\\dev\\one')).toEqual(offer)
  })

  it('answers nothing when the shell refuses the read, and keeps the reason out of the view', async () => {
    expect(await removalOffer('C:\\dev\\one')).toBeNull()
  })

  it('sends the removal with the confirmation only the dialog gives', async () => {
    setHandler('remove_workspace', () => null)

    await removeWorkspace('C:\\dev\\one')

    expect(handlerArgs('remove_workspace')).toEqual([
      { path: 'C:\\dev\\one', confirmed: true },
    ])
  })

  it('swallows a refused removal, because the push carries the notice', async () => {
    setHandler('remove_workspace', () => {
      throw new Error('a chat is still working')
    })

    await expect(removeWorkspace('C:\\dev\\one')).resolves.toBeUndefined()
  })
})
