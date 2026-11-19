// The composer's files as plain data (design/composer-dock T4, S3, R5, R6).
//
// Rust decides what each path the dialog or a drop named IS (`describe_attachments`);
// this module is what the composer does with that answer, kept out of the
// component so each rule is testable without a window:
//
// - the images keep the road they already had: their PATHS go to `attach_images`,
//   which reads them and serves their thumbnails (R6);
// - every other file becomes a chip, and the send carries its whole path below
//   the text, one per line;
// - a refused path is one line, never a chip;
// - a file outside the chat's folder is attached anyway (R5): nothing here blocks.

import type { AttachmentRejectView, AttachmentView } from '../contract/bindings'

/** The most chips drawn before the row collapses into a "+N more" button. */
export const MAX_VISIBLE = 3

/** The paths that still travel as images, in the order they were named. */
export function imagesOf(views: AttachmentView[]): string[] {
  return views.filter((view) => view.kind.type === 'Image').map((view) => view.path)
}

/** The files that became chips, in the order they were named. */
export function filesOf(views: AttachmentView[]): AttachmentView[] {
  return views.filter((view) => view.kind.type === 'File')
}

/** Why `view` was not attached, `null` when it was. */
export function rejectionOf(view: AttachmentView): AttachmentRejectView | null {
  return view.kind.type === 'Rejected' ? view.kind.data : null
}

/**
 * The one line a refused path draws: the first one named, with its own reason.
 * Artboard D draws one line, never a list of everything that failed.
 */
export function firstRejection(
  views: AttachmentView[],
): { name: string; reason: AttachmentRejectView } | null {
  for (const view of views) {
    const reason = rejectionOf(view)
    if (reason !== null) return { name: view.name, reason }
  }
  return null
}

/**
 * What is attached now plus what just arrived, with a file attached once: the
 * first of a repeated path wins, so the same file never rides twice (sheet D).
 */
export function merged(existing: AttachmentView[], adding: AttachmentView[]): AttachmentView[] {
  const seen = new Set(existing.map((view) => view.path))
  const fresh = adding.filter((view) => {
    if (seen.has(view.path)) return false
    seen.add(view.path)
    return true
  })
  return [...existing, ...fresh]
}

/**
 * The text a send dispatches: the typed draft with the chips' whole paths
 * appended below it, one per line. A box that is empty (or only spaces) sends
 * the paths alone; no chip means the text is dispatched exactly as it was.
 */
export function promptWithPaths(text: string, paths: string[]): string {
  const typed = text.trim()
  if (paths.length === 0) return typed
  const list = paths.join('\n')
  return typed === '' ? list : `${typed}\n${list}`
}

/**
 * The chips drawn right now, and how many the collapsed row hides: the first
 * [`MAX_VISIBLE`] until the row is expanded, then all of them.
 */
export function collapsedOf(
  views: AttachmentView[],
  expanded: boolean,
  max = MAX_VISIBLE,
): { shown: AttachmentView[]; hidden: number } {
  if (views.length <= max) return { shown: views, hidden: 0 }
  // `hidden` is what the button names, so it stays the same number while the row
  // is open; `shown` is what the row draws right now.
  return { shown: expanded ? views : views.slice(0, max), hidden: views.length - max }
}
