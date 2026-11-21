// The Notifications section of the Settings dialog (notifications S4, S5; the
// approved design `design/notifications/`): ONE choice in the same segmented row
// Language uses - "when the window is not in front" (the default) or off - the
// explanation of what it does, and a "Try it" button that plays the tone and
// sends one toast, with the two real outcomes it can have.
//
// The words are quoted literally, in BOTH languages: they are the artboards'
// copy (01-pane, 02-try-denied, 03-off), so a change of copy is a change here.
// The pane is drawn on its own with the setting as a prop, the way the dialog
// hands it down; the dialog's own rail and keys are the other files' business.
import '../test-support/dom-env'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { cleanup, fireEvent, render, within } from '@testing-library/svelte'
import { tick } from 'svelte'
import type { NotificationsMode } from '../contract/bindings'
import { setLanguage } from '../i18n/language.svelte'
import { tryNotification } from '../notifications/try'
import { handlerArgs, resetTauriMock, setHandler } from '../test-support/tauri-mock'
import NotificationsPane from './NotificationsPane.svelte'

vi.mock('@tauri-apps/api/core', () => import('../test-support/tauri-mock').then((m) => m.coreModule()))
vi.mock('@tauri-apps/api/event', () => import('../test-support/tauri-mock').then((m) => m.eventModule()))
// The tone and the toast are T3's (`../notifications/try`): this file is about
// what the pane does with the two answers it can give.
vi.mock('../notifications/try', () => ({ tryNotification: vi.fn() }))

/** The English words, exactly as the artboards draw them. */
const EN = {
  group: 'When to notify',
  whenUnfocused: 'When the window is not in front',
  off: 'Off',
  explain:
    'A short tone and a system notification when a reply finishes, when the agent needs you or when something fails. They only happen while the window is not in front: with the window in front, the app stays silent.',
  tryIt: 'Try it',
  sentTag: 'Sent',
  sentLine: 'The tone played and the notification was sent.',
  deniedTag: 'No notification',
  deniedLine: 'The tone played, but Windows did not allow the notification.',
  deniedNote:
    'Allow notifications for Gentle Shell Desktop in Windows Settings \u203a System \u203a Notifications, then try again.',
  offReason:
    'There is nothing to try while notifications are off. Choose \u201cWhen the window is not in front\u201d to hear the tone and see the notification.',
}

/** The Spanish words, exactly as the artboards draw them. */
const ES = {
  group: 'Cuándo avisar',
  whenUnfocused: 'Cuando la ventana no está al frente',
  off: 'Desactivado',
  explain:
    'Un tono breve y un aviso del sistema cuando una respuesta termina, cuando el agente te necesita o cuando algo falla. Sólo ocurren mientras la ventana no está al frente: con la ventana al frente, la app se queda en silencio.',
  tryIt: 'Probar',
  sentTag: 'Enviado',
  sentLine: 'El tono sonó y el aviso se envió.',
  deniedTag: 'Sin aviso',
  deniedLine: 'El tono sonó, pero Windows no permitió el aviso.',
  deniedNote:
    'Habilita las notificaciones de Gentle Shell Desktop en Ajustes de Windows \u203a Sistema \u203a Notificaciones y vuelve a probar.',
  offReason:
    'Con los avisos desactivados no hay nada que probar. Elige \u00abCuando la ventana no está al frente\u00bb para escuchar el tono y ver el aviso.',
}

/** The tone and the toast, as the mocked module answers. */
const tone = vi.mocked(tryNotification)

// The testing library only cleans up on its own with vitest globals on.
afterEach(cleanup)

beforeEach(() => {
  resetTauriMock()
  setLanguage('en')
  tone.mockReset()
  tone.mockResolvedValue('sent')
  // The bridge answers the intent the pane sends: the dialog's own command.
  setHandler('dispatch_intent', () => ({}))
})

/** The pane, drawn with the setting it is given. */
function pane(notifications: NotificationsMode = 'when-unfocused'): HTMLElement {
  return render(NotificationsPane, { props: { notifications } }).container
}

/** The two choices, in the order the row draws them. */
function choices(panel: HTMLElement): HTMLElement[] {
  return within(panel).getAllByRole('radio')
}

/** The Try it button. */
function tryButton(panel: HTMLElement): HTMLElement {
  const labels = [EN.tryIt, ES.tryIt]
  const found = labels
    .map((label) => within(panel).queryByRole('button', { name: label }))
    .filter((button): button is HTMLElement => button !== null)
  if (found.length === 0) throw new Error('the pane drew no Try it button')
  return found[0]
}

/** The `SetNotifications` modes the bridge received, oldest first. */
function modes(): NotificationsMode[] {
  return handlerArgs('dispatch_intent')
    .map((args) => args.intent as { type: string; data?: { mode: NotificationsMode } })
    .filter((intent) => intent.type === 'SetNotifications')
    .map((intent) => intent.data!.mode)
}

/**
 * The intents the PANE sent, oldest first. `beforeEach` sets the language before
 * drawing anything, and that call is the harness's, not the pane's - the same
 * filter `SettingsSearch.test.ts` uses.
 */
function sent(): unknown[] {
  return handlerArgs('dispatch_intent')
    .map((args) => args.intent)
    .filter((intent) => (intent as { type: string }).type !== 'SetLanguage')
}

describe('the Notifications pane', () => {
  it('draws the two choices with the mode the app holds selected', () => {
    const panel = pane('when-unfocused')

    expect(choices(panel).map((choice) => choice.textContent?.trim())).toEqual([
      EN.whenUnfocused,
      EN.off,
    ])
    expect(
      choices(panel).map((choice) => choice.getAttribute('aria-checked')),
    ).toEqual(['true', 'false'])
    expect(within(panel).getByRole('radiogroup', { name: EN.group })).toBeTruthy()
    expect(within(panel).getByText(EN.explain)).toBeTruthy()
  })

  it('marks off when the app holds off, and keeps the explanation readable', () => {
    const panel = pane('off')

    expect(
      choices(panel).map((choice) => choice.getAttribute('aria-checked')),
    ).toEqual(['false', 'true'])
    // Turning it off does not hide what was turned off (03-off).
    expect(within(panel).getByText(EN.explain)).toBeTruthy()
  })

  it('sends one SetNotifications with the chosen mode and nothing else', async () => {
    const panel = pane('when-unfocused')

    await fireEvent.click(choices(panel)[1])

    expect(modes()).toEqual(['off'])
    expect(sent()).toEqual([{ type: 'SetNotifications', data: { mode: 'off' } }])
  })

  it('sends nothing when the mode it already holds is chosen again', async () => {
    const panel = pane('when-unfocused')

    await fireEvent.click(choices(panel)[0])

    expect(sent()).toEqual([])
  })

  it('tries once and says the tone played and the notification was sent', async () => {
    const panel = pane('when-unfocused')

    await fireEvent.click(tryButton(panel))
    await tick()

    expect(tone).toHaveBeenCalledTimes(1)
    // The tone is synthesized in the language the app's own words are in.
    expect(tone).toHaveBeenCalledWith('en')
    const status = panel.querySelector('[role="status"]')
    expect(status?.textContent).toContain(EN.sentTag)
    expect(status?.textContent).toContain(EN.sentLine)
    // Nothing was persisted: trying is a test, not a setting.
    expect(sent()).toEqual([])
  })

  it('says the tone played and Windows denied the toast, with the way to allow it', async () => {
    tone.mockResolvedValue('denied')
    const panel = pane('when-unfocused')

    await fireEvent.click(tryButton(panel))
    await tick()

    const status = panel.querySelector('[role="status"]')
    expect(status?.textContent).toContain(EN.deniedTag)
    expect(status?.textContent).toContain(EN.deniedLine)
    expect(within(panel).getByText(EN.deniedNote)).toBeTruthy()
    expect(within(panel).queryByText(EN.sentLine)).toBeNull()
  })

  it('draws no result line before the first try', () => {
    const panel = pane('when-unfocused')

    expect(panel.querySelector('[role="status"]')).toBeNull()
    expect(within(panel).queryByText(EN.deniedNote)).toBeNull()
  })

  it('clears the result when the choice changes', async () => {
    tone.mockResolvedValue('denied')
    const panel = pane('when-unfocused')
    await fireEvent.click(tryButton(panel))
    await tick()
    expect(panel.querySelector('[role="status"]')).not.toBeNull()

    await fireEvent.click(choices(panel)[1])
    await tick()

    expect(panel.querySelector('[role="status"]')).toBeNull()
  })

  it('keeps Try it reachable but dead while notifications are off, with its reason', async () => {
    const panel = pane('off')
    const button = tryButton(panel)

    // `aria-disabled`, not `disabled`: the reason under it stays reachable from
    // the keyboard (03-off).
    expect(button.getAttribute('aria-disabled')).toBe('true')
    expect(button.hasAttribute('disabled')).toBe(false)
    expect(within(panel).getByText(EN.offReason)).toBeTruthy()

    await fireEvent.click(button)
    await tick()

    expect(tone).not.toHaveBeenCalled()
    expect(sent()).toEqual([])
    expect(panel.querySelector('[role="status"]')).toBeNull()
  })

  it('draws every word in Spanish when the app holds Spanish', async () => {
    setLanguage('es')
    const panel = pane('when-unfocused')
    await tick()

    expect(choices(panel).map((choice) => choice.textContent?.trim())).toEqual([
      ES.whenUnfocused,
      ES.off,
    ])
    expect(within(panel).getByRole('radiogroup', { name: ES.group })).toBeTruthy()
    expect(within(panel).getByText(ES.explain)).toBeTruthy()
    expect(tryButton(panel).textContent?.trim()).toBe(ES.tryIt)

    await fireEvent.click(tryButton(panel))
    await tick()
    expect(tone).toHaveBeenCalledWith('es')
    const status = panel.querySelector('[role="status"]')
    expect(status?.textContent).toContain(ES.sentTag)
    expect(status?.textContent).toContain(ES.sentLine)
  })

  it('draws the denied result and the off reason in Spanish too', async () => {
    setLanguage('es')
    tone.mockResolvedValue('denied')
    const denied = pane('when-unfocused')
    await tick()

    await fireEvent.click(tryButton(denied))
    await tick()

    expect(denied.querySelector('[role="status"]')?.textContent).toContain(ES.deniedTag)
    expect(denied.querySelector('[role="status"]')?.textContent).toContain(ES.deniedLine)
    expect(within(denied).getByText(ES.deniedNote)).toBeTruthy()

    cleanup()
    const off = pane('off')
    await tick()
    expect(within(off).getByText(ES.offReason)).toBeTruthy()
  })
})
