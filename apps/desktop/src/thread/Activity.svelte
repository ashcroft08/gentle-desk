<script lang="ts">
  // The line under the last message that says the app is busy: the chat is
  // opening (its process takes seconds to come up) or the model is working on a
  // turn. Three dots pulse next to the word; a screen reader skips them. Every
  // other mark draws nothing.
  import type { ChatActivityMark } from '../contract/bindings'
  import { OPENING_STATUS, WORKING_STATUS } from './copy'

  let { mark = null }: { mark?: ChatActivityMark | null } = $props()

  const text = $derived(
    mark === 'Opening' ? OPENING_STATUS : mark === 'Working' ? WORKING_STATUS : null,
  )
</script>

{#if text !== null}
  <p class="activity" role={mark === 'Opening' ? 'status' : undefined}>
    {text}<span class="dots" aria-hidden="true"
      ><span class="dot"></span><span class="dot"></span><span class="dot"></span></span
    >
  </p>
{/if}
