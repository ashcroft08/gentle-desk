// The Notifications pane's own words (notifications S4, S5; the approved design
// `design/notifications/`): the one choice and its two answers, the explanation of
// what the app does while the window is not in front, the one action and the two
// results it can draw, plus the reason Try it is dead while the setting is off.
//
// This file pins the TWO columns against the artboards' text, line by line, and
// the one rule every language-keyed dictionary here has: nothing is left in
// English in the Spanish column. What a DRAWN pane shows is
// `NotificationsPane.test.ts`'s.
//
// The one line the artboards draw only in Spanish (the explanation) is pinned in
// English too: the English column is a translation of it and says the same thing,
// nothing more.
import { describe, expect, it } from 'vitest'
import { notificationsCopy } from './notifications-copy'

/** English: the artboards' English column. */
const ENGLISH = {
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

/** Neutral Spanish, quoted from the artboards: accents and guillemets kept. */
const SPANISH = {
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

/** The plain words of one language: every entry is a string. */
const WORDS = Object.keys(ENGLISH) as (keyof typeof ENGLISH)[]

describe('the notifications copy', () => {
  it('draws the artboards’ English words', () => {
    expect(notificationsCopy('en')).toEqual(ENGLISH)
  })

  it('draws the artboards’ Spanish words', () => {
    expect(notificationsCopy('es')).toEqual(SPANISH)
  })

  it('translates every word: nothing is left in English', () => {
    const english = notificationsCopy('en')
    const spanish = notificationsCopy('es')

    expect(Object.keys(spanish).sort()).toEqual(Object.keys(english).sort())
    for (const word of WORDS) {
      expect(spanish[word], word).not.toBe(english[word])
    }
  })
})
