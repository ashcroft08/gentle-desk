<script lang="ts">
  // The shell-owned lines the bridge pushes, drawn as plain text for now: a
  // damaged config file (`config_notice`), why the last workspace action was
  // refused (`workspace_notice`) and why the last session action was refused
  // (T2's rename, `sessions_notice`). The real notice component with its dismiss
  // action is T7; until then each line is a `role="status"` paragraph and
  // nothing here is dismissible.
  let {
    configNotice = null,
    workspaceNotice = null,
    sessionsNotice = null,
  }: {
    configNotice?: string | null
    workspaceNotice?: string | null
    sessionsNotice?: string | null
  } = $props()
</script>

{#if configNotice}
  <p class="notice-line" role="status">{configNotice}</p>
{/if}
{#if workspaceNotice}
  <p class="notice-line" role="status">{workspaceNotice}</p>
{/if}
{#if sessionsNotice}
  <p class="notice-line" role="status">{sessionsNotice}</p>
{/if}
