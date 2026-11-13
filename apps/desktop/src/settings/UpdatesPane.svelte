<script lang="ts">
  // The Updates pane (0.1.12, S2, S3, S4, S7): ONE card, from the check the app
  // already ran when it opened. Nothing here checks on its own, so a pane that
  // is visited before the check answers says so quietly and never blocks.
  //
  // The card has three shapes, and the rules that pick one live in `./updates`:
  // available (a warning tag, one headline per row that is behind, the quiet
  // versions), up to date (the success tag and the same quiet row), or the
  // checking line with the versions it already had.
  //
  // The Update button is the terminal's - the shell's own update runs there -
  // and it is DEAD while any chat is open (S4), with the reason written under
  // it: a chat whose child is mid-turn must not have its pi change underneath.
  // "Check again" only ever asks for a check; no update is installed from here.
  import { dispatchIntent, openTerminal } from '../bridge'
  import type { UpdatesView_Serialize as UpdatesView } from '../contract/bindings'
  import { cardOf } from './updates'
  import {
    APP_UPDATES_ITSELF,
    CHECKING,
    CHECK_AGAIN,
    HEADLINE,
    NOT_CHECKED,
    UNKNOWN_VERSION,
    UPDATE,
    UPDATE_AVAILABLE,
    UPDATE_IN_TERMINAL,
    UP_TO_DATE,
    VERSIONS,
    gateHint,
  } from './updates-copy'

  let { updates, openChats = 0 }: { updates: UpdatesView | null; openChats?: number } = $props()

  /** What the terminal said when it could not open, or `null`. */
  let terminalError = $state<string | null>(null)

  const card = $derived(cardOf(updates))
  const blocked = $derived(openChats > 0)
  const hint = $derived(gateHint(openChats))
  const hintId = 'updates-gate'
  const versions = $derived(
    VERSIONS(
      card.versions.map((line) => `${line.label} ${line.version ?? UNKNOWN_VERSION}`),
    ),
  )
  const showVersions = $derived(card.versions.some((line) => line.version !== null))

  function check(): void {
    void dispatchIntent({ type: 'CheckUpdates' })
  }

  async function update(): Promise<void> {
    if (blocked) return
    terminalError = await openTerminal('Update')
  }
</script>

{#if card.checked}
  <div class="card updates">
    <div class="head">
      <span class="tag {card.available ? 'warn' : 'ok'}">
        {card.available ? UPDATE_AVAILABLE : UP_TO_DATE}
      </span>
      {#each card.headlines as row (row.label)}
        <p class="headline">{HEADLINE(row.label, row.current, row.latest)}</p>
      {/each}
    </div>
    {#if showVersions}
      <p class="versions">{versions}</p>
    {/if}
    <div class="acts">
      {#if card.available && card.terminal}
        <button
          class="btn accent"
          type="button"
          disabled={blocked}
          aria-describedby={blocked ? hintId : undefined}
          onclick={() => void update()}>{UPDATE}</button
        >
      {/if}
      <button class="btn" type="button" onclick={check}>{CHECK_AGAIN}</button>
    </div>
    {#if card.available && card.terminal}
      {#if blocked}
        <p class="gate" id={hintId} role="status">{hint}</p>
      {:else}
        <p class="note">{UPDATE_IN_TERMINAL}</p>
      {/if}
    {:else if card.appOnly}
      <p class="note">{APP_UPDATES_ITSELF}</p>
    {/if}
    {#if terminalError !== null}
      <p class="quiet" role="status">{terminalError}</p>
    {/if}
  </div>
{:else}
  <!-- Before the first answer, and while a check runs: a quiet line, the
       versions already known, and the check button. The pane never waits. -->
  {#if card.checking}
    <p class="quiet" role="status">{CHECKING}</p>
  {:else}
    <p class="quiet">{NOT_CHECKED}</p>
  {/if}
  {#if showVersions}
    <p class="versions">{versions}</p>
  {/if}
  <div class="acts">
    <button class="btn accent" type="button" onclick={check}>{CHECK_AGAIN}</button>
  </div>
{/if}
