<script lang="ts">
  // The folder chip (design/composer-dock S2, R4): the LAST SEGMENT of the chat's
  // working directory - the name its sidebar row draws, computed by Rust - with
  // the whole path only on hover. No box around it and no control: it is text in
  // the composer's row, and clicking a folder has nowhere to go in this slice.
  //
  // The 1px hairline belongs to this group, not to the row: the line separates
  // what the message runs ON (model, effort) from where it runs (folder, and the
  // branch in a later slice), so a chat with no folder to name leaves no
  // dangling separator either (R3's rule, applied one group earlier).
  import type { FolderView } from '../contract/bindings'
  import Icon from '../icons/Icon.svelte'

  let { folder = null }: { folder?: FolderView | null } = $props()
</script>

{#if folder !== null && folder.name !== ''}
  <span class="vsep" aria-hidden="true"></span>
  <span class="folder" title={folder.path}>
    <Icon name="folder" size={12} />
    <span class="t">{folder.name}</span>
  </span>
{/if}
