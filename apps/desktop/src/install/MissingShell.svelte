<script lang="ts">
  // The screen for a machine with no gentle-shell. It replaces the app, because
  // nothing here can work without it, and it asks for permission: the install
  // command runs only after the user presses the install button. It says what
  // the command installs (the program, with its size) and that where chats and
  // sign-ins live is the question after it, so the install is not mistaken for
  // the whole of gentle-shell. It follows the install the button started -
  // running, done or failed - and offers to look for gentle-shell again, which
  // is how it moves on once it is there.
  import { dispatchIntent } from '../bridge'
  import type { InstallView } from '../contract/bindings'
  import {
    CHECK_AGAIN,
    COMMAND,
    COMMAND_INTRO,
    DONE,
    FAILED_HINT,
    FAILED_TITLE,
    HOME,
    INSTALL,
    INSTALLS,
    RUNNING,
    TITLE,
    WARNING,
    WHY,
  } from './copy'

  let { install }: { install: InstallView } = $props()

  /** Set by the first press: until the push says Running, no second one goes out. */
  let asked = $state(false)

  const running = $derived(install.type === 'Running')
  const done = $derived(install.type === 'Done')
  const failure = $derived(install.type === 'Failed' ? install.data.message : null)

  // A failure ends the wait, so the button works again.
  $effect(() => {
    if (install.type === 'Failed') asked = false
  })

  function ask(): void {
    if (asked || running) return
    asked = true
    void dispatchIntent({ type: 'InstallShell' })
  }

  function again(): void {
    void dispatchIntent({ type: 'CheckShellAgain' })
  }
</script>

<main class="welcome" aria-labelledby="missing-title">
  <div class="welcome-card">
    <h1 id="missing-title">{TITLE}</h1>
    <p class="welcome-intro">{WHY}</p>

    <section class="welcome-found" aria-label={COMMAND_INTRO}>
      <h2>{COMMAND_INTRO}</h2>
      <p class="welcome-path">{COMMAND}</p>
    </section>
    <p class="welcome-intro">{INSTALLS}</p>
    <p class="welcome-intro">{HOME}</p>
    <p class="welcome-skip">{WARNING}</p>

    {#if failure !== null}
      <div class="install-failed" role="alert">
        <strong>{FAILED_TITLE}</strong>
        <pre>{failure}</pre>
        <p class="install-hint">{FAILED_HINT}</p>
      </div>
    {/if}
    {#if running}
      <p class="welcome-applying" role="status">{RUNNING}</p>
    {:else if done}
      <p class="welcome-applying" role="status">{DONE}</p>
    {/if}

    <div class="install-actions">
      {#if !done}
        <button
          type="button"
          class="welcome-go"
          disabled={running || asked}
          onclick={ask}
        >
          {INSTALL}
        </button>
      {/if}
      <button
        type="button"
        class="welcome-go {done ? '' : 'welcome-quiet'}"
        disabled={running}
        onclick={again}
      >
        {CHECK_AGAIN}
      </button>
    </div>
  </div>
</main>
