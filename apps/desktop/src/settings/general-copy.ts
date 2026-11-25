// The General pane's words (0.9.0 general S2, S5; the approved artboard
// `design/settings-redesign/08-general.html`, its G1/G2 sheets and the ES/EN
// table in G4): the four tool names, the three states, the one word the pane says
// while it reads, and the fix hint each tool carries when it is missing or
// unreadable.
//
// One dictionary per language and a builder that answers the whole object, the
// shape every pane of this dialog uses (D4): choosing Español redraws these words
// in the same tick, and missing and unreadable are two different words AND two
// different sentences, so a row can never draw "there is no tool" as "it is there
// and did not answer" (S5).
//
// A fix hint is the LIST of the pieces its sentence is drawn from, because the
// exact command the app names is drawn in `<code>` inside that sentence (G2's
// `.twhy code`): the command is a fact of the copy here, never a string the
// component splits back apart.
//
// What does NOT move: the four tool names (the terminal's own words, spelled the
// same in both columns of G4), the versions the check read, and the `—` a row
// draws while nobody has read it yet (G2).
//
// The hints the artboard does not spell out sentence by sentence are built from
// the pieces it DOES give: G4's footer names the three commands
// (`npm install -g gentle-pi`, `winget install OpenJS.NodeJS.LTS`,
// `winget install Git.Git`), the table gives the install sentence and the two
// sentences under it, and the pi row carries the line the task named for it when
// the artboard had none.
import type { LanguageView, ToolKind, ToolState } from '../contract/bindings'

/**
 * One piece of a fix hint: the sentence's own words, or the command it names,
 * which the pane draws in `<code>`.
 */
export type Piece =
  /** Words of the sentence, with the spaces around them as written. */
  | { readonly kind: 'words'; readonly text: string }
  /** The exact command this page names for that tool. */
  | { readonly kind: 'command'; readonly text: string }

/** The General pane's own words for one language. */
export interface GeneralCopy {
  /** What a row says while no check has answered yet (G2). */
  checking: string
  /**
   * What a row's version column holds while nobody read it: the artboard's `—`,
   * an absence and not a word of ours, so it reads the same in both languages.
   */
  noVersion: string
  /** Each tool's own name, mono in the row and never translated. */
  tools: Record<ToolKind, string>
  /** The three states, worded apart (S5). */
  state: Record<ToolState, string>
  /**
   * The fix hint for a tool in a state that has one, or `null` when it has none:
   * an installed tool has nothing to fix and draws no `.twhy` (S2).
   */
  fix: (tool: ToolKind, state: ToolState) => readonly Piece[] | null
}

/** The words of one sentence piece. */
const words = (text: string): Piece => ({ kind: 'words', text })

/** The command one sentence piece names, drawn in `<code>`. */
const command = (text: string): Piece => ({ kind: 'command', text })

/** The shell's own installer: the package that brings `pi` with it (G4). */
const SHELL = 'npm install -g gentle-pi'
/** node's installer (G2), and git's (S6): the two winget lines of G4's footer. */
const NODE = 'winget install OpenJS.NodeJS.LTS'
const GIT = 'winget install Git.Git'
/** pi's own source, for a user who does not run the shell's pi (G4's footer). */
const PI = 'npm install -g @earendil-works/pi-coding-agent'

/** The four tool names: the terminal's words, the same in both columns. */
const TOOLS: Record<ToolKind, string> = {
  GentleShell: 'Gentle Shell',
  Pi: 'pi',
  Node: 'node',
  Git: 'git',
}

/** The installer this page names for a tool. */
function installer(tool: ToolKind): string {
  return tool === 'Node' ? NODE : tool === 'Git' ? GIT : SHELL
}

/**
 * What a tool being unreadable or absent MEANS for this app, as the approved
 * table words it: nothing for the shell, node's use for the usage script, git
 * never blocking the app (S6), and pi's own line.
 */
function tail(tool: ToolKind, language: LanguageView): readonly Piece[] {
  if (tool === 'Node') {
    return [
      words(
        language === 'es'
          ? ' La app lo usa para el script de uso.'
          : ' The app uses it for the usage script.',
      ),
    ]
  }
  if (tool === 'Git') {
    return [
      words(
        language === 'es'
          ? ' Sin git la app anda: solo faltan los chips de rama.'
          : ' Without git the app still works: only the branch chips are missing.',
      ),
    ]
  }
  if (tool === 'Pi') {
    return [
      words(
        language === 'es'
          ? ' pi viene con Gentle Shell; si usas tu propio pi, instálalo con '
          : ' pi comes with Gentle Shell; if you use your own pi, install it with ',
      ),
      command(PI),
      words('.'),
    ]
  }
  return []
}

/** English: the words of the approved table, in its own columns. */
const ENGLISH: GeneralCopy = {
  checking: 'Checking...',
  noVersion: '—',
  tools: TOOLS,
  state: { Installed: 'Installed', Missing: 'Missing', Unreadable: 'Could not read' },
  fix: (tool, state) =>
    state === 'Installed'
      ? null
      : state === 'Missing'
        ? [words('Install it with '), command(installer(tool)), words(' and check again.'), ...tail(tool, 'en')]
        : [
            words('The version of '),
            command(TOOLS[tool]),
            words(' could not be read. Install it with '),
            command(installer(tool)),
            words('.'),
            ...tail(tool, 'en'),
          ],
}

/** Neutral Spanish: no voseo, no regionalisms, nothing left in English. */
const SPANISH: typeof ENGLISH = {
  checking: 'Comprobando...',
  noVersion: '—',
  tools: TOOLS,
  state: { Installed: 'Instalado', Missing: 'Falta', Unreadable: 'No se pudo leer' },
  fix: (tool, state) =>
    state === 'Installed'
      ? null
      : state === 'Missing'
        ? [
            words('Instálalo con '),
            command(installer(tool)),
            words(' y vuelve a comprobar.'),
            ...tail(tool, 'es'),
          ]
        : [
            words('No se pudo leer la versión de '),
            command(TOOLS[tool]),
            words('. Instálalo con '),
            command(installer(tool)),
            words('.'),
            ...tail(tool, 'es'),
          ],
}

/** The pane's own words in `language`, the whole dictionary at once (D4). */
export function generalCopy(language: LanguageView): GeneralCopy {
  return language === 'es' ? SPANISH : ENGLISH
}
