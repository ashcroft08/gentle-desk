// What the update line says, case by case. On Windows the updater closes the app
// by itself to hand over to the installer, so the line must say that BEFORE the
// button is pressed: it never promises a restart it cannot see happen.

import { afterEach, describe, expect, it, vi } from 'vitest'
import { cleanup, fireEvent, render, screen } from '@testing-library/svelte'
import UpdateNotice from './UpdateNotice.svelte'
import { available, downloading, failed, ready, stillIdle } from './state'

afterEach(cleanup)

const noop = () => {}

function draw(state: ReturnType<typeof stillIdle>, handlers: Partial<Record<'onInstall' | 'onRetry' | 'onDismiss', () => void>> = {}) {
  return render(UpdateNotice, {
    props: { state, onInstall: noop, onRetry: noop, onDismiss: noop, ...handlers },
  })
}

describe('the update line', () => {
  it('draws nothing while there is nothing to say', () => {
    const { container } = draw(stillIdle())

    expect(container.querySelector('.update')).toBeNull()
  })

  it('names the version and warns that the app will close while it installs', () => {
    draw(available('0.2.0'))

    const line = screen.getByRole('status').textContent ?? ''
    expect(line).toContain('0.2.0')
    expect(line).toContain('will close')
    expect(screen.getByRole('button', { name: 'Install update' })).toBeTruthy()
    expect(screen.queryByText(/restart/i)).toBeNull()
  })

  it('installs only when the button is pressed, and can be put away', async () => {
    const onInstall = vi.fn()
    const onDismiss = vi.fn()
    draw(available('0.2.0'), { onInstall, onDismiss })
    expect(onInstall).not.toHaveBeenCalled()

    await fireEvent.click(screen.getByRole('button', { name: 'Install update' }))
    await fireEvent.click(screen.getByRole('button', { name: 'Not now' }))

    expect(onInstall).toHaveBeenCalledTimes(1)
    expect(onDismiss).toHaveBeenCalledTimes(1)
  })

  it('shows the percentage while it downloads, and takes the buttons away', () => {
    draw(downloading(available('0.2.0'), 50, 100))

    expect(screen.getByRole('status').textContent).toContain('50%')
    expect(screen.queryByRole('button')).toBeNull()
  })

  it('says it is installed without promising a restart', () => {
    draw(ready(available('0.2.0')))

    const line = screen.getByRole('status').textContent ?? ''
    expect(line).toContain('0.2.0')
    expect(line).toContain('Open the app again')
  })

  it('says why it failed and offers to try again', async () => {
    const onRetry = vi.fn()
    draw(failed(available('0.2.0'), 'no network'), { onRetry })

    expect(screen.getByRole('status').textContent).toContain('no network')
    await fireEvent.click(screen.getByRole('button', { name: 'Try again' }))

    expect(onRetry).toHaveBeenCalledTimes(1)
  })
})
