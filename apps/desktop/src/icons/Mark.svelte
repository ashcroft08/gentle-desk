<script lang="ts">
  // One of the six chat-state marks. Shape and the English state name travel
  // together, so the state is never carried by colour alone; the colour itself
  // only ever comes from a design-system CSS variable.
  import { MARK_LABELS, MARK_SHAPES, MARK_VARS, type MarkKind } from './shapes'

  let { kind }: { kind: MarkKind } = $props()
</script>

<svg
  class="mark"
  style={`color: var(${MARK_VARS[kind]})`}
  viewBox="0 0 24 24"
  role="img"
  aria-label={MARK_LABELS[kind]}
>
  <title>{MARK_LABELS[kind]}</title>
  {#each MARK_SHAPES[kind] as shape, index (index)}
    {#if shape.kind === 'path'}
      <path d={shape.d} />
    {:else}
      <circle cx={shape.cx} cy={shape.cy} r={shape.r} />
    {/if}
  {/each}
</svg>

<style>
  .mark {
    width: 16px;
    height: 16px;
    flex: none;
    fill: none;
    stroke: currentColor;
    stroke-width: 2;
    stroke-linecap: round;
    stroke-linejoin: round;
  }
</style>
