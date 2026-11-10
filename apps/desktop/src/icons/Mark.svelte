<script lang="ts">
  // One of the six chat-state marks. Shape and the English state name travel
  // together, so the state is never carried by colour alone; the colour itself
  // only ever comes from a design-system CSS variable.
  //
  // `label` is for a caller whose text is not the chat state's own word: a
  // helper card names the mark with the helper status Rust sent, and the tab's
  // badge names it with how many helpers run.
  import { MARK_LABELS, MARK_SHAPES, MARK_VARS, type MarkKind } from './shapes'

  let { kind, label = MARK_LABELS[kind] }: { kind: MarkKind; label?: string } = $props()
</script>

<svg
  class="mark"
  style={`color: var(${MARK_VARS[kind]})`}
  viewBox="0 0 24 24"
  role="img"
  aria-label={label}
>
  <title>{label}</title>
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
