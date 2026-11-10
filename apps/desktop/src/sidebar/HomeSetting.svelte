<script lang="ts">
  // The footer control that changes the home later (shell-choice S5): which home
  // gentle shell runs on - the user's own pi or a space of its own - as two
  // answers with the current one pressed. A choice only applies to a new run, so
  // while one waits the control says so and offers the restart. With no pi on
  // the machine there is nothing to choose between and nothing is drawn.
  import { dispatchIntent } from '../bridge'
  import type { ShellView_Serialize as ShellView } from '../contract/bindings'
  import { GROUP, LINK, RESTART_NOW, RESTART_NOTE, SEPARATE } from './home-copy'

  let { shell }: { shell: ShellView } = $props()

  // No choice is the separate home.
  const current = $derived(shell.choice ?? 'isolated')
  const drawn = $derived(shell.pi_path !== null || shell.choice === 'link')

  function choose(choice: 'link' | 'isolated'): void {
    if (choice === current) return
    void dispatchIntent({ type: 'SetShellHome', data: { choice } })
  }

  function restart(): void {
    void dispatchIntent({ type: 'RestartApp' })
  }
</script>

{#if drawn}
  <div class="home-setting">
    <div class="home-choices" role="group" aria-label={GROUP}>
      <button
        type="button"
        class="home-choice"
        aria-pressed={current === 'link'}
        onclick={() => choose('link')}>{LINK}</button
      >
      <button
        type="button"
        class="home-choice"
        aria-pressed={current === 'isolated'}
        onclick={() => choose('isolated')}>{SEPARATE}</button
      >
    </div>
    {#if shell.restart_needed}
      <p class="home-restart" role="status">
        {RESTART_NOTE}
        <button type="button" class="home-restart-go" onclick={restart}>{RESTART_NOW}</button>
      </p>
    {/if}
  </div>
{/if}
