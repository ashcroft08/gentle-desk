// The geometry of the two inline SVG primitives. Path data is copied verbatim
// from the Lucide icons the design artboard already uses
// (design/gentle-desk-v2/01a-tree-chat.html), so the drawing matches the design
// and the app adds no icon dependency and no external URL.
//
// Lucide is licensed under the ISC license: https://lucide.dev/license

/** One drawn element: a path outline or a circle. */
export type Shape =
  | { kind: 'path'; d: string }
  | { kind: 'circle'; cx: number; cy: number; r: number }

const path = (d: string): Shape => ({ kind: 'path', d })
const circle = (cx: number, cy: number, r: number): Shape => ({ kind: 'circle', cx, cy, r })

/**
 * One element of an icon: the three shapes above plus a filled rectangle, which
 * the artboard draws for Stop (`<rect width="18" height="18" x="3" y="3"
 * rx="2">` inside `fill="currentColor"`). Marks never need it, so the filled
 * shape stays with the icons instead of widening what a chat mark may draw.
 */
export type IconShape = Shape | { kind: 'rect'; x: number; y: number; width: number; height: number; rx: number }

const rect = (x: number, y: number, width: number, height: number, rx: number): IconShape => ({
  kind: 'rect',
  x,
  y,
  width,
  height,
  rx,
})

/** The named icons the controls need: the sidebar's and the composer's. */
export type IconName =
  | 'plus'
  | 'search'
  | 'message-square'
  | 'folder'
  | 'folder-open'
  | 'folder-plus'
  | 'circle-plus'
  // The chat row's actions menu (T2), 01a's `i-more` symbol.
  | 'more'
  | 'send'
  | 'stop'
  | 'restart'
  | 'x'
  | 'back'
  | 'theme'
  // The footer's gear, which opens the Settings dialog (settings-dialog S5):
  // Lucide's `settings` icon, one outline around the hub.
  | 'settings'
  // The Settings dialog's rail (settings-dialog L4): Lucide's `house` for Shell
  // home and its `info` for About and updates, in the gear's own stroke.
  | 'house'
  | 'info'
  // The 0.1.12 rail's own two entries: Lucide's `users` for Profiles, whose
  // rows are the agents a profile routes, and its `key` for Account.
  | 'users'
  | 'key'
  // The model chip's two dropdowns (0.1.11): Lucide's `chevron-down`.
  | 'chevronDown'
  // The sidebar collapse toggle (responsive-sidebar S1): Lucide's `menu`.
  | 'menu'

const FOLDER = 'M20 20a2 2 0 0 0 2-2V8a2 2 0 0 0-2-2h-7.9a2 2 0 0 1-1.69-.9L9.6 3.9A2 2 0 0 0 7.93 3H4a2 2 0 0 0-2 2v13a2 2 0 0 0 2 2Z'
const FOLDER_OPEN =
  'm6 14 1.5-2.9A2 2 0 0 1 9.24 10H20a2 2 0 0 1 1.94 2.5l-1.54 6a2 2 0 0 1-1.95 1.5H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h3.9a2 2 0 0 1 1.69.9l.81 1.2a2 2 0 0 0 1.67.9H18a2 2 0 0 1 2 2v2'

export const ICON_SHAPES: Record<IconName, IconShape[]> = {
  plus: [path('M5 12h14'), path('M12 5v14')],
  search: [path('m21 21-4.34-4.34'), circle(11, 11, 8)],
  'message-square': [
    path('M22 17a2 2 0 0 1-2 2H6.828a2 2 0 0 0-1.414.586l-2.202 2.202A.71.71 0 0 1 2 21.286V5a2 2 0 0 1 2-2h16a2 2 0 0 1 2 2z'),
  ],
  folder: [path(FOLDER)],
  // The header row's two states: closed and open folder, no chevron (D6).
  'folder-open': [path(FOLDER_OPEN)],
  'folder-plus': [path('M12 10v6'), path('M9 13h6'), path(FOLDER)],
  'circle-plus': [circle(12, 12, 10), path('M8 12h8'), path('M12 8v8')],
  // The row's "..." menu (T2): the artboard's `i-more` symbol, three dots.
  more: [circle(12, 12, 1), circle(19, 12, 1), circle(5, 12, 1)],
  // The composer's two actions and the restart notice's: the artboard's own
  // paths (01a, the `i-send`, `i-stop`, `i-restart` and `i-x` symbols).
  send: [path('m5 12 7-7 7 7'), path('M12 19V5')],
  stop: [rect(3, 3, 18, 18, 2)],
  restart: [path('M3 12a9 9 0 1 0 9-9 9.75 9.75 0 0 0-6.74 2.74L3 8'), path('M3 3v5h5')],
  x: [path('M18 6 6 18'), path('m6 6 12 12')],
  // The model chip's two dropdowns (0.1.11): Lucide's `chevron-down`.
  chevronDown: [path('m6 9 6 6 6-6')],
  // The sidebar collapse toggle (responsive-sidebar S1): Lucide's `menu`.
  menu: [path('M3 6h18'), path('M3 12h18'), path('M3 18h18')],
  // The helper thread's Back, 01b's `i-back` symbol.
  back: [path('m12 19-7-7 7-7'), path('M19 12H5')],
  // The footer theme control, 01a's `i-theme` symbol: a half-filled circle.
  theme: [circle(12, 12, 10), path('M12 18a6 6 0 0 0 0-12v12z')],
  // The footer's gear button: Lucide's `settings` icon, the toothed outline and
  // its hub, drawn in the same two-shape style as the theme control above.
  settings: [
    path('M12.22 2h-.44a2 2 0 0 0-2 2v.18a2 2 0 0 1-1 1.73l-.43.25a2 2 0 0 1-2 0l-.15-.08a2 2 0 0 0-2.73.73l-.22.38a2 2 0 0 0 .73 2.73l.15.1a2 2 0 0 1 1 1.72v.51a2 2 0 0 1-1 1.74l-.15.09a2 2 0 0 0-.73 2.73l.22.38a2 2 0 0 0 2.73.73l.15-.08a2 2 0 0 1 2 0l.43.25a2 2 0 0 1 1 1.73V20a2 2 0 0 0 2 2h.44a2 2 0 0 0 2-2v-.18a2 2 0 0 1 1-1.73l.43-.25a2 2 0 0 1 2 0l.15.08a2 2 0 0 0 2.73-.73l.22-.39a2 2 0 0 0-.73-2.73l-.15-.08a2 2 0 0 1-1-1.74v-.5a2 2 0 0 1 1-1.74l.15-.09a2 2 0 0 0 .73-2.73l-.22-.38a2 2 0 0 0-2.73-.73l-.15.08a2 2 0 0 1-2 0l-.43-.25a2 2 0 0 1-1-1.73V4a2 2 0 0 0-2-2z'),
    circle(12, 12, 3),
  ],
  // The rail's Shell home entry: Lucide's `house`, the door and the roof wall.
  house: [
    path('M15 21v-8a1 1 0 0 0-1-1h-4a1 1 0 0 0-1 1v8'),
    path('M3 10a2 2 0 0 1 .709-1.528l7-5.999a2 2 0 0 1 2.582 0l7 5.999A2 2 0 0 1 21 10v9a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z'),
  ],
  // The rail's About entry: Lucide's `info`, the ring, the stem and the dot. The
  // same three-shape style as the two above.
  info: [circle(12, 12, 10), path('M12 16v-4'), path('M12 8h.01')],
  // The rail's Profiles entry (0.1.12): Lucide's `users`, two heads on one line.
  users: [
    path('M16 21v-2a4 4 0 0 0-4-4H6a4 4 0 0 0-4 4v2'),
    circle(9, 7, 4),
    path('M22 21v-2a4 4 0 0 0-3-3.87'),
    path('M16 3.13a4 4 0 0 1 0 7.75'),
  ],
  // The rail's Account entry (0.1.12): Lucide's `key`, the ring and the bit.
  key: [
    circle(7.5, 15.5, 5.5),
    path('m21 2-9.6 9.6'),
    path('m15.5 7.5 3 3L22 7l-3-3'),
  ],
}

/** The six chat states a session row can mark, as the contract names them. */
export type MarkKind = 'Working' | 'Queued' | 'Opening' | 'NewReply' | 'NeedsAttention' | 'Ended'

export const MARK_SHAPES: Record<MarkKind, Shape[]> = {
  Working: [path('M21 12a9 9 0 1 1-6.219-8.56')],
  Queued: [circle(12, 12, 10), path('M12 6v6l4 2')],
  Opening: [
    path('M12 2v4'),
    path('m16.2 7.8 2.9-2.9'),
    path('M18 12h4'),
    path('m16.2 16.2 2.9 2.9'),
    path('M12 18v4'),
    path('m4.9 19.1 2.9-2.9'),
    path('M2 12h4'),
    path('m4.9 4.9 2.9 2.9'),
  ],
  NewReply: [
    path('M2.992 16.342a2 2 0 0 1 .094 1.167l-1.065 3.29a1 1 0 0 0 1.236 1.168l3.413-.998a2 2 0 0 1 1.099.092 10 10 0 1 0-4.777-4.719'),
  ],
  NeedsAttention: [
    path('m21.73 18-8-14a2 2 0 0 0-3.48 0l-8 14A2 2 0 0 0 4 21h16a2 2 0 0 0 1.73-3'),
    path('M12 9v4'),
    path('M12 17h.01'),
  ],
  Ended: [circle(12, 12, 10), path('m15 9-6 6'), path('m9 9 6 6')],
}

/** The English name of each mark: the `aria-label`, never a bare colour. */
export const MARK_LABELS: Record<MarkKind, string> = {
  Working: 'Working',
  Queued: 'Queued',
  Opening: 'Opening',
  NewReply: 'New reply',
  NeedsAttention: 'Needs attention',
  Ended: 'Ended',
}

/** The one design-system token each mark takes its colour from. */
export const MARK_VARS: Record<MarkKind, string> = {
  Working: '--accent',
  Queued: '--muted',
  Opening: '--muted',
  NewReply: '--success',
  NeedsAttention: '--warning',
  Ended: '--danger',
}
