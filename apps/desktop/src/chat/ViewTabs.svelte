<script lang="ts">
  // The Chat | Helpers tablist of the chat header (01b): the two views of one
  // chat, and the working badge on Helpers while a helper runs. The active tab
  // is the shell's view-local choice; this component only reports the tab the
  // user asked for and keeps the roving tab order (one tab stop, arrows walk).
  import Mark from '../icons/Mark.svelte'
  import { language } from '../i18n/language.svelte'
  import { chatCopy } from './copy'
  import { PANEL_IDS, TABS, TAB_IDS, tabAfter, type ViewTab } from './tabs'

  let {
    tab,
    running = 0,
    onSelect,
  }: {
    /** The active tab, as the shell keeps it. */
    tab: ViewTab
    /** How many helpers run right now; the badge is drawn only above zero. */
    running?: number
    /** The tab the user asked for, by click or by key. */
    onSelect: (tab: ViewTab) => void
  } = $props()

  /** The tablist's words in the language the app holds (0.2.0, S3, D4). */
  const copy = $derived(chatCopy(language.current))

  /** A key that walks the tabs also takes the focus with it. */
  function keyed(event: KeyboardEvent, from: ViewTab): void {
    const next = tabAfter(event.key, from)
    if (next === null) return
    event.preventDefault()
    onSelect(next)
    document.getElementById(TAB_IDS[next])?.focus()
  }
</script>

<div class="seg" role="tablist" aria-label={copy.tabsLabel}>
  {#each TABS as item (item)}
    <button
      type="button"
      role="tab"
      id={TAB_IDS[item]}
      aria-controls={PANEL_IDS[item]}
      aria-selected={tab === item}
      tabindex={tab === item ? 0 : -1}
      onclick={() => onSelect(item)}
      onkeydown={(event) => keyed(event, item)}
    >
      {copy.tabs[item]}
      {#if item === 'helpers' && running > 0}
        <!-- The badge is the chat's Working mark; its name says how many run,
             so the count is text, never a number read off a picture. The name is
             the header's own word, so it is drawn in the tablist's language. -->
        <Mark kind="Working" label={copy.workingBadge(running)} />
      {/if}
    </button>
  {/each}
</div>

<style>
  .seg {
    display: inline-flex;
    gap: 2px;
    padding: 2px;
    border: 1px solid var(--border);
    border-radius: 10px;
    background: var(--surface);
  }

  button {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    border: 0;
    border-radius: 8px;
    padding: 4px 14px;
    background: transparent;
    color: var(--muted);
    font-size: 13px;
    cursor: pointer;
  }

  button:hover {
    color: var(--text);
  }

  button[aria-selected='true'] {
    background: var(--tint);
    color: var(--text);
    box-shadow: inset 0 0 0 1px var(--accent);
  }
</style>
