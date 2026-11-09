import { describe, expect, it } from 'vitest'
import { chatRow, workspaceRow } from '../test-support/tauri-mock'
import { isWorkspaceRow, markKindOf, ROW_HEIGHT, virtualWindow } from './rows'

describe('ROW_HEIGHT', () => {
  it('is the fixed row height of the design system', () => {
    expect(ROW_HEIGHT).toBe(34)
  })
})

describe('virtualWindow', () => {
  it('draws nothing for an empty list', () => {
    expect(virtualWindow(0, 0, 340)).toEqual({ first: 0, count: 0, top: 0, bottom: 0 })
  })

  it('draws the one row of a one-row list and nothing on the last page', () => {
    expect(virtualWindow(1, 0, 340)).toEqual({ first: 0, count: 1, top: 0, bottom: 0 })
  })

  it('draws every row when the list is shorter than the viewport', () => {
    expect(virtualWindow(3, 0, 340)).toEqual({ first: 0, count: 3, top: 0, bottom: 0 })
  })

  it('reserves the spacers around the viewport of a long list', () => {
    expect(virtualWindow(100, 0, 340)).toEqual({ first: 0, count: 10, top: 0, bottom: 3060 })
  })

  it('stops at the last full page instead of scrolling past it', () => {
    expect(virtualWindow(100, 5000, 340)).toEqual({ first: 90, count: 10, top: 3060, bottom: 0 })
  })

  it('rounds a partly visible row into the viewport', () => {
    expect(virtualWindow(100, 40, 35)).toEqual({ first: 1, count: 2, top: 34, bottom: 3298 })
  })

  it('draws nothing when the viewport has no height', () => {
    expect(virtualWindow(10, 0, 0)).toEqual({ first: 0, count: 0, top: 0, bottom: 0 })
  })

  it('measures the tree by the rows Rust counts, headers included', () => {
    // Two workspace headers and eight chats: ten rows of one height, so the
    // spacers of a viewport of five rows leave the last five below it.
    expect(virtualWindow(10, 0, 5 * ROW_HEIGHT)).toEqual({
      first: 0,
      count: 5,
      top: 0,
      bottom: 5 * ROW_HEIGHT,
    })
    expect(virtualWindow(10, 5 * ROW_HEIGHT, 5 * ROW_HEIGHT)).toEqual({
      first: 5,
      count: 5,
      top: 5 * ROW_HEIGHT,
      bottom: 0,
    })
  })
})

describe('isWorkspaceRow', () => {
  it('tells a workspace header from a chat', () => {
    const header = workspaceRow('repo')
    const chat = chatRow('a')
    expect(isWorkspaceRow(header)).toBe(true)
    expect(isWorkspaceRow(chat)).toBe(false)
  })

  it('keeps the fields the row was served with', () => {
    const header = workspaceRow('repo', { collapsed: true, chat_count: 3, available: false })
    if (!isWorkspaceRow(header)) throw new Error('a header row')
    expect(header.label).toBe('repo')
    expect(header.collapsed).toBe(true)
    expect(header.chat_count).toBe(3)
    expect(header.available).toBe(false)
    const chat = chatRow('a', { title: 'first chat' })
    if (isWorkspaceRow(chat)) throw new Error('a chat row')
    expect(chat.title).toBe('first chat')
  })
})

describe('markKindOf', () => {
  it('names every mark the contract carries', () => {
    for (const kind of ['Working', 'Queued', 'Opening', 'NewReply', 'NeedsAttention', 'Ended']) {
      expect(markKindOf(kind)).toBe(kind)
    }
  })

  it('draws no mark for a name the contract does not carry', () => {
    expect(markKindOf('Idle')).toBeNull()
  })
})
