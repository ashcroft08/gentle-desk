// The Profiles pane's words (0.1.12, S1, T2, L9, L17). Read-only: every edit
// happens in the terminal, so the pane's own copy says what it shows and where the
// writing happens.
//
// 0.2.0 (S3, D4): those words are written in BOTH languages here - one dictionary
// per language and a builder that answers the whole object, so a parametrized line
// (the failure reason after its colon) stays a function and no string-keyed lookup
// is introduced. The two sub-tabs' own words live here too, beside the pane that
// draws them (D5): the pane's area is this module's. The English column is what the
// pane has drawn since 0.1.12, unchanged; the Spanish one is neutral - no voseo, no
// regionalisms - and keeps its accents.
//
// What does NOT move: the profile name, the model names, the effort words and the
// role names, which are pi's own document and arrive exactly as written (D7).
import type { LanguageView } from '../contract/bindings'
import type { ProfileTab } from './profile-tabs'

/** The Profiles pane's own words for one language. */
export interface ProfilesCopy {
  /** The switch's own label, and the one line under it that keeps it honest. */
  modeLabel: string
  modeOrganic: string
  modeFullDelegation: string
  modeNote: string
  /** The two routes the summary block draws, and the table's three columns. */
  orchestratorLabel: string
  fallbackLabel: string
  agentColumn: string
  modelColumn: string
  effortColumn: string
  /** The status line of the active profile (L11: the active one only). */
  activeNote: string
  /** The pane's three states, in the order the pushes walk them. */
  reading: string
  noProfiles: string
  failed: (reason: string) => string
  /** What a retry means, and the one button that does it. */
  retrySafe: string
  tryAgain: string
  /** The one action, and the note that says it opens a terminal. */
  editInTerminal: string
  editNote: string
  /** The tablist's accessible name: the pane's own title already says Profiles. */
  tabsLabel: string
  /** Each sub-tab's own word, in the order the pane draws them (L17). */
  tabs: Record<ProfileTab, string>
}

/** English: the words the pane already drew. */
const ENGLISH: ProfilesCopy = {
  modeLabel: 'Orchestrator mode',
  modeOrganic: 'Organic',
  modeFullDelegation: 'Full delegation',
  modeNote: 'For chats opened from now on \u00b7 not stored',
  orchestratorLabel: 'Orchestrator',
  fallbackLabel: 'Fallback',
  agentColumn: 'Agent',
  modelColumn: 'Model',
  effortColumn: 'Effort',
  activeNote: 'Default for new sessions',
  reading: 'Reading profiles...',
  noProfiles: 'No profiles found.',
  failed: (reason) => `The profiles could not be read: ${reason}`,
  retrySafe: 'Reading them again is safe.',
  tryAgain: 'Try again',
  editInTerminal: 'Edit in terminal',
  editNote:
    'Opens the shell: type /gentle:profiles there to edit, create and switch profiles - the panel refreshes when the terminal closes.',
  tabsLabel: 'Profile details',
  tabs: { profile: 'Profile', agents: 'Agents' },
}

/** Neutral Spanish: no voseo, no regionalisms, nothing left in English. */
const SPANISH: typeof ENGLISH = {
  modeLabel: 'Modo de orquestación',
  modeOrganic: 'Orgánico',
  modeFullDelegation: 'Delegación total',
  modeNote: 'Para los chats abiertos desde ahora \u00b7 no se guarda',
  orchestratorLabel: 'Orquestador',
  // `Fallback` names the role pi's own profiles document calls `fallback`: the one
  // word the terminal and this pane must not disagree on.
  fallbackLabel: 'Fallback',
  agentColumn: 'Agente',
  modelColumn: 'Modelo',
  effortColumn: 'Esfuerzo',
  activeNote: 'Predeterminado para las sesiones nuevas',
  reading: 'Leyendo los perfiles...',
  noProfiles: 'No se encontró ningún perfil.',
  failed: (reason) => `No se pudieron leer los perfiles: ${reason}`,
  retrySafe: 'Volver a leerlos es seguro.',
  tryAgain: 'Reintentar',
  editInTerminal: 'Editar en la terminal',
  editNote:
    'Abre la shell: escribe /gentle:profiles allí para editar, crear y cambiar de perfil - el panel se actualiza cuando se cierra la terminal.',
  tabsLabel: 'Detalles del perfil',
  tabs: { profile: 'Perfil', agents: 'Agentes' },
}

/** The pane's own words in `language`, the whole dictionary at once (D4). */
export function profilesCopy(language: LanguageView): typeof ENGLISH {
  return language === 'es' ? SPANISH : ENGLISH
}
