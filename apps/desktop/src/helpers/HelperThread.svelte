<script lang="ts">
  // One helper's thread (T7b-2): what it said, thought and ran. The header is
  // 01b's bar - the Back control, the helper's name and task, its status mark -
  // plus the status text and the prompt iced echoes above the items.
  //
  // Two parts of this view are decisions the design does not draw: the Follow
  // live toggle and the Show tool details toggle are NOT here (iced always
  // follows and always shows tools, decided in T7b-1), and there is no Stop:
  // the only control is the drawn Back. So the thread sticks to the end while
  // the reader is at the end, and shows every tool call, with no switch.
  import { onMount } from 'svelte'
  import type { HelperThreadView as HelperThread } from '../contract/bindings'
  import Icon from '../icons/Icon.svelte'
  import Mark from '../icons/Mark.svelte'
  import { createMarkdownCache } from '../thread/markdown'
  import { nearBottom, nextScrollTop } from '../thread/scroll'
  import HelperThreadItem from './HelperThreadItem.svelte'
  import {
    BACK_TO_HELPERS,
    HELPER_THREAD_LABEL,
    NO_ACTIVITY,
    droppedLine,
    markOfStatus,
    threadHeading,
  } from './copy'

  let {
    thread,
    onBack,
  }: {
    /** The selected helper's thread, as Rust pushed it. */
    thread: HelperThread
    /** Back to the helpers list: the panel sends CloseHelper. */
    onBack: () => void
  } = $props()

  /** The one markdown cache of this thread, keyed by the item's place. */
  const cache = createMarkdownCache()

  const heading = $derived(threadHeading(thread.agent, thread.title))

  /** Every item with the html only a `Said` one draws. */
  const rows = $derived.by(() =>
    thread.items.map((item, index) => ({
      item,
      html: item.kind === 'Said' ? cache.htmlFor(`${thread.id}:${index}`, item.text) : '',
    })),
  )

  let box: HTMLElement | null = $state(null)
  let back: HTMLButtonElement | null = $state(null)
  /** Whether the reader is stuck to the end; scrolling up unsticks them. */
  let stick = true

  // The Back control takes the focus when the thread opens: the card the reader
  // came from is gone, and the way out is the one control here.
  onMount(() => back?.focus())

  // The cache follows the items on screen: a thread that moved on keeps no
  // render of the items it no longer draws.
  $effect(() => {
    cache.retain(rows.map((row, index) => `${thread.id}:${index}`))
  })

  // A pushed update follows the end while the reader is there (a helper that is
  // running keeps appending), and leaves the reader where they are otherwise.
  $effect(() => {
    const list = rows
    if (box === null || list.length === 0) return
    const top = nextScrollTop(
      { scrollTop: box.scrollTop, scrollHeight: box.scrollHeight, clientHeight: box.clientHeight },
      stick,
    )
    if (top !== null) box.scrollTop = top
  })

  function onScroll(): void {
    if (box === null) return
    stick = nearBottom(box.scrollTop, box.scrollHeight, box.clientHeight)
  }
</script>

<section class="ht" aria-label={HELPER_THREAD_LABEL}>
  <div class="bar">
    <button
      class="back"
      type="button"
      aria-label={BACK_TO_HELPERS}
      title={BACK_TO_HELPERS}
      onclick={onBack}
      bind:this={back}
    >
      <Icon name="back" />
    </button>
    <h2 class="name">{heading}</h2>
    <Mark kind={markOfStatus(thread.status)} label={thread.status_label} />
  </div>
  <p class="prompt">{thread.prompt}</p>
  <div class="body" bind:this={box} onscroll={onScroll}>
    {#if thread.dropped > 0}
      <p class="dropped">{droppedLine(thread.dropped)}</p>
    {/if}
    {#if rows.length === 0}
      <p class="empty">{NO_ACTIVITY}</p>
    {/if}
    {#each rows as row, index (index)}
      <HelperThreadItem item={row.item} html={row.html} />
    {/each}
  </div>
</section>

<style>
  .ht {
    display: flex;
    flex-direction: column;
    min-height: 0;
    height: 100%;
  }

  .bar {
    display: flex;
    align-items: center;
    gap: 8px;
    height: 48px;
    padding: 0 12px;
    border-bottom: 1px solid var(--border);
  }

  .back {
    display: inline-grid;
    place-items: center;
    width: 32px;
    height: 32px;
    flex: none;
    padding: 0;
    border: 0;
    border-radius: 8px;
    background: transparent;
    color: var(--muted);
    cursor: pointer;
  }

  .back:hover {
    background: var(--surface2);
    color: var(--text);
    box-shadow: inset 0 0 0 1px var(--border);
  }

  .name {
    flex: 1;
    min-width: 0;
    margin: 0;
    font-size: 14px;
    font-weight: 600;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .prompt {
    margin: 0;
    padding: 8px 16px 0;
    color: var(--muted);
    font-size: 12px;
  }

  .body {
    flex: 1;
    min-height: 0;
    overflow: auto;
    display: flex;
    flex-direction: column;
    gap: 12px;
    padding: 16px 24px 20px;
  }

  .dropped,
  .empty {
    margin: 0;
    color: var(--muted);
    font-size: 12px;
  }
</style>
