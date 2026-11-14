// The Usage pane's rules as plain data (settings-0.1.12 S6's deferred half):
// what one read of the shell's own usage sources draws, decided once here so the
// states are testable without a window.
//
// The rule that matters most is the same one the context chip keeps: a provider
// that answered "no data" is NO DATA and never a zero percent. The numbers
// themselves are the sources' own readings, rounded the way the shell's panel
// rounds them; the gauge's tone is the reader's own scale, so colour never
// carries a reading alone.

import type {
  UsageProviderStatusView,
  UsageView_Serialize as UsageView,
} from '../contract/bindings'
import { barWidth, toneOf, type ContextTone } from '../composer/context'

/** One window of a limit, as the gauge and its line draw it. */
export interface UsageWindowRow {
  label: string
  /** The share used, rounded to a whole percent like the context chip. */
  percent: number
  /** The bar's width, clamped to its own ends. */
  width: string
  /** The reader's own scale: quiet, warning from 75%, danger from 95%. */
  tone: ContextTone
  /** The countdown the shell computed ("resets in 4h"), or null when none. */
  resets: string | null
}

/** One limit: its name, whether the provider marked it reached, and its windows. */
export interface UsageLimitRow {
  name: string
  reached: boolean
  windows: UsageWindowRow[]
}

/** One provider, as its card draws it. */
export interface UsageProviderRow {
  provider: string
  status: UsageProviderStatusView
  plan: string | null
  error: string | null
  limits: UsageLimitRow[]
}

/** Which of the pane's five drawn states it is in. */
export type UsagePaneState =
  /** Nothing read yet, or the read is running: one quiet line. */
  | { kind: 'reading' }
  /** The sources answered: one card per provider. */
  | { kind: 'installed'; providers: UsageProviderRow[] }
  /** The sources are not installed for this shell: the install button. */
  | { kind: 'missing' }
  /** Something could not be read: the reason and the safe retry. */
  | { kind: 'unreadable'; reason: string }
  /** The sources ran and their answer could not be used: reason and retry. */
  | { kind: 'failed'; reason: string }

/** The pane's whole state, from the slice Rust pushed. */
export function paneOf(usage: UsageView | null): UsagePaneState {
  if (usage === null) return { kind: 'reading' }
  const status = usage.status
  if (status.type === 'Installed') {
    return { kind: 'installed', providers: worthDrawing(usage.providers.map(providerRow)) }
  }
  if (status.type === 'Missing') return { kind: 'missing' }
  if (status.type === 'Unreadable') return { kind: 'unreadable', reason: status.data }
  if (status.type === 'Failed') return { kind: 'failed', reason: status.data }
  // `Unknown` and `Reading` are both "nothing to draw yet".
  return { kind: 'reading' }
}

/** The highest percentage a provider reported, which is how the cards sort. */
function peakOf(provider: UsageProviderRow): number {
  return provider.limits.reduce(
    (peak, limit) => Math.max(peak, ...limit.windows.map((window) => window.percent)),
    0,
  )
}

/**
 * What the pane actually draws: the providers that HAVE a reading.
 *
 * The shell's own panel lists a provider only once its source answered with
 * numbers, so a provider that answered "no data" (a machine without that
 * provider's credentials) is not drawn at all here either - the maintainer saw
 * the pane list providers he has never signed in to, which is noise, not
 * information. A provider that FAILED stays, with its reason: that one is worth
 * reading, because it explains missing numbers. The cards then sort by what is
 * actually being spent, so the reading that matters is the first one: peak
 * usage first, then the quiet ones, then failures.
 */
export function worthDrawing(providers: UsageProviderRow[]): UsageProviderRow[] {
  return providers
    .filter((provider) => provider.status !== 'NoData')
    .sort((left, right) => {
      const failed = Number(left.status === 'Failed') - Number(right.status === 'Failed')
      if (failed !== 0) return failed
      const peak = peakOf(right) - peakOf(left)
      if (peak !== 0) return peak
      return left.provider.localeCompare(right.provider)
    })
}

/** One provider and its limits. */
function providerRow(provider: UsageView['providers'][number]): UsageProviderRow {
  return {
    provider: provider.provider,
    status: provider.status,
    plan: provider.plan,
    error: provider.error,
    limits: provider.limits.map((limit) => ({
      name: limit.name,
      reached: limit.limit_reached,
      windows: limit.windows.flatMap((window) => {
        // The generated type marks every float nullable (JSON cannot carry each
        // one), and the shell's parser already refuses a window without a
        // percentage: a null here is no reading at all, so the window is left
        // out of the card rather than drawn as 0%.
        if (window.used_percent === null) return []
        const percent = Math.round(window.used_percent)
        return [
          {
            label: window.label,
            percent,
            width: barWidth(percent),
            tone: toneOf(percent),
            resets: window.resets_in,
          },
        ]
      }),
    })),
  }
}
