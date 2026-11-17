<script lang="ts">
  // One accordion group of the Profile tab (06-profile-tab.html, `.group`): the
  // button that opens it, its summary at the right edge, and the body it controls.
  // The chevron is the house's own `chevronDown` glyph, turned by CSS rather than
  // swapped for a picture, and the whole head is one button, so a group is one tab
  // stop and Enter or Space opens it.
  import type { Snippet } from 'svelte'
  import Icon from '../icons/Icon.svelte'

  let {
    title,
    summary = '',
    expanded,
    panelId,
    onToggle,
    children,
  }: {
    /** The group's own name: a profile's name, or the pin group's word. */
    title: string
    /** The quiet line at the right edge: what the group holds, in a word. */
    summary?: string
    /** Whether its body is drawn. Exactly one group is open at a time. */
    expanded: boolean
    /** The id of the body, so the head can point at it with `aria-controls`. */
    panelId: string
    /** Opens this group, or closes it when it is the open one. */
    onToggle: () => void
    /** The group's body. */
    children: Snippet
  } = $props()
</script>

<div class="group">
  <h3 class="gh3">
    <button
      class="ghead"
      type="button"
      aria-expanded={expanded}
      aria-controls={panelId}
      onclick={onToggle}
    >
      <span class="gtitle">{title}</span>
      {#if summary !== ''}
        <span class="gsum">{summary}</span>
      {/if}
      <span class="chev" class:open={expanded} aria-hidden="true">
        <Icon name="chevronDown" size={12} />
      </span>
    </button>
  </h3>
  <div class="gb" id={panelId} hidden={!expanded}>
    {@render children()}
  </div>
</div>

<style>
  /* The artboard's `.group`: a card whose head is a full-width button. */
  .group {
    border: 1px solid var(--border);
    border-radius: 12px;
    overflow: hidden;
    background: var(--surface);
  }

  .gh3 {
    margin: 0;
    font-size: inherit;
    font-weight: inherit;
  }

  .ghead {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    padding: 10px 12px;
    border: 0;
    background: transparent;
    color: var(--text);
    font-size: 13px;
    font-weight: 600;
    text-align: left;
    cursor: pointer;
  }

  .ghead:hover {
    background: var(--surface2);
  }

  .gtitle {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .gsum {
    margin-left: auto;
    overflow: hidden;
    color: var(--faint);
    font-size: 11.5px;
    font-weight: 400;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .chev {
    display: inline-flex;
    flex: none;
    color: var(--muted);
    transition: transform 120ms ease;
  }

  .chev.open {
    transform: rotate(180deg);
  }

  .gb {
    padding: 2px 12px 12px;
    border-top: 1px solid var(--border);
  }

  /* Reduced motion: the chevron jumps instead of turning, and nothing animates. */
  @media (prefers-reduced-motion: reduce) {
    .chev {
      transition: none;
    }
  }
</style>
