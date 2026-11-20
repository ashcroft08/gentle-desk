<script lang="ts">
  // The folder chip (design/composer-dock S2, R4): the LAST SEGMENT of the chat's
  // working directory - the name its sidebar row draws, computed by Rust - with
  // the whole path only on hover. No box around it and no control: it is text in
  // the composer's row, and clicking a folder has nowhere to go in this slice.
  //
  // It draws the folder and nothing else. The 1px hairline it used to lead with
  // is gone: each group of the row wears its own quiet border since the
  // maintainer read the row as one big rectangle (L11, artboard C), so a
  // separator here would only double the border it sits inside.
  import type { FolderView } from '../contract/bindings'
  import Icon from '../icons/Icon.svelte'

  let { folder = null }: { folder?: FolderView | null } = $props()
</script>

{#if folder !== null && folder.name !== ''}
  <span class="folder" title={folder.path}>
    <Icon name="folder" size={12} />
    <span class="t">{folder.name}</span>
  </span>
{/if}
