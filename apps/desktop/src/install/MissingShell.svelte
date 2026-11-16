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
  import { language } from '../i18n/language.svelte'
  import { COMMAND, installCopy } from './copy'

  let { install }: { install: InstallView } = $props()

  /** The screen's own words in the language the app holds (0.2.0, S3/D4). */
  const copy = $derived(installCopy(language.current))

  /** Set by the first press: until the push says Running, no second one goes out. */
  let asked = $state(false)

  // Seconds waited since the push said Running. npm prints nothing while it
  // works, so this count is what shows the screen is alive; it is never turned
  // into a percentage.
  let seconds = $state(0)

  const running = $derived(install.type === 'Running')
  const done = $derived(install.type === 'Done')
  const failure = $derived(install.type === 'Failed' ? install.data.message : null)

  // The count runs only while the install does. The interval is taken down with
  // the Running state, so nothing ticks once the install ends, and the effect's
  // own cleanup takes it down again if the screen goes away mid-install.
  $effect(() => {
    if (install.type !== 'Running') return
    seconds = 0
    const tick = setInterval(() => {
      seconds += 1
    }, 1000)
    return () => clearInterval(tick)
  })

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
    <h1 id="missing-title">{copy.title}</h1>
    <p class="welcome-intro">{copy.why}</p>
    <p class="welcome-intro">{copy.lookedIn}</p>

    <section class="welcome-found" aria-label={copy.commandIntro}>
      <h2>{copy.commandIntro}</h2>
      <p class="welcome-path">{COMMAND}</p>
    </section>
    <p class="welcome-intro">{copy.installs}</p>
    <p class="welcome-intro">{copy.home}</p>
    <p class="welcome-skip">{copy.warning}</p>

    {#if failure !== null}
      <div class="install-failed" role="alert">
        <strong>{copy.failedTitle}</strong>
        <pre>{failure}</pre>
        <p class="install-hint">{copy.failedHint}</p>
      </div>
    {/if}
    {#if running}
      <p class="welcome-applying" role="status">{copy.running}</p>
      <div class="install-progress" role="progressbar" aria-label={copy.progress}>
        <span class="install-progress-bar"></span>
      </div>
      <p class="welcome-elapsed">{copy.elapsed(seconds)}</p>
    {:else if done}
      <p class="welcome-applying" role="status">{copy.done}</p>
    {/if}

    <div class="install-actions">
      {#if !done}
        <button
          type="button"
          class="welcome-go"
          disabled={running || asked}
          onclick={ask}
        >
          {copy.install}
        </button>
      {/if}
      <button
        type="button"
        class="welcome-go {done ? '' : 'welcome-quiet'}"
        disabled={running}
        onclick={again}
      >
        {copy.checkAgain}
      </button>
    </div>
  </div>
</main>
