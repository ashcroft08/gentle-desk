// A Chat | Helpers tablist DRAWN from its dictionary (0.2.0, S2, S3): one local
// language write redraws the tablist's name, both tabs and the working badge in
// the same tick - no push, no restart (S2) - while the mark's own name stays the
// count Rust pushed, never a word of ours (D7).
//
// The English words are asserted literally: they are what 01b draws. The two
// columns themselves are pinned in `copy.test.ts`.
import '../test-support/dom-env'
import { afterEach, describe, expect, it } from 'vitest'
import { cleanup, render, screen, within } from '@testing-library/svelte'
import { tick } from 'svelte'
import { setLanguage } from '../i18n/language.svelte'
import ViewTabs from './ViewTabs.svelte'

afterEach(cleanup)

describe('a drawn tablist', () => {
  it('flips the tablist name and both tabs in the same tick', async () => {
    setLanguage('en')
    render(ViewTabs, { props: { tab: 'chat', running: 0, onSelect: () => {} } })

    expect(screen.getByRole('tablist', { name: 'View' })).toBeTruthy()
    expect(screen.getByRole('tab', { name: 'Chat' })).toBeTruthy()
    expect(screen.getByRole('tab', { name: 'Helpers' })).toBeTruthy()

    setLanguage('es')
    await tick()

    const list = screen.getByRole('tablist', { name: 'Vista' })
    expect(within(list).getByRole('tab', { name: 'Chat' })).toBeTruthy()
    expect(within(list).getByRole('tab', { name: 'Ayudantes' })).toBeTruthy()

    setLanguage('en')
  })

  it('names the working badge in the language of the moment', async () => {
    setLanguage('en')
    render(ViewTabs, { props: { tab: 'helpers', running: 2, onSelect: () => {} } })

    expect(screen.getByRole('img', { name: '2 helpers working' })).toBeTruthy()

    setLanguage('es')
    await tick()

    expect(screen.getByRole('img', { name: '2 ayudantes trabajando' })).toBeTruthy()

    setLanguage('en')
  })
})
