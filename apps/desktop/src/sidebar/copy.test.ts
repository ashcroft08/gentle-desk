// The sidebar's own words (0.2.0, S3 and D4): the empty state, the reading line,
// the New workspace menu and its create dialog, the footer's chip and gear, the
// theme control's own two names, the new-chat icon of a workspace header and the
// words around a chat row live in ONE language-keyed dictionary. This file pins
// the two columns and the lines that carry a number, a label or a title; the
// words a drawn sidebar shows are pinned in `copy.flip.test.ts`.
//
// The English column is quoted literally: it is what the sidebar has drawn since
// 01a and the states board, so a change of copy is a change here. The titles, the
// workspace labels and the timestamps Rust sends are NOT words of this module:
// they travel through it unchanged (D7).
import { describe, expect, it } from 'vitest'
import { sidebarCopy } from './copy'

/** The English words, exactly as the sidebar drew them before 0.2.0. */
const ENGLISH = {
  workspaces: 'Workspaces',
  newSession: 'New session',
  workspaceTitle: 'Workspace',
  newWorkspace: 'New workspace',
  menuChoose: 'Choose an existing folder',
  menuCreate: 'Create a new folder',
  newFolderTitle: 'Create a new folder',
  newFolderHint: 'Enter a name, then choose where the folder goes.',
  newFolderLabel: 'Folder name',
  createIdle: 'Create',
  createBusy: 'Creating...',
  cancel: 'Cancel',
  emptyTitle: 'No workspaces yet',
  emptyReason:
    'Use New workspace to choose a folder or create a new one. A new session needs a workspace.',
  noWorkspaceReason: 'Select a workspace to start a session.',
  preview: 'Preview',
  previewTitle: 'Preview: under active development - fixes ship fast',
  themeTip: 'Theme',
  themeListName: 'Theme',
  chatActionsTip: 'Chat actions',
  chatName: 'Chat name',
  rename: 'Rename',
  closeChat: 'Close chat',
  deleteChat: 'Delete',
  newChatTip: 'New chat',
  copyPath: 'Copy file path',
  workspaceActionsTip: 'Workspace actions',
  removeWorkspace: 'Remove workspace...',
  removeTitle: 'Remove this workspace?',
  removeKeepConfirm: 'Remove',
  deleteTitle: 'Delete this chat?',
  deleteConfirm: 'Delete',
}

/** The Spanish words: neutral Spanish, accents allowed. */
const SPANISH = {
  workspaces: 'Espacios de trabajo',
  newSession: 'Nueva sesión',
  workspaceTitle: 'Espacio de trabajo',
  newWorkspace: 'Nuevo espacio de trabajo',
  menuChoose: 'Elegir una carpeta existente',
  menuCreate: 'Crear una carpeta nueva',
  newFolderTitle: 'Crear una carpeta nueva',
  newFolderHint: 'Escribe un nombre y elige dónde se crea la carpeta.',
  newFolderLabel: 'Nombre de la carpeta',
  createIdle: 'Crear',
  createBusy: 'Creando...',
  cancel: 'Cancelar',
  emptyTitle: 'Aún no hay espacios de trabajo',
  emptyReason:
    'Usa Nuevo espacio de trabajo para elegir una carpeta o crear una nueva. Una sesión nueva necesita un espacio de trabajo.',
  noWorkspaceReason: 'Elige un espacio de trabajo para iniciar una sesión.',
  preview: 'Vista previa',
  previewTitle: 'Vista previa: en desarrollo activo; las correcciones llegan rápido',
  themeTip: 'Tema',
  themeListName: 'Tema',
  chatActionsTip: 'Acciones del chat',
  chatName: 'Nombre del chat',
  rename: 'Renombrar',
  closeChat: 'Cerrar chat',
  deleteChat: 'Eliminar',
  newChatTip: 'Chat nuevo',
  copyPath: 'Copiar ruta del archivo',
  workspaceActionsTip: 'Acciones del espacio de trabajo',
  removeWorkspace: 'Quitar espacio de trabajo...',
  removeTitle: '¿Quitar este espacio de trabajo?',
  removeKeepConfirm: 'Quitar',
  deleteTitle: '¿Eliminar este chat?',
  deleteConfirm: 'Eliminar',
}

/** The plain words of one language, as the sidebar draws them. */
const WORDS = Object.keys(ENGLISH) as (keyof typeof ENGLISH)[]

/** Nothing in the sidebar is spelled the same in both languages. */
const SAME_IN_BOTH = new Set<keyof typeof ENGLISH>([])

/** A chat title Rust sends: never translated, only wrapped in our words (D7). */
const TITLE = 'First chat'

/** The words of one language, as a template reads them. */
function words(language: 'en' | 'es'): Record<string, string> {
  const copy = sidebarCopy(language)
  return Object.fromEntries(WORDS.map((word) => [word, copy[word]]))
}

describe('the sidebar copy', () => {
  it('draws the English words the sidebar already had', () => {
    expect(words('en')).toEqual(ENGLISH)
  })

  it('draws Spanish for the same shape', () => {
    expect(words('es')).toEqual(SPANISH)
  })

  it('translates every word: nothing is left in English', () => {
    const english = sidebarCopy('en')
    const spanish = sidebarCopy('es')

    expect(Object.keys(spanish).sort()).toEqual(Object.keys(english).sort())
    for (const word of WORDS) {
      if (SAME_IN_BOTH.has(word)) {
        expect(spanish[word], word).toBe(english[word])
        continue
      }
      expect(spanish[word], word).not.toBe(english[word])
    }
  })
})

describe('the sidebar lines that carry a number, a name or a title', () => {
  const english = sidebarCopy('en')
  const spanish = sidebarCopy('es')

  it('spells the reading line, before and after a total is known', () => {
    expect(english.reading(0, 0)).toBe('Reading chats...')
    expect(english.reading(4, 9)).toBe('Reading chats... 4 / 9')
    expect(spanish.reading(0, 0)).toBe('Leyendo chats...')
    expect(spanish.reading(4, 9)).toBe('Leyendo chats... 4 / 9')
  })

  it('names the theme control with its label, and never renames the label', () => {
    expect(english.themeName('Dark')).toBe('Theme: Dark')
    expect(spanish.themeName('Oscuro')).toBe('Tema: Oscuro')
  })

  it('names a row menu with the title Rust sent, word for word', () => {
    expect(english.chatActions(TITLE)).toBe(`Chat actions for ${TITLE}`)
    expect(spanish.chatActions(TITLE)).toBe(`Acciones del chat: ${TITLE}`)
    // D7: the title is Rust's, so both languages carry it exactly as it arrived.
    expect(english.chatActions(TITLE)).toContain(TITLE)
    expect(spanish.chatActions(TITLE)).toContain(TITLE)
  })

  it('names the new-chat icon with the workspace label Rust sent, word for word', () => {
    expect(english.newChatIn('one')).toBe('New chat in one')
    expect(spanish.newChatIn('one')).toBe('Chat nuevo en one')
  })

  it('names the gear with the Settings word it is given, so one word has one owner', () => {
    expect(english.updateWaiting('Settings')).toBe('Settings - an update is available')
    expect(spanish.updateWaiting('Ajustes')).toBe('Ajustes - hay una actualización disponible')
  })

  it('names a workspace row menu with the label Rust sent, word for word', () => {
    expect(english.workspaceActions('one')).toBe('Workspace actions for one')
    expect(spanish.workspaceActions('one')).toBe('Acciones del espacio de trabajo: one')
  })

  it('writes each consequence line with the scope as its own run (S3)', () => {
    expect(english.removeKeepLine).toEqual({
      before: 'gentle-desk only ',
      scope: 'forgets the bookmark',
      after:
        ". Its chats stay in gentle-shell's session store - add the folder back and they come back with it.",
    })
    expect(spanish.removeKeepLine.scope).toBe('olvida el marcador')

    const scope = '2 chats (1.7 MB)'
    const removed = english.removeChatsLine(scope)
    expect(removed.scope).toBe(scope)
    expect(removed.before).toBe('Removing deletes its ')
    expect(removed.after).toContain('cannot be regenerated')
    // The count and the size are drawn by `removal.ts`, never re-spelled here.
    expect(spanish.removeChatsLine(scope).scope).toBe(scope)
  })

  it('labels the disclosure and the answers with the scope, and never a parenthetical (L6)', () => {
    expect(english.showChats('2 chats (1.7 MB)')).toBe('Show 2 chats (1.7 MB)')
    expect(spanish.showChats('2 chats (1.7 MB)')).toBe('Mostrar 2 chats (1.7 MB)')
    expect(english.removeChatsConfirm('2 chats')).toBe('Delete 2 chats and remove')
    expect(spanish.removeChatsConfirm('2 chats')).toBe('Eliminar 2 chats y quitar')
    expect(english.removeChatsConfirm('2 chats')).not.toContain('(')
  })

  it('never prints the words of the store\u2019s own files (S3)', () => {
    const lines = [
      english.removeKeepLine,
      english.removeChatsLine('1 chat'),
      spanish.removeKeepLine,
      spanish.removeChatsLine('1 chat'),
      english.deleteLine,
      spanish.deleteLine,
    ]
    for (const line of lines) {
      const whole = `${line.before}${line.scope}${line.after}`
      expect(whole).not.toMatch(/chat file/i)
      expect(line.scope.length).toBeGreaterThan(0)
    }
    expect(english.deleteLine.scope).toBe('chat')
  })
})
