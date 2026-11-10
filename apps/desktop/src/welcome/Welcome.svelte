<script lang="ts">
  // The first-start question: the app found pi on this machine, so it asks
  // whether to run on it (the recommended answer) or to keep gentle shell in a
  // space of its own. One click is one intent, and the answer is remembered, so
  // this is asked once. The keyboard starts on the recommended answer.
  import { dispatchIntent } from '../bridge'
  import {
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

  $effect(() => {
    recommended?.focus()
  })

  function choose(choice: 'link' | 'isolated'): void {
    void dispatchIntent({ type: 'SetShellHome', data: { choice } })
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
        <button type="button" class="welcome-go" bind:this={recommended} onclick={() => choose('link')}>
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
        <button type="button" class="welcome-go welcome-quiet" onclick={() => choose('isolated')}>
          {SEPARATE_TITLE}
        </button>
      </section>
    </div>

    <p class="welcome-skip">{SKIP_NOTE}</p>
  </div>
</main>
