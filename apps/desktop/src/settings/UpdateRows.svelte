<script lang="ts">
  // The Updates card's three rows (0.8.1 settings-polish S8, D9, D10, D14): one
  // piece of this machine each, with the version it runs, the version published
  // when there is something newer, and the row's OWN action - so a button finally
  // names the piece it updates and the shell can be behind at all (S6, S7).
  //
  // Nothing is decided here: `cardOf` in `./updates` says which row offers which
  // action, which row carries the one skip, and which sentence goes under it. This
  // file only draws that answer with the app's own pieces (`.urows`/`.urow`, the
  // Account pane's row rhythm, `.headline`, `.mono`, `.reason`) and asks the pane
  // above it for the action.
  //
  // The terminal buttons are DEAD while any chat is open (S4), with the reason
  // written under the rows by the pane. The skip is held back with them (04a·2):
  // while a chat is open the whole action row waits, and "Check again" - which
  // installs nothing - is the one thing that stays alive.
  import type { RowAction, UpdateRow } from './updates'
  import type { UpdatesCopy } from './updates-copy'

  let {
    rows,
    copy,
    blocked = false,
    hintId = undefined,
    onAction,
    onSkip,
    onUnskip,
  }: {
    /** The three rows as `cardOf` worked them out. */
    rows: UpdateRow[]
    /** The pane's own words in the language the app holds. */
    copy: UpdatesCopy
    /** Whether a chat is open: the terminal actions and the skip wait for it. */
    blocked?: boolean
    /** The id of the gate's line, which a dead button points at. */
    hintId?: string
    /** Opens a row's own terminal window. */
    onAction: (action: RowAction) => void
    /** Skips the version on offer. */
    onSkip: (version: string) => void
    /** Stops skipping it. */
    onUnskip: () => void
  } = $props()

  /** The action's own words: the one place a row's button is named. */
  function actionLabel(action: RowAction): string {
    return action === 'upgrade' ? copy.updateShell : copy.update
  }

  /** The one click handler a skip half needs, in the row that carries it. */
  function skipClick(row: UpdateRow): void {
    if (row.skip === 'skip' && row.latest !== null) onSkip(row.latest)
    else if (row.skip === 'unskip') onUnskip()
  }
</script>

<ul class="urows">
  {#each rows as row (row.key)}
    <li class="urow" data-row={row.key}>
      <span class="headline">{row.label}</span>
      <span class="mono">{row.version}</span>
      {#if row.action !== null}
        {@const action = row.action}
        <button
          class="btn accent"
          type="button"
          disabled={blocked}
          aria-describedby={blocked ? hintId : undefined}
          onclick={() => onAction(action)}>{actionLabel(action)}</button
        >
      {/if}
      {#if row.skip !== null}
        <button
          class="btn"
          type="button"
          disabled={blocked}
          aria-describedby={blocked ? hintId : undefined}
          onclick={() => skipClick(row)}>{row.skip === 'skip' ? copy.skipVersion : copy.stopSkipping}</button
        >
      {/if}
      {#if row.note !== null}
        <p class="reason">{row.note}</p>
      {/if}
    </li>
  {/each}
</ul>
