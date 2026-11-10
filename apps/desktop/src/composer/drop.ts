// The file drop the composer answers to.
//
// A dropped file's PATH exists only in Tauri's own drop event: the webview's
// `File` object carries the bytes and the name, never where the file is, so the
// page can attach an image only by NAMING it and letting Rust read it (S4).
// This module is that one boundary - it subscribes to the event and turns each
// payload into what the composer does about it - so the component draws a
// decision instead of reading a protocol.

import { getCurrentWebview } from '@tauri-apps/api/webview'

/** The part of a Tauri drag and drop payload this window reads. A drag that is
 * merely over the window has no paths; only `enter` and `drop` do. */
export type DragPayload = {
  type: 'enter' | 'over' | 'drop' | 'leave'
  paths?: string[]
}

/** What one drag and drop event asks of the composer. */
export type DropAction =
  | { kind: 'over' }
  | { kind: 'clear' }
  | { kind: 'drop'; paths: string[] }

/**
 * What the composer does with one payload: say it can take the drag, attach the
 * paths Rust will read, or stop saying anything. An unknown payload clears the
 * hint too, so the composer never keeps a hint for a drag that already ended.
 */
export function dropAction(payload: DragPayload): DropAction {
  switch (payload.type) {
    case 'enter':
    case 'over':
      return { kind: 'over' }
    case 'drop':
      return { kind: 'drop', paths: payload.paths ?? [] }
    default:
      return { kind: 'clear' }
  }
}

/**
 * Subscribes to Tauri's drop events. The answer unsubscribes, and it arrives as
 * a promise because the listener is registered over IPC: a component that goes
 * away first still unsubscribes when it lands.
 */
export function watchDrops(handle: (action: DropAction) => void): Promise<() => void> {
  return getCurrentWebview().onDragDropEvent((event) =>
    handle(dropAction(event.payload as DragPayload)),
  )
}
