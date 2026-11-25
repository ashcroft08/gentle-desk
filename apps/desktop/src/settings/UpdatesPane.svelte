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
  // The second group is "Update extensions" (D14; packages-status T2, S1, S3): the
  // shell's updater with pi's `--extensions` flag only touches the packages the
  // shell has installed, so that button is always offered - but WHICH of them are
  // behind is knowable while a chat is open, through pi's own check. So the block
  // counts and names them, says they are up to date only when the read answered
  // that, says the read could not run when it could not, and keeps the approved
  // honest sentence plus its condition when nothing asked. Same chats-closed gate.
  //
  // The terminal actions are the shell's own - `update` on pi's row and `upgrade`
  // on the shell's (S7) - and both are DEAD while any chat is open (S4), with the
  // reason written under the rows: a chat whose child is mid-turn must not have its
  // pi change underneath. "Check again" only ever asks for a check; no update is
  // installed from here, and it is never held back by the gate because reading
  // changes nothing.
  //
  // WHICH command an action runs is the home's (updates-by-home S4): under each
  // button is one quiet mono line with the exact line Rust will start -
  // `gentle-shell ...` on Separado, `pi ...` on Mi pi - read from the one table in
  // `./update-commands`, and never a guess when Mi pi has no measured stable
  // version to install. A terminal that refuses answers with its own line, drawn
  // here as it always was, under the button that asked and cleared by the next one.
  //
  // "Skip this version" (S1, S2, D9) is the quiet one: a plain secondary `.btn` on
  // the row whose version is the newest on offer, and "Stop skipping" once that row
  // publishes the version the user skipped. Both are intents the bridge answers,
  // and the pane asks for the dialog's one read again after them, so the card shows
  // the skip it just made.
  import { dispatchIntent, openTerminal } from '../bridge'
  import type {
    HomeChoiceView,
    TerminalActionView,
    UpdatesView_Serialize as UpdatesView,
  } from '../contract/bindings'
  import { ago, isJustNow } from '../home/ago'
  import { language } from '../i18n/language.svelte'
  import UpdateRows from './UpdateRows.svelte'
  import { commandAction, commandLine, drawnLine } from './update-commands'
  import { cardOf, packagesOf, type RowAction } from './updates'
  import { updatesCopy } from './updates-copy'

  let {
    updates,
    openChats = 0,
    refreshing = false,
    home = null,
    onRefresh = () => {},
  }: {
    /** The check as the dialog's one load answered, or null before any read. */
    updates: UpdatesView | null
    /** How many chats are open: the Update gate (S4). */
    openChats?: number
    /** Whether a load the user asked for is in flight: the checking line. */
    refreshing?: boolean
    /**
     * The home the live push carries (`shell.choice`), which decides WHICH command
     * every terminal action runs (updates-by-home S4): it is drawn under each
     * button, so the choice is never a surprise.
     */
    home?: HomeChoiceView | null
    /** Asks the dialog for one more load: the pane's only read path. */
    onRefresh?: () => void
  } = $props()

  /** The pane's own words in the language the app holds (0.2.0, D4). */
  const copy = $derived(updatesCopy(language.current))

  /**
   * What the terminal answered when it could not open, and the action that asked
   * (updates-by-home S4): ONE quiet line, drawn under the button it belongs to.
   */
  let terminalError = $state<{ action: TerminalActionView; line: string } | null>(null)

  /**
   * The refusal under the card's two actions, and the one under the extensions
   * button: the same state, so the next press clears whatever was said before.
   */
  const rowsError = $derived(
    terminalError !== null && terminalError.action !== 'UpdateExtensions'
      ? terminalError.line
      : null,
  )
  const extensionsError = $derived(
    terminalError !== null && terminalError.action === 'UpdateExtensions'
      ? terminalError.line
      : null,
  )

  const card = $derived(cardOf(updates, copy))
  /**
   * The Extensions block's one honest state (packages-status T2, S1, S3): the
   * packages slice of the same load, never a read of its own.
   */
  const ext = $derived(packagesOf(updates?.packages ?? null))
  /** Whether a check is on its way: the pushed one, or the one the user asked for. */
  const checking = $derived(card.checking || refreshing)
  const blocked = $derived(openChats > 0)
  const hint = $derived(copy.gateHint(openChats))
  const hintId = 'updates-gate'

  /**
   * The measured Gentle Shell version the Mi pi install names (the same
   * `updates.shell.latest` the row is drawn from), and the extensions line - which
   * needs no version, so a dialog that has not checked yet still says it.
   */
  const shellLatest = $derived(updates?.shell.latest ?? null)
  const extensionsLine = $derived(
    drawnLine(commandLine('UpdateExtensions', home, shellLatest), copy.noStableLatest),
  )

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
    const target = commandAction(action)
    const line = await openTerminal(target)
    terminalError = line === null ? null : { action: target, line }
  }

  /** The same terminal path, with pi's `--extensions` flag passed through. */
  async function extensions(): Promise<void> {
    if (blocked) return
    const line = await openTerminal('UpdateExtensions')
    terminalError = line === null ? null : { action: 'UpdateExtensions', line }
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
      {home}
      {shellLatest}
      onAction={(action) => void open(action)}
      onSkip={(version) => void skip(version)}
      onUnskip={() => void unskip()}
    />
    {#if rowsError !== null}
      <!-- The terminal's own answer when the row's button could not run (S4): ONE
           quiet line, under the actions that run those commands. -->
      <p class="quiet" role="status">{rowsError}</p>
    {/if}
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

<!-- The extensions block: pi's own check when it could ask (packages-status T2),
     and the approved honest sentence plus its condition when it could not (S3). -->
<div class="card ext">
  {#if ext.state === 'behind'}
    <p class="quiet">{copy.packagesBehind(ext.count)}</p>
    <ul class="urows">
      {#each ext.rows as one (one.name)}
        <li class="urow" data-package={one.name}>
          <span class="headline">{one.name}</span>
          {#if one.version !== null}
            <span class="mono">{one.version}</span>
          {/if}
        </li>
      {/each}
    </ul>
  {:else if ext.state === 'upToDate'}
    <p class="quiet">{copy.packagesUpToDate}</p>
  {:else if ext.state === 'unreadable'}
    <p class="quiet">{copy.packagesUnreadable}</p>
  {:else}
    <!-- No chat open, or a shell that does not advertise the read (S5): the
         honest sentence, and the condition that would let it check (S3). -->
    <p class="quiet">{copy.extensionsNote}</p>
    <p class="quiet">{copy.packagesNeedsSession}</p>
  {/if}
  <div class="acts">
    <button
      class="btn"
      type="button"
      disabled={blocked}
      aria-describedby={blocked ? hintId : undefined}
      onclick={() => void extensions()}>{copy.updateExtensions}</button
    >
  </div>
  <!-- The exact line that button runs, on the home the push carries (S4), and the
       terminal's own refusal when it could not run it. -->
  <p class="reason mono" data-command>{extensionsLine}</p>
  {#if extensionsError !== null}
    <p class="quiet" role="status">{extensionsError}</p>
  {/if}
</div>
