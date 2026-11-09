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

/** The named icons the sidebar's own controls need. */
export type IconName =
  | 'plus'
  | 'search'
  | 'message-square'
  | 'folder'
  | 'folder-open'
  | 'folder-plus'
  | 'circle-plus'

const FOLDER = 'M20 20a2 2 0 0 0 2-2V8a2 2 0 0 0-2-2h-7.9a2 2 0 0 1-1.69-.9L9.6 3.9A2 2 0 0 0 7.93 3H4a2 2 0 0 0-2 2v13a2 2 0 0 0 2 2Z'
const FOLDER_OPEN =
  'm6 14 1.5-2.9A2 2 0 0 1 9.24 10H20a2 2 0 0 1 1.94 2.5l-1.54 6a2 2 0 0 1-1.95 1.5H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h3.9a2 2 0 0 1 1.69.9l.81 1.2a2 2 0 0 0 1.67.9H18a2 2 0 0 1 2 2v2'

export const ICON_SHAPES: Record<IconName, Shape[]> = {
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
