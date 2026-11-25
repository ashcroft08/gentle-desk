// The Settings dialog's rail (settings-dialog L4; 0.1.12 S1-S7; 0.2.0 S2/S4): the
// seven sections as entries in a left rail, the selected entry highlighted, and
// that section's content in the pane on the right. The order here is the order
// the rail draws and the first one is the section the dialog opens on. Each
// entry's glyph carries no meaning on its own - the entry's word is its name -
// and the seven follow the house style the settings gear set: Lucide outline
// geometry on one 24x24 grid. That word is NOT here: the rail is drawn in the
// language the app holds (0.2.0, D4), so `settings/copy.ts` keeps the words - the
// rail's own accessible name included - and this module keeps the order, the
// glyphs and the ids.
//
// Profiles left the rail in profiles-editor T2d (L21, L22): the maintainer ruled
// that a Settings section made the profiles look global, and they live in the
// chat's own Profile tab now. The order below is what is left, unchanged.
//
// The keys are the chat header's tablist keys (chat/tabs.ts): arrows walk the
// rail and wrap, Home and End jump to the ends, and one entry is the tab stop.
//
// The search field above the rail reads the KEYWORDS below (the reference's
// `navigation.rs` keeps the same list per page): they are the words a user may
// look a section up by that the section does not draw. They are NOT drawn words,
// which is why they live here and not in `copy.ts`: a list that must answer a
// Spanish query while the app draws English is written in BOTH languages at
// once, and a per-language dictionary cannot hold that. Lowercase, one word each
// where a term is one word - the matcher folds case and accents on both sides,
// so the accents stay here for the reader.
import type { IconName } from '../icons/shapes'
import type { UpdatesView_Serialize as UpdatesView } from '../contract/bindings'
import { updateWaiting } from '../update/dot'

/** The nine sections, as the dialog names them. */
export type Section =
  // The environment check (0.9.0 general S1): FIRST in the rail, and the section
  // the dialog opens on. It asks what the machine has before anything else is
  // asked, which is why it goes above the pane that changes what it reports.
  | 'general'
  | 'appearance'
  | 'home'
  | 'updates'
  | 'account'
  | 'usage'
  | 'language'
  // The tone and the toast (notifications T4): after Language and before About.
  | 'notifications'
  | 'about'

/** One rail entry: the section it draws, its glyph and its search words. */
export interface SectionEntry {
  id: Section
  icon: IconName
  /** What a search may find this entry by, in both languages. Lowercase. */
  keywords: readonly string[]
}

/** The rail, in the order the dialog draws it: General comes first (S1). */
export const SECTIONS: SectionEntry[] = [
  // The environment check (0.9.0 general S1, S2): the four tools the app and the
  // shell depend on. Its glyph is Lucide's `activity`, the line the approved
  // artboard draws first in the rail; the keywords are the words the page does
  // not draw - the other language's, and the two tools' own names.
  {
    id: 'general',
    icon: 'activity',
    keywords: [
      'general', 'environment', 'tools', 'node', 'git', 'pi', 'version', 'check',
      'entorno', 'herramientas', 'versión', 'comprobación',
    ],
  },
  {
    id: 'appearance',
    icon: 'theme',
    keywords: [
      'theme', 'themes', 'color', 'colors', 'colours', 'palette', 'dark', 'light', 'look', 'style',
      'window', 'tema', 'temas', 'colores', 'oscuro', 'claro', 'aspecto', 'estilo',
      'ventana',
    ],
  },
  {
    id: 'home',
    icon: 'house',
    keywords: [
      'home', 'shell', 'pi', 'installation', 'machine', 'terminal', 'signin', 'signout', 'models',
      'chats', 'restart', 'isolated', 'separate', 'hogar', 'inicio', 'instalación', 'máquina',
      'sesión', 'modelos', 'reiniciar', 'aislado', 'separado',
    ],
  },
  {
    id: 'updates',
    icon: 'restart',
    keywords: [
      'update', 'updates', 'upgrade', 'version', 'newer', 'release', 'notes', 'check', 'restart',
      'actualización', 'actualizaciones', 'actualizar', 'versión', 'nueva', 'notas', 'comprobar',
      'reiniciar',
    ],
  },
  {
    id: 'account',
    icon: 'key',
    keywords: [
      'account', 'provider', 'providers', 'signed', 'signin', 'signout', 'auth', 'login', 'logout',
      'credentials', 'cuenta', 'proveedor', 'proveedores', 'sesión', 'iniciar', 'cerrar', 'clave',
    ],
  },
  {
    id: 'usage',
    icon: 'gauge',
    keywords: [
      'usage', 'limits', 'plan', 'subscription', 'tokens', 'consumed', 'sources', 'uso', 'límites',
      'suscripción', 'consumido', 'fuentes',
    ],
  },
  // The language of the app's own words (0.2.0): beside Usage and before About,
  // which stays the last entry Home and End walk to. Its glyph is the speech
  // bubble the house already draws (`message-square`), since the rail's set has
  // no letterform of its own.
  {
    id: 'language',
    icon: 'message-square',
    keywords: [
      'language', 'words', 'english', 'spanish', 'idioma', 'palabras', 'inglés', 'español',
    ],
  },
  // The tone and the toast (notifications S4, S5; L3): the section sits after
  // Idioma and before Acerca de, which stays the last entry Home and End walk
  // to. Its glyph is Lucide's `bell`, the shape the approved artboard draws.
  {
    id: 'notifications',
    icon: 'bell',
    keywords: [
      'notifications', 'notificaciones', 'aviso', 'avisos', 'sonido', 'sound', 'alerta', 'alert',
      'toast', 'probar', 'try',
    ],
  },
  {
    id: 'about',
    icon: 'info',
    keywords: [
      'about', 'version', 'build', 'name', 'info', 'acerca', 'versión', 'compilación', 'nombre',
      'información',
    ],
  },
]

/** The section the dialog opens on: the first one in the rail. */
export const DEFAULT_SECTION: Section = SECTIONS[0].id

/** The id of each rail entry, so the pane can name it with `aria-labelledby`. */
export const TAB_IDS: Record<Section, string> = {
  general: 'settings-tab-general',
  appearance: 'settings-tab-appearance',
  home: 'settings-tab-home',
  updates: 'settings-tab-updates',
  account: 'settings-tab-account',
  usage: 'settings-tab-usage',
  language: 'settings-tab-language',
  notifications: 'settings-tab-notifications',
  about: 'settings-tab-about',
}

/** The pane's one id, since one section is drawn there at a time. */
export const PANEL_ID = 'settings-pane'

/**
 * The entry a vertical move asks for: the arrow keys walk the rail and wrap,
 * Home and End jump to the ends. `null` for every other key, which keeps its own
 * meaning - Escape still closes the dialog.
 *
 * `rail` is the list the rail is DRAWING, which a search query narrows: the
 * arrows then walk the entries on screen and wrap at the ends of that list, so a
 * key never lands on an entry the field has hidden. With no query it is every
 * entry, exactly as before.
 */
export function sectionAfter(
  key: string,
  from: Section,
  rail: readonly SectionEntry[] = SECTIONS,
): Section | null {
  if (rail.length === 0) return null
  const index = rail.findIndex((entry) => entry.id === from)
  // A move starts from an entry the rail is drawing: a search can hide the one
  // the pane shows, and that entry has no key of its own to walk from.
  if (index < 0) return null
  switch (key) {
    case 'ArrowDown':
    case 'ArrowRight':
      return rail[(index + 1) % rail.length].id
    case 'ArrowUp':
    case 'ArrowLeft':
      return rail[(index - 1 + rail.length) % rail.length].id
    case 'Home':
      return rail[0].id
    case 'End':
      return rail[rail.length - 1].id
    default:
      return null
  }
}

/**
 * The id of the hidden line that says the rail's update in words (packages-status
 * S8, T5): the one "Actualizaciones" entry DESCRIBES itself with it
 * (`aria-describedby`) while something waits, so the news has a text channel that
 * adds no word to the entry's own name - the name a user and a test look it up
 * by, and the reason the phrase is not a span inside the entry.
 */
export const RAIL_NOTE_ID = 'settings-rail-note'

/**
 * The note a rail entry carries while something waits (packages-status S8, T5):
 * the phrase the hidden line carries and the same answer that draws the dot
 * beside the entry's word - one condition, so the mark and its description can
 * never disagree.
 *
 * Only `updates` has anything to say. `updates` is the dialog's LIVE slice - the
 * very expression the Updates pane draws - and the rule is `updateWaiting`
 * (update/dot.ts), asked here rather than restated: the skip rule, the two rows
 * and the packages slice stay that module's, so the rail and the gear promise
 * exactly the same thing. Every other entry - and every state that is not a
 * promise: unasked, unknown, up to date - answers `null`, which is no note and no
 * dot.
 */
export function railNote(
  id: Section,
  updates: UpdatesView | null,
  words: { updateWaiting: string },
): string | null {
  return id === 'updates' && updateWaiting(updates) ? words.updateWaiting : null
}
