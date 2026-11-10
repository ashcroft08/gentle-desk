<script lang="ts">
  // The first-start question: the app found pi on this machine, so it asks
  // whether to run on it (the recommended answer) or to keep gentle shell in a
  // space of its own. The answer is remembered, so this is asked once, and it
  // applies to a new run: the app saves it and starts again (the second intent),
  // so the window says it is applying the choice and takes both answers away
  // meanwhile. The keyboard starts on the recommended answer.
  import { dispatchIntent } from '../bridge'
  import {
    APPLYING,
    FOUND_TITLE,
    INTRO,
    LINK_BADGE,
    LINK_PATH,
    LINK_POINTS,
    LINK_TITLE,
    SEPARATE_PATH,
    SEPARATE_POINTS,
    SEPARATE_TITLE,
    SKIP_NOTE,
    TITLE,
  } from './copy'

  let { piPath }: { piPath: string } = $props()

  let recommended: HTMLButtonElement | null = $state(null)
  /** Set by the first click: the answer is on its way and cannot change. */
  let applying = $state(false)

  $effect(() => {
    recommended?.focus()
  })

  async function choose(choice: 'link' | 'isolated'): Promise<void> {
    if (applying) return
    applying = true
    // In this order: the restart flushes the config, so the choice is in it.
    await dispatchIntent({ type: 'SetShellHome', data: { choice } })
    await dispatchIntent({ type: 'RestartApp' })
  }
</script>

<main class="welcome" aria-labelledby="welcome-title">
  <div class="welcome-card">
    <h1 id="welcome-title">{TITLE}</h1>
    <p class="welcome-intro">{INTRO}</p>

    <section class="welcome-found" aria-label={FOUND_TITLE}>
      <h2>{FOUND_TITLE}</h2>
      <p class="welcome-path">{piPath}</p>
    </section>

    <div class="welcome-choices">
      <section class="welcome-choice welcome-pick" aria-labelledby="welcome-link">
        <h2 id="welcome-link">
          {LINK_TITLE}<span class="welcome-badge">{LINK_BADGE}</span>
        </h2>
        <ul>
          {#each LINK_POINTS as point (point)}
            <li>{point}</li>
          {/each}
        </ul>
        <p class="welcome-where">{LINK_PATH}</p>
        <button type="button" class="welcome-go" bind:this={recommended}
          disabled={applying}
          onclick={() => choose('link')}
        >
          {LINK_TITLE}
        </button>
      </section>

      <section class="welcome-choice" aria-labelledby="welcome-separate">
        <h2 id="welcome-separate">{SEPARATE_TITLE}</h2>
        <ul>
          {#each SEPARATE_POINTS as point (point)}
            <li>{point}</li>
          {/each}
        </ul>
        <p class="welcome-where">{SEPARATE_PATH}</p>
        <button
          type="button"
          class="welcome-go welcome-quiet"
          disabled={applying}
          onclick={() => choose('isolated')}
        >
          {SEPARATE_TITLE}
        </button>
      </section>
    </div>

    {#if applying}
      <p class="welcome-applying" role="status">{APPLYING}</p>
    {:else}
      <p class="welcome-skip">{SKIP_NOTE}</p>
    {/if}
  </div>
</main>
