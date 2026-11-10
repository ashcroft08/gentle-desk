<script lang="ts">
  // One message of the thread, and the three shapes `crates/ui/src/view/thread.rs`
  // draws: a user bubble of plain text, an assistant reply of markdown (with Copy
  // once it is finished), and - in place of a reply whose turn ended in an error -
  // the model-error box. Copy never shows on an errored message: the iced thread
  // returns its danger box before the Copy row.
  import type { MessageView } from '../contract/bindings'
  import AssistantMessage from './AssistantMessage.svelte'
  import ModelError from './ModelError.svelte'
  import UserMessage from './UserMessage.svelte'
  import { ASSISTANT_SAID, YOU_SAID } from './copy'

  let { message, html = '' }: { message: MessageView; html?: string } = $props()

  const fromUser = $derived(message.role === 'User')
  const whoId = $derived(`msg-who-${message.id}`)
</script>

<article class="msg" class:u={fromUser} class:a={!fromUser} aria-labelledby={whoId}>
  <!-- The author, for a screen reader only: the bubble is drawn without a
       visible name, and the article is what a reader navigates by. -->
  <p class="vh" id={whoId}>{fromUser ? YOU_SAID : ASSISTANT_SAID}</p>
  {#if fromUser}
    <UserMessage text={message.text} />
  {:else if message.error != null}
    <ModelError error={message.error} />
  {:else}
    <AssistantMessage id={message.id} {html} copyable={!message.streaming} />
  {/if}
</article>
