import { describe, expect, it } from 'vitest'
import { render } from '@testing-library/svelte'
import Icon from './Icon.svelte'
import type { IconName } from './shapes'

const NAMES: IconName[] = [
  'plus',
  'search',
  'message-square',
  'folder',
  'folder-open',
  'folder-plus',
  'circle-plus',
  'more',
  'settings',
]

describe('Icon', () => {
  it('stays out of the accessibility tree', () => {
    const { container } = render(Icon, { props: { name: 'plus' } })
    const svg = container.querySelector('svg')
    expect(svg).not.toBeNull()
    expect(svg?.getAttribute('aria-hidden')).toBe('true')
    expect(svg?.getAttribute('role')).toBeNull()
  })

  it('draws the copied Lucide geometry of every named icon', () => {
    for (const name of NAMES) {
      const { container, unmount } = render(Icon, { props: { name } })
      expect(container.querySelectorAll('path, circle').length).toBeGreaterThan(0)
      unmount()
    }
  })

  it('draws the plus of the design artboard exactly', () => {
    const { container } = render(Icon, { props: { name: 'plus' } })
    const paths = [...container.querySelectorAll('path')].map((path) => path.getAttribute('d'))
    expect(paths).toEqual(['M5 12h14', 'M12 5v14'])
  })

  it('draws the Settings gear as one outline around its hub', () => {
    // The gear the footer's gear button draws (settings-dialog S5): Lucide's
    // `settings` icon, the outline plus the one circle, never a filled shape.
    const { container } = render(Icon, { props: { name: 'settings' } })
    const paths = [...container.querySelectorAll('path')].map((path) => path.getAttribute('d'))
    const hub = container.querySelector('circle')

    expect(paths.length).toBe(1)
    expect(paths[0]?.startsWith('M12.22 2h-.44')).toBe(true)
    expect(hub?.getAttribute('cx')).toBe('12')
    expect(hub?.getAttribute('cy')).toBe('12')
    expect(hub?.getAttribute('r')).toBe('3')
    expect(container.querySelector('rect')).toBeNull()
  })
})
