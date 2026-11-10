<script lang="ts">
  // The thread: one scrollable column of the foreground chat's messages. It owns
  // two things and nothing else - the incremental markdown cache (one render of
  // the growing tail per delta, never the finished blocks again) and the scroll
  // rule (stick to the bottom while the reader is there, leave them be if they
  // scrolled up).
  import type { MessageView } from '../contract/bindings'
  import Message from './Message.svelte'
  import { REPLYING_STATUS, THREAD_LABEL } from './copy'
  import { createMarkdownCache } from './markdown'
  import { nearBottom, nextScrollTop } from './scroll'

  let {
    messages = [],
    foregroundChat = null,
  }: {
    /** The foreground chat's messages, oldest first, as Rust pushed them. */
    messages?: MessageView[]
    /** The foreground chat id: a change means the reader starts at the bottom. */
    foregroundChat?: string | null
  } = $props()

  /** The one cache of this thread: per message id, only the growing tail renders. */
  const cache = createMarkdownCache()

  /** The scroll container. */
  let box: HTMLElement | null = $state(null)
  /** Whether the reader is stuck to the bottom; scrolling up unsticks them. */
  let stick = true
  /** The chat the box was last laid out for. */
  let shown: string | null = null

  const streaming = $derived(messages.some((message) => message.streaming))

  /** Every message with its html: a user message and a model error draw no markdown. */
  const rows = $derived.by(() =>
    messages.map((message) => ({
      message,
      html:
        message.role === 'Assistant' && message.error == null
          ? cache.htmlFor(message.id, message.text)
          : '',
    })),
  )

  // The cache follows the window Rust pushes: a message that left the last fifty
  // keeps nothing, so a long session does not keep a render cache per turn.
  $effect(() => {
    cache.retain(messages.map((message) => message.id))
  })

  $effect(() => {
    // Reading these is what makes the effect follow a push: the foreground chat
    // and the whole drawn list, whose height changes on any delta.
    const chat = foregroundChat
    const list = rows
    const moved = chat !== shown
    if (moved) {
      shown = chat
      // A new chat starts at the bottom, whatever the previous one was scrolled to.
      stick = true
    }
    if (box === null || list.length === 0) return
    const top = nextScrollTop(
      { scrollTop: box.scrollTop, scrollHeight: box.scrollHeight, clientHeight: box.clientHeight },
      stick || moved,
    )
    if (top !== null) box.scrollTop = top
  })

  function onScroll(): void {
    if (box === null) return
    stick = nearBottom(box.scrollTop, box.scrollHeight, box.clientHeight)
  }
</script>

<section class="thread" aria-label={THREAD_LABEL} bind:this={box} onscroll={onScroll}>
  <div class="thread-in">
    {#each rows as row (row.message.id)}
      <Message message={row.message} html={row.html} />
    {/each}
  </div>
  {#if streaming}
    <!-- The only live region of the thread: the reply bodies stay out of it, so
         a screen reader is never made to re-read the whole answer per delta. -->
    <p class="vh" role="status" aria-live="polite">{REPLYING_STATUS}</p>
  {/if}
</section>
