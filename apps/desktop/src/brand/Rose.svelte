<script lang="ts">
  // The rose of gentle-shell-gpui as the welcome screen draws it: a canvas that
  // blooms once from its heart outward over 2.6 s and then STOPS, so a screen left
  // open asks for no frame at all. It floats and sways while it opens, a glint
  // crosses the petals, and then it comes to rest on its own art. With reduced
  // motion the rose is simply there, drawn once, open: the animation only ever
  // said how it arrives. The screen names the picture (it owns the words); this
  // owns the dots.
  import { onMount } from 'svelte'
  import {
    BASE_COLOUR,
    DOT_SIDE,
    ROSE_HEIGHT,
    ROSE_PADDING,
    ROSE_WIDTH,
    WRITE_MS,
    frameAt,
  } from './rose'

  let { label, height = 96 }: { label: string; height?: number } = $props()

  /** One art dot in CSS pixels, and the canvas the art needs, padding aside. */
  const dot = $derived(height / ROSE_HEIGHT)
  const width = $derived(ROSE_WIDTH * dot + ROSE_PADDING * 2)
  const tall = $derived(height + ROSE_PADDING * 2)

  let canvas: HTMLCanvasElement | null = $state(null)
  /** The frame this rose is waiting for, so it can be cancelled when it leaves. */
  let waiting = 0
  /**
   * The frame last drawn: how far the bloom got, whether it was the still form,
   * and at which height. A height that changes after mount redraws THAT frame once
   * (the screen resizes the rose with the window), never the bloom again.
   */
  let reveal = 0
  let rested = false
  let drawnAt: number | null = null

  function draw(revealNow: number, still: boolean): void {
    reveal = revealNow
    rested = still
    drawnAt = height
    const element = canvas
    if (!element) return
    const ratio = globalThis.devicePixelRatio || 1
    element.style.width = `${width}px`
    element.style.height = `${tall}px`
    element.width = Math.ceil(width * ratio)
    element.height = Math.ceil(tall * ratio)
    // A browser without a 2d surface still gets a canvas of the right size.
    const context = element.getContext('2d')
    if (!context) return
    context.setTransform(ratio, 0, 0, ratio, 0, 0)
    const side = Math.max(1, DOT_SIDE * dot)
    for (const cell of frameAt(revealNow, still)) {
      // A folded dot is not drawn at all, as it is not in the artboard's frame.
      if (cell.alpha <= 0) continue
      const { hue } = BASE_COLOUR
      context.fillStyle = `hsla(${hue},${cell.saturation * 100}%,${cell.lightness * 100}%,${cell.alpha})`
      context.fillRect(
        ROSE_PADDING + cell.x * dot,
        ROSE_PADDING + cell.y * dot,
        side,
        side,
      )
    }
  }

  onMount(() => {
    const reduce =
      typeof globalThis.matchMedia === 'function' &&
      globalThis.matchMedia('(prefers-reduced-motion: reduce)').matches
    if (reduce) {
      draw(1, true)
      return
    }
    const started = performance.now()
    const step = (now: number) => {
      const progress = Math.min(1, (now - started) / WRITE_MS)
      draw(progress, false)
      if (progress < 1) waiting = requestAnimationFrame(step)
    }
    step(started)
    return () => cancelAnimationFrame(waiting)
  })

  // A new size is not the bloom's business: it draws the frame the screen already
  // has at the new geometry, once, and asks for no frame of its own. While the
  // bloom runs, the frames it already asked for keep coming at the new size; at
  // rest this is the ONE redraw, and the rose stays at rest (nothing is waiting).
  $effect(() => {
    const asked = height
    if (drawnAt === null || drawnAt === asked) return
    draw(reveal, rested)
  })
</script>

<!-- The canvas IS the picture: it is not a control and takes no click, so its
     name and its role belong on it, and a wrapper would leave the one thing a
     screen reader must describe unnamed. The compiler reads `canvas` as
     interactive and would move the name off it. -->
<!-- svelte-ignore a11y_no_interactive_element_to_noninteractive_role -->
<canvas bind:this={canvas} role="img" aria-label={label}></canvas>
