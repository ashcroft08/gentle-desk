// The context chip's rules, one test per rule (0.1.12, S5): a null is never a
// zero, so "just compacted" draws a dash and "nothing read yet" draws nothing at
// all; the scale turns at 75% and 95%; and the token detail is the hover text.
import { describe, expect, it } from 'vitest'
import type { ContextUsageView_Serialize as ContextUsageView } from '../contract/bindings'
import { barWidth, chipOf, toneOf } from './context'

function usage(over: Partial<ContextUsageView> = {}): ContextUsageView {
  return { tokens: null, percent: null, window: null, ...over }
}

const READING = usage({ tokens: 70_000, percent: 35, window: 200_000 })

describe('the context chip', () => {
  it('draws the percentage pi computed, in the quiet tone', () => {
    const chip = chipOf(READING)

    expect(chip).toMatchObject({ kind: 'reading', percent: 35, tone: 'normal' })
    expect(chip.kind === 'reading' && chip.label).toContain('35')
  })

  it('turns to the warning tone from 75 percent and the danger tone from 95', () => {
    expect(toneOf(74.9)).toBe('normal')
    expect(toneOf(75)).toBe('warning')
    expect(toneOf(94.9)).toBe('warning')
    expect(toneOf(95)).toBe('danger')
    expect(toneOf(100)).toBe('danger')
  })

  it('rounds the percentage it draws, so the number and the bar agree', () => {
    const chip = chipOf(usage({ tokens: 1200, percent: 34.6, window: 3500 }))
    expect(chip).toMatchObject({ kind: 'reading', percent: 35 })
    expect(barWidth(35)).toBe('35%')
    expect(barWidth(104)).toBe('100%')
    expect(barWidth(-2)).toBe('0%')
  })

  it('draws a dash when only the window is known, never a zero', () => {
    // The state pi answers right after a compaction.
    const chip = chipOf(usage({ tokens: null, percent: null, window: 200_000 }))

    expect(chip.kind).toBe('unknown')
    expect(chip.kind === 'unknown' && chip.label).not.toContain('0')
  })

  it('draws nothing at all when there is no reading', () => {
    expect(chipOf(null).kind).toBe('hidden')
    expect(chipOf(usage()).kind).toBe('hidden')
  })

  it('keeps the tokens and the window as the chip hover, not in the way', () => {
    expect(chipOf(READING)).toMatchObject({ title: '70k / 200k tokens' })
    // A known token count without a window still says what it can.
    expect(chipOf(usage({ tokens: 950 }))).toMatchObject({ title: '950 tokens' })
    // A compaction leaves nothing to describe, so there is no hover text.
    expect(chipOf(usage({ window: 200_000 }))).toMatchObject({ title: '' })
  })
})
