<script lang="ts">
  // The calm line the Profile tab draws instead of a broken pane (profiles-editor
  // T2c, state 10 of 06-profile-tab.html): one headline, the one reason under it,
  // and the single read it offers. It is NEVER an error box - a child that did not
  // answer is a state, not a failure - and its edge is the warning the artboard
  // gives it, never the danger of a rejection.
  let {
    title,
    why,
    tryAgain,
    onTryAgain,
  }: {
    /** What could not be read, in the tab's own words. */
    title: string
    /** Why, in one short line. */
    why: string
    /** The one action's word. */
    tryAgain: string
    /** Reads again through the tab's own coalesced read. */
    onTryAgain: () => void
  } = $props()
</script>

<div class="unavailable" role="status">
  <p class="bt">{title}</p>
  <p class="why">{why}</p>
  <button class="btn" type="button" onclick={onTryAgain}>{tryAgain}</button>
</div>

<style>
  .unavailable {
    padding: 10px 12px;
    border: 1px solid var(--border);
    border-radius: 10px;
    background: var(--surface2);
    box-shadow: inset 3px 0 0 var(--warning);
    font-size: 12.5px;
  }

  .bt {
    margin: 0 0 4px;
    font-weight: 600;
  }

  .why {
    margin: 0;
    color: var(--muted);
    font-size: 12px;
    line-height: 1.5;
  }

  .btn {
    margin-top: 8px;
  }
</style>
