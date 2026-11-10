<script lang="ts">
  // The "Gentle Shell" lettering above New session. When the app opens the pixels
  // are typed in from left to right, each one flashing bright before it settles,
  // and then it stays. It is a picture with a name; a screen reader hears
  // "Gentle Shell" once and skips the cells. With reduced motion the cells are
  // simply there (the animation only describes how they arrive).
  import { cellsOf } from './cells'
  import { MASK } from './mask'

  const cells = cellsOf(MASK)
  const columns = MASK[0].length
  const rows = MASK.length
</script>

<div class="wordmark" role="img" aria-label="Gentle Shell">
  <svg
    viewBox={`0 0 ${columns} ${rows}`}
    aria-hidden="true"
    focusable="false"
    shape-rendering="crispEdges"
  >
    {#each cells as cell (`${cell.x}:${cell.y}`)}
      <rect
        class={cell.kind === 'letter' ? 'wm-cell' : 'wm-shadow'}
        x={cell.x}
        y={cell.y}
        width="1"
        height="1"
        style:--d={`${cell.delay}ms`}
      />
    {/each}
  </svg>
</div>
