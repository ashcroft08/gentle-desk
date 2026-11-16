<script lang="ts">
  // The footer control that changes the home later (shell-choice S5): which home
  // gentle shell runs on - the user's own pi or a space of its own - as two
  // answers with the current one pressed. A choice only applies to a new run, so
  // while one waits the control says so and offers the restart. With no pi on
  // the machine there is nothing to choose between and nothing is drawn.
  import { dispatchIntent } from '../bridge'
  import type { ShellView_Serialize as ShellView } from '../contract/bindings'
  import { language } from '../i18n/language.svelte'
  import { homeCopy } from './home-copy'

  let { shell }: { shell: ShellView } = $props()

  /** The control's own words, in the language this run is set to (0.2.0, S3/D4). */
  const copy = $derived(homeCopy(language.current))

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
    <div class="home-choices" role="group" aria-label={copy.group}>
      <button
        type="button"
        class="home-choice"
        aria-pressed={current === 'link'}
        onclick={() => choose('link')}>{copy.link}</button
      >
      <button
        type="button"
        class="home-choice"
        aria-pressed={current === 'isolated'}
        onclick={() => choose('isolated')}>{copy.separate}</button
      >
    </div>
    {#if shell.restart_needed}
      <p class="home-restart" role="status">
        {copy.restartNote}
        <button type="button" class="home-restart-go" onclick={restart}>{copy.restartNow}</button>
      </p>
    {/if}
  </div>
{/if}
