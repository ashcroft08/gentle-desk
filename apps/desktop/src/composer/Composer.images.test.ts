// The composer's drop and its attachments, one test per rule: the paths of a
// drop are all that travels up, the images the chat holds are drawn with a way
// to take each one away, a refusal is said out loud, and the composer says where
// files go while they are in the air. Nothing here sends anything by itself.
//
// The paths come from Tauri's own drop event (`./drop`), which is the only place
// a dropped file's path exists: the webview's `File` has none.

import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { cleanup, fireEvent, render, screen } from '@testing-library/svelte'
import { tick } from 'svelte'
import type { ChatActivityMark, ImageView } from '../contract/bindings'
import { handlerArgs, resetTauriMock, setHandler, viewState } from '../test-support/tauri-mock'
import { dragEvent } from '../test-support/webview'
import Composer from './Composer.svelte'
import { DROP_HINT, REMOVE_IMAGE } from './copy'

vi.mock('@tauri-apps/api/core', () => import('../test-support/tauri-mock').then((m) => m.coreModule()))
vi.mock('@tauri-apps/api/event', () => import('../test-support/tauri-mock').then((m) => m.eventModule()))
vi.mock('@tauri-apps/api/webview', () => import('../test-support/webview').then((m) => m.webviewModule()))

/** One image a chat holds, as Rust pushes it: a name and where its thumbnail is
 * served from, never a byte. */
function image(id: string, name: string): ImageView {
  return { id, name, preview_url: `http://gentle-image.localhost/${id}` }
}

type Props = {
  chat: string | null
  mark?: ChatActivityMark | null
  images?: ImageView[]
  imagesNotice?: string | null
}

/** Draws the composer for one chat with whatever it already holds. */
function draw(props: Props) {
  return render(Composer, { props })
}

afterEach(cleanup)

beforeEach(() => {
  resetTauriMock()
  setHandler('dispatch_intent', () => viewState())
  setHandler('attach_images', () => viewState())
  setHandler('remove_image', () => viewState())
})

// What a drop attaches and what the composer draws of it. The paths come from
// Tauri's own drop event: a browser `File` has no path, so the page that shows a
// thumbnail is never the page that reads the file.
describe('Composer images', () => {
  /** One Tauri drag and drop payload, as the webview reports it. */
  const at = { x: 10, y: 10 }

  it('attaches the paths of a drop, and nothing else', async () => {
    draw({ chat: 'chat-1' })
    await tick()

    dragEvent({ type: 'drop', paths: ['C:\\shots\\a.png', 'C:\\shots\\b.png'], position: at })
    await tick()

    expect(handlerArgs('attach_images')).toEqual([
      { paths: ['C:\\shots\\a.png', 'C:\\shots\\b.png'] },
    ])
  })

  it('draws the images the chat holds, each with a way to take it away', async () => {
    draw({ chat: 'chat-1', images: [image('img-1', 'a.png'), image('img-2', 'b.png')] })

    const first = screen.getByRole('img', { name: 'a.png' })
    expect(first.getAttribute('src')).toBe('http://gentle-image.localhost/img-1')

    await fireEvent.click(screen.getByRole('button', { name: `${REMOVE_IMAGE} a.png` }))

    expect(handlerArgs('remove_image')).toEqual([{ id: 'img-1' }])
    expect(handlerArgs('dispatch_intent')).toEqual([])
  })

  it('says why a dropped file was refused', () => {
    const notice = 'notes.txt was not attached: only png, jpg, jpeg, webp and gif images can be.'

    draw({ chat: 'chat-1', imagesNotice: notice })

    expect(screen.getByRole('status').textContent).toBe(notice)
  })

  it('says where the files go while they are dragged over the window', async () => {
    draw({ chat: 'chat-1' })
    await tick()
    expect(screen.queryByText(DROP_HINT)).toBeNull()

    dragEvent({ type: 'enter', paths: ['C:\\shots\\a.png'], position: at })
    await tick()
    expect(screen.getByText(DROP_HINT)).toBeTruthy()

    dragEvent({ type: 'leave' })
    await tick()
    expect(screen.queryByText(DROP_HINT)).toBeNull()
  })

  it('draws no image list and no refusal for a chat that dropped nothing', () => {
    // PRESERVE: a composer with no images is the one it always was.
    draw({ chat: 'chat-1' })

    expect(screen.queryByRole('img')).toBeNull()
    expect(screen.queryByRole('status')).toBeNull()
  })
})
