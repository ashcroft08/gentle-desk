// The helpers panel a component test draws: one card of the stand-in's frame
// per helper status, and the counters line Rust spells.

import type { HelperRowView_Serialize, HelpersView_Serialize } from '../contract/bindings'

/** The middle dot Rust's counters line uses, as an escape so this file stays ASCII. */
const DOT = '\u00b7'

/** The counters line Rust sends, spelled the way `helper_text::summary_line` does. */
export function summaryLine(counts: Partial<HelpersView_Serialize> = {}): string {
  const { running = 0, queued = 0, waiting = 0, finished = 0 } = counts
  return `${running} running ${DOT} ${queued} queued ${DOT} ${waiting} waiting ${DOT} ${finished} finished`
}

/** One helper card, as `ViewState.helpers` carries it. */
export function helperRow(
  id: string,
  over: Partial<HelperRowView_Serialize> = {},
): HelperRowView_Serialize {
  return {
    id,
    title: `${id} task`,
    agent: 'explore',
    status: 'Running',
    status_label: 'running',
    turns: 2,
    tool_calls: 1,
    last_step: null,
    error: null,
    ...over,
  }
}

/** The helpers panel of the foreground chat, with a counters line that matches it. */
export function helpersPanel(
  tasks: HelperRowView_Serialize[] = [],
  over: Partial<HelpersView_Serialize> = {},
): HelpersView_Serialize {
  return {
    summary_line: summaryLine(over),
    running: 0,
    queued: 0,
    waiting: 0,
    finished: 0,
    tasks,
    ...over,
  }
}

