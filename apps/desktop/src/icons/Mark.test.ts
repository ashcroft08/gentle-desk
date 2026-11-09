import { describe, expect, it } from 'vitest'
import { render, screen } from '@testing-library/svelte'
import Mark from './Mark.svelte'
import type { MarkKind } from './shapes'

const LABELS: [MarkKind, string][] = [
  ['Working', 'Working'],
  ['Queued', 'Queued'],
  ['Opening', 'Opening'],
  ['NewReply', 'New reply'],
  ['NeedsAttention', 'Needs attention'],
  ['Ended', 'Ended'],
]

describe('Mark', () => {
  it.each(LABELS)('draws %s as an image named "%s"', (kind, label) => {
    render(Mark, { props: { kind } })
    const mark = screen.getByRole('img', { name: label })
    expect(mark.tagName.toLowerCase()).toBe('svg')
    expect(mark.getAttribute('aria-label')).toBe(label)
  })

  it('draws a shape for every mark', () => {
    for (const [kind] of LABELS) {
      const { container, unmount } = render(Mark, { props: { kind } })
      const svg = container.querySelector('svg')
      expect(svg?.querySelectorAll('path, circle').length).toBeGreaterThan(0)
      expect(svg?.querySelector('title')?.textContent).toBeTruthy()
      unmount()
    }
  })
})
