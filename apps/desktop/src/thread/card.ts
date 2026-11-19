// What a tool card has to decide before anything is drawn (chat-transcript T2b,
// L4): which kind of tool a call is - and therefore which glyph and which chip it
// carries -, what its ONE monospace subject line says, which lines of the payload
// are its body, how many of them are folded away, and how its elapsed time reads.
//
// The rules live here and not in the component for the reason L14 gives the bound:
// a chat of 400 tool calls redraws a bounded list on every push, so a row does a
// FIXED amount of work - string slicing and counting - and never measures the DOM.
// Nothing here is language-dependent: the words the card draws are the thread's
// dictionary (`copy.ts`), and the tool's own name, command and output arrive from
// the child exactly as written (D7).
//
// Nothing is invented either: a call with no duration states its state alone, a
// payload with no text is printed as its JSON, and a tool whose name implies no
// language gets no chip.

import type { ToolCallItemView_Serialize as ToolCall } from '../contract/bindings'

/** The kinds the card draws a glyph for. */
export type ToolKind = 'shell' | 'search' | 'edit' | 'read' | 'other'

/** How many lines of a body are shown before the head is folded away (L14). */
export const FOLD_LINES = 20

/** The longest subject line a card builds before it elides its own summary. */
const SUBJECT_LIMIT = 180

/** The longest string the argument summary keeps before it elides it. */
const VALUE_LIMIT = 32

/**
 * The words a tool's NAME is built from, lowercased and split on anything that is
 * not a letter or a digit: `apply_patch` is an `apply` and a `patch`, and the
 * table answers for the segment it knows.
 */
const KINDS: Record<string, ToolKind> = {
  bash: 'shell',
  sh: 'shell',
  shell: 'shell',
  zsh: 'shell',
  pwsh: 'shell',
  powershell: 'shell',
  cmd: 'shell',
  terminal: 'shell',
  run: 'shell',
  exec: 'shell',
  execute: 'shell',
  grep: 'search',
  rg: 'search',
  ripgrep: 'search',
  search: 'search',
  find: 'search',
  glob: 'search',
  ls: 'search',
  list: 'search',
  edit: 'edit',
  write: 'edit',
  patch: 'edit',
  replace: 'edit',
  read: 'read',
  cat: 'read',
  view: 'read',
  open: 'read',
}

/**
 * The chips a tool name implies: the name of a language (`bash`) or of a search
 * dialect (`regex`). A name that implies neither gets no chip - the artboard draws
 * one on Bash and Grep and on nothing else.
 */
const CHIPS: Record<string, string> = {
  bash: 'bash',
  sh: 'bash',
  shell: 'bash',
  zsh: 'bash',
  pwsh: 'bash',
  powershell: 'bash',
  cmd: 'bash',
  terminal: 'bash',
  run: 'bash',
  exec: 'bash',
  execute: 'bash',
  grep: 'regex',
  rg: 'regex',
  ripgrep: 'regex',
}

/** The argument keys a shell-like tool keeps its command under. */
const COMMAND_KEYS = ['command', 'cmd', 'script', 'input']

/** The keys a payload keeps its own text under, in the order they are tried. */
const TEXT_KEYS = ['output', 'stdout', 'stderr', 'text', 'content', 'message', 'error']

/**
 * The same keys for a FAILED call, tried error-first.
 *
 * A failure's reason lives in `stderr` or `error` far more often than in the
 * progress it printed to `output`/`stdout` first: reading the normal order would
 * put the tool's progress line where the reason belongs, which is the one line a
 * card promises never to lose (native review R3-001).
 */
const ERROR_KEYS = ['error', 'stderr', 'message', 'output', 'stdout', 'text', 'content']

/** The segments of a tool's name, lowercased, each non-alphanumeric a separator. */
function segments(name: string): string[] {
  return name
    .trim()
    .toLowerCase()
    .split(/[^a-z0-9]+/)
    .filter((segment) => segment !== '')
}

/** The kind of tool `name` names, or `other` when it names none of them. */
export function toolKind(name: string): ToolKind {
  for (const segment of segments(name)) {
    const kind = KINDS[segment]
    if (kind !== undefined) return kind
  }
  return 'other'
}

/** The chip `name` implies, or `null` when the name implies no language. */
export function kindChip(name: string): string | null {
  for (const segment of segments(name)) {
    const chip = CHIPS[segment]
    if (chip !== undefined) return chip
  }
  return null
}

/** One subject line: its text, and whether it is a shell command to prompt for. */
export type ToolSubject = { text: string; shell: boolean }

/**
 * The ONE line a card always shows: a shell-like tool's own command, or every
 * other tool's name followed by a compact summary of its arguments - the artboard's
 * `lookup({ long_key_that_collided: … })`.
 */
export function toolSubject(tool: Pick<ToolCall, 'tool_name' | 'args'>): ToolSubject {
  const command = toolKind(tool.tool_name) === 'shell' ? commandOf(tool.args) : null
  if (command !== null) return { text: command, shell: true }
  const summary = summarize(tool.args)
  return { text: summary === '' ? tool.tool_name : `${tool.tool_name}(${summary})`, shell: false }
}

/** The command a shell-like tool was given, when it was given one. */
function commandOf(args: unknown): string | null {
  if (typeof args === 'string') {
    const text = args.trim()
    return text === '' ? null : text
  }
  if (!isRecord(args)) return null
  for (const key of COMMAND_KEYS) {
    const value = args[key]
    if (typeof value === 'string' && value.trim() !== '') return value
  }
  return null
}

/** A compact, bounded one-line summary of a tool's arguments. */
export function summarize(args: unknown): string {
  if (args === null || args === undefined) return ''
  const text = typeof args === 'string' ? args : compactJson(collapse(args))
  const flat = text.replace(/\s+/g, ' ').trim()
  return flat.length > SUBJECT_LIMIT ? `${flat.slice(0, SUBJECT_LIMIT)}…` : flat
}

/**
 * A copy of `value` with every long string elided and every collection capped, so
 * summarizing a payload never walks a whole output. Two levels deep is all a
 * one-line summary can honestly show.
 */
function collapse(value: unknown, depth = 0): unknown {
  if (typeof value === 'string') {
    return value.length > VALUE_LIMIT ? `${value.slice(0, VALUE_LIMIT)}…` : value
  }
  if (value === null || typeof value !== 'object') return value
  if (depth >= 2) return '…'
  if (Array.isArray(value)) return value.slice(0, 4).map((item) => collapse(item, depth + 1))
  const out: Record<string, unknown> = {}
  for (const [key, item] of Object.entries(value).slice(0, 6)) {
    out[key] = collapse(item, depth + 1)
  }
  return out
}

/** The body of a card: its lines, and the error line a failure carries apart. */
export type ToolBody = { error: string | null; lines: string[] }

/**
 * What a card's body draws: the payload's own text as lines, and - when the child
 * called it a failure - its FIRST line lifted out as the error line, because a
 * folded body must never swallow the reason a call failed. The rest stays folded
 * under it, and an output that fits is untouched.
 */
export function toolBody(
  tool: Pick<ToolCall, 'result' | 'partial_result' | 'is_error'>,
): ToolBody {
  const payload = tool.result ?? tool.partial_result ?? null
  const normal = splitLines(textOf(payload))
  if (!tool.is_error) return { error: null, lines: normal }
  // A failure is read error-first, so the REASON becomes the error line and never
  // the progress the tool happened to print first (native review R3-001). The
  // progress is not thrown away: it stays folded under the reason.
  const failure = splitLines(textOf(payload, ERROR_KEYS))
  const [first, ...rest] = failure.length > 0 ? failure : normal
  if (first === undefined) return { error: null, lines: [] }
  const kept = failure.length > 0 ? rest : []
  // Only appended when the two reads disagree, which is exactly the case where the
  // error lives in another field than the output.
  const tail = failure.length > 0 && failure.join('\n') !== normal.join('\n') ? normal : []
  return { error: first, lines: [...kept, ...tail] }
}

/** A payload's text as lines, with the tool's own final newline dropped. */
function splitLines(text: string): string[] {
  const lines = text.split('\n')
  // A tool's own final newline is not an empty last line.
  if (lines.length > 0 && lines[lines.length - 1] === '') lines.pop()
  return lines
}

/** The text of a payload: its own, or its JSON when it carries none. */
export function textOf(value: unknown, keys: readonly string[] = TEXT_KEYS): string {
  if (value === null || value === undefined) return ''
  if (typeof value === 'string') return value
  if (typeof value === 'number' || typeof value === 'boolean') return String(value)
  if (Array.isArray(value)) {
    // A content list of text parts is its text, one part per line.
    const parts = value.filter((part) => isRecord(part) && typeof part.text === 'string')
    if (parts.length === value.length && parts.length > 0) {
      return parts.map((part) => String((part as Record<string, unknown>).text)).join('\n')
    }
    return json(value)
  }
  if (isRecord(value)) {
    // A tool result is usually `{ content: [{ type: 'text', text }], ... }`: its
    // answer lives one level DOWN. Rendering the raw JSON instead buries the text
    // the user asked for (maintainer's screenshot, 2026-11-18).
    const parts = value.content
    if (
      Array.isArray(parts) &&
      parts.length > 0 &&
      parts.every((part) => isRecord(part) && typeof part.text === 'string')
    ) {
      return parts.map((part) => String((part as Record<string, unknown>).text)).join('\n')
    }
    // A key whose value IS a string settles the question even when it is blank: an
    // all-blank payload has no text, so it renders as an empty body - NOT as its
    // own JSON, which would put `{"output":""}` on the card (review regression of
    // R3-001, found by the targeted validator).
    let sawString = false
    for (const key of keys) {
      const text = value[key]
      if (typeof text !== 'string') continue
      sawString = true
      // An EMPTY string is not the payload's text: taking it would hand back a
      // blank body while a later key held the real one (native review R3-001).
      if (text.trim() !== '') return text
    }
    // Only a payload whose EVERY value is blank has no text at all; anything else
    // keeps its JSON, which may hold the only reason a call failed (R3-001, round 3:
    // `{ output: '', error: { message: 'denied' } }` must still say "denied").
    if (sawString && Object.values(value).every((v) => typeof v === 'string' && v.trim() === ''))
      return ''
  }
  return json(value)
}

/** `value` as one compact line of JSON, for a subject a card draws in one line. */
function compactJson(value: unknown): string {
  try {
    return JSON.stringify(value) ?? ''
  } catch {
    return ''
  }
}

/** `value` as indented JSON, or nothing when it cannot be written as such. */
function json(value: unknown): string {
  try {
    return JSON.stringify(value, null, 2) ?? ''
  } catch {
    return ''
  }
}

/** Whether `value` is a plain object to read keys from. */
function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null && !Array.isArray(value)
}

/** How a body is drawn right now: the lines on screen and how many are hidden. */
export type Fold = { shown: string[]; hidden: number }

/**
 * The fold: a closed body keeps its TAIL - the end of an output is the part a
 * reader is after - and counts the earlier lines it hid, exactly as the artboard's
 * "14 earlier lines" head does. An open body shows everything.
 */
export function foldLines(lines: string[], open: boolean): Fold {
  if (open || lines.length <= FOLD_LINES) return { shown: lines, hidden: 0 }
  return {
    shown: lines.slice(lines.length - FOLD_LINES),
    hidden: lines.length - FOLD_LINES,
  }
}

/**
 * How long a row took, as the card's footer reads it ("0.2s"), or `null` when the
 * child's clock never saw it - the footer then draws the state alone rather than a
 * number nothing measured.
 */
export function formatDuration(ms: number | null | undefined): string | null {
  if (ms === null || ms === undefined || !Number.isFinite(ms)) return null
  return `${(Math.max(0, ms) / 1000).toFixed(1)}s`
}
