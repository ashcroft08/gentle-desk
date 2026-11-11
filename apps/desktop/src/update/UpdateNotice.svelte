<script lang="ts">
  // The update line, drawn only when there is something to say: a newer version
  // to install, its progress, or the reason it could not be installed. It sits
  // above everything else and never blocks the window (S3).
  import { DOWNLOADING, DISMISS, FAILED, INSTALL, READY, RETRY, found } from './copy'
  import type { UpdateState } from './state'

  let {
    state,
    onInstall,
    onRetry,
    onDismiss,
  }: {
    state: UpdateState
    onInstall: () => void
    onRetry: () => void
    onDismiss: () => void
  } = $props()

  const version = $derived('version' in state ? state.version : null)
  const percent = $derived(
    state.kind === 'downloading' && state.total !== null && state.total > 0
      ? Math.min(100, Math.round((state.done / state.total) * 100))
      : null,
  )
</script>

{#if state.kind === 'available' || state.kind === 'downloading' || state.kind === 'ready' || state.kind === 'failed'}
  <div class="update" role="status">
    {#if state.kind === 'available'}
      <span class="update-m">{found(version ?? '')}</span>
      <button type="button" class="update-go" onclick={onInstall}>{INSTALL}</button>
      <button type="button" class="update-quiet" onclick={onDismiss}>{DISMISS}</button>
    {:else if state.kind === 'downloading'}
      <span class="update-m">{DOWNLOADING(version ?? '', percent)}</span>
      {#if percent !== null}
        <progress class="update-bar" max="100" value={percent}></progress>
      {/if}
    {:else if state.kind === 'ready'}
      <span class="update-m">{READY(version ?? '')}</span>
    {:else if state.kind === 'failed'}
      <span class="update-m">{FAILED(version, state.message)}</span>
      <button type="button" class="update-go" onclick={onRetry}>{RETRY}</button>
      <button type="button" class="update-quiet" onclick={onDismiss}>{DISMISS}</button>
    {/if}
  </div>
{/if}
