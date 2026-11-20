<script lang="ts">
  // The Updates pane (0.1.12, S2, S3, S4, S7): ONE card, from the check the
  // dialog's one load ran when it opened (settings-snapshot). Nothing here
  // checks on its own, so a pane that is visited while the load is still on its
  // way says so quietly and never blocks.
  //
  // The card has three shapes, and the rules that pick one live in `./updates`:
  // available (a warning tag, one headline per row that is behind, the quiet
  // versions), up to date (the success tag and the same quiet row), or the
  // checking line with the versions it already had. `refreshing` is what puts the
  // last shape on screen while a load the USER asked for is on its way.
  //
  // The Update button is the terminal's - the shell's own update runs there -
  // and it is DEAD while any chat is open (S4), with the reason written under
  // it: a chat whose child is mid-turn must not have its pi change underneath.
  // "Check again" only ever asks for a check; no update is installed from here.
  // It sits FIRST in the row and wears no accent (L22): the two buttons after it
  // open a terminal window, so the cheap one must not be the middle of the row
  // a hand lands on by accident.
  //
  // "Update extensions" sits beside them and is the one action NO data drives:
  // the shell's own updater with pi's `--extensions` flag passed through only
  // touches the packages the shell has installed, and nothing can tell which of
  // them are behind, so there is no check to wait for and it is drawn in every
  // state, safe to press any time. It obeys the same chats-closed gate.
  import { openTerminal } from '../bridge'
  import type { UpdatesView_Serialize as UpdatesView } from '../contract/bindings'
  import { language } from '../i18n/language.svelte'
  import { cardOf } from './updates'
  import { updatesCopy } from './updates-copy'

  let {
    updates,
    openChats = 0,
    refreshing = false,
    onRefresh = () => {},
  }: {
    /** The check as the dialog's one load answered, or null before any read. */
    updates: UpdatesView | null
    /** How many chats are open: the Update gate (S4). */
    openChats?: number
    /** Whether a load the user asked for is in flight: the checking line. */
    refreshing?: boolean
    /** Asks the dialog for one more load: the pane's only read path. */
    onRefresh?: () => void
  } = $props()

  /** The pane's own words in the language the app holds (0.2.0, D4). */
  const copy = $derived(updatesCopy(language.current))

  /** What the terminal said when it could not open, or `null`. */
  let terminalError = $state<string | null>(null)

  const card = $derived(cardOf(updates, copy))
  /** Whether a check is on its way: the pushed one, or the one the user asked for. */
  const checking = $derived(card.checking || refreshing)
  const blocked = $derived(openChats > 0)
  const hint = $derived(copy.gateHint(openChats))
  const hintId = 'updates-gate'
  const versions = $derived(
    copy.versions(
      card.versions.map((line) => `${line.label} ${line.version ?? copy.unknownVersion}`),
    ),
  )
  const showVersions = $derived(card.versions.some((line) => line.version !== null))

  async function update(): Promise<void> {
    if (blocked) return
    terminalError = await openTerminal('Update')
  }

  /** The same terminal path, with pi's `--extensions` flag passed through. */
  async function extensions(): Promise<void> {
    if (blocked) return
    terminalError = await openTerminal('UpdateExtensions')
  }
</script>

<!-- The extensions button is drawn in every state of the pane, because nothing
     the check says drives it; only the chats-closed gate can hold it back. -->
{#snippet extensionsButton()}
  <button
    class="btn"
    type="button"
    disabled={blocked}
    aria-describedby={blocked ? hintId : undefined}
    onclick={() => void extensions()}>{copy.updateExtensions}</button
  >
{/snippet}

{#if card.checked && !refreshing}
  <div class="card updates">
    <div class="head">
      <span class="tag {card.available ? 'warn' : 'ok'}">
        {card.available ? copy.updateAvailable : copy.upToDate}
      </span>
      {#each card.headlines as row (row.label)}
        <p class="headline">{copy.headline(row.label, row.current, row.latest)}</p>
      {/each}
    </div>
    {#if showVersions}
      <p class="versions">{versions}</p>
    {/if}
    <div class="acts">
      <button class="btn" type="button" onclick={onRefresh}>{copy.checkAgain}</button>
      {#if card.available && card.terminal}
        <button
          class="btn accent"
          type="button"
          disabled={blocked}
          aria-describedby={blocked ? hintId : undefined}
          onclick={() => void update()}>{copy.update}</button
        >
      {/if}
      {@render extensionsButton()}
    </div>
    {#if blocked}
      <p class="gate" id={hintId} role="status">{hint}</p>
    {:else}
      {#if card.available && card.terminal}
        <p class="note">{copy.updateInTerminal}</p>
      {/if}
      <p class="note">{copy.updateExtensionsInTerminal}</p>
    {/if}
    {#if card.appOnly}
      <p class="note">{copy.appUpdatesItself}</p>
    {/if}
    {#if terminalError !== null}
      <p class="quiet" role="status">{terminalError}</p>
    {/if}
  </div>
{:else}
  <!-- Before the first answer, and while a check runs: a quiet line, the
       versions already known, and the check button. The pane never waits. -->
  {#if checking}
    <p class="quiet" role="status">{copy.checking}</p>
  {:else}
    <p class="quiet">{copy.notChecked}</p>
  {/if}
  {#if showVersions}
    <p class="versions">{versions}</p>
  {/if}
  <div class="acts">
    <button class="btn" type="button" onclick={onRefresh}>{copy.checkAgain}</button>
    {@render extensionsButton()}
  </div>
  {#if blocked}
    <p class="gate" id={hintId} role="status">{hint}</p>
  {:else}
    <p class="note">{copy.updateExtensionsInTerminal}</p>
  {/if}
{/if}
