<script lang="ts">
  // One helper's thread (T7b-2): what it said, thought and ran. The header is
  // 01b's bar - the Back control, the helper's name and task, its status mark -
  // plus the status text and the prompt iced echoes above the items.
  //
  // Two parts of this view are decisions the design does not draw: the Follow
  // live toggle and the Show tool details toggle are NOT here (iced always
  // follows and always shows tools, decided in T7b-1), so the thread sticks to
  // the end while the reader is at the end and shows every tool call, with no
  // switch.
  //
  // The bar's other control IS the design's (helper-stop T1/T4): the red square
  // 01b draws at the right of the bar. It stops THIS helper and nothing else -
  // the chat keeps running - so it is enabled only while the helper is running
  // and stays a real, disabled button afterwards, with the tip saying so.
  //
  // Since T3 the thread is drawn BESIDE the helper list, never in its place, so
  // it takes no focus when it appears: the list it sits next to keeps the
  // keyboard, which is what walking it with a thread open needs.
  import type { HelperThreadView as HelperThread } from '../contract/bindings'
  import Icon from '../icons/Icon.svelte'
  import Mark from '../icons/Mark.svelte'
  import { createMarkdownCache } from '../thread/markdown'
  import { nearBottom, nextScrollTop } from '../thread/scroll'
  import { language } from '../i18n/language.svelte'
  import HelperThreadItem from './HelperThreadItem.svelte'
  import { helpersCopy, markOfStatus, threadHeading } from './copy'

  let {
    thread,
    onBack,
    onStop,
  }: {
    /** The selected helper's thread, as Rust pushed it. */
    thread: HelperThread
    /** Back to the helpers list: the panel sends CloseHelper. */
    onBack: () => void
    /** Stop this helper: the panel sends StopHelper with the thread's own id. */
    onStop: (id: string) => void
  } = $props()

  /** The one markdown cache of this thread, keyed by the item's place. */
  const cache = createMarkdownCache()

  /** The thread's own words in the language the panel is drawn in (D4). */
  const copy = $derived(helpersCopy(language.current))

  const heading = $derived(threadHeading(thread.agent, thread.title))

  /**
   * Whether this helper can be stopped right now. Only a RUNNING helper can: the
   * reducer refuses every other mark, so the square says the same thing the
   * state does instead of asking a child to cancel something that is not working.
   */
  const running = $derived(thread.status === 'Running')

  /** Every item with the html only a `Said` one draws. */
  const rows = $derived.by(() =>
    thread.items.map((item, index) => ({
      item,
      html: item.kind === 'Said' ? cache.htmlFor(`${thread.id}:${index}`, item.text) : '',
    })),
  )

  let box: HTMLElement | null = $state(null)
  /** Whether the reader is stuck to the end; scrolling up unsticks them. */
  let stick = true

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

<section class="ht" aria-label={copy.threadLabel}>
  <div class="bar">
    <button
      class="back"
      type="button"
      aria-label={copy.back}
      title={copy.back}
      onclick={onBack}
    >
      <Icon name="back" />
    </button>
    <h2 class="name">{heading}</h2>
    <Mark kind={markOfStatus(thread.status)} label={thread.status_label} />
    <button
      class="ic danger tip-r stop"
      type="button"
      data-tip={running ? copy.stop : copy.stopped}
      aria-label={running ? copy.stopNamed(heading) : copy.alreadyStopped(heading)}
      disabled={!running}
      onclick={() => onStop(thread.id)}
    >
      <Icon name="stop" size={14} />
    </button>
  </div>
  <p class="prompt">{thread.prompt}</p>
  <div class="body" bind:this={box} onscroll={onScroll}>
    {#if thread.dropped > 0}
      <p class="dropped">{copy.droppedLine(thread.dropped)}</p>
    {/if}
    {#if rows.length === 0}
      <p class="empty">{copy.noActivity}</p>
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

  /* The bar's stop square (01b's `.ht-bar .stop`): 32x32, the danger colour,
     and the hover IS the fill - the same rule the composer's Stop follows. A
     helper that is not running keeps the button (and its place in the bar) and
     goes faint instead. Nothing here animates, so there is no motion to reduce. */
  .stop {
    position: relative;
    flex: none;
    width: 32px;
    height: 32px;
    padding: 0;
    border-radius: 8px;
    color: var(--danger);
  }

  .stop:hover:not(:disabled),
  .stop:focus-visible:not(:disabled) {
    background: var(--danger);
    color: var(--bg);
    box-shadow: none;
  }

  .stop:disabled {
    color: var(--faint);
    cursor: default;
  }

  .stop:disabled:hover {
    background: transparent;
    box-shadow: none;
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
