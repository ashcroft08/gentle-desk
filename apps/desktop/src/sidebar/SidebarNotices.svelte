<script lang="ts">
  // The two shell-owned lines the bridge pushes, drawn as plain text for now: a
  // damaged config file (`config_notice`) and why the last workspace action was
  // refused (`workspace_notice`). The real notice component with its dismiss
  // action is T7; until then each line is a `role="status"` paragraph and
  // nothing here is dismissible.
  let {
    configNotice = null,
    workspaceNotice = null,
  }: {
    configNotice?: string | null
    workspaceNotice?: string | null
  } = $props()
</script>

{#if configNotice}
  <p class="notice-line" role="status">{configNotice}</p>
{/if}
{#if workspaceNotice}
  <p class="notice-line" role="status">{workspaceNotice}</p>
{/if}
