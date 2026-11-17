// The thread bar's stop square (helper-stop T4, 01b's `#h-stop`): one test per
// rule. It sits at the right of the bar after the helper's name, it is enabled
// only while that helper is RUNNING and stays a real disabled button with the
// "Stopped" tip and the helper's own name otherwise, it sends this thread's id
// once, and it reaches the keyboard as an ordinary button.

import '../test-support/dom-env'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { cleanup, fireEvent, render, screen, within } from '@testing-library/svelte'
import type { HelperThreadView as HelperThreadModel } from '../contract/bindings'
import { helperThread } from '../test-support/helper-thread'
import { resetTauriMock, setHandler, viewState } from '../test-support/tauri-mock'
import HelperThread from './HelperThread.svelte'
import { helpersCopy, threadHeading } from './copy'

vi.mock('@tauri-apps/api/core', () => import('../test-support/tauri-mock').then((m) => m.coreModule()))
vi.mock('@tauri-apps/api/event', () => import('../test-support/tauri-mock').then((m) => m.eventModule()))

/** The thread's own words, as it drew them before 0.2.0 (English). */
const copy = helpersCopy('en')

/** Draws one thread and answers its region plus the Back and Stop spies. */
function draw(thread: HelperThreadModel = helperThread()) {
  const onBack = vi.fn()
  const onStop = vi.fn()
  render(HelperThread, { props: { thread, onBack, onStop } })
  const region = screen.getByRole('region', { name: copy.threadLabel })
  /** The bar's controls, in the order the bar draws them. */
  const controls = within(region.querySelector('.bar') as HTMLElement).getAllByRole('button')
  return { onBack, onStop, region, controls }
}

afterEach(cleanup)

beforeEach(() => {
  resetTauriMock()
  setHandler('dispatch_intent', () => viewState())
  setHandler('open_link', () => null)
})

describe("the thread bar's stop square", () => {
  it('sits at the right of the bar, after the helper name, named for the helper', () => {
    const thread = helperThread({ agent: 'explore', title: 'Inventory of sidebar modules' })
    const { controls } = draw(thread)

    expect(controls).toHaveLength(2)
    expect(controls[0].getAttribute('aria-label')).toBe(copy.back)
    expect(controls[1].getAttribute('aria-label')).toBe(
      copy.stopNamed(threadHeading(thread.agent, thread.title)),
    )
    expect(controls[1].getAttribute('data-tip')).toBe(copy.stop)
  })

  it('is enabled while the helper is running, and stops it once when pressed', async () => {
    const { region, onStop } = draw(helperThread({ id: 'h1', status: 'Running' }))

    const stop = within(region).getByRole('button', {
      name: copy.stopNamed(threadHeading('explore', 'Inventory of sidebar modules')),
    })
    expect((stop as HTMLButtonElement).disabled).toBe(false)

    await fireEvent.click(stop)

    expect(onStop).toHaveBeenCalledTimes(1)
    expect(onStop).toHaveBeenCalledWith('h1')
  })

  it.each(['Waiting', 'Queued', 'Done', 'Failed', 'Cancelled'] as const)(
    'is disabled with the Stopped tip once the helper is %s',
    (status) => {
      const thread = helperThread({ id: 'h1', status, status_label: status.toLowerCase() })
      const { region } = draw(thread)
      const heading = threadHeading(thread.agent, thread.title)

      const stop = within(region).getByRole('button', { name: copy.alreadyStopped(heading) })
      // A real `disabled` button, exactly as 01b draws it: the name and the tip
      // say the helper's name and that there is nothing left to stop. (No click
      // is simulated: a browser cannot activate a disabled button, while jsdom
      // delivers a programmatically dispatched event regardless.)
      expect((stop as HTMLButtonElement).disabled).toBe(true)
      expect(stop.getAttribute('data-tip')).toBe(copy.stopped)
    },
  )

  it('reaches the keyboard as an ordinary button in the tab order', () => {
    const { controls } = draw()

    const stop = controls[1] as HTMLButtonElement
    expect(stop.tagName).toBe('BUTTON')
    expect(stop.getAttribute('tabindex')).toBeNull()
    stop.focus()
    expect(document.activeElement).toBe(stop)
  })
})
