// The helper thread a component test draws: the header iced echoes above the
// items, and one builder per item kind, as `ViewState.helper_thread` carries
// them.

import type {
  HelperThreadItemView,
  HelperThreadView,
} from '../contract/bindings'

/** The thread of one helper, with an empty body unless `over` says otherwise. */
export function helperThread(
  over: Partial<HelperThreadView> = {},
): HelperThreadView {
  return {
    id: 'h1',
    title: 'Inventory of sidebar modules',
    agent: 'explore',
    status: 'Running',
    status_label: 'running',
    prompt: 'List what the bridge must expose.',
    dropped: 0,
    items: [],
    ...over,
  }
}

/** What the helper said: markdown, drawn as the chat thread is. */
export function saidItem(text: string): HelperThreadItemView {
  return { kind: 'Said', text }
}

/** The helper's reasoning, drawn muted. */
export function thoughtItem(text: string): HelperThreadItemView {
  return { kind: 'Thought', text }
}

/** A note the helper left, drawn as plain muted text. */
export function noteItem(text: string): HelperThreadItemView {
  return { kind: 'Note', text }
}

/** One tool call, with the strings Rust already built. */
export function ranItem(
  name: string,
  over: Partial<Extract<HelperThreadItemView, { kind: 'Ran' }>> = {},
): HelperThreadItemView {
  return {
    kind: 'Ran',
    name,
    status: 'Ok',
    args_line: 'path=rpc.md',
    output_preview: 'wrote 12 lines',
    ...over,
  }
}
