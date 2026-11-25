<script lang="ts">
  // The General pane (0.9.0 general S1, S2, S5; the approved artboard
  // `design/settings-redesign/08-general.html`, sheets G1 and G2): the environment
  // check, one row per tool the app and the shell depend on - Gentle Shell, pi,
  // node, git - each with its state (Installed / Missing / Could not read, worded
  // apart by S5), the version it printed when it printed one, and the fix hint
  // ONLY when it is missing or unreadable, the exact command drawn in `<code>`.
  //
  // Nothing here reads the machine: the rows are the General slice of the dialog's
  // ONE read (S4), handed down by `SettingsDialog.svelte`, and the pane's one
  // button asks that same read again through `onRefresh` - the same wire the
  // Updates and Usage panes use, so the card still costs one load per open and a
  // burst of asks is one extra load.
  //
  // Until that read answers - no snapshot yet, or a payload that carries no row at
  // all - the pane says "Checking..." with the artboard's `—` in the version
  // column (G2). It never invents an "Installed" row, and it never draws a hint it
  // has no fact for: "nothing was read" is not "nothing is installed".
  //
  // The bottom row is the house's own: the last check worded by the Updates pane's
  // sentence and `home/ago.ts`'s age - one word, one owner, never a second
  // formatter and never a ticker - and the one button beside it.
  import type {
    GeneralView_Serialize as GeneralView,
    ToolState,
  } from '../contract/bindings'
  import { ago, isJustNow } from '../home/ago'
  import { language } from '../i18n/language.svelte'
  import { generalCopy } from './general-copy'
  import { updatesCopy } from './updates-copy'

  let {
    general = null,
    refreshing = false,
    onRefresh = () => {},
  }: {
    /** The environment check of the dialog's one read, or null before it lands. */
    general?: GeneralView | null
    /** Whether a load the user asked for is in flight: the button waits. */
    refreshing?: boolean
    /** Asks the dialog for one more load: the pane's only read path (S4). */
    onRefresh?: () => void
  } = $props()

  /** The pane's own words in the language the app holds (0.2.0, D4). */
  const copy = $derived(generalCopy(language.current))
  /** The Updates pane's words: the last-checked sentence and the one button (S4). */
  const check = $derived(updatesCopy(language.current))

  /** The rows the check reported, or null while there is nothing to draw yet. */
  const rows = $derived(
    general !== null && general.tools.length > 0 ? general.tools : null,
  )

  /** The one tag tone each state draws (G1's green, G2's two). */
  const TONE: Record<ToolState, string> = {
    Installed: 'ok',
    Missing: 'warn',
    Unreadable: 'bad',
  }

  /**
   * Asks the dialog for its one load again. The button is dead while any read is
   * out, so a burst of presses cannot become a burst of probes.
   */
  function again(): void {
    if (refreshing || general === null) return
    onRefresh()
  }

  /** When the check last read the machine, worded the way the card words it. */
  const checkedLine = $derived.by(() => {
    const ms = general?.checked_ms ?? null
    if (ms === null) return check.checkedNever
    const now = Date.now()
    return check.checkedAt(ago(language.current, now, ms), isJustNow(now, ms))
  })
</script>

{#if rows === null}
  <!-- G2: nobody has read the machine yet. One row says so, in the checking
       words, and the version column holds the artboard's dash - never a state this
       pane has no fact for. -->
  <div class="tool">
    <div class="thead">
      <span class="tname">{copy.tools.GentleShell}</span>
      <span class="tag warn">{copy.checking}</span>
    </div>
    <p class="tver">{copy.noVersion}</p>
  </div>
{:else}
  <!-- One block per row, in the order the check reported them (the contract's own
       order: the shell, the pi it runs, then the two tools the shell's work
       needs). The hint is drawn only where there is something to fix (S2). -->
  {#each rows as row (row.tool)}
    {@const hint = copy.fix(row.tool, row.state)}
    <div class="tool" data-tool={row.tool}>
      <div class="thead">
        <span class="tname">{copy.tools[row.tool]}</span>
        <span class="tag {TONE[row.state]}">{copy.state[row.state]}</span>
      </div>
      {#if row.version}
        <p class="tver">{row.version}</p>
      {/if}
      {#if hint !== null}
        <p class="twhy">
          {#each hint as piece, index (index)}{#if piece.kind === 'command'}<code
              >{piece.text}</code
            >{:else}{piece.text}{/if}{/each}
        </p>
      {/if}
    </div>
  {/each}
{/if}

<!-- The last check and the one button: the card's own sentence and the card's
     own load, so this page owns neither a clock nor a read. -->
<div class="tcheck">
  <span>{checkedLine}</span>
  <button class="btn" type="button" disabled={refreshing || general === null} onclick={again}>
    {check.checkAgain}
  </button>
</div>
