// The updater's notice, as a state machine: what the app says when it finds a
// release, while it downloads it, and when something goes wrong. Pure, so none
// of this needs a window or a network.

import { describe, expect, it } from 'vitest'
import { available, dismissed, downloading, failed, ready, stillIdle } from './state'

describe('the update notice', () => {
  it('says nothing while there is no newer version', () => {
    expect(stillIdle()).toEqual({ kind: 'idle' })
    expect(stillIdle().kind).toBe('idle')
  })

  it('names the version it found', () => {
    expect(available('0.2.0')).toEqual({ kind: 'available', version: '0.2.0' })
  })

  it('reports how much it downloaded, with or without a total', () => {
    const started = downloading(available('0.2.0'))
    expect(started).toEqual({ kind: 'downloading', version: '0.2.0', done: 0, total: null })

    const half = downloading(started, 50, 100)
    expect(half).toMatchObject({ done: 50, total: 100 })

    const unknown = downloading(started, 50)
    expect(unknown).toMatchObject({ done: 50, total: null })
  })

  it('says it is ready and never claims that while still downloading', () => {
    expect(ready(available('0.2.0'))).toEqual({ kind: 'ready', version: '0.2.0' })
    expect(ready(downloading(available('0.2.0'))).kind).toBe('ready')
  })

  it('keeps the version when a download fails, so the retry knows what to fetch', () => {
    expect(failed(downloading(available('0.2.0')), 'no network')).toEqual({
      kind: 'failed',
      version: '0.2.0',
      message: 'no network',
    })
    expect(failed(stillIdle(), 'no network')).toEqual({
      kind: 'failed',
      version: null,
      message: 'no network',
    })
  })

  it('can be put away for this run, except while it is finishing', () => {
    expect(dismissed(available('0.2.0'))).toEqual({ kind: 'dismissed' })
    expect(dismissed(downloading(available('0.2.0'))).kind).toBe('downloading')
    expect(dismissed(ready(available('0.2.0'))).kind).toBe('ready')
  })
})
