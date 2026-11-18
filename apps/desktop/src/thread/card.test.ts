// What a tool row has to decide before anything is drawn (chat-transcript T2b):
// which kind of tool it is (and therefore which glyph and which chip), what the
// one monospace subject line says, which lines of the payload are the body, how
// many of them are folded away, and how the elapsed time reads. The rules live
// here, out of the component, so a 400-row chat does the same fixed amount of
// work per row and never measures the DOM.
import { describe, expect, it } from 'vitest'
import {
  FOLD_LINES,
  foldLines,
  formatDuration,
  kindChip,
  toolBody,
  toolKind,
  toolSubject,
} from './card'

describe('the kind of a tool', () => {
  it.each([
    ['bash', 'shell'],
    ['Bash', 'shell'],
    ['run_command', 'shell'],
    ['powershell', 'shell'],
    ['grep', 'search'],
    ['ripgrep', 'search'],
    ['apply_patch', 'edit'],
    ['write_file', 'edit'],
    ['read_file', 'read'],
  ] as const)('reads %s as %s', (name, kind) => {
    expect(toolKind(name)).toBe(kind)
  })

  it('draws no kind it does not know', () => {
    expect(toolKind('lookup')).toBe('other')
    expect(toolKind('')).toBe('other')
  })

  it('chips only the names that name a language', () => {
    expect(kindChip('bash')).toBe('bash')
    // The artboard's Grep card carries a `regex` chip: a search dialect, not prose.
    expect(kindChip('grep')).toBe('regex')
    expect(kindChip('read')).toBeNull()
    expect(kindChip('lookup')).toBeNull()
  })
})

describe('the subject line of a tool card', () => {
  it('shows a shell-like tool its own command, marked as a shell line', () => {
    const subject = toolSubject({
      tool_name: 'bash',
      args: { command: 'cd "…/.worktrees/pos-tienda/frontend" && ls node_modules' },
    })

    expect(subject).toEqual({
      text: 'cd "…/.worktrees/pos-tienda/frontend" && ls node_modules',
      shell: true,
    })
  })

  it('takes a command out of a string argument too', () => {
    const subject = toolSubject({ tool_name: 'bash', args: 'ls node_modules' })

    expect(subject).toEqual({ text: 'ls node_modules', shell: true })
  })

  it('summarises the arguments of every other tool after its name', () => {
    const subject = toolSubject({ tool_name: 'read', args: { path: 'src/app.css' } })

    expect(subject.shell).toBe(false)
    expect(subject.text).toBe('read({"path":"src/app.css"})')
  })

  it('names a tool that carried no argument at all', () => {
    expect(toolSubject({ tool_name: 'read', args: null })).toEqual({ text: 'read', shell: false })
  })

  it('cuts a summary that would be longer than a line', () => {
    const long = 'x'.repeat(400)
    const subject = toolSubject({ tool_name: 'read', args: { path: long } })

    expect(subject.text.length).toBeLessThan(220)
    // The argument itself is elided, so no line of a card ever grows with its payload.
    expect(subject.text).toContain('…')
    expect(subject.text).not.toContain('x'.repeat(64))
  })
})

describe('the body of a tool card', () => {
  it('takes the output of a result, one line each, without the tool’s final newline', () => {
    const body = toolBody({ result: 'mocker.d.ts\nnode.d.ts\n', partial_result: null, is_error: false })

    expect(body).toEqual({ error: null, lines: ['mocker.d.ts', 'node.d.ts'] })
  })

  it('reads the output while the call is still running', () => {
    const body = toolBody({
      result: null,
      partial_result: 'Waiting for OneDrive to release the lock…',
      is_error: false,
    })

    expect(body.lines).toEqual(['Waiting for OneDrive to release the lock…'])
  })

  it('pulls the text out of a payload object instead of printing its JSON', () => {
    expect(toolBody({ result: { output: 'opus 1.2', exit_code: 0 }, is_error: false }).lines).toEqual([
      'opus 1.2',
    ])
    expect(toolBody({ result: { error: 'ERR_PNPM_NO_LOCKFILE' }, is_error: true })).toEqual({
      error: 'ERR_PNPM_NO_LOCKFILE',
      lines: [],
    })
  })

  it('prints a payload with no text of its own as JSON', () => {
    const body = toolBody({ result: { rows: [{ id: 1 }] }, is_error: false })

    expect(body.lines.join('\n')).toContain('"rows"')
  })

  it('makes the first line of a failed call its error line, and keeps the rest', () => {
    const body = toolBody({
      result: 'ERR_PNPM_NO_LOCKFILE  Cannot install with "frozen-lockfile"\n  at install (pnpm.js:1)',
      is_error: true,
    })

    expect(body.error).toBe('ERR_PNPM_NO_LOCKFILE  Cannot install with "frozen-lockfile"')
    expect(body.lines).toEqual(['  at install (pnpm.js:1)'])
  })

  it('has nothing to draw for a call with no payload', () => {
    expect(toolBody({ result: null, partial_result: null, is_error: false })).toEqual({
      error: null,
      lines: [],
    })
  })
})

describe('the fold of a body', () => {
  const lines = Array.from({ length: 34 }, (_, i) => `line ${i + 1}`)

  it('folds at twenty lines and keeps the tail', () => {
    const fold = foldLines(lines, false)

    expect(FOLD_LINES).toBe(20)
    expect(fold.shown).toHaveLength(20)
    expect(fold.shown[0]).toBe('line 15')
    expect(fold.hidden).toBe(14)
  })

  it('hides nothing when the body already fits', () => {
    expect(foldLines(lines.slice(0, 20), false)).toEqual({
      shown: lines.slice(0, 20),
      hidden: 0,
    })
  })

  it('shows every line once it is open', () => {
    expect(foldLines(lines, true)).toEqual({ shown: lines, hidden: 0 })
  })
})

describe('the elapsed time of a row', () => {
  it.each([
    [0, '0.0s'],
    [200, '0.2s'],
    [4000, '4.0s'],
    [12800, '12.8s'],
  ])('reads %ims as %s', (ms, text) => {
    expect(formatDuration(ms)).toBe(text)
  })

  it('never invents a number for a call with no clock', () => {
    expect(formatDuration(null)).toBeNull()
    expect(formatDuration(undefined)).toBeNull()
    expect(formatDuration(Number.NaN)).toBeNull()
  })
})
