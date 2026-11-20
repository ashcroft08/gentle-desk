// The sidebar's words, in one place: the design's own text, quoted from
// design/gentle-desk-v2/00-states-board.html, so the states board and the app
// cannot drift apart silently. Everything here is plain text.
//
// 0.2.0 (S3, D4): those words are written in BOTH languages here - one dictionary
// per language and a builder that answers the whole object, so a parametrized
// entry (a reading line with its counter, a row's actions menu, a name with a
// theme label in it) stays a function and no string-keyed lookup is introduced.
// A sidebar drawn from it redraws its words in the same tick it is told the new
// language (S2). The English column is what the sidebar has drawn since 01a and
// the states board, unchanged; the Spanish one is neutral - no voseo, no
// regionalisms - and keeps its accents.
//
// What does NOT move: the chat titles, the workspace labels and the timestamps
// Rust sends, and the drawn shapes of the marks. They travel through these words
// exactly as they arrived (D7), which is why every function below that carries one
// keeps it untouched.
import type { LanguageView } from '../contract/bindings'
import type { ConsequenceLine } from './removal'

/** The id of the `Select a workspace to start a session.` line. */
export const NEW_SESSION_WHY_ID = 'new-session-why'

/** The id of the empty first-run reason, drawn by the empty state. */
export const EMPTY_WHY_ID = 'workspace-empty-why'

/** The sidebar's own words for one language. */
export interface SidebarCopy {
  /** The sidebar's own name, and the tree's: the same word for both. */
  workspaces: string
  /** The first-run empty state's title. */
  emptyTitle: string
  /** The empty first run's one line: what to do, and why nothing else works. */
  emptyReason: string
  /** New session's reason while a workspace exists but none is current (D4). */
  noWorkspaceReason: string
  /** The sidebar header: the section name of the tree. */
  workspaceTitle: string
  /** New session: one new chat in the current workspace. */
  newSession: string
  /** The New workspace control's own name: the icon, its menu and its dialog. */
  newWorkspace: string
  /** The two items the New workspace menu draws (01a, the board's menu card). */
  menuChoose: string
  menuCreate: string
  /** The create-folder dialog. No artboard draws it, so these are its own words. */
  newFolderTitle: string
  newFolderHint: string
  newFolderLabel: string
  createIdle: string
  createBusy: string
  cancel: string
  /**
   * The first-pass line and its counter, mirroring the rule the iced sidebar
   * tests (`crates/ui/src/view/sidebar/status.rs`): no counter before a file has
   * been found, so a zero total never reads like a settled answer.
   */
  reading: (done: number, total: number) => string
  /** The run's Preview chip, and the promise its tooltip carries. */
  preview: string
  previewTitle: string
  /**
   * The gear's name while an update waits. The Settings word itself is the
   * dialog's (settings/copy.ts), so it is passed in and only the news is ours.
   */
  updateWaiting: (settings: string) => string
  /** A chat row's actions: the button's tooltip, the menu's name and its items. */
  chatActionsTip: string
  chatActions: (title: string) => string
  chatName: string
  rename: string
  closeChat: string
  deleteChat: string
  /** The new-chat icon of a workspace header, named with the workspace's label. */
  newChatTip: string
  newChatIn: (label: string) => string
  /** The chat row menu's copy of the session file's own name (S3). */
  copyPath: string
  /**
   * The workspace header's `...` menu (S8): the button's tooltip, the menu's name
   * around the workspace's label, and its one destructive item.
   */
  workspaceActionsTip: string
  workspaceActions: (label: string) => string
  removeWorkspace: string
  /**
   * The removal dialog (S1, S3): the question, its one consequence line with the
   * scope to be drawn in bold, the disclosure over the chats that go, and the two
   * answers. The count and the size inside those lines are `removal.ts`'s, so the
   * words here only hold the sentence around them.
   */
  removeTitle: string
  removeKeepLine: ConsequenceLine
  removeChatsLine: (scope: string) => ConsequenceLine
  showChats: (scope: string) => string
  removeKeepConfirm: string
  removeChatsConfirm: (count: string) => string
  /**
   * The dialog's two failure lines, in the consequence line's own anatomy (S3):
   * one for an offer that could not be read (R3-002), one for a removal the shell
   * refused before it answered (R3-003). Those two are the dialog's words, not
   * Rust's: the reason a command carries is drawn where a notice is drawn.
   */
  removeOfferError: ConsequenceLine
  removeError: ConsequenceLine
  /** The delete confirm dialog, in the same anatomy (S3). */
  deleteTitle: string
  deleteLine: ConsequenceLine
  deleteConfirm: string
}

/** English: the words the sidebar already drew. */
const ENGLISH: SidebarCopy = {
  workspaces: 'Workspaces',
  emptyTitle: 'No workspaces yet',
  emptyReason:
    'Use New workspace to choose a folder or create a new one. A new session needs a workspace.',
  noWorkspaceReason: 'Select a workspace to start a session.',
  workspaceTitle: 'Workspace',
  newSession: 'New session',
  newWorkspace: 'New workspace',
  menuChoose: 'Choose an existing folder',
  menuCreate: 'Create a new folder',
  newFolderTitle: 'Create a new folder',
  newFolderHint: 'Enter a name, then choose where the folder goes.',
  newFolderLabel: 'Folder name',
  createIdle: 'Create',
  createBusy: 'Creating...',
  cancel: 'Cancel',
  reading: (done, total) =>
    total === 0 ? 'Reading chats...' : `Reading chats... ${done} / ${total}`,
  preview: 'Preview',
  previewTitle: 'Preview: under active development - fixes ship fast',
  updateWaiting: (settings) => `${settings} - an update is available`,
  chatActionsTip: 'Chat actions',
  chatActions: (title) => `Chat actions for ${title}`,
  chatName: 'Chat name',
  rename: 'Rename',
  closeChat: 'Close chat',
  deleteChat: 'Delete',
  newChatTip: 'New chat',
  newChatIn: (label) => `New chat in ${label}`,
  copyPath: 'Copy file path',
  workspaceActionsTip: 'Workspace actions',
  workspaceActions: (label) => `Workspace actions for ${label}`,
  removeWorkspace: 'Remove workspace...',
  removeTitle: 'Remove this workspace?',
  removeKeepLine: {
    before: 'gentle-desk only ',
    scope: 'forgets the bookmark',
    after:
      ". Its chats stay in gentle-shell's session store - add the folder back and they come back with it.",
  },
  removeChatsLine: (scope) => ({
    before: 'Removing deletes its ',
    scope,
    after: " from gentle-shell's session store. This history cannot be regenerated.",
  }),
  showChats: (scope) => `Show ${scope}`,
  removeKeepConfirm: 'Remove',
  removeChatsConfirm: (count) => `Delete ${count} and remove`,
  removeOfferError: {
    before: 'The chats of this workspace could not be read. ',
    scope: 'Nothing was removed.',
    after: ' Try again from the workspace menu, or cancel.',
  },
  removeError: {
    before: 'The removal did not finish. ',
    scope: 'Nothing more will be deleted',
    after: ' unless you try again.',
  },
  deleteTitle: 'Delete this chat?',
  deleteLine: {
    before: 'Removing deletes this ',
    scope: 'chat',
    after: " from gentle-shell's session store. This history cannot be regenerated.",
  },
  deleteConfirm: 'Delete',
}

/** Neutral Spanish: no voseo, no regionalisms, nothing left in English. */
const SPANISH: typeof ENGLISH = {
  workspaces: 'Espacios de trabajo',
  emptyTitle: 'Aún no hay espacios de trabajo',
  emptyReason:
    'Usa Nuevo espacio de trabajo para elegir una carpeta o crear una nueva. Una sesión nueva necesita un espacio de trabajo.',
  noWorkspaceReason: 'Elige un espacio de trabajo para iniciar una sesión.',
  workspaceTitle: 'Espacio de trabajo',
  newSession: 'Nueva sesión',
  newWorkspace: 'Nuevo espacio de trabajo',
  menuChoose: 'Elegir una carpeta existente',
  menuCreate: 'Crear una carpeta nueva',
  newFolderTitle: 'Crear una carpeta nueva',
  newFolderHint: 'Escribe un nombre y elige dónde se crea la carpeta.',
  newFolderLabel: 'Nombre de la carpeta',
  createIdle: 'Crear',
  createBusy: 'Creando...',
  cancel: 'Cancelar',
  reading: (done, total) =>
    total === 0 ? 'Leyendo chats...' : `Leyendo chats... ${done} / ${total}`,
  preview: 'Vista previa',
  previewTitle: 'Vista previa: en desarrollo activo; las correcciones llegan rápido',
  updateWaiting: (settings) => `${settings} - hay una actualización disponible`,
  chatActionsTip: 'Acciones del chat',
  chatActions: (title) => `Acciones del chat: ${title}`,
  chatName: 'Nombre del chat',
  rename: 'Renombrar',
  closeChat: 'Cerrar chat',
  deleteChat: 'Eliminar',
  newChatTip: 'Chat nuevo',
  newChatIn: (label) => `Chat nuevo en ${label}`,
  copyPath: 'Copiar ruta del archivo',
  workspaceActionsTip: 'Acciones del espacio de trabajo',
  workspaceActions: (label) => `Acciones del espacio de trabajo: ${label}`,
  removeWorkspace: 'Quitar espacio de trabajo...',
  removeTitle: '¿Quitar este espacio de trabajo?',
  removeKeepLine: {
    before: 'gentle-desk solo ',
    scope: 'olvida el marcador',
    after:
      '. Sus chats siguen en el almacén de sesiones de gentle-shell: agrega la carpeta de nuevo y vuelven con ella.',
  },
  removeChatsLine: (scope) => ({
    before: 'Al quitarlo se eliminan sus ',
    scope,
    after: ' del almacén de sesiones de gentle-shell. Este historial no se puede regenerar.',
  }),
  showChats: (scope) => `Mostrar ${scope}`,
  removeKeepConfirm: 'Quitar',
  removeChatsConfirm: (count) => `Eliminar ${count} y quitar`,
  removeOfferError: {
    before: 'No se pudieron leer los chats de este espacio de trabajo. ',
    scope: 'No se quitó nada.',
    after: ' Inténtalo de nuevo desde el menú del espacio de trabajo, o cancela.',
  },
  removeError: {
    before: 'La operación no terminó. ',
    scope: 'No se eliminará nada más',
    after: ' a menos que lo intentes de nuevo.',
  },
  deleteTitle: '¿Eliminar este chat?',
  deleteLine: {
    before: 'Quitarlo elimina este ',
    scope: 'chat',
    after: ' del almacén de sesiones de gentle-shell. Este historial no se puede regenerar.',
  },
  deleteConfirm: 'Eliminar',
}

/** The sidebar's own words in `language`, the whole dictionary at once (D4). */
export function sidebarCopy(language: LanguageView): typeof ENGLISH {
  return language === 'es' ? SPANISH : ENGLISH
}
