<script lang="ts">
  // The thread: one scrollable column of the foreground chat's TRANSCRIPT. It
  // owns three things and nothing else - the incremental markdown cache (one
  // render of the growing tail per delta, never the finished blocks again), the
  // scroll rule (stick to the bottom while the reader is there, leave them be if
  // they scrolled up), and the one list that draws every kind of row.
  //
  // chat-transcript T2b: the transcript is the ITEMS the contract pushes - text,
  // reasoning and tool calls, in the child's own order - so this list is keyed by
  // the id each row carries. A tool row and a reasoning row draw no markdown and
  // keep no cache entry. `messages` is still read while the shell finishes moving
  // to the list: when no row list arrives, the text rows it carries are the only
  // rows there are, drawn exactly as they were before.
  //
  // Nothing here measures a row: the window is Rust's (`last_items`), the fold of
  // a body is a slice, so a chat of 400 tool calls stays a bounded list of bounded
  // rows and the thread keeps ONE scroll container.
  import type {
    ChatActivityMark,
    ChatItemView_Serialize as ChatItem,
    MessageView,
    ThinkingItemView_Serialize as ThinkingItem,
    ToolCallItemView_Serialize as ToolItem,
  } from '../contract/bindings'
  import { language } from '../i18n/language.svelte'
  import Activity from './Activity.svelte'
  import Message from './Message.svelte'
  import ReasoningRow from './ReasoningRow.svelte'
  import ToolCard from './ToolCard.svelte'
  import WorkingLine from './WorkingLine.svelte'
  import { threadCopy } from './copy'
  import { createMarkdownCache } from './markdown'
  import { nearBottom, nextScrollTop } from './scroll'

  let {
    messages = [],
    items = [],
    foregroundChat = null,
    visible = true,
    scrollTop = $bindable(0),
    mark = null,
  }: {
    /** The foreground chat's messages, oldest first, as Rust pushed them. */
    messages?: MessageView[]
    /** The same chat's whole transcript: text, reasoning and tool rows. */
    items?: ChatItem[]
    /** The foreground chat id: a change means the reader starts at the bottom. */
    foregroundChat?: string | null
    /**
     * Whether the Chat view is the one on screen. The thread stays mounted
     * while the Helpers view covers it (T7b-1), so its scroll position and its
     * render cache survive the trip; this only tells it when to put the reader
     * back.
     */
    visible?: boolean
    /** Where the reader was, kept by the shell across a trip to Helpers. */
    scrollTop?: number
    /** What the chat is doing: the activity line under the last message follows it. */
    mark?: ChatActivityMark | null
  } = $props()

  /** The thread's own words in the language the app holds (0.2.0, S3/D4). */
  const copy = $derived(threadCopy(language.current))

  /**
   * One drawn row: the id it is keyed by and the row itself, told apart by `kind`
   * so the template draws each one with its own component.
   */
  type Row =
    | { kind: 'message'; id: string; message: MessageView; html: string }
    | { kind: 'thinking'; id: string; thinking: ThinkingItem }
    | { kind: 'tool'; id: string; tool: ToolItem }

  /** The one cache of this thread: per message id, only the growing tail renders. */
  const cache = createMarkdownCache()

  /** The scroll container. */
  let box: HTMLElement | null = $state(null)
  /** Whether the reader is stuck to the bottom; scrolling up unsticks them. */
  let stick = true
  /** The chat the box was last laid out for. */
  let shown: string | null = null
  /** Whether the Chat view was the one on screen last time this ran. */
  let shownVisible: boolean | null = null

  /** A reply draws markdown; a user message and a model error draw none. */
  function htmlOf(message: MessageView): string {
    return message.role === 'Assistant' && message.error == null
      ? cache.htmlFor(message.id, message.text)
      : ''
  }

  /** One pushed transcript row, as this thread draws it. */
  function rowOf(item: ChatItem): Row {
    if (item.kind === 'Message') {
      return { kind: 'message', id: item.data.id, message: item.data, html: htmlOf(item.data) }
    }
    if (item.kind === 'Thinking') return { kind: 'thinking', id: item.data.id, thinking: item.data }
    return { kind: 'tool', id: item.data.id, tool: item.data }
  }

  /**
   * Every row with its html: the pushed transcript when there is one, and the
   * text rows alone while the shell still hands this thread `messages`.
   */
  const rows = $derived.by<Row[]>(() => {
    if (items.length > 0) return items.map(rowOf)
    return messages.map((message) => ({
      kind: 'message',
      id: message.id,
      message,
      html: htmlOf(message),
    }))
  })

  const streaming = $derived(
    rows.some((row) => row.kind === 'message' && row.message.streaming),
  )

  // The cache follows the window Rust pushes: a message that left the last fifty
  // keeps nothing, so a long session does not keep a render cache per turn.
  $effect(() => {
    cache.retain(rows.filter((row) => row.kind === 'message').map((row) => row.id))
  })

  $effect(() => {
    // Reading these is what makes the effect follow a push: the foreground chat
    // and the whole drawn list, whose height changes on any delta.
    const chat = foregroundChat
    const list = rows
    const busy = mark
    const moved = chat !== shown
    if (moved) {
      shown = chat
      // A new chat starts at the bottom, whatever the previous one was scrolled to.
      stick = true
    }
    if (box === null || (list.length === 0 && busy === null)) return
    const top = nextScrollTop(
      { scrollTop: box.scrollTop, scrollHeight: box.scrollHeight, clientHeight: box.clientHeight },
      stick || moved,
    )
    if (top !== null) box.scrollTop = top
  })

  // The Helpers view covers the thread instead of replacing it: coming back puts
  // the reader where they were, or at the bottom when they were following the
  // reply (a box hidden with `hidden` may come back scrolled to the top).
  $effect(() => {
    const onScreen = visible
    const returned = shownVisible === false && onScreen
    shownVisible = onScreen
    if (!returned || box === null) return
    const top = nextScrollTop(
      { scrollTop, scrollHeight: box.scrollHeight, clientHeight: box.clientHeight },
      stick,
    )
    box.scrollTop = top ?? scrollTop
  })

  function onScroll(): void {
    if (box === null) return
    scrollTop = box.scrollTop
    stick = nearBottom(box.scrollTop, box.scrollHeight, box.clientHeight)
  }
</script>

<section class="thread" aria-label={copy.threadLabel} bind:this={box} onscroll={onScroll}>
  <div class="thread-in">
    {#each rows as row (row.id)}
      {#if row.kind === 'message'}
        <Message message={row.message} html={row.html} />
      {:else if row.kind === 'thinking'}
        <ReasoningRow item={row.thinking} />
      {:else}
        <ToolCard tool={row.tool} />
      {/if}
    {/each}
    <Activity {mark} />
    <!-- The working line (design/composer-dock S1): the reasoning row's anatomy,
         the web's own clock, and the place the read-only session mode takes. It
         shares the last slot with the opening line, never both at once. -->
    <WorkingLine {mark} />
  </div>
  {#if streaming}
    <!-- The only live region of the thread: the reply bodies stay out of it, so
         a screen reader is never made to re-read the whole answer per delta. -->
    <p class="vh" role="status" aria-live="polite">{copy.replying}</p>
  {/if}
</section>
