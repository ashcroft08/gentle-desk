// The composer's FILE attachments, as one seam the component reads (T4, S3,
// R6). Split out of Composer.svelte so the composer stays within its line
// budget and the attachment rules are testable on their own; the behaviour is
// the one `Composer.attachments.test.ts` pins.
import type { AttachmentRejectView, AttachmentView } from '../contract/bindings'
import {
  attachImages,
  describeAttachments,
  pickFiles,
} from '../bridge'
import { filesOf, firstRejection, imagesOf, merged } from './attach'

/** One composer's attachment state: the chips and the one rejection line. */
export type AttachmentDraft = {
  /** The FILES this message will carry, as the chips draw them. Images keep
   * their own road: their paths go to `attachImages` and Rust answers the
   * thumbnails. This list is the view's own state - Rust never sees a chip. */
  attached: AttachmentView[]
  /** Why the last clip or drop attached nothing, in the words of the language
   * the composer is drawn in (sheet D): ONE line, never a list. */
  attachNotice: string | null
}

/** An empty draft: no chips, no notice. */
export function emptyDraft(): AttachmentDraft {
  return { attached: [], attachNotice: null }
}

/**
 * Attaches what the clip chose or a drop named (T4, S3, R6): Rust says what
 * each path is, an image keeps the road it already had, every other file
 * becomes a chip, and a refused path draws ONE line instead. A picker or drop
 * that arrives after the chat changed attaches nothing: the chips belong to
 * the message of the chat they were chosen for.
 */
export async function attachPaths(
  state: { attached: AttachmentView[]; attachNotice: string | null },
  chat: string | null,
  paths: string[],
  folderPath: string | null,
  attachFailed: (name: string, reason: AttachmentRejectView) => string,
): Promise<AttachmentDraft | null> {
  if (chat === null || paths.length === 0) return null
  const views = await describeAttachments(paths, folderPath)
  if (views.length === 0) return null
  const images = imagesOf(views)
  if (images.length > 0) void attachImages(images)
  const refused = firstRejection(views)
  return {
    attached: merged(state.attached, filesOf(views)),
    attachNotice: refused === null ? null : attachFailed(refused.name, refused.reason),
  }
}

/** The clip: the system's file dialog, in this composer's own words. A
 * cancelled pick answers no path, so nothing changes. */
export async function pickPaths(title: string): Promise<string[]> {
  return pickFiles(title)
}

/** Takes one chip away before the send. Nothing is sent and nothing is read;
 * a file outside the chat's folder is attached like any other (R5). */
export function withoutPath(
  state: AttachmentDraft,
  path: string,
): AttachmentDraft {
  return {
    ...state,
    attached: state.attached.filter((view) => view.path !== path),
  }
}
