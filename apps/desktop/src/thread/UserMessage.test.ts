// A user message draws what the person sent: their plain text, and the
// thumbnails of the images the message carried (S2). The thumbnails are the same
// ones the composer showed, so a message never looks different once it is sent.

import { afterEach, describe, expect, it } from 'vitest'
import { cleanup, render, screen } from '@testing-library/svelte'
import type { ImageView } from '../contract/bindings'
import UserMessage from './UserMessage.svelte'

/** One image a sent message carried, as Rust pushes it: a name and the URL its
 * thumbnail is served from, never a byte of the image. */
function image(id: string, name: string): ImageView {
  return { id, name, preview_url: `http://gentle-image.localhost/${id}` }
}

function draw(text: string, images: ImageView[] = []) {
  return render(UserMessage, { props: { text, images } })
}

afterEach(cleanup)

describe('UserMessage', () => {
  it('draws the images the message was sent with, beside its text', () => {
    draw('look at this', [image('img-1', 'a.png'), image('img-2', 'b.png')])

    expect(screen.getByRole('img', { name: 'a.png' }).getAttribute('src')).toBe(
      'http://gentle-image.localhost/img-1',
    )
    expect(screen.getByRole('img', { name: 'b.png' })).toBeTruthy()
    expect(screen.getByText('look at this')).toBeTruthy()
  })

  it('draws no images for a message that carried none', () => {
    // PRESERVE: every message typed without a drop draws exactly what it drew.
    draw('just words')

    expect(screen.queryByRole('img')).toBeNull()
    expect(screen.getByText('just words')).toBeTruthy()
  })
})
