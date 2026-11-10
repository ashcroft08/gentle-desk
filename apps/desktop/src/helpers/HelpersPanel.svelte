<script lang="ts">
  // The helpers view of one chat: the counters line Rust spells and one card per
  // delegated helper, or - when a card is selected and its thread arrived - that
  // helper's thread in place of the list (T7b-2). The thread is iced's drawer:
  // Back sends CloseHelper and the panel is the list again.
  //
  // The panel owns the two focus moves the switch needs: opening the thread puts
  // the focus on its Back control (the card is gone), and closing it puts the
  // focus back on the card that was open.
  import { dispatchIntent } from '../bridge'
  import type {
    HelperThreadView as HelperThread,
    HelpersView_Serialize as Helpers,
  } from '../contract/bindings'
  import { HELPERS_LIST_LABEL, NO_HELPERS } from './copy'
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
   * The thread the panel draws: only the selected card's own. A missing thread
   * (the helper left the list, or the push has not caught up) or one that names
   * another helper keeps the list, so the panel never draws a stranger's
   * transcript under the selected card.
   */
  const open = $derived(thread !== null && selected !== null && thread.id === selected ? thread : null)

  /** The card the keyboard is on: the selected one, else the first. */
  let focused = $state<string | null>(null)
  let list = $state<HTMLElement | null>(null)

  const cards = $derived(helpers.tasks)
  /** The one card in the tab order: focus, else the selection, else the first. */
  const active = $derived(focused ?? selected ?? cards[0]?.id ?? null)

  /** The card the open thread belongs to, kept to give it the focus back. */
  let openCard: string | null = null

  // Opening the thread hands the focus to its Back control (the thread does
  // that when it mounts); closing it hands the focus back to the card that was
  // open, which is what a reader who pressed Enter expects.
  $effect(() => {
    const now = open?.id ?? null
    if (now === openCard) return
    const left = openCard
    openCard = now
    if (now === null && left !== null) moveTo(left)
  })

  function select(id: string): void {
    void dispatchIntent({ type: 'SelectHelper', data: { helper_id: id } })
  }

  /** Back from a helper's thread to the list, iced's only way out. */
  function back(): void {
    void dispatchIntent({ type: 'CloseHelper' })
  }

  /** Moves the roving tab stop and the focus to `id`. */
  function moveTo(id: string): void {
    focused = id
    const nodes = list?.querySelectorAll<HTMLElement>('[data-helper-id]') ?? []
    for (const node of nodes) {
      if (node.dataset.helperId === id) node.focus()
    }
  }

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

<div class="panel" class:thread={open !== null}>
  {#if open !== null}
    <!-- The thread replaces the list and its counters: the bar names the helper
         and the counters are the list's, as iced draws them. -->
    <HelperThreadView thread={open} onBack={back} />
  {:else}
    <!-- The counters line is Rust's own text: no counter is spelled again here. -->
    <p class="counters">{helpers.summary_line}</p>
    {#if cards.length === 0}
      <p class="empty">{NO_HELPERS}</p>
    {:else}
      <div class="list" role="listbox" aria-label={HELPERS_LIST_LABEL} bind:this={list}>
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
    {/if}
  {/if}
</div>

<style>
  .panel {
    display: flex;
    flex: 1 1 0;
    flex-direction: column;
    gap: 8px;
    min-height: 0;
    padding: 12px;
    overflow: auto;
  }

  /* The thread fills the panel and scrolls its own body; the list pads itself. */
  .panel.thread {
    gap: 0;
    padding: 0;
    overflow: hidden;
  }
  .counters,
  .empty {
    margin: 0;
    padding: 0 10px;
    color: var(--muted);
    font-size: 12px;
  }

  .empty {
    font-size: 13px;
  }

  .list {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
</style>
