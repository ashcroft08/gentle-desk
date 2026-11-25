// The General pane's own words (0.9.0 general S2, S5; the approved artboard
// `design/settings-redesign/08-general.html`, sheets G1, G2 and the G4 table):
// the four tool names, the three states plus the one the pane says while it
// reads, and the fix hint each tool carries when it is missing or unreadable -
// with the command the app names drawn in `<code>` inside the sentence.
//
// One dictionary per language and a builder that answers the whole object, the
// shape every pane of this dialog uses (D4): a hint is the LIST of the pieces the
// sentence is drawn from, so the exact command is a fact of the copy and never a
// word the component glues back together.
//
// The English column is quoted literally, so a change of copy is a change here.
// What never moves: the four tool names (the terminal's own words), the versions
// the check reported and the `—` the pane draws while it reads (G2).
import { describe, expect, it } from 'vitest'
import type { ToolKind, ToolState } from '../contract/bindings'
import { generalCopy, type Piece } from './general-copy'

/** The four tools, in the order the contract lists them (and the pane draws). */
const TOOLS: ToolKind[] = ['GentleShell', 'Pi', 'Node', 'Git']

/** The two states that carry a fix hint. Installed carries none (S2). */
const FIXED: ToolState[] = ['Missing', 'Unreadable']

/** The plain words of one language: everything that is not a sentence. */
const ENGLISH = {
  checking: 'Checking...',
  noVersion: '—',
  tools: { GentleShell: 'Gentle Shell', Pi: 'pi', Node: 'node', Git: 'git' },
  state: { Installed: 'Installed', Missing: 'Missing', Unreadable: 'Could not read' },
}

/** Neutral Spanish: no voseo, no regionalisms, nothing left in English. */
const SPANISH = {
  checking: 'Comprobando...',
  noVersion: '—',
  tools: { GentleShell: 'Gentle Shell', Pi: 'pi', Node: 'node', Git: 'git' },
  state: { Installed: 'Instalado', Missing: 'Falta', Unreadable: 'No se pudo leer' },
}

/**
 * One hint as it is drawn: its words as they read, with every command in
 * brackets, so a test pins BOTH the sentence and where the `<code>` starts.
 */
function drawn(pieces: readonly Piece[]): string {
  return pieces.map((piece) => (piece.kind === 'command' ? `[${piece.text}]` : piece.text)).join('')
}

/** The hints, tool by tool, as one table per language (G2/G4). */
const ENGLISH_FIX: Record<string, string> = {
  'GentleShell/Missing': 'Install it with [npm install -g gentle-pi] and check again.',
  'GentleShell/Unreadable':
    'The version of [Gentle Shell] could not be read. Install it with [npm install -g gentle-pi].',
  'Pi/Missing':
    'Install it with [npm install -g gentle-pi] and check again. pi comes with Gentle Shell; if you use your own pi, install it with [npm install -g @earendil-works/pi-coding-agent].',
  'Pi/Unreadable':
    'The version of [pi] could not be read. Install it with [npm install -g gentle-pi]. pi comes with Gentle Shell; if you use your own pi, install it with [npm install -g @earendil-works/pi-coding-agent].',
  'Node/Missing':
    'Install it with [winget install OpenJS.NodeJS.LTS] and check again. The app uses it for the usage script.',
  'Node/Unreadable':
    'The version of [node] could not be read. Install it with [winget install OpenJS.NodeJS.LTS]. The app uses it for the usage script.',
  'Git/Missing':
    'Install it with [winget install Git.Git] and check again. Without git the app still works: only the branch chips are missing.',
  'Git/Unreadable':
    'The version of [git] could not be read. Install it with [winget install Git.Git]. Without git the app still works: only the branch chips are missing.',
}

const SPANISH_FIX: Record<string, string> = {
  'GentleShell/Missing': 'Instálalo con [npm install -g gentle-pi] y vuelve a comprobar.',
  'GentleShell/Unreadable':
    'No se pudo leer la versión de [Gentle Shell]. Instálalo con [npm install -g gentle-pi].',
  'Pi/Missing':
    'Instálalo con [npm install -g gentle-pi] y vuelve a comprobar. pi viene con Gentle Shell; si usas tu propio pi, instálalo con [npm install -g @earendil-works/pi-coding-agent].',
  'Pi/Unreadable':
    'No se pudo leer la versión de [pi]. Instálalo con [npm install -g gentle-pi]. pi viene con Gentle Shell; si usas tu propio pi, instálalo con [npm install -g @earendil-works/pi-coding-agent].',
  'Node/Missing':
    'Instálalo con [winget install OpenJS.NodeJS.LTS] y vuelve a comprobar. La app lo usa para el script de uso.',
  'Node/Unreadable':
    'No se pudo leer la versión de [node]. Instálalo con [winget install OpenJS.NodeJS.LTS]. La app lo usa para el script de uso.',
  'Git/Missing':
    'Instálalo con [winget install Git.Git] y vuelve a comprobar. Sin git la app anda: solo faltan los chips de rama.',
  'Git/Unreadable':
    'No se pudo leer la versión de [git]. Instálalo con [winget install Git.Git]. Sin git la app anda: solo faltan los chips de rama.',
}

/** Every tool and state that has a hint, as `tool/state`. */
const HINTS = TOOLS.flatMap((tool) => FIXED.map((state) => `${tool}/${state}`))

describe('the General copy', () => {
  it('draws the English words of the approved table', () => {
    const copy = generalCopy('en')

    expect(copy.checking).toBe(ENGLISH.checking)
    expect(copy.noVersion).toBe(ENGLISH.noVersion)
    expect(copy.tools).toEqual(ENGLISH.tools)
    expect(copy.state).toEqual(ENGLISH.state)
    for (const hint of HINTS) {
      const [tool, state] = hint.split('/') as [ToolKind, ToolState]
      expect(drawn(copy.fix(tool, state) ?? []), hint).toBe(ENGLISH_FIX[hint])
    }
  })

  it('draws Spanish for the same shape', () => {
    const copy = generalCopy('es')

    expect(copy.checking).toBe(SPANISH.checking)
    expect(copy.noVersion).toBe(SPANISH.noVersion)
    expect(copy.tools).toEqual(SPANISH.tools)
    expect(copy.state).toEqual(SPANISH.state)
    for (const hint of HINTS) {
      const [tool, state] = hint.split('/') as [ToolKind, ToolState]
      expect(drawn(copy.fix(tool, state) ?? []), hint).toBe(SPANISH_FIX[hint])
    }
  })

  it('carries a hint only for a tool that is missing or unreadable', () => {
    for (const language of ['en', 'es'] as const) {
      const copy = generalCopy(language)
      for (const tool of TOOLS) {
        expect(copy.fix(tool, 'Installed'), `${language}/${tool}`).toBeNull()
        for (const state of FIXED) {
          const pieces = copy.fix(tool, state)
          expect(pieces, `${language}/${tool}/${state}`).not.toBeNull()
          expect(pieces!.length).toBeGreaterThan(0)
        }
      }
    }
  })

  // The two facts S5 keeps apart: "the tool is not there" and "it is there and
  // did not answer" are two different words AND two different sentences, so a
  // row can never draw one as the other.
  it('words missing and unreadable apart, per tool and per language', () => {
    for (const language of ['en', 'es'] as const) {
      const copy = generalCopy(language)
      expect(copy.state.Missing).not.toBe(copy.state.Unreadable)
      for (const tool of TOOLS) {
        expect(drawn(copy.fix(tool, 'Missing')!), `${language}/${tool}`).not.toBe(
          drawn(copy.fix(tool, 'Unreadable')!),
        )
      }
    }
  })

  it('translates every sentence and every state, and renames no tool', () => {
    const english = generalCopy('en')
    const spanish = generalCopy('es')

    expect(spanish.checking).not.toBe(english.checking)
    for (const state of Object.keys(english.state) as ToolState[]) {
      expect(spanish.state[state], state).not.toBe(english.state[state])
    }
    // A tool's name is the terminal's own word, so it reads the same twice.
    expect(spanish.tools).toEqual(english.tools)
    // The `—` is an absence, not a word of ours (G2).
    expect(spanish.noVersion).toBe(english.noVersion)
    for (const hint of HINTS) {
      const [tool, state] = hint.split('/') as [ToolKind, ToolState]
      expect(drawn(spanish.fix(tool, state)!), hint).not.toBe(drawn(english.fix(tool, state)!))
    }
  })
})
