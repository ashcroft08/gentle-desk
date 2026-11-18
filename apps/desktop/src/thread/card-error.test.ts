// A failed call must never lose the reason it failed. Two ways that used to happen
// (native review R3-001, CRITICAL): the payload's first recognised text field was
// an EMPTY string, and the failure text lived in `stderr`/`error` while
// `output`/`stdout` carried the tool's progress. Both put nothing - or the wrong
// line - where the reason belongs.
import { describe, expect, it } from 'vitest'
import { toolBody, textOf } from './card'

describe('a failed call names its reason', () => {
  it('reads the error before the output the tool printed first', () => {
    const body = toolBody({
      is_error: true,
      result: {
        output: 'Cloning into repo...\nreceiving objects',
        stderr: 'fatal: could not resolve host github.com',
      },
    })

    expect(body.error).toBe('fatal: could not resolve host github.com')
    // What was on stdout stays folded under the reason, rather than replacing it.
    expect(body.lines.join('\n')).toContain('Cloning into repo...')
  })

  it('does not mistake an empty field for the payload text', () => {
    const body = toolBody({
      is_error: true,
      result: { output: '', stderr: 'ERR_PNPM_NO_LOCKFILE' },
    })

    expect(body.error).toBe('ERR_PNPM_NO_LOCKFILE')
  })

  it('falls back to the normal read when the error keys hold nothing', () => {
    const body = toolBody({
      is_error: true,
      result: { output: 'the tool said why here', stderr: '' },
    })

    expect(body.error).toBe('the tool said why here')
  })

  it('a successful call keeps reading the output first', () => {
    const lines = textOf({ output: 'rows\nmore rows', error: 'ignored' })

    expect(lines.startsWith('rows')).toBe(true)
  })
})

describe('a payload with no text at all', () => {
  it('renders as an empty body, never as its own JSON', () => {
    // The targeted validator caught this: skipping the blank keys fell through to
    // `json(value)`, so a payload of blank strings put `{"output":""}` on the card.
    expect(textOf({ output: '' })).toBe('')
    expect(textOf({ output: '', stderr: '   ' })).toBe('')
  })

  it('a failed call with blanks shows no error line and no body', () => {
    const body = toolBody({ is_error: true, result: { output: '', stderr: '' } })

    expect(body.error).toBeNull()
    expect(body.lines).toEqual([])
  })
})
describe('a payload with blank text AND other data', () => {
  it('keeps the data instead of hiding it behind an empty body', () => {
    // R3-001 round 3: a blank `output` must not mask a structured error.
    expect(textOf({ output: '', error: { message: 'permission denied' } })).toContain('denied')
    expect(textOf({ output: '', exitCode: 1 })).toContain('exitCode')
  })
})
