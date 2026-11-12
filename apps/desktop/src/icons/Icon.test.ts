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
  'house',
  'info',
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

  it('draws the Settings rail icons in the same Lucide style: a house and an info', () => {
    // The rail's Shell home and About and updates entries (settings-dialog L4):
    // Lucide's `house` and `info`, outline-only like the gear above, so the
    // three entries share one stroke and one 24x24 grid.
    const house = render(Icon, { props: { name: 'house' } })
    const housePaths = [...house.container.querySelectorAll('path')].map((path) =>
      path.getAttribute('d'),
    )
    expect(housePaths).toEqual([
      'M15 21v-8a1 1 0 0 0-1-1h-4a1 1 0 0 0-1 1v8',
      'M3 10a2 2 0 0 1 .709-1.528l7-5.999a2 2 0 0 1 2.582 0l7 5.999A2 2 0 0 1 21 10v9a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z',
    ])
    expect(house.container.querySelector('rect')).toBeNull()
    house.unmount()

    const info = render(Icon, { props: { name: 'info' } })
    const ring = info.container.querySelector('circle')
    expect([...info.container.querySelectorAll('path')].map((path) => path.getAttribute('d'))).toEqual(
      ['M12 16v-4', 'M12 8h.01'],
    )
    expect(ring?.getAttribute('r')).toBe('10')
    expect(info.container.querySelector('rect')).toBeNull()
  })
})
