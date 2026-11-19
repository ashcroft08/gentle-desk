// The folder chip (design/composer-dock S2, R4): the LAST SEGMENT of the chat's
// working directory - the same name its sidebar row draws - with the whole path
// only on hover. No box around it: it is text in the composer's row, and the 1px
// hairline that separates where the message runs from what it runs on belongs to
// the group, so a chat with no folder leaves no dangling line.
import { afterEach, describe, expect, it } from 'vitest'
import { cleanup, render } from '@testing-library/svelte'
import FolderChip from './FolderChip.svelte'

afterEach(cleanup)

const FOLDER = { name: 'Kallari_Pagina_web', path: '/dev/proyectos/Kallari_Pagina_web' }

describe('the folder chip', () => {
  it('draws the last segment only, and keeps the whole path on hover', () => {
    const { container } = render(FolderChip, { props: { folder: FOLDER } })
    const chip = container.querySelector('.folder') as HTMLElement

    expect(chip.textContent?.trim()).toBe(FOLDER.name)
    expect(chip.textContent).not.toContain('/dev/proyectos')
    expect(chip.getAttribute('title')).toBe(FOLDER.path)
  })

  it('is text, not a control: nothing to click and nothing to focus', () => {
    const { container } = render(FolderChip, { props: { folder: FOLDER } })

    expect(container.querySelector('button')).toBeNull()
    expect(container.querySelector('[tabindex]')).toBeNull()
  })

  it('leads its group with the hairline that separates what from where', () => {
    const { container } = render(FolderChip, { props: { folder: FOLDER } })
    const parts = Array.from(container.children).map((child) => child.className)

    expect(parts).toEqual(['vsep', 'folder'])
  })

  it('draws neither the hairline nor a placeholder when there is no folder', () => {
    const { container } = render(FolderChip, { props: { folder: null } })

    expect(container.children).toHaveLength(0)
  })

  it('treats a folder with no name as no folder', () => {
    const { container } = render(FolderChip, { props: { folder: { name: '', path: '/' } } })

    expect(container.children).toHaveLength(0)
  })
})
