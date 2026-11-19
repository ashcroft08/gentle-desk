// The composer's dock as the artboard draws it (design/composer-dock/01-dock.html,
// approved): the context rail ABOVE the box and outside its border, one borderless
// row inside it - attach clip, the two selects, a hairline, the folder, Send - and
// the key hint BELOW the border, left-aligned.
//
// This file pins the PLACEMENT, which is what the slice changes: what surrounds
// the box, what shares the row, and the order of the row's groups. Each part's
// own behaviour lives beside its component.
//
// What this file deliberately does NOT draw: the permission mode (T3), whose
// behaviour lives beside its own component. The attach clip (T4) is the row's
// first part now, and it is pinned here only by its place: what it DOES lives in
// `Composer.attachments.test.ts`.
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { cleanup, render } from '@testing-library/svelte'
import type { ModelChipView } from '../contract/bindings'
import { resetTauriMock, setHandler, viewState } from '../test-support/tauri-mock'
import Composer from './Composer.svelte'
import { composerCopy } from './copy'

vi.mock('@tauri-apps/api/core', () => import('../test-support/tauri-mock').then((m) => m.coreModule()))
vi.mock('@tauri-apps/api/event', () => import('../test-support/tauri-mock').then((m) => m.eventModule()))
vi.mock('@tauri-apps/api/webview', () => import('../test-support/webview').then((m) => m.webviewModule()))

const READY: ModelChipView = {
  status: { type: 'Ready' },
  models: [{ provider: 'prov', model_id: 'm1', name: 'Model One', reasoning: true }],
  current: { provider: 'prov', model_id: 'm1', name: 'Model One', reasoning: true },
  thinking_level: 'high',
  thinking_levels: ['off', 'medium', 'high'],
}

const FOLDER = { name: 'gentle-desk', path: 'C:\\Users\\josep\\dev\\gentle-desk' }
const BRANCH = { name: 'feat/0.7.0-transcript', path: FOLDER.path }
const USAGE = { tokens: 70_000, percent: 35, window: 200_000 }
const { keysHint: KEYS } = composerCopy('en')

beforeEach(() => {
  resetTauriMock()
  setHandler('dispatch_intent', () => viewState())
})

afterEach(cleanup)

/** Draws the dock with everything this slice can fill. */
function draw() {
  return render(Composer, { props: { chat: 'c1', chip: READY, contextUsage: USAGE, folder: FOLDER } })
}
describe('the dock', () => {
  it('puts the context rail above the box, outside its border', () => {
    const { container } = draw()

    const rail = container.querySelector('.composer-dock > .ctx-rail')
    expect(rail).toBeTruthy()
    expect(rail?.nextElementSibling?.tagName).toBe('FORM')
    // Outside the border: the box holds the input and the row, nothing else.
    expect(container.querySelector('.composer .ctx-rail')).toBeNull()
  })

  it('keeps the key hint below the box, outside the form', () => {
    const { container } = draw()
    const hint = container.querySelector('.composer-dock > .keys') as HTMLElement

    expect(hint.textContent).toBe(KEYS)
    expect(hint.closest('form')).toBeNull()
    expect(hint.previousElementSibling?.tagName).toBe('FORM')
  })

  it('regroups the row: the clip, the two selects, a hairline, the folder, then Send', () => {
    const { container } = draw()
    const row = container.querySelector('.composer .row') as HTMLElement
    const parts = Array.from(row.children).map((child) => child.className)

    expect(parts).toEqual(['att', 'chip', 'vsep', 'folder', 'grow', 'ic fill tip-r'])
  })

  it('leaves the gap that is not a group empty: no branch and no placeholder', () => {
    const { container } = draw()
    const row = container.querySelector('.composer .row') as HTMLElement

    // One hairline (the model group's), one folder, nothing between them and the
    // send but the flexible space.
    expect(row.querySelectorAll('.vsep')).toHaveLength(1)
    expect(row.querySelector('.branch')).toBeNull()
    expect(row.querySelector('[data-glyph="branch"]')).toBeNull()
  })

  it('draws the branch after the folder, and the one hairline stays before the group', () => {
    const { container } = render(Composer, {
      props: { chat: 'c1', chip: READY, folder: FOLDER, branch: BRANCH },
    })
    const row = container.querySelector('.composer .row') as HTMLElement
    const parts = Array.from(row.children).map((child) => child.className)

    // What the message runs ON, the hairline, then where it runs - the folder
    // and the branch on it, in that order - and Send.
    expect(parts).toEqual(['att', 'chip', 'vsep', 'folder', 'branch', 'grow', 'ic fill tip-r'])
    expect(row.querySelectorAll('.vsep')).toHaveLength(1)
    expect((row.querySelector('.branch') as HTMLElement).textContent?.trim()).toBe(BRANCH.name)
  })

  it('leaves the branch out for a folder no repository holds, with no gap either', () => {
    // R3: no repo means no chip at all, so the row keeps its folder and its
    // flexible space and nothing stands where the branch would have been.
    const { container } = render(Composer, {
      props: { chat: 'c1', chip: READY, folder: FOLDER, branch: null },
    })
    const row = container.querySelector('.composer .row') as HTMLElement
    const parts = Array.from(row.children).map((child) => child.className)

    expect(parts).toEqual(['att', 'chip', 'vsep', 'folder', 'grow', 'ic fill tip-r'])
    expect(row.querySelector('.branch')).toBeNull()
    expect(row.querySelectorAll('.vsep')).toHaveLength(1)
  })

  it('draws no context chip in the row any more: the rail is the only reading', () => {
    const { container } = draw()

    expect(container.querySelector('.ctx')).toBeNull()
    expect(container.querySelector('.bar')).toBeNull()
  })

  it('keeps the row together when a chat has no folder to name', () => {
    const { container } = render(Composer, { props: { chat: 'c1', chip: READY } })
    const row = container.querySelector('.composer .row') as HTMLElement
    const parts = Array.from(row.children).map((child) => child.className)

    expect(parts).toEqual(['att', 'chip', 'grow', 'ic fill tip-r'])
    expect(container.querySelector('.folder')).toBeNull()
  })
})
