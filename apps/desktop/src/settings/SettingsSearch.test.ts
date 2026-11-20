// The field above the Settings rail (the reference's "Find a setting…" box,
// crates/app/src/features/settings/navigation.rs:105-145): it filters the entries
// the rail lists by the words each one already carries - its title, its hint and
// its keywords - and it changes nothing else. No index, no debounce, no read of
// the machine: nine strings checked per keystroke.
//
// What this file pins is the behaviour the user sees: the entries that stay, the
// words a query may reach them by (including the OTHER language's, so a Spanish
// query finds a pane while the app is in English), the one quiet line when
// nothing is left, and what a pick and an Escape do. The words themselves are
// `copy.test.ts`'s and the keyword lists are data in `sections.ts`; how the match
// is computed is not this file's business.
import '../test-support/dom-env'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { cleanup, fireEvent, within } from '@testing-library/svelte'
import { tick } from 'svelte'
import { intents, openSettings, pane, pick } from '../test-support/settings'
import { resetTauriMock } from '../test-support/tauri-mock'
import { setLanguage } from '../i18n/language.svelte'

vi.mock('@tauri-apps/api/core', () => import('../test-support/tauri-mock').then((m) => m.coreModule()))
vi.mock('@tauri-apps/api/event', () => import('../test-support/tauri-mock').then((m) => m.eventModule()))

/** The field's accessible name, and the one quiet line, quoted per language. */
const FIND = 'Find a setting'
const FIND_ES = 'Buscar un ajuste'
const NONE = 'No settings match.'
const NONE_ES = 'Ningún ajuste coincide.'

/** The eight entries in the order the dialog draws them, per language. */
const RAIL_EN = [
  'Appearance',
  'Shell home',
  'Updates',
  'Account',
  'Usage',
  'Language',
  'Notifications',
  'About',
]
const RAIL_ES = [
  'Apariencia',
  'Home del shell',
  'Actualizaciones',
  'Cuenta',
  'Uso',
  'Idioma',
  'Notificaciones',
  'Acerca de',
]

// The testing library only cleans up on its own with vitest globals on.
afterEach(cleanup)

beforeEach(() => {
  resetTauriMock()
  setLanguage('en')
})

/**
 * The entries the rail draws right now, in order. Read with the QUERY form, so a
 * field that leaves nothing to draw is an empty list and not a thrown lookup.
 */
function names(dialog: HTMLElement): string[] {
  return within(dialog)
    .queryAllByRole('tab')
    .map((entry) => entry.textContent?.trim() ?? '')
}

/**
 * The intents the DIALOG sent, oldest first. This file sets the language before
 * it opens anything (so the words are the same on every machine), and that call
 * is the harness's, not the dialog's.
 */
function sent(): unknown[] {
  return intents().filter((intent) => (intent as { type: string }).type !== 'SetLanguage')
}

/** The search field, by the name it announces. */
function field(dialog: HTMLElement): HTMLElement {
  return within(dialog).getByRole('searchbox')
}

/** Types `text` into the field the way the user does. */
async function search(dialog: HTMLElement, text: string): Promise<void> {
  await fireEvent.input(field(dialog), { target: { value: text } })
}

describe('the Settings search field', () => {
  it('draws every entry while the field is empty', async () => {
    const dialog = await openSettings()

    expect(names(dialog)).toEqual(RAIL_EN)
    // The field says what it does, and its name is the whole instruction.
    expect(within(dialog).getByRole('searchbox', { name: FIND })).toBeTruthy()

    await search(dialog, 'account')
    expect(names(dialog)).toEqual(['Account'])

    // An emptied field is the dialog as it was: every entry, nothing hidden.
    await search(dialog, '')
    expect(names(dialog)).toEqual(RAIL_EN)
    expect(within(dialog).queryByText(NONE)).toBeNull()
  })

  it("filters the rail by a section's own word and by the hint it draws", async () => {
    const dialog = await openSettings()

    await search(dialog, 'account')
    expect(names(dialog)).toEqual(['Account'])

    // "newer" is a word of the Updates hint and of no entry's name.
    await search(dialog, 'newer')
    expect(names(dialog)).toEqual(['Updates'])
  })

  it('reaches a pane by the words of the other language, either way round', async () => {
    const dialog = await openSettings()

    // English drawn, Spanish query: the keyword list carries both columns.
    await search(dialog, 'actualizaciones')
    expect(names(dialog)).toEqual(['Updates'])

    setLanguage('es')
    await tick()
    await search(dialog, '')
    expect(names(dialog)).toEqual(RAIL_ES)
    await search(dialog, 'account')
    expect(names(dialog)).toEqual(['Cuenta'])
    expect(within(dialog).getByRole('searchbox').getAttribute('aria-label')).toBe(FIND_ES)
  })

  it('ignores case and accents', async () => {
    const dialog = await openSettings()

    await search(dialog, 'ACCOUNT')
    expect(names(dialog)).toEqual(['Account'])

    setLanguage('es')
    await tick()

    // 'máquina', typed without its accent: the Home hint names the machine and
    // the Updates hint says what runs on it, so both are answers.
    await search(dialog, 'MAQUINA')
    expect(names(dialog)).toEqual(['Home del shell', 'Actualizaciones'])
    expect(within(dialog).getByRole('searchbox').getAttribute('aria-label')).toBe(FIND_ES)
  })

  it('needs every term of the query to land on the same entry', async () => {
    const dialog = await openSettings()

    // Both words are Language's: its name and its hint.
    await search(dialog, 'language words')
    expect(names(dialog)).toEqual(['Language'])

    // One word from Language, one from Account: no single entry holds both.
    await search(dialog, 'language account')
    expect(names(dialog)).toEqual([])
  })

  it('finds the Notifications section by the words of its own sound, either way round', async () => {
    const dialog = await openSettings()

    // English drawn, Spanish query: 'sonido' is the keyword both columns carry.
    await search(dialog, 'sonido')
    expect(names(dialog)).toEqual(['Notifications'])

    setLanguage('es')
    await tick()

    // Spanish drawn, English query, the way the field is meant to work.
    await search(dialog, 'sound')
    expect(names(dialog)).toEqual(['Notificaciones'])

    setLanguage('en')
  })

  it('says so in one quiet line when nothing is left, sending nothing', async () => {
    const dialog = await openSettings()

    await search(dialog, 'nothing-matches-this')
    expect(names(dialog)).toEqual([])
    expect(within(dialog).getByText(NONE)).toBeTruthy()
    // Searching is reading: the dialog asked the shell for nothing.
    expect(sent()).toEqual([])

    setLanguage('es')
    await tick()
    expect(within(dialog).getByText(NONE_ES)).toBeTruthy()
  })

  it('opens the pane of the result the user picks', async () => {
    const dialog = await openSettings()

    await search(dialog, 'usage')
    await fireEvent.click(within(dialog).getByRole('tab', { name: 'Usage' }))

    expect(within(pane(dialog)).getByRole('heading', { level: 3 }).textContent).toBe('Usage')
    expect(within(dialog).getByRole('tab', { name: 'Usage' }).getAttribute('aria-selected')).toBe(
      'true',
    )
  })

  it('keeps the open pane when the query stops matching it', async () => {
    const dialog = await openSettings()
    await pick(dialog, 'Account')
    expect(within(pane(dialog)).getByText('Which providers are signed in, and where to sign in or out.')).toBeTruthy()

    // The query leaves the rail with nothing, and the pane the user opened is
    // still the pane the user sees: searching never moves the section.
    await search(dialog, 'zzz')
    expect(names(dialog)).toEqual([])
    expect(
      within(pane(dialog)).getByText('Which providers are signed in, and where to sign in or out.'),
    ).toBeTruthy()
  })

  it('clears the field with Escape and leaves the dialog open', async () => {
    const dialog = await openSettings()
    await search(dialog, 'account')
    expect(names(dialog)).toEqual(['Account'])

    await fireEvent.keyDown(field(dialog), { key: 'Escape' })

    expect(names(dialog)).toEqual(RAIL_EN)
    expect(field(dialog).getAttribute('value')).toBeNull()
    expect((field(dialog) as HTMLInputElement).value).toBe('')
    expect(dialog.isConnected).toBe(true)
    expect(sent()).toEqual([])
  })

  it('walks the entries the field leaves, and wraps at the ends of that list', async () => {
    const dialog = await openSettings()

    // Two entries hold the word "version": Updates and About, in rail order.
    await search(dialog, 'version')
    expect(names(dialog)).toEqual(['Updates', 'About'])

    await fireEvent.keyDown(within(dialog).getByRole('tab', { name: 'Updates' }), {
      key: 'ArrowDown',
    })
    expect(within(dialog).getByRole('tab', { name: 'About' }).getAttribute('aria-selected')).toBe(
      'true',
    )

    // The rail wraps inside the two it is drawing, not across the eight.
    await fireEvent.keyDown(within(dialog).getByRole('tab', { name: 'About' }), {
      key: 'ArrowDown',
    })
    expect(within(dialog).getByRole('tab', { name: 'Updates' }).getAttribute('aria-selected')).toBe(
      'true',
    )
  })

  it('keeps the selected entry in the rail while it still matches', async () => {
    const dialog = await openSettings()
    await pick(dialog, 'Account')
    const before = within(dialog).getByRole('tab', { name: 'Account' }).getAttribute('tabindex')

    await search(dialog, 'signed')
    expect(names(dialog)).toEqual(['Account'])
    expect(within(dialog).getByRole('tab', { name: 'Account' }).getAttribute('tabindex')).toBe(
      before,
    )
    expect(
      within(dialog).getByRole('tab', { name: 'Account' }).getAttribute('aria-selected'),
    ).toBe('true')
  })
})
