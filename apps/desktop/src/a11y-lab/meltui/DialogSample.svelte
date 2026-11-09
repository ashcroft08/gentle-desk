<!-- a11y lab: melt-ui widget 1: modal dialog (focus trap, Esc, focus return). -->
<script lang="ts">
  import { createDialog, melt } from '@melt-ui/svelte'

  const { elements, states } = createDialog()
  const { trigger, content, title, description, close } = elements
  const open = states.open
</script>

<section>
  <h3>Dialog</h3>
  <button use:melt={$trigger}>Open dialog</button>
  {#if $open}
    <div use:melt={$content}>
      <h3 use:melt={$title}>Lab dialog</h3>
      <p use:melt={$description}>melt-ui dialog sample</p>
      <button use:melt={$close}>Close</button>
    </div>
  {/if}
</section>
