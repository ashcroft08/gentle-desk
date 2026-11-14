<script lang="ts">
  // The first-start question: the app found pi on this machine, so it asks
  // whether to run on it (the recommended answer) or to keep gentle shell in a
  // space of its own. The answer is remembered, so this is asked once, and it
  // applies to a new run: the app saves it and starts again (the second intent),
  // so the window says it is applying the choice and takes both answers away
  // meanwhile. The keyboard starts on the recommended answer.
  import { dispatchIntent } from '../bridge'
  import { language } from '../i18n/language.svelte'
  import { welcomeCopy } from './copy'

  let { piPath }: { piPath: string } = $props()

  /** The screen's own words in the language the app holds (0.2.0, S3/D4). */
  const copy = $derived(welcomeCopy(language.current))

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
    <h1 id="welcome-title">{copy.title}</h1>
    <p class="welcome-intro">{copy.intro}</p>

    <section class="welcome-found" aria-label={copy.foundTitle}>
      <h2>{copy.foundTitle}</h2>
      <p class="welcome-path">{piPath}</p>
    </section>

    <div class="welcome-choices">
      <section class="welcome-choice welcome-pick" aria-labelledby="welcome-link">
        <h2 id="welcome-link">
          {copy.linkTitle}<span class="welcome-badge">{copy.linkBadge}</span>
        </h2>
        <ul>
          {#each copy.linkPoints as point (point)}
            <li>{point}</li>
          {/each}
        </ul>
        <p class="welcome-where">{copy.linkPath}</p>
        <button type="button" class="welcome-go" bind:this={recommended}
          disabled={applying}
          onclick={() => choose('link')}
        >
          {copy.linkTitle}
        </button>
      </section>

      <section class="welcome-choice" aria-labelledby="welcome-separate">
        <h2 id="welcome-separate">{copy.separateTitle}</h2>
        <ul>
          {#each copy.separatePoints as point (point)}
            <li>{point}</li>
          {/each}
        </ul>
        <p class="welcome-where">{copy.separatePath}</p>
        <button
          type="button"
          class="welcome-go welcome-quiet"
          disabled={applying}
          onclick={() => choose('isolated')}
        >
          {copy.separateTitle}
        </button>
      </section>
    </div>

    {#if applying}
      <p class="welcome-applying" role="status">{copy.applying}</p>
    {:else}
      <p class="welcome-skip">{copy.skipNote}</p>
    {/if}
  </div>
</main>
