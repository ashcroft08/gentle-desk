// The age of a chat, worded exactly as the sidebar words it: these are the cases
// of Rust's own `time_ago` (`crates/runtime/src/time_ago/tests.rs`, the rule the
// rows' `modified_ago` is printed by), so the welcome can never drift from the
// sidebar. The unit suffixes are symbols in both languages; the one word is
// "now"/"ahora", which Rust keeps in `gentle_core::i18n::messages`.
import { describe, expect, it } from 'vitest'
import type { LanguageView } from '../contract/bindings'
import { ago } from './ago'

const SEC = 1000
const MIN = 60 * SEC
const HOUR = 60 * MIN
const DAY = 24 * HOUR
/** The same instant Rust's tests hold. */
const NOW = 1_790_000_000_000

/** `delta` milliseconds before NOW, as the sidebar would word it. */
function age(delta: number, language: LanguageView = 'en'): string {
  return ago(language, NOW, NOW - delta)
}

describe('the age of a chat', () => {
  it('is "now" under a minute, in the language the caller holds', () => {
    expect(age(0)).toBe('now')
    expect(age(59 * SEC)).toBe('now')
    expect(age(59 * SEC, 'es')).toBe('ahora')
  })

  it('is "now" for a time in the future', () => {
    expect(ago('en', NOW, NOW + 5 * MIN)).toBe('now')
    expect(ago('es', NOW, NOW + 5 * MIN)).toBe('ahora')
  })

  it('counts minutes, hours and days down', () => {
    expect(age(MIN)).toBe('1m')
    expect(age(59 * MIN + 59 * SEC)).toBe('59m')
    expect(age(HOUR)).toBe('1h')
    expect(age(23 * HOUR + 59 * MIN)).toBe('23h')
    expect(age(DAY)).toBe('1d')
    expect(age(29 * DAY + 23 * HOUR)).toBe('29d')
  })

  it('counts months and years past thirty days', () => {
    expect(age(30 * DAY)).toBe('1mo')
    expect(age(364 * DAY)).toBe('12mo')
    expect(age(365 * DAY)).toBe('1y')
    expect(age(800 * DAY)).toBe('2y')
  })

  it("words an unknown zero time as Rust does, without panicking", () => {
    expect(ago('en', NOW, 0)).toBe('56y')
  })
})
