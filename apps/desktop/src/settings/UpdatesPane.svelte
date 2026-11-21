<script lang="ts">
  // The Updates pane (0.1.12, S2, S3, S4, S7; 0.8.1 settings-polish S6, S8): TWO
  // cards, from the check the dialog's one load ran when it opened
  // (settings-snapshot). Nothing here checks on its own, so a pane that is visited
  // while the load is still on its way says so quietly and never blocks.
  //
  // The first card is the three rows - this app, pi and Gentle Shell - each with
  // what it runs, what is published and its own action, the tag that summarises
  // them, the sentence of a read that failed, and when it all was last read (S8).
  // The rules that pick a shape live in `./updates`; the rows are drawn by
  // `UpdateRows.svelte`.
  //
  // The second card is "Update extensions", and it is a card of its own because it
  // is the one action NO data drives (D14): the shell's updater with pi's
  // `--extensions` flag only touches the packages the shell has installed, nothing
  // can tell which of them are behind, so there is no state to wait for and the
  // sentence says exactly that. It is drawn in every state and obeys the same
  // chats-closed gate.
  //
  // The terminal actions are the shell's own - `update` on pi's row and `upgrade`
  // on the shell's (S7) - and both are DEAD while any chat is open (S4), with the
  // reason written under the rows: a chat whose child is mid-turn must not have its
  // pi change underneath. "Check again" only ever asks for a check; no update is
  // installed from here, and it is never held back by the gate because reading
  // changes nothing.
  //
  // "Skip this version" (S1, S2, D9) is the quiet one: a plain secondary `.btn` on
  // the row whose version is the newest on offer, and "Stop skipping" once that row
  // publishes the version the user skipped. Both are intents the bridge answers,
  // and the pane asks for the dialog's one read again after them, so the card shows
  // the skip it just made.
  import { dispatchIntent, openTerminal } from '../bridge'
  import type { UpdatesView_Serialize as UpdatesView } from '../contract/bindings'
  import { ago, isJustNow } from '../home/ago'
  import { language } from '../i18n/language.svelte'
  import UpdateRows from './UpdateRows.svelte'
  import { cardOf, type RowAction } from './updates'
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

  /** The card's one tag: something newer, nothing newer, or a row nobody read. */
  const tag = $derived(
    card.available
      ? { text: copy.updateAvailable, tone: 'warn' }
      : card.partial
        ? { text: copy.notCheckedTag, tone: 'quiet' }
        : { text: copy.upToDate, tone: 'ok' },
  )

  /**
   * The last-checked line (S8, D11): the house's own age rule, never a second
   * formatter. It is composed when the line is drawn and never on a ticker, like
   * the welcome's rows, so a dialog left open repeats what it said.
   */
  const checkedLine = $derived.by(() => {
    const ms = card.checkedMs
    if (ms === null) return copy.checkedNever
    const now = Date.now()
    return copy.checkedAt(ago(language.current, now, ms), isJustNow(now, ms))
  })

  async function open(action: RowAction): Promise<void> {
    if (blocked) return
    terminalError = await openTerminal(action === 'upgrade' ? 'UpgradeShell' : 'Update')
  }

  /** The same terminal path, with pi's `--extensions` flag passed through. */
  async function extensions(): Promise<void> {
    if (blocked) return
    terminalError = await openTerminal('UpdateExtensions')
  }

  /**
   * Skips the version on offer: the one that row publishes, never one this pane
   * guessed. Nothing is installed or uninstalled by it - the next check simply
   * stops offering that version - and the dialog's one read is asked again so the
   * card draws the skip it just made.
   */
  async function skip(version: string): Promise<void> {
    await dispatchIntent({ type: 'SkipUpdate', data: { version } })
    onRefresh()
  }

  /** Stops skipping that version, so the next check offers it again. */
  async function unskip(): Promise<void> {
    await dispatchIntent({ type: 'UnskipUpdate' })
    onRefresh()
  }
</script>

{#if card.checked && !refreshing}
  <div class="card updates">
    <div class="head">
      <span class="tag {tag.tone}">{tag.text}</span>
    </div>
    <UpdateRows
      rows={card.rows}
      {copy}
      {blocked}
      {hintId}
      onAction={(action) => void open(action)}
      onSkip={(version) => void skip(version)}
      onUnskip={() => void unskip()}
    />
    {#if blocked}
      <p class="gate" id={hintId} role="status">{hint}</p>
    {:else if card.terminal}
      <p class="note">{copy.updateInTerminal}</p>
    {/if}
    <p class="versions">{checkedLine}</p>
    <div class="acts">
      <button class="btn" type="button" onclick={onRefresh}>{copy.checkAgain}</button>
    </div>
  </div>
{:else}
  <!-- Before the first answer, and while a check runs: a quiet line while it runs,
       when the rows were last read, and the check button. The pane never waits,
       and the last time is NOT erased by the check on its way. -->
  {#if checking}
    <p class="quiet" role="status">{copy.checking}</p>
  {/if}
  <p class="versions">{checkedLine}</p>
  <div class="acts">
    <button class="btn" type="button" onclick={onRefresh}>{copy.checkAgain}</button>
  </div>
  {#if blocked}
    <p class="gate" id={hintId} role="status">{hint}</p>
  {/if}
{/if}
{#if terminalError !== null}
  <p class="quiet" role="status">{terminalError}</p>
{/if}

<!-- The one action no data drives, in a card of its own that says so (D14). -->
<div class="card ext">
  <p class="quiet">{copy.extensionsNote}</p>
  <div class="acts">
    <button
      class="btn"
      type="button"
      disabled={blocked}
      aria-describedby={blocked ? hintId : undefined}
      onclick={() => void extensions()}>{copy.updateExtensions}</button
    >
  </div>
</div>
