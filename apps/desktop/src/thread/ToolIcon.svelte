<script lang="ts">
  // The glyph a tool card carries, one per kind of tool (`card.ts` decides the
  // kind, never the name's spelling). House stroke glyphs only: `fill: none`,
  // `currentColor` and a 2px stroke, so the six themes draw it with no colour of
  // its own - the same language as `.mark` in `app.css`.
  import type { ToolKind } from './card'

  let { kind }: { kind: ToolKind } = $props()
</script>

<svg class="tc-glyph" viewBox="0 0 24 24" aria-hidden="true">
  {#if kind === 'shell'}
    <path d="M4 17l6-5-6-5" />
    <path d="M12 19h8" />
  {:else if kind === 'search'}
    <circle cx="11" cy="11" r="7" />
    <path d="M20 20l-3.5-3.5" />
  {:else if kind === 'edit'}
    <path d="M4 20h4L20 8l-4-4L4 16z" />
    <path d="M14 6l4 4" />
  {:else if kind === 'read'}
    <path d="M6 3h8l4 4v14H6z" />
    <path d="M14 3v4h4" />
  {:else}
    <path d="M12 3v18" />
    <path d="M3 12h18" />
  {/if}
</svg>
