<script lang="ts">
  // The sidebar header: the section name and the New workspace icon button (D7).
  // This slice adds a folder straight away, as W4b draws the two-item menu
  // ("Choose an existing folder", "Create a new folder") and the dialog.
  import { WORKSPACE_TITLE } from './copy'
  import Icon from '../icons/Icon.svelte'

  let { onNewWorkspace }: { onNewWorkspace: () => void } = $props()
</script>

<div class="wh">
  <h2>{WORKSPACE_TITLE}</h2>
  <button
    class="ic tip-r"
    type="button"
    aria-label="New workspace"
    data-tip="New workspace"
    onclick={onNewWorkspace}
  >
    <Icon name="folder-plus" />
  </button>
</div>
