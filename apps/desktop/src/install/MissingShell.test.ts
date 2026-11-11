// The screen for a machine with no gentle-shell (shell-choice S4). It explains
// that the app cannot work without it, says what the command installs and that
// where chats and sign-ins live is the choice after it, and asks for
// permission: nothing is installed until the user presses the install button,
// and the screen follows the install it started (running, done, failed) and
// offers to look again.

import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { cleanup, fireEvent, render, screen } from '@testing-library/svelte'
import type { InstallView } from '../contract/bindings'
import { handlerArgs, resetTauriMock, setHandler, viewState } from '../test-support/tauri-mock'
import MissingShell from './MissingShell.svelte'

vi.mock('@tauri-apps/api/core', () => import('../test-support/tauri-mock').then((m) => m.coreModule()))
vi.mock('@tauri-apps/api/event', () => import('../test-support/tauri-mock').then((m) => m.eventModule()))

afterEach(cleanup)

beforeEach(() => {
  resetTauriMock()
  setHandler('dispatch_intent', () => viewState())
})

function intents(): unknown[] {
  return handlerArgs('dispatch_intent').map((args) => (args as { intent: unknown }).intent)
}

function draw(install: InstallView = { type: 'Idle' }) {
  return render(MissingShell, { props: { install } })
}

describe('the missing gentle-shell screen', () => {
  it('says gentle-shell is required and shows the exact command it would run', () => {
    draw()

    expect(screen.getByRole('heading', { name: 'gentle-shell is not installed' })).toBeTruthy()
    expect(screen.getByText('npm install -g gentle-pi')).toBeTruthy()
  })

  it('says the command installs the program only, and how big it is', () => {
    draw()

    expect(
      screen.getByText(
        'This installs the gentle-shell program only, about 250 MB, in the npm global folder. It makes no place for chats or sign-ins.',
      ),
    ).toBeTruthy()
  })

  it('says where chats and sign-ins live is a separate choice, made next', () => {
    draw()

    expect(
      screen.getByText(
        'Where your chats and sign-ins live is a separate choice, made next: your own pi, or a space of its own.',
      ),
    ).toBeTruthy()
  })

  it('says it needs Node and npm, changes this computer and can take a few minutes', () => {
    draw()

    expect(
      screen.getByText(
        'The install changes this computer: it needs Node and npm, and it can take a few minutes.',
      ),
    ).toBeTruthy()
  })

  it('installs nothing until the user presses the button', () => {
    draw()

    expect(intents()).toEqual([])
  })

  it('asks for the install once, however many times the button is pressed', async () => {
    draw()
    const button = screen.getByRole('button', { name: 'Install gentle-shell' })

    await fireEvent.click(button)
    await fireEvent.click(button)

    await vi.waitFor(() => expect(intents()).toEqual([{ type: 'InstallShell' }]))
  })

  it('shows progress while it runs and takes both buttons away', () => {
    draw({ type: 'Running' })

    expect(screen.getByRole('status').textContent).toContain('Installing gentle-shell')
    for (const name of ['Install gentle-shell', 'Check again']) {
      expect((screen.getByRole('button', { name }) as HTMLButtonElement).disabled).toBe(true)
    }
  })

  it('says what went wrong and lets the user try again', () => {
    draw({ type: 'Failed', data: { message: 'the install ended with exit code 1\nnpm ERR! 404' } })

    expect(screen.getByRole('alert').textContent).toContain('npm ERR! 404')
    expect((screen.getByRole('button', { name: 'Install gentle-shell' }) as HTMLButtonElement).disabled).toBe(
      false,
    )
  })

  it('tells the user what to do when npm itself could not run', () => {
    draw({ type: 'Failed', data: { message: 'could not start npm.cmd: not found' } })

    expect(screen.getByRole('alert').textContent).toContain('install Node.js')
  })

  it('says it finished and offers to look again instead of installing again', () => {
    draw({ type: 'Done' })

    expect(screen.getByRole('status').textContent).toContain('gentle-shell was installed')
    expect(screen.queryByRole('button', { name: 'Install gentle-shell' })).toBeNull()
    expect(screen.getByRole('button', { name: 'Check again' })).toBeTruthy()
  })

  it('looks for gentle-shell again when asked', async () => {
    draw()

    await fireEvent.click(screen.getByRole('button', { name: 'Check again' }))

    await vi.waitFor(() => expect(intents()).toEqual([{ type: 'CheckShellAgain' }]))
  })
})

// The wait itself: npm prints nothing while it works, so the screen shows a bar
// that moves and the seconds it has waited. Neither is a percentage, because
// nobody here knows one.
describe('the wait while the install runs', () => {
  beforeEach(() => {
    vi.useFakeTimers()
  })

  afterEach(() => {
    vi.useRealTimers()
  })

  it('shows an indeterminate progress bar, with no value to read', () => {
    draw({ type: 'Running' })

    const bar = screen.getByRole('progressbar')
    expect(bar.getAttribute('aria-valuenow')).toBeNull()
    expect(bar.getAttribute('aria-valuetext')).toBeNull()
  })

  it('shows no progress bar when the install is not running', () => {
    const views: InstallView[] = [
      { type: 'Idle' },
      { type: 'Done' },
      { type: 'Failed', data: { message: 'the install ended with exit code 1' } },
    ]

    for (const view of views) {
      const { unmount } = draw(view)
      expect(screen.queryByRole('progressbar')).toBeNull()
      unmount()
    }
  })

  it('counts the seconds waited, so the screen is visibly alive', async () => {
    draw({ type: 'Running' })

    expect(screen.getByText('0 s elapsed.')).toBeTruthy()

    await vi.advanceTimersByTimeAsync(1000)
    expect(screen.getByText('1 s elapsed.')).toBeTruthy()

    await vi.advanceTimersByTimeAsync(71_000)
    expect(screen.getByText('1 min 12 s elapsed.')).toBeTruthy()
  })

  it('stops counting when the install leaves Running', async () => {
    const { rerender } = draw({ type: 'Running' })
    await vi.advanceTimersByTimeAsync(2000)

    await rerender({ install: { type: 'Done' } })
    expect(screen.queryByText(/elapsed\./)).toBeNull()
    await vi.advanceTimersByTimeAsync(5000)

    expect(vi.getTimerCount()).toBe(0)
    expect(screen.getByRole('status').textContent).toContain('gentle-shell was installed')
  })

  it('stops counting when the screen goes away', async () => {
    const { unmount } = draw({ type: 'Running' })
    await vi.advanceTimersByTimeAsync(1000)

    unmount()

    expect(vi.getTimerCount()).toBe(0)
  })
})
