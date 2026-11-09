import { describe, expect, it } from 'vitest'
import { render } from '@testing-library/svelte'
import Icon from './Icon.svelte'
import type { IconName } from './shapes'

const NAMES: IconName[] = ['plus', 'search', 'message-square', 'folder', 'folder-plus']

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
})
