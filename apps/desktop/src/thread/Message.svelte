<script lang="ts">
  // One message of the thread, and the three shapes `crates/ui/src/view/thread.rs`
  // draws: a user bubble of plain text, an assistant reply of markdown (with Copy
  // once it is finished), and - in place of a reply whose turn ended in an error -
  // the model-error box. Copy never shows on an errored message: the iced thread
  // returns its danger box before the Copy row.
  //
  // Copy belongs to a row the CHAT owns. The first frame of a stored chat is read
  // from its session file (chat-open-latency T1), and those rows are drawn but
  // never acted on: no Copy, because the chat has not answered for them yet.
  import type { MessageView } from '../contract/bindings'
  import { language } from '../i18n/language.svelte'
  import AssistantMessage from './AssistantMessage.svelte'
  import ModelError from './ModelError.svelte'
  import UserMessage from './UserMessage.svelte'
  import { threadCopy } from './copy'

  let {
    message,
    html = '',
    provisional = false,
  }: {
    message: MessageView
    html?: string
    /** The row came from the session file, not from the child (T1). */
    provisional?: boolean
  } = $props()

  /** The author line in the language the app holds (0.2.0, S3/D4). */
  const copy = $derived(threadCopy(language.current))

  const fromUser = $derived(message.role === 'User')
  const whoId = $derived(`msg-who-${message.id}`)
</script>

<article class="msg" class:u={fromUser} class:a={!fromUser} aria-labelledby={whoId}>
  <!-- The author, for a screen reader only: the bubble is drawn without a
       visible name, and the article is what a reader navigates by. -->
  <p class="vh" id={whoId}>{fromUser ? copy.youSaid : copy.assistantSaid}</p>
  {#if fromUser}
    <UserMessage text={message.text} images={message.images} />
  {:else if message.error != null}
    <ModelError error={message.error} />
  {:else}
    <AssistantMessage id={message.id} {html} copyable={!message.streaming && !provisional} />
  {/if}
</article>
