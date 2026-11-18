// The Profile tab's two APPLY outcomes and the answers that write nothing
// (profiles-editor T3b, S4, S8, S9, L30).
//
// One dictionary per language and a builder that answers the whole object (the
// house pattern, D4), kept in a file of its own so the tab's read-only words
// (`copy.ts`) and these writing ones stay separately readable - and so neither
// file grows past the house's line limit.
//
// What is NOT here: the channel's own confirmation. Its title and message are
// the child's text, in the child's language, and are drawn exactly as they
// arrive (L30 ruling 2); only the two buttons and the line that says whose dialog
// it is are ours. Nor are the NOTES of an apply that ran: they are the child's own
// answer, in its own voice, and the tab draws them verbatim with no wrapper word to
// translate (T3b, L32).
import type {
  ChatProfileApplyNoticeView,
  LanguageView,
} from '../contract/bindings'

/** The two outcomes, their scopes, and the one line each failure leaves. */
export interface ApplyCopy {
  /** `Use in this chat`: the profile binds to THIS chat's own child. */
  useInChat: string
  /** What that touches: this chat, and no other open chat. */
  useInChatNote: string
  /** `Make default`: the channel's own GLOBAL apply. */
  makeDefault: string
  /** What that touches: every chat that starts with the default (S9, L30). */
  makeDefaultNote: string
  /**
   * One short line per answer that wrote nothing (S4): the profile is gone, an
   * apply is already running, the store moved, the owner refused the data, the
   * child did not answer, its shell cannot apply, the screen itself did not end
   * `applied`, or any other refusal.
   */
  notices: Record<ChatProfileApplyNoticeView, string>
  /** The two buttons of the child's own dialog. */
  confirmAccept: string
  confirmCancel: string
  /** The line that says whose dialog this is, so the child's text is never read
   * as the app's own words. */
  confirmNote: string
}

/** English: short enough that the Spanish column fits the same controls. */
const ENGLISH: ApplyCopy = {
  useInChat: 'Use in this chat',
  useInChatNote:
    "Sends this profile to this chat's own child. The other open chats keep the routing they started with.",
  makeDefault: 'Make default',
  makeDefaultNote:
    'Global: every chat that starts with the default uses this profile, and gentle-pi asks first.',
  notices: {
    unknown_profile: 'That profile is no longer saved, so nothing was applied.',
    busy: 'Another profile is already being applied. Try again when it finishes.',
    revision_conflict: 'The profile changed while applying. Nothing was written; try again.',
    invalid_data: 'The profile store refused the data. Nothing was written.',
    no_answer: "This chat's child did not answer, so nothing was applied.",
    unsupported: "This chat's shell cannot apply profiles.",
    not_applied: 'The profile was not applied. Nothing was written.',
    refused: 'The profile could not be applied. Nothing was written.',
  },
  confirmAccept: 'Accept',
  confirmCancel: 'Cancel',
  confirmNote: 'gentle-pi asks this itself; this tab only offers the action.',
}

/** Neutral Spanish: no voseo, no regionalisms, nothing left in English. */
const SPANISH: ApplyCopy = {
  useInChat: 'Usar en este chat',
  useInChatNote:
    'Envía este perfil al hijo de este chat. Los otros chats abiertos conservan la ruta con la que empezaron.',
  makeDefault: 'Hacer predeterminado',
  makeDefaultNote:
    'Global: todos los chats que empiezan con el perfil predeterminado usan este perfil, y gentle-pi pregunta antes.',
  notices: {
    unknown_profile: 'Ese perfil ya no está guardado, así que no se aplicó nada.',
    busy: 'Ya se está aplicando otro perfil. Reintenta cuando termine.',
    revision_conflict: 'El perfil cambió mientras se aplicaba. No se escribió nada; reintenta.',
    invalid_data: 'El almacén de perfiles rechazó los datos. No se escribió nada.',
    no_answer: 'El hijo de este chat no respondió, así que no se aplicó nada.',
    unsupported: 'La shell de este chat no puede aplicar perfiles.',
    not_applied: 'El perfil no se aplicó. No se escribió nada.',
    refused: 'No se pudo aplicar el perfil. No se escribió nada.',
  },
  confirmAccept: 'Aceptar',
  confirmCancel: 'Cancelar',
  confirmNote: 'gentle-pi pregunta esto por su cuenta; esta pestaña solo ofrece la acción.',
}

/** These words in `language`, the whole dictionary at once (D4). */
export function applyCopy(language: LanguageView): ApplyCopy {
  return language === 'es' ? SPANISH : ENGLISH
}
