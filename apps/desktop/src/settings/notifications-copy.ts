// The Notifications pane's words (notifications S4, S5): the section's one choice,
// the explanation of what the app does while the window is not in front, the one
// "Try it" action and the two real answers it can give - the toast went out, or
// Windows did not allow it and the pane says how to allow it.
//
// They come from the approved artboards (`design/notifications/`, 01-pane,
// 02-try-denied and 03-off), quoted verbatim in both languages: the Spanish column
// is the artboards' own text - neutral Spanish, no voseo, no regionalisms, accents
// kept - and the English column is what the same artboards name as their English
// column. The one line the artboards only draw in Spanish (the explanation) is
// written in English here to say the same thing, nothing more.
//
// The dictionary shape is the house's (updates-copy, usage-copy): one object per
// language and a builder that answers the whole object, so every entry stays a
// plain string and no string-keyed lookup is introduced.
import type { LanguageView } from '../contract/bindings'

/** The Notifications pane's own words for one language. */
export interface NotificationsCopy {
  /** The name of the choice group: the two answers under it are when to notify. */
  group: string
  /** The two answers, in the order the row draws them: the default, then off. */
  whenUnfocused: string
  off: string
  /**
   * What the app does while the window is not in front: the three triggers
   * (a reply finishes, the agent needs the user, something fails) and the one
   * promise that matters - with the window in front, silence.
   */
  explain: string
  /** The one action: play the tone and send one toast, right now. */
  tryIt: string
  /** The tag of the answer that came through, and its one short line. */
  sentTag: string
  sentLine: string
  /** The tag of a toast Windows refused, its line, and the way to allow it. */
  deniedTag: string
  deniedLine: string
  deniedNote: string
  /**
   * Why Try it is dead while the setting is off, and what to choose to hear the
   * tone: `aria-disabled` keeps this line reachable from the keyboard (03-off).
   */
  offReason: string
}

/** English: the artboards' English column, verified line by line. */
const ENGLISH: NotificationsCopy = {
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

/** Neutral Spanish, the artboards' own text: no voseo, no regionalisms. */
const SPANISH: typeof ENGLISH = {
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

/** The pane's own words in `language`, the whole dictionary at once (D4). */
export function notificationsCopy(language: LanguageView): typeof ENGLISH {
  return language === 'es' ? SPANISH : ENGLISH
}
