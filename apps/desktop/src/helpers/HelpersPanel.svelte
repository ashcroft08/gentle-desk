<script lang="ts">
  // The helpers view of one chat (T7b-1): the counters line Rust spells, then
  // one card per delegated helper. Selecting a card only MARKS it in this
  // slice: opening the helper's said/thought/ran thread is T7b-2, and nothing
  // here draws a control that would promise it today.
  import { dispatchIntent } from '../bridge'
  import type { HelpersView_Serialize as Helpers } from '../contract/bindings'
  import { HELPERS_LIST_LABEL, NO_HELPERS } from './copy'
  import HelperCard from './HelperCard.svelte'

  let {
    helpers,
    selected = null,
  }: {
    /** The foreground chat's helpers, counters and cards alike. */
    helpers: Helpers
    /** The card the reducer selected, or `null` for the list itself. */
    selected?: string | null
  } = $props()

  /** The card the keyboard is on: the selected one, else the first. */
  let focused = $state<string | null>(null)
  let list = $state<HTMLElement | null>(null)

  const cards = $derived(helpers.tasks)
  /** The one card in the tab order: focus, else the selection, else the first. */
  const active = $derived(focused ?? selected ?? cards[0]?.id ?? null)

  function select(id: string): void {
    void dispatchIntent({ type: 'SelectHelper', data: { helper_id: id } })
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

<div class="panel">
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
</div>

<style>
  .panel {
    display: flex;
    flex-direction: column;
    gap: 8px;
    min-height: 0;
    padding: 12px;
    overflow: auto;
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
