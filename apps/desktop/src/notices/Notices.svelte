<script lang="ts">
// The notices Rust pushes, stacked in its own order (oldest first), one card
// each. A push with no notice draws nothing: the strip is one child of the main
// grid, so an empty strip would still take its space under the header.

import type { NoticeView } from '../contract/bindings'
import Notice from './Notice.svelte'

let { notices = [] }: { notices?: NoticeView[] } = $props()
</script>

{#each notices as notice (notice.id)}
  <Notice {notice} />
{/each}
