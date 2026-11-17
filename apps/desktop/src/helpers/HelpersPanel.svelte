<script lang="ts">
  // The helpers view of one chat, as 01b draws it (T3): two columns, the helper
  // list always on the left and the selected helper's thread on the right.
  // Selecting a card swaps the right column in place - the list is never
  // replaced, so the reader keeps walking it with the keyboard while a thread is
  // drawn beside it.
  //
  // The counters line is Rust's text and stays above the list; a chat with no
  // helper keeps the plain line instead of an empty two-column shell. The thread
  // bar's Back is iced's CloseHelper: it closes the thread and the list stays as
  // it is.
  import { dispatchIntent } from '../bridge'
  import type {
    HelperThreadView as HelperThread,
    HelpersView_Serialize as Helpers,
  } from '../contract/bindings'
  import { language } from '../i18n/language.svelte'
  import { helpersCopy } from './copy'
  import HelperCard from './HelperCard.svelte'
  import HelperThreadView from './HelperThread.svelte'

  let {
    helpers,
    selected = null,
    thread = null,
  }: {
    /** The foreground chat's helpers, counters and cards alike. */
    helpers: Helpers
    /** The card the reducer selected, or `null` for the list itself. */
    selected?: string | null
    /** The selected helper's thread, or `null` while there is none. */
    thread?: HelperThread | null
  } = $props()

  /**
   * The thread the right column draws: only the selected card's own. A missing
   * thread (the helper left the list, or the push has not caught up) or one that
   * names another helper leaves the column empty, so the panel never draws a
   * stranger's transcript under the selected card.
   */
  const open = $derived(thread !== null && selected !== null && thread.id === selected ? thread : null)

  /** The panel's own words in the language it is drawn in (D4). */
  const copy = $derived(helpersCopy(language.current))

  /** The card the keyboard is on: the selected one, else the first. */
  let focused = $state<string | null>(null)
  let list = $state<HTMLElement | null>(null)
  /**
   * The card whose thread the right column drew, so closing it can give the
   * keyboard back to the list.
   */
  let drawn: string | null = null

  const cards = $derived(helpers.tasks)
  /** The one card in the tab order: focus, else the selection, else the first. */
  const active = $derived(focused ?? selected ?? cards[0]?.id ?? null)

  function select(id: string): void {
    void dispatchIntent({ type: 'SelectHelper', data: { helper_id: id } })
  }

  /** Back from a helper's thread: iced's CloseHelper, whose list stays drawn. */
  function back(): void {
    void dispatchIntent({ type: 'CloseHelper' })
  }

  /**
   * The thread bar's red square (helper-stop S2, S4): stop THAT helper, named by
   * the thread's own id, so the panel can never stop the card next to it. The
   * reducer decides whether it can be stopped at all (only a running helper of
   * the foreground chat is), and the child's answer comes back as a push.
   */
  function stopHelper(id: string): void {
    void dispatchIntent({ type: 'StopHelper', data: { helper_id: id } })
  }

  /** Moves the roving tab stop and the focus to `id`. */
  function moveTo(id: string): void {
    focused = id
    const nodes = list?.querySelectorAll<HTMLElement>('[data-helper-id]') ?? []
    for (const node of nodes) {
      if (node.dataset.helperId === id) node.focus()
    }
  }

  // Closing the thread - Back, or a push that clears the selection - takes the
  // control the keyboard was on off the screen with it, so the focus goes back to
  // the list: the card that was open if its row is still there, else the first
  // row. The roving tab stop moves with it, so the list keeps one tab stop.
  $effect(() => {
    const shown = open?.id ?? null
    const previous = drawn
    drawn = shown
    // Only a close is answered here: an opening thread keeps the caret where the
    // user left it, and so does a selection that moves to another card.
    if (shown !== null || previous === null) return
    const target = cards.some((card) => card.id === previous) ? previous : (cards[0]?.id ?? null)
    if (target !== null) moveTo(target)
  })

  /**
   * The listbox's key rule: the arrows walk the cards, Home and End jump to the
   * ends, and Enter selects the card the focus is on. Every other key keeps its
   * own meaning, so Escape in particular does nothing here: iced's panel closes
   * a helper's thread from its Back button, never from a key.
   */
  function keyed(event: KeyboardEvent): void {
    const ids = cards.map((card) => card.id)
    const index = active === null ? -1 : ids.indexOf(active)
    if (event.key === 'Enter') {
      if (active === null) return
      event.preventDefault()
      select(active)
      return
    }
    let next: string | undefined
    if (event.key === 'ArrowDown') next = ids[Math.min(index + 1, ids.length - 1)]
    else if (event.key === 'ArrowUp') next = ids[Math.max(index - 1, 0)]
    else if (event.key === 'Home') next = ids[0]
    else if (event.key === 'End') next = ids[ids.length - 1]
    if (next === undefined) return
    event.preventDefault()
    moveTo(next)
  }
</script>

<div class="panel">
  {#if cards.length === 0}
    <!-- No helper to select: the plain line, never an empty two-column shell. -->
    <div class="plain">
      <p class="counters">{helpers.summary_line}</p>
      <p class="empty">{copy.noHelpers}</p>
    </div>
  {:else}
    <!-- The list column (01b's `.hl`): the counters line is Rust's own text
         and no counter is spelled again here. -->
    <div class="listcol">
      <p class="counters">{helpers.summary_line}</p>
      <div class="list" role="listbox" aria-label={copy.listLabel} bind:this={list}>
        {#each cards as card (card.id)}
          <HelperCard
            row={card}
            selected={card.id === selected}
            tabIndex={card.id === active ? 0 : -1}
            onSelect={select}
            onKey={keyed}
          />
        {/each}
      </div>
    </div>
    <!-- The thread column (01b's `.ht`): the selected helper's thread, or the
         empty column while the push has not arrived. -->
    <div class="threadcol">
      {#if open !== null}
        <HelperThreadView thread={open} onBack={back} onStop={stopHelper} />
      {/if}
    </div>
  {/if}
</div>

<style>
  /* 01b's `.hv`: the list column takes what it needs up to 280px, the thread
     takes the rest; a narrow window pins the list at its 200px floor. */
  .panel {
    display: grid;
    grid-template-columns: minmax(200px, 280px) 1fr;
    flex: 1 1 0;
    height: 100%;
    min-height: 0;
  }

  @media (max-width: 1000px) {
    .panel {
      grid-template-columns: 200px 1fr;
    }
  }

  /* The empty chat keeps the single padded column it always drew. */
  .plain {
    grid-column: 1 / -1;
    display: flex;
    flex-direction: column;
    gap: 8px;
    min-height: 0;
    padding: 12px;
    overflow: auto;
  }

  /* 01b's `.hl`: the column's own padding, its border and its surface. */
  .listcol {
    min-height: 0;
    padding: 10px;
    overflow: auto;
    border-right: 1px solid var(--border);
    background: var(--surface);
  }

  .counters,
  .empty {
    margin: 0;
    color: var(--muted);
    font-size: 12px;
  }

  .counters {
    margin-bottom: 6px;
  }

  .empty {
    font-size: 13px;
  }

  /* 01b's `.ht`: the thread fills the column and scrolls its own body. */
  .threadcol {
    display: flex;
    flex-direction: column;
    min-width: 0;
    min-height: 0;
  }

  .list {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
</style>
