// The Profile tab's own words (profiles-editor T2c, S6): one dictionary per
// language and a builder that answers the whole object (the house pattern, D4),
// so a parametrized line stays a function and a drawn tab redraws in the same tick
// the language changes. The English column is the artboard's copy where the
// artboard has it (06-profile-tab.html) and kept short so Spanish fits; the
// Spanish one is the app's own neutral Spanish, no voseo and no regionalisms.
//
// What does NOT live here: the profile names, the agent keys, the models, the
// efforts and the tiers. They are the store's data and are drawn exactly as they
// arrive (D7); so is the `*` key, whose LABEL is ours and is `profileWide`.
import { describe, expect, it } from 'vitest'
import type { StoreName } from './rules'
import { chainName, pinLayerName, pinMarkWord, profileCopy, profileSummary } from './copy'
import { FAST, TEAM } from '../test-support/chat-profile'

/** The English words, exactly as the tab draws them. */
const ENGLISH = {
  kicker: 'This chat',
  tagDefault: 'default',
  tagSession: 'this chat only',
  lineDefault: 'This chat uses the default profile, team.',
  lineDefaultNone: 'This chat uses the default profile, and none is named yet.',
  lineSession: 'This chat uses fast, in this chat only.',
  approximation: "Approximate: this chat's session file says which profile it uses.",
  agentCount: '4 agents',
  agentCountOne: '1 agent',
  chainCount: '2 chains',
  chainCountOne: '1 chain',
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
  storeProfiles: 'profiles',
  storeFallbacks: 'fallbacks',
  storePins: 'pins',
  storeMissing: 'The pins store could not be read.',
  modeNote: 'Saved in the profile "team", the setting every chat with that profile uses.',
  modeNotSaved: 'The mode could not be saved. Nothing was written.',
}

/** The same words in the app's neutral Spanish. */
const SPANISH = {
  kicker: 'Este chat',
  tagDefault: 'predeterminado',
  tagSession: 'solo este chat',
  lineDefault: 'Este chat usa el perfil predeterminado, team.',
  lineDefaultNone: 'Este chat usa el perfil predeterminado y todavía no hay ninguno elegido.',
  lineSession: 'Este chat usa fast, solo en este chat.',
  approximation: 'Aproximado: el archivo de sesión de este chat dice qué perfil usa.',
  agentCount: '4 agentes',
  agentCountOne: '1 agente',
  chainCount: '2 cadenas',
  chainCountOne: '1 cadena',
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
  storeProfiles: 'perfiles',
  storeFallbacks: 'cadenas',
  storePins: 'fijados',
  storeMissing: 'No se pudo leer el almacén de fijados.',
  modeNote: 'Se guarda en el perfil "team", el ajuste que usan todos los chats con ese perfil.',
  modeNotSaved: 'No se pudo guardar el modo. No se escribió nada.',
}

/** The one word both languages spell the same on purpose (`Local` is Spanish too). */
const SAME_IN_BOTH = new Set<keyof typeof ENGLISH>(['pinLocal'])

/** The words of one language, read the way the tab reads them. */
function words(language: 'en' | 'es'): typeof ENGLISH {
  const copy = profileCopy(language)
  return {
    kicker: copy.kicker,
    tagDefault: copy.tagDefault,
    tagSession: copy.tagSession,
    lineDefault: copy.lineDefault('team'),
    lineDefaultNone: copy.lineDefaultNone,
    lineSession: copy.lineSession('fast'),
    approximation: copy.approximation,
    agentCount: copy.agentCount(4),
    agentCountOne: copy.agentCount(1),
    chainCount: copy.chainCount(2),
    chainCountOne: copy.chainCount(1),
    agentColumn: copy.agentColumn,
    modelColumn: copy.modelColumn,
    thinkingColumn: copy.thinkingColumn,
    tierColumn: copy.tierColumn,
    profileWide: copy.profileWide,
    chainTitle: copy.chainTitle,
    pinTitle: copy.pinTitle,
    noProfiles: copy.noProfiles,
    noAgents: copy.noAgents,
    noChains: copy.noChains,
    pinRepo: copy.pinRepo,
    pinLocal: copy.pinLocal,
    pinWinner: copy.pinWinner,
    pinStale: copy.pinStale,
    pinInvalid: copy.pinInvalid,
    pinNone: copy.pinNone,
    pinsNone: copy.pinsNone,
    noWorktree: copy.noWorktree,
    noWorktreeWhy: copy.noWorktreeWhy,
    unavailable: copy.unavailable,
    unavailableWhy: copy.unavailableWhy,
    tryAgain: copy.tryAgain,
    reading: copy.reading,
    storeProfiles: copy.stores.profiles,
    storeFallbacks: copy.stores.fallbacks,
    storePins: copy.stores.pins,
    storeMissing: copy.storeMissing('pins'),
    modeNote: copy.modeNote('team'),
    modeNotSaved: copy.modeNotSaved,
  }
}

describe('the profile copy', () => {
  it('draws the English words the tab draws', () => {
    expect(words('en')).toEqual(ENGLISH)
  })

  it('draws Spanish for the same shape', () => {
    expect(words('es')).toEqual(SPANISH)
  })

  it('translates every word and every line: nothing is left in English', () => {
    const english = words('en')
    const spanish = words('es')

    expect(Object.keys(spanish).sort()).toEqual(Object.keys(english).sort())
    for (const key of Object.keys(english) as (keyof typeof ENGLISH)[]) {
      if (SAME_IN_BOTH.has(key)) {
        expect(spanish[key], key).toBe(english[key])
        continue
      }
      expect(spanish[key], key).not.toBe(english[key])
    }
    // Each store has its own name, so the marking line of each one differs.
    const stores: StoreName[] = ['profiles', 'fallbacks', 'pins']
    const marked = stores.map((store) => profileCopy('en').storeMissing(store))
    expect(new Set(marked).size).toBe(stores.length)
    for (const store of stores) {
      expect(profileCopy('es').storeMissing(store)).not.toBe(profileCopy('en').storeMissing(store))
    }
  })
})

describe('the labels the view adds to the store data', () => {
  it('labels the `*` chain as the profile-wide one and keeps every other key as it is', () => {
    expect(chainName('*', profileCopy('en'))).toBe('* profile-wide')
    expect(chainName('worker', profileCopy('en'))).toBe('worker')
    expect(chainName('*', profileCopy('es'))).toBe('* de todo el perfil')
  })

  it('names each pin layer and each mark', () => {
    const copy = profileCopy('en')
    expect(pinLayerName('repo', copy)).toBe('Repo')
    expect(pinLayerName('local', copy)).toBe('Local')
    expect(pinMarkWord('winner', copy)).toBe('wins here')
    expect(pinMarkWord('stale', copy)).toBe('stale')
    expect(pinMarkWord('invalid', copy)).toBe('invalid')
  })

  it('sums up one profile as its agents and its chains', () => {
    expect(profileSummary(TEAM, profileCopy('en'))).toBe('3 agents \u00b7 1 chain')
    expect(profileSummary(FAST, profileCopy('en'))).toBe('1 agent \u00b7 1 chain')
    expect(profileSummary(FAST, profileCopy('es'))).toBe('1 agente \u00b7 1 cadena')
  })
})
