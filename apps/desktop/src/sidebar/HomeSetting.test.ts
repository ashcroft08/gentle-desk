// The footer control that changes the home later (shell-choice S5): two
// answers, the current one pressed. A choice only applies to a new run, so while
// a change is waiting the control says so and offers the restart. It is not drawn
// when there is nothing to choose between.

import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { cleanup, fireEvent, render, screen } from '@testing-library/svelte'
import type { ShellView_Serialize as ShellView } from '../contract/bindings'
import { handlerArgs, resetTauriMock, setHandler, viewState } from '../test-support/tauri-mock'
import HomeSetting from './HomeSetting.svelte'

vi.mock('@tauri-apps/api/core', () => import('../test-support/tauri-mock').then((m) => m.coreModule()))
vi.mock('@tauri-apps/api/event', () => import('../test-support/tauri-mock').then((m) => m.eventModule()))

const PI = 'C:/Users/someone/.pi/agent'

function shell(over: Partial<ShellView> = {}): ShellView {
  return {
    pi_path: PI,
    choice: null,
    ask: false,
    launcher_found: true,
    install: { type: 'Idle' },
    restart_needed: false,
    ...over,
  }
}

afterEach(cleanup)

beforeEach(() => {
  resetTauriMock()
  setHandler('dispatch_intent', () => viewState())
})

function intents(): unknown[] {
  return handlerArgs('dispatch_intent').map((args) => (args as { intent: unknown }).intent)
}

function pressed(name: string): string | null {
  return screen.getByRole('button', { name }).getAttribute('aria-pressed')
}

describe('the home setting', () => {
  it('is not drawn when there is no pi to choose instead', () => {
    const { container } = render(HomeSetting, { props: { shell: shell({ pi_path: null }) } })

    expect(container.querySelector('.home-setting')).toBeNull()
  })

  it('presses the separate home while nothing is chosen', () => {
    render(HomeSetting, { props: { shell: shell() } })

    expect(pressed('Separate')).toBe('true')
    expect(pressed('My pi')).toBe('false')
  })

  it('presses the chosen home', () => {
    render(HomeSetting, { props: { shell: shell({ choice: 'link' }) } })

    expect(pressed('My pi')).toBe('true')
    expect(pressed('Separate')).toBe('false')
  })

  it('sends the other home when it is pressed, once', async () => {
    render(HomeSetting, { props: { shell: shell({ choice: 'isolated' }) } })

    await fireEvent.click(screen.getByRole('button', { name: 'My pi' }))

    expect(intents()).toEqual([{ type: 'SetShellHome', data: { choice: 'link' } }])
  })

  it('sends nothing for the home that is already the one', async () => {
    render(HomeSetting, { props: { shell: shell({ choice: 'isolated' }) } })

    await fireEvent.click(screen.getByRole('button', { name: 'Separate' }))

    expect(intents()).toEqual([])
  })

  it('says a change waits for a restart and offers it, only then', async () => {
    const waiting = render(HomeSetting, {
      props: { shell: shell({ choice: 'link', restart_needed: true }) },
    })

    expect(screen.getByRole('status').textContent).toContain('Restart to use this home')
    await fireEvent.click(screen.getByRole('button', { name: 'Restart now' }))
    expect(intents()).toEqual([{ type: 'RestartApp' }])

    waiting.unmount()
    render(HomeSetting, { props: { shell: shell({ choice: 'link', restart_needed: false }) } })
    expect(screen.queryByRole('button', { name: 'Restart now' })).toBeNull()
  })
})
