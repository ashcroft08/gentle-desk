// The Profiles pane's own words (0.2.0, S3 and D4): the orchestrator switch, the
// two sub-tabs, the columns of the agent table, the pane's three states and the one
// terminal hand-off live in ONE language-keyed dictionary. This file pins the two
// columns; the words a DRAWN pane shows are pinned in `profiles-copy.flip.test.ts`.
//
// The English column is quoted literally: it is what the pane has drawn since
// 0.1.12, so a change of copy is a change here. Profile names, model names, role
// names and the shell's own reason are not words of this module: they travel
// through it exactly as they arrived (D7).
import { describe, expect, it } from 'vitest'
import { profilesCopy } from './profiles-copy'

/** The English words, exactly as the pane drew them before 0.2.0. */
const ENGLISH = {
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
  retrySafe: 'Reading them again is safe.',
  tryAgain: 'Try again',
  editInTerminal: 'Edit in terminal',
  editNote:
    'Opens the shell: type /gentle:profiles there to edit, create and switch profiles - the panel refreshes when the terminal closes.',
  tabsLabel: 'Profile details',
}

/** The Spanish words: neutral Spanish, accents allowed. */
const SPANISH = {
  modeLabel: 'Modo de orquestación',
  modeOrganic: 'Orgánico',
  modeFullDelegation: 'Delegación total',
  modeNote: 'Para los chats abiertos desde ahora \u00b7 no se guarda',
  orchestratorLabel: 'Orquestador',
  fallbackLabel: 'Fallback',
  agentColumn: 'Agente',
  modelColumn: 'Modelo',
  effortColumn: 'Esfuerzo',
  activeNote: 'Predeterminado para las sesiones nuevas',
  reading: 'Leyendo los perfiles...',
  noProfiles: 'No se encontró ningún perfil.',
  retrySafe: 'Volver a leerlos es seguro.',
  tryAgain: 'Reintentar',
  editInTerminal: 'Editar en la terminal',
  editNote:
    'Abre la shell: escribe /gentle:profiles allí para editar, crear y cambiar de perfil - el panel se actualiza cuando se cierra la terminal.',
  tabsLabel: 'Detalles del perfil',
}

/** The two sub-tabs' own words, per language. */
const ENGLISH_TABS = { profile: 'Profile', agents: 'Agents' }
const SPANISH_TABS = { profile: 'Perfil', agents: 'Agentes' }

/** The plain words of one language: the tab map is pinned below them. */
const WORDS = Object.keys(ENGLISH) as (keyof typeof ENGLISH)[]

/**
 * `Fallback` is the one word kept as it is: it names the role pi's own profiles
 * document calls `fallback`, so renaming it would rename what the terminal shows.
 */
const SAME_IN_BOTH = new Set<keyof typeof ENGLISH>(['fallbackLabel'])

/** The words of one language, as a template reads them. */
function words(language: 'en' | 'es'): Record<string, string> {
  const copy = profilesCopy(language)
  return Object.fromEntries(WORDS.map((word) => [word, copy[word]]))
}

describe('the profiles copy', () => {
  it('draws the English words the pane already had', () => {
    expect(words('en')).toEqual(ENGLISH)
    expect(profilesCopy('en').tabs).toEqual(ENGLISH_TABS)
  })

  it('draws Spanish for the same shape', () => {
    expect(words('es')).toEqual(SPANISH)
    expect(profilesCopy('es').tabs).toEqual(SPANISH_TABS)
  })

  it('translates every word, the two sub-tabs included', () => {
    const english = profilesCopy('en')
    const spanish = profilesCopy('es')

    expect(Object.keys(spanish).sort()).toEqual(Object.keys(english).sort())
    expect(Object.keys(spanish.tabs).sort()).toEqual(Object.keys(english.tabs).sort())
    for (const word of WORDS) {
      if (SAME_IN_BOTH.has(word)) {
        expect(spanish[word], word).toBe(english[word])
        continue
      }
      expect(spanish[word], word).not.toBe(english[word])
    }
    for (const tab of Object.keys(english.tabs) as (keyof typeof ENGLISH_TABS)[]) {
      expect(spanish.tabs[tab], `the ${tab} tab`).not.toBe(english.tabs[tab])
    }
  })

  it('wraps the shell own reason after the colon, word for word in both languages', () => {
    const reason = 'the file is not a profiles document'

    expect(profilesCopy('en').failed(reason)).toBe(`The profiles could not be read: ${reason}`)
    expect(profilesCopy('es').failed(reason)).toBe(`No se pudieron leer los perfiles: ${reason}`)
    expect(profilesCopy('en').failed(reason)).toContain(reason)
    expect(profilesCopy('es').failed(reason)).toContain(reason)
  })
})
