<script lang="ts">
  // One helper as a card: the mark whose accessible name is the status label
  // Rust sent, the agent and the task title 01b's row draws, and the counts, the
  // last step and the failure iced's card adds. Nothing here decides: the mark
  // is the approved status mapping, and every text is Rust's.
  import type { HelperRowView_Serialize as HelperRow } from '../contract/bindings'
  import Mark from '../icons/Mark.svelte'
  import { cardMeta, markOfStatus } from './copy'

  let {
    row,
    selected,
    tabIndex,
    onSelect,
    onKey,
  }: {
    row: HelperRow
    /** Whether the reducer holds this card as the selected one. */
    selected: boolean
    /** The roving tab index: 0 for the card the list is on, -1 for the rest. */
    tabIndex: number
    /** The card the user asked for, by click. */
    onSelect: (id: string) => void
    /** The list's key rule, so the card is reachable and Enter selects it. */
    onKey: (event: KeyboardEvent) => void
  } = $props()
</script>

<div
  class="card"
  role="option"
  aria-selected={selected}
  tabindex={tabIndex}
  data-helper-id={row.id}
  onclick={() => onSelect(row.id)}
  onkeydown={onKey}
>
  <Mark kind={markOfStatus(row.status)} label={row.status_label} />
  <div class="body">
    <p class="agent">{row.agent}</p>
    <p class="title">{row.title}</p>
    <p class="meta">{cardMeta(row.turns, row.tool_calls)}</p>
    {#if row.last_step !== null}
      <p class="step">{row.last_step}</p>
    {/if}
    {#if row.error !== null}
      <p class="error">{row.error}</p>
    {/if}
  </div>
</div>

<style>
  .card {
    display: flex;
    align-items: flex-start;
    gap: 8px;
    padding: 8px 10px;
    border-radius: 8px;
    color: var(--text);
    cursor: pointer;
    user-select: none;
  }

  .card:hover {
    background: var(--surface2);
    box-shadow: inset 0 0 0 1px var(--border);
  }

  .card[aria-selected='true'] {
    background: var(--tint);
    box-shadow: inset 3px 0 0 var(--accent);
  }

  .card:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: -2px;
  }

  .body {
    flex: 1;
    min-width: 0;
  }

  p {
    margin: 0;
    font-size: 13px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .agent {
    font-weight: 600;
  }

  .title,
  .step {
    color: var(--muted);
    font-size: 12px;
  }

  .meta {
    color: var(--faint);
    font-size: 12px;
  }

  .error {
    color: var(--danger);
    font-size: 12px;
    white-space: normal;
  }
</style>
