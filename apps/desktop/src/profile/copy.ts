// The Profile tab's own words (profiles-editor T2c, S6). One dictionary per
// language and a builder that answers the whole object (the house pattern, D4), so
// a parametrized line stays a function and a drawn tab redraws in the same tick
// the language changes (S2). The English column is the artboard's copy where the
// artboard has it (06-profile-tab.html), kept short so Spanish fits; the Spanish
// one is the app's own neutral Spanish - no voseo, no regionalisms - and keeps its
// accents.
//
// What does NOT live here, and is drawn exactly as it arrives (D7): the profile
// names, the agent keys, the models, the efforts and the tiers. The one key the
// view INTERPRETS is `*`, whose label is ours (`profileWide`).
import type { LanguageView } from '../contract/bindings'
import { DOT } from '../helpers/copy'
import type { StoreName } from './rules'

/**
 * The separator between the parts of a line: the one the house already uses
 * (U+00B7, `helpers/copy.ts`), imported so the two views cannot drift apart.
 */

/** The Profile tab's own words for one language. */
export interface ProfileCopy {
  /** The line above the name: which chat this view belongs to. */
  kicker: string
  /** The tag of a chat that follows the global default (state 1). */
  tagDefault: string
  /** The tag of a chat bound to a profile of its own (state 2). */
  tagSession: string
  /** State 1 with a default profile named. */
  lineDefault: (name: string) => string
  /** State 1 with no default named at all: the neutral line. */
  lineDefaultNone: string
  /** State 2: the profile this chat bound to itself. */
  lineSession: (name: string) => string
  /**
   * The approximation L9 accepted: the binding is read from the chat's own session
   * file, never from the channel, so the tab says so in one line.
   */
  approximation: string
  /** "4 agents", singular at one. */
  agentCount: (count: number) => string
  /** "2 chains", singular at one. */
  chainCount: (count: number) => string
  /** The agents table's four headings. */
  agentColumn: string
  modelColumn: string
  thinkingColumn: string
  tierColumn: string
  /** The label of the `*` chain: the profile-wide fallback, and the group it is
   * named in, which is ours because the store has no word for it. */
  profileWide: string
  chainTitle: string
  /** The pin group's own name (state 5). */
  pinTitle: string
  /** No profile is defined at all. */
  noProfiles: string
  /** A profile that names no agents / no chains. */
  noAgents: string
  noChains: string
  /** The pin group's two layer names (state 5). */
  pinRepo: string
  pinLocal: string
  /** The three marks a layer can carry. */
  pinWinner: string
  pinStale: string
  pinInvalid: string
  /** A layer that names no profile. */
  pinNone: string
  /** A workspace that wrote no pin, which is not a failure. */
  pinsNone: string
  /** State 5's no-worktree case: the headline and its one reason line. */
  noWorktree: string
  noWorktreeWhy: string
  /** State 10: the calm headline and the line under it. */
  unavailable: string
  unavailableWhy: string
  /** The read the unavailable state offers, and nothing else. */
  tryAgain: string
  /** Before the first answer ever lands. */
  reading: string
  /** The name of each store, for the line that marks it missing. */
  stores: Record<StoreName, string>
  /** One store could not be read: the line drawn where its content would be. */
  storeMissing: (store: StoreName) => string
  /** The footer's one word: nothing in this tab writes yet. */
  readOnly: string
}

/** English: the artboard's copy, and the house's own words where it has none. */
const ENGLISH: ProfileCopy = {
  kicker: 'This chat',
  tagDefault: 'default',
  tagSession: 'this chat only',
  lineDefault: (name) => `This chat uses the default profile, ${name}.`,
  lineDefaultNone: 'This chat uses the default profile, and none is named yet.',
  lineSession: (name) => `This chat uses ${name}, in this chat only.`,
  approximation: "Approximate: this chat's session file says which profile it uses.",
  agentCount: (count) => (count === 1 ? '1 agent' : `${count} agents`),
  chainCount: (count) => (count === 1 ? '1 chain' : `${count} chains`),
  agentColumn: 'Agent',
  modelColumn: 'Model',
  thinkingColumn: 'Thinking',
  tierColumn: 'Tier',
  profileWide: '* profile-wide',
  chainTitle: 'Fallback chain',
  pinTitle: 'Pin',
  noProfiles: 'No profiles are defined.',
  noAgents: 'No agents.',
  noChains: 'No fallback chains.',
  pinRepo: 'Repo',
  pinLocal: 'Local',
  pinWinner: 'wins here',
  pinStale: 'stale',
  pinInvalid: 'invalid',
  pinNone: 'none',
  pinsNone: 'No pin for this workspace.',
  noWorktree: 'Pins need a Git workspace',
  noWorktreeWhy:
    'This chat has no working directory in a resolvable worktree, so there is no pin to read.',
  unavailable: "Could not read this chat's profile configuration.",
  unavailableWhy: "This chat's child may not be answering. Nothing was changed.",
  tryAgain: 'Try again',
  reading: "Reading this chat's profile configuration...",
  stores: { profiles: 'profiles', fallbacks: 'fallbacks', pins: 'pins' },
  storeMissing: (store) => `The ${ENGLISH.stores[store]} store could not be read.`,
  readOnly: 'Read-only view',
}

/**
 * Neutral Spanish: no voseo, no regionalisms, nothing left in English. The two
 * counted lines mark the singular, which the English column does too.
 */
const SPANISH: ProfileCopy = {
  kicker: 'Este chat',
  tagDefault: 'predeterminado',
  tagSession: 'solo este chat',
  lineDefault: (name) => `Este chat usa el perfil predeterminado, ${name}.`,
  lineDefaultNone: 'Este chat usa el perfil predeterminado y todavía no hay ninguno elegido.',
  lineSession: (name) => `Este chat usa ${name}, solo en este chat.`,
  approximation: 'Aproximado: el archivo de sesión de este chat dice qué perfil usa.',
  agentCount: (count) => (count === 1 ? '1 agente' : `${count} agentes`),
  chainCount: (count) => (count === 1 ? '1 cadena' : `${count} cadenas`),
  agentColumn: 'Agente',
  modelColumn: 'Modelo',
  thinkingColumn: 'Razonamiento',
  tierColumn: 'Nivel',
  profileWide: '* de todo el perfil',
  chainTitle: 'Cadena de respaldo',
  pinTitle: 'Perfil fijado',
  noProfiles: 'No hay perfiles definidos.',
  noAgents: 'No hay agentes.',
  noChains: 'No hay cadenas de respaldo.',
  pinRepo: 'Repositorio',
  // "Local" is the same word in Spanish, so it is shared on purpose.
  pinLocal: 'Local',
  pinWinner: 'manda aquí',
  pinStale: 'obsoleto',
  pinInvalid: 'inválido',
  pinNone: 'ninguno',
  pinsNone: 'No hay ningún perfil fijado en este espacio de trabajo.',
  noWorktree: 'Los perfiles fijados necesitan un espacio de trabajo Git',
  noWorktreeWhy:
    'Este chat no tiene un directorio de trabajo en un worktree resoluble, así que no hay ningún perfil fijado que leer.',
  unavailable: 'No se pudo leer la configuración de perfiles de este chat.',
  unavailableWhy: 'Puede que el hijo de este chat no esté respondiendo. No se cambió nada.',
  tryAgain: 'Reintentar',
  reading: 'Leyendo la configuración de perfiles de este chat...',
  stores: { profiles: 'perfiles', fallbacks: 'cadenas', pins: 'fijados' },
  storeMissing: (store) => `No se pudo leer el almacén de ${SPANISH.stores[store]}.`,
  readOnly: 'Vista de solo lectura',
}

/** The tab's own words in `language`, the whole dictionary at once (D4). */
export function profileCopy(language: LanguageView): ProfileCopy {
  return language === 'es' ? SPANISH : ENGLISH
}

/**
 * One chain's name: the store's own key, except `*`, which is OUR label for the
 * profile-wide fallback. Any other key is drawn exactly as stored (D7).
 */
export function chainName(agent: string, copy: ProfileCopy): string {
  return agent === '*' ? copy.profileWide : agent
}

/** One pin layer's name: the file it is kept in. */
export function pinLayerName(source: string, copy: ProfileCopy): string {
  return source === 'local' ? copy.pinLocal : copy.pinRepo
}

/** One layer's mark, in the tab's own words. */
export function pinMarkWord(mark: string, copy: ProfileCopy): string {
  switch (mark) {
    case 'winner':
      return copy.pinWinner
    case 'stale':
      return copy.pinStale
    default:
      return copy.pinInvalid
  }
}

/** "3 agents · 1 chain": what one profile definition holds. */
export function profileSummary(
  profile: { agents: unknown[]; chains: unknown[] },
  copy: ProfileCopy,
): string {
  return `${copy.agentCount(profile.agents.length)} ${DOT} ${copy.chainCount(profile.chains.length)}`
}
