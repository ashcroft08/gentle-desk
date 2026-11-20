// The composer's file attachments (design/composer-dock T4, S3, R5, R6): the
// clip and a drop name PATHS, Rust says what each one is, an image still travels
// as an image, every other file becomes a chip, a folder or a missing file is one
// error line, a file outside the chat's folder attaches anyway and says so, and
// the send carries the chips' absolute paths below the text.
//
// Nothing here ever sends on an attach: the only intent these tests allow is the
// one a send itself dispatches.

import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { cleanup, fireEvent, render, screen } from '@testing-library/svelte'
import { tick } from 'svelte'
import type { AttachmentView, ChatActivityMark, FolderView } from '../contract/bindings'
import { handlerArgs, resetTauriMock, setHandler, viewState } from '../test-support/tauri-mock'
import { dragEvent } from '../test-support/webview'
import Composer from './Composer.svelte'
import { composerCopy } from './copy'

vi.mock('@tauri-apps/api/core', () => import('../test-support/tauri-mock').then((m) => m.coreModule()))
vi.mock('@tauri-apps/api/event', () => import('../test-support/tauri-mock').then((m) => m.eventModule()))
vi.mock('@tauri-apps/api/webview', () => import('../test-support/webview').then((m) => m.webviewModule()))

const copy = composerCopy('en')

/** One attachment as `describe_attachments` answers it. */
function attachment(
  path: string,
  kind: AttachmentView['kind'] = { type: 'File' },
  outside = false,
): AttachmentView {
  return { path, name: path.split('\\').pop() ?? path, kind, outside }
}

const FILE = attachment('C:\\dev\\repo\\src\\header.tsx')
const SECOND = attachment('C:\\dev\\repo\\docs\\plan.md')
const IMAGE = attachment('C:\\dev\\repo\\shots\\a.png', { type: 'Image' })
const FOLDER = attachment('C:\\dev\\repo\\assets', { type: 'Rejected', data: 'IsDirectory' })
const FOLDER_VIEW: FolderView = { name: 'repo', path: 'C:\\dev\\repo' }

type Props = {
  chat: string | null
  mark?: ChatActivityMark | null
  folder?: FolderView | null
}

/** The views the mocked command answers, per test. */
let described: AttachmentView[] = []

/** Draws the composer for one chat. */
function draw(props: Props) {
  return render(Composer, { props })
}

/** Lets every promise of an attach settle, then redraws. */
async function flush(): Promise<void> {
  await Promise.resolve()
  await Promise.resolve()
  await Promise.resolve()
  await tick()
}

/** The clip: the dialog answers `paths` and the command says what they are. */
async function choose(paths: string[], views: AttachmentView[]): Promise<void> {
  described = views
  setHandler('pick_files', () => paths)
  await fireEvent.click(screen.getByRole('button', { name: copy.attachFiles }))
  await flush()
}

/** One Enter as the browser reports it, on the box the name reaches. */
function enter(box: HTMLElement): KeyboardEvent {
  const event = new KeyboardEvent('keydown', { key: 'Enter', bubbles: true, cancelable: true })
  box.dispatchEvent(event)
  return event
}

/** The intents the composer dispatched, oldest first. */
function intents(): unknown[] {
  return handlerArgs('dispatch_intent').map((args) => args.intent)
}

afterEach(cleanup)

beforeEach(() => {
  resetTauriMock()
  described = []
  setHandler('dispatch_intent', () => viewState())
  setHandler('attach_images', () => viewState())
  setHandler('remove_image', () => viewState())
  setHandler('describe_attachments', () => described)
  setHandler('pick_files', () => [])
})

describe('the clip', () => {
  it('turns a picked file into a chip, and sends nothing', async () => {
    draw({ chat: 'chat-1' })

    await choose([FILE.path], [FILE])

    // The dialog was asked in the composer's own words.
    expect(handlerArgs('pick_files')).toEqual([{ title: copy.attachFiles }])
    expect(screen.getByRole('listitem').textContent).toContain(FILE.name)
    expect(handlerArgs('attach_images')).toEqual([])
    expect(intents()).toEqual([])
  })

  it('gives the chip the whole path as its hover, never as its text', async () => {
    draw({ chat: 'chat-1' })

    await choose([FILE.path], [FILE])

    expect(screen.getByRole('listitem').getAttribute('title')).toBe(FILE.path)
    expect(screen.getByRole('listitem').textContent).not.toContain('C:\\')
  })

  it('still hands an image path to the image command', async () => {
    draw({ chat: 'chat-1' })

    await choose([IMAGE.path], [IMAGE])

    // R6: an image travels as an image, so no chip stands in for it.
    expect(handlerArgs('attach_images')).toEqual([{ paths: [IMAGE.path] }])
    expect(screen.queryAllByRole('listitem')).toHaveLength(0)
    expect(intents()).toEqual([])
  })

  it('says why a folder was not attached, and draws no chip for it', async () => {
    draw({ chat: 'chat-1' })

    await choose([FOLDER.path], [FOLDER])

    expect(screen.getByRole('alert').textContent).toBe(
      copy.attachFailed('assets', 'IsDirectory'),
    )
    expect(screen.queryAllByRole('listitem')).toHaveLength(0)
  })

  it('changes nothing when the dialog is cancelled', async () => {
    draw({ chat: 'chat-1' })

    await choose([], [])

    expect(handlerArgs('describe_attachments')).toEqual([])
    expect(screen.queryAllByRole('listitem')).toHaveLength(0)
    expect(screen.queryByRole('alert')).toBeNull()
    expect(intents()).toEqual([])
  })
})

describe('a drop', () => {
  it('does both of what a mixed drop is: the image attaches, the file becomes a chip', async () => {
    const at = { x: 10, y: 10 }
    draw({ chat: 'chat-1' })
    described = [IMAGE, FILE]
    await tick()

    dragEvent({ type: 'drop', paths: [IMAGE.path, FILE.path], position: at })
    await flush()

    expect(handlerArgs('attach_images')).toEqual([{ paths: [IMAGE.path] }])
    expect(screen.getByRole('listitem').textContent).toContain(FILE.name)
    expect(intents()).toEqual([])
  })

  it('asks what a dropped path is with the chat folder it must compare against', async () => {
    const at = { x: 10, y: 10 }
    draw({ chat: 'chat-1', folder: FOLDER_VIEW })
    described = [FILE]

    dragEvent({ type: 'drop', paths: [FILE.path], position: at })
    await flush()

    expect(handlerArgs('describe_attachments')).toEqual([
      { paths: [FILE.path], folder: FOLDER_VIEW.path },
    ])
  })
})

describe('a file outside the chat folder', () => {
  const AWAY = attachment('C:\\Users\\josep\\Desktop\\notas.md', { type: 'File' }, true)

  it('attaches anyway, says so on the chip and in ONE line under the chips', async () => {
    draw({ chat: 'chat-1', folder: FOLDER_VIEW })

    await choose([AWAY.path], [AWAY])

    expect(screen.getByRole('listitem').textContent).toContain(copy.outsideWorkspace)
    expect(screen.getByRole('status').textContent).toBe(copy.outsideWarning)
    // R5: nothing is blocked.
    expect(screen.queryAllByRole('listitem')).toHaveLength(1)
    expect(intents()).toEqual([])
  })
})

describe('the chip row', () => {
  it('takes back the chip the user removes, and sends nothing', async () => {
    draw({ chat: 'chat-1' })
    await choose([FILE.path, SECOND.path], [FILE, SECOND])

    await fireEvent.click(screen.getByRole('button', { name: `${copy.removeFile} ${FILE.name}` }))
    await flush()

    expect(screen.queryAllByRole('listitem')).toHaveLength(1)
    expect(screen.getByRole('listitem').textContent).toContain(SECOND.name)
    expect(intents()).toEqual([])
  })

  it('shows the first three and expands to the whole list', async () => {
    const more = [
      FILE,
      SECOND,
      attachment('C:\\dev\\repo\\routes.ts'),
      attachment('C:\\dev\\repo\\footer.tsx'),
    ]
    draw({ chat: 'chat-1' })
    await choose(more.map((view) => view.path), more)

    expect(screen.getAllByRole('listitem')).toHaveLength(3)
    const expand = screen.getByRole('button', { name: copy.showMoreLabel(1) })
    expect(expand.textContent).toBe(copy.showMore(1))
    expect(expand.getAttribute('aria-expanded')).toBe('false')

    await fireEvent.click(expand)

    expect(screen.getAllByRole('listitem')).toHaveLength(4)
    expect(
      screen.getByRole('button', { name: copy.showMoreLabel(1) }).getAttribute('aria-expanded'),
    ).toBe('true')
  })
})

describe('a send with chips', () => {
  it('appends exactly the text and then one path per line', async () => {
    draw({ chat: 'chat-1' })
    const box = screen.getByRole('textbox', { name: copy.messageLabel }) as HTMLTextAreaElement
    await choose([FILE.path, SECOND.path], [FILE, SECOND])

    await fireEvent.input(box, { target: { value: 'review these' } })
    enter(box)
    await flush()

    expect(intents()).toEqual([
      {
        type: 'SendPrompt',
        data: { text: `review these\n${FILE.path}\n${SECOND.path}` },
      },
    ])
  })

  it('sends the paths alone when the box is empty, and is allowed to', async () => {
    draw({ chat: 'chat-1' })
    const box = screen.getByRole('textbox', { name: copy.messageLabel }) as HTMLTextAreaElement
    await choose([FILE.path], [FILE])

    enter(box)
    await flush()

    expect(intents()).toEqual([{ type: 'SendPrompt', data: { text: FILE.path } }])
  })

  it('leaves no chip behind', async () => {
    draw({ chat: 'chat-1' })
    const box = screen.getByRole('textbox', { name: copy.messageLabel }) as HTMLTextAreaElement
    await choose([FILE.path], [FILE])

    enter(box)
    await flush()

    expect(screen.queryAllByRole('listitem')).toHaveLength(0)
    expect(box.value).toBe('')
  })

  it('drops the chips when the chat changes, before anything is sent', async () => {
    const drawn = draw({ chat: 'chat-1' })
    await choose([FILE.path], [FILE])

    await drawn.rerender({ chat: 'chat-2' })
    await flush()

    expect(screen.queryAllByRole('listitem')).toHaveLength(0)
    expect(intents()).toEqual([])
  })
})
