// The branch chip (design/composer-dock S2, R3, R4): the branch the chat's
// folder is on, read by Rust, drawn as TEXT beside the folder - no box, no
// control, no colour of its own. The whole directory the branch was read from
// lives in the hover and is never the text, exactly as the folder chip does.
//
// R3 is the loudest rule here: a chat outside any repository has no chip at all,
// so nothing stands in its place - no gap, no dash, no "detached".
import { afterEach, describe, expect, it } from 'vitest'
import { cleanup, render } from '@testing-library/svelte'
import BranchChip from './BranchChip.svelte'

afterEach(cleanup)

const BRANCH = { name: 'feat/0.7.0-transcript', path: 'C:\\Users\\josep\\dev\\gentle-desk' }

describe('the branch chip', () => {
  it('draws the branch name only, and keeps the whole directory on hover', () => {
    const { container } = render(BranchChip, { props: { branch: BRANCH } })
    const chip = container.querySelector('.branch') as HTMLElement

    expect(chip.textContent?.trim()).toBe(BRANCH.name)
    expect(chip.textContent).not.toContain('Users')
    expect(chip.getAttribute('title')).toBe(BRANCH.path)
  })

  it('is text, not a control: nothing to click and nothing to focus', () => {
    const { container } = render(BranchChip, { props: { branch: BRANCH } })

    expect(container.querySelector('button')).toBeNull()
    expect(container.querySelector('[tabindex]')).toBeNull()
  })

  it('draws nothing at all when there is no branch to name', () => {
    // R3: no repository draws no chip - not an empty one, not a dash, not a
    // placeholder standing in for one.
    const { container } = render(BranchChip, { props: { branch: null } })

    expect(container.children).toHaveLength(0)
    expect(container.textContent).toBe('')
  })

  it('treats a branch with no name as no branch', () => {
    const { container } = render(BranchChip, { props: { branch: { name: '', path: '/' } } })

    expect(container.children).toHaveLength(0)
  })

  it('brings no hairline of its own: the one before the folder group stays there', () => {
    const { container } = render(BranchChip, { props: { branch: BRANCH } })
    const parts = Array.from(container.children).map((child) => child.className)

    expect(parts).toEqual(['branch'])
    expect(container.querySelector('.vsep')).toBeNull()
  })
})
