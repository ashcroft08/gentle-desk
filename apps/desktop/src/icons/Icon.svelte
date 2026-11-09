<script lang="ts">
  // A named inline icon. Decoration only: every control that draws one carries
  // its own accessible name, so the svg stays out of the accessibility tree.
  import { ICON_SHAPES, type IconName } from './shapes'

  let { name, size = 18 }: { name: IconName; size?: number } = $props()
</script>

<svg class="icon" width={size} height={size} viewBox="0 0 24 24" aria-hidden="true">
  {#each ICON_SHAPES[name] as shape, index (index)}
    {#if shape.kind === 'path'}
      <path d={shape.d} />
    {:else}
      <circle cx={shape.cx} cy={shape.cy} r={shape.r} />
    {/if}
  {/each}
</svg>

<style>
  .icon {
    flex: none;
    fill: none;
    stroke: currentColor;
    stroke-width: 1.8;
    stroke-linecap: round;
    stroke-linejoin: round;
  }
</style>
