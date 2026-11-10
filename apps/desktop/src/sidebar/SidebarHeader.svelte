<script lang="ts">
  // The sidebar header: the section name of the tree and the New workspace
  // control (D7). The control is the folder-plus icon with its two-item menu,
  // and the create-folder dialog the second item opens; both commands go through
  // the bridge, never through `invoke` here.
  import { addWorkspace, createWorkspace } from '../bridge'
  import { WORKSPACE_TITLE } from './copy'
  import NewWorkspaceDialog from './NewWorkspaceDialog.svelte'
  import NewWorkspaceMenu from './NewWorkspaceMenu.svelte'

  let {
    menuOpen = $bindable(false),
  }: {
    /** Whether the New workspace menu is open; the empty first-run prompt opens it. */
    menuOpen?: boolean
  } = $props()

  /** The icon, so a closed menu or dialog gives the focus back to it. */
  let trigger = $state<HTMLElement | null>(null)
  let createOpen = $state(false)
</script>

<div class="wh">
  <h2>{WORKSPACE_TITLE}</h2>
  <NewWorkspaceMenu
    bind:open={menuOpen}
    bind:trigger
    onChooseFolder={() => void addWorkspace()}
    onCreateFolder={() => (createOpen = true)}
  />
</div>
<NewWorkspaceDialog bind:open={createOpen} {trigger} onCreate={(name) => createWorkspace(name)} />
