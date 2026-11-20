<script lang="ts">
  // The line under the last message that says the app is busy opening: a chat's
  // process takes seconds to come up. Three dots pulse next to the word; a screen
  // reader skips them. Every other mark draws nothing here - a working turn is the
  // status line's own line (design/composer-dock S1, `./WorkingLine`), so the
  // thread never says "working" twice.
  import type { ChatActivityMark } from '../contract/bindings'
  import { language } from '../i18n/language.svelte'
  import { threadCopy } from './copy'

  let { mark = null }: { mark?: ChatActivityMark | null } = $props()

  /** The line's own words in the language the app holds (0.2.0, S3/D4). */
  const copy = $derived(threadCopy(language.current))
  const text = $derived(mark === 'Opening' ? copy.opening : null)
</script>

{#if text !== null}
  <p class="activity" role="status">
    {text}<span class="dots" aria-hidden="true"
      ><span class="dot"></span><span class="dot"></span><span class="dot"></span></span
    >
  </p>
{/if}
