// How long ago a chat was written, in the sidebar's own words: the web mirror of
// Rust's `time_ago` (`crates/runtime/src/time_ago.rs`), which words every row's
// `modified_ago`. The thresholds and the unit suffixes are Rust's, exactly, so a
// chat cannot read "21m" in the sidebar and something else here.
//
// The unit suffixes (`m`, `h`, `d`, `mo`, `y`) are symbols in both languages; the
// one word is "now"/"ahora", which Rust keeps in
// `gentle_core::i18n::messages::time_now`, so this takes the language the caller
// holds. `nowMs` is the caller's clock: the welcome takes it ONCE when it renders
// and never ticks, so a screen left open says what it said when it opened.
import type { LanguageView } from '../contract/bindings'

/** "now", the one word of the rule, per language (Rust's `time_now`). */
const NOW: Record<LanguageView, string> = { en: 'now', es: 'ahora' }

const MINUTE_MS = 60_000

/** `thenMs` as a short age seen at `nowMs` (both epoch milliseconds). */
export function ago(language: LanguageView, nowMs: number, thenMs: number): string {
  // Rust saturates: a future time, or the epoch's own zero, is "now" rather than
  // a negative age, and nothing below can go below zero.
  const minutes = Math.floor(Math.max(0, nowMs - thenMs) / MINUTE_MS)
  const hours = Math.floor(minutes / 60)
  const days = Math.floor(hours / 24)
  if (minutes < 1) return NOW[language]
  if (hours < 1) return `${minutes}m`
  if (days < 1) return `${hours}h`
  if (days < 30) return `${days}d`
  if (days < 365) return `${Math.floor(days / 30)}mo`
  return `${Math.floor(days / 365)}y`
}
