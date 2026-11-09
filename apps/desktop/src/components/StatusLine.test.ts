import { describe, it, expect } from 'vitest'
import { render, screen } from '@testing-library/svelte'
import StatusLine from './StatusLine.svelte'

describe('StatusLine', () => {
  it('renders its visible text', () => {
    render(StatusLine, { props: { text: 'ready' } })
    expect(screen.getByText('ready')).toBeTruthy()
  })
})
