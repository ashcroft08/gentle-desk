<script lang="ts">
  // The Extensions block of the Updates pane (packages-status T2, S1, S3; T6, S6):
  // the one group no version of this app drives. The shell's updater with pi's
  // `--extensions` flag only touches the packages that shell installed, so the
  // button is always offered - but WHICH of them are behind is knowable while a
  // chat is open, through pi's own check. So this block counts and names them,
  // says they are up to date only when pi's own check has run for the whole
  // listing, says nobody has checked yet when it has not, says the read could not
  // run when it could not, and keeps the approved honest sentence plus its
  // condition when nothing asked.
  //
  // It draws and asks nothing of its own: the pane owns the press ("Check again"
  // asks the dialog's one load AND pi's explicit check) and hands down the state
  // the push carried, the exact command the button runs (updates-by-home S4) and
  // the terminal's refusal when it could not run it. See `./updates` for the five
  // states and `./updates-copy` for their words.
  import type { PackagesView_Serialize as PackagesView } from '../contract/bindings'
  import { packagesOf } from './updates'
  import type { UpdatesCopy } from './updates-copy'

  let {
    packages,
    checking,
    blocked,
    hintId,
    line,
    error,
    onExtensions,
    copy,
  }: {
    /** The packages slice the dialog draws, or null when there is none at all. */
    packages: PackagesView | null
    /** Whether the explicit check the user asked for is on its way. */
    checking: boolean
    /** Whether any chat is open: the Update gate (S4). */
    blocked: boolean
    /** The gate's line, which the dead button points at. */
    hintId: string
    /** The exact command "Update extensions" runs on this home (S4). */
    line: string
    /** What the terminal answered when it could not open, or null. */
    error: string | null
    /** Opens that terminal window: the block's only action. */
    onExtensions: () => void
    /** The block's own words, in the language the pane is drawn in (D4). */
    copy: UpdatesCopy
  } = $props()

  /** The block's one honest state, from the slice the dialog handed down. */
  const ext = $derived(packagesOf(packages))
</script>

<div class="card ext">
  {#if checking}
    <!-- The check the user asked for is out: the pane's own quiet idiom, for this
         block only - the card of the three rows is not claimed to be checking. -->
    <p class="quiet" role="status">{copy.checking}</p>
  {:else if ext.state === 'behind'}
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
    {#if ext.unchecked}
      <!-- Some packages are known behind and others are still `unknown`: the list
           can never read as the whole story (S6). -->
      <p class="quiet">{copy.packagesUnchecked}</p>
    {/if}
  {:else if ext.state === 'unchecked'}
    <!-- The listing exists and pi's own check has not run for it: never "up to
         date", and never the failed read's clause either (S6). -->
    <p class="quiet">{copy.packagesUnchecked}</p>
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
      onclick={onExtensions}>{copy.updateExtensions}</button
    >
  </div>
  <!-- The exact line that button runs, on the home the push carries (S4), and the
       terminal's own refusal when it could not run it. -->
  <p class="reason mono" data-command>{line}</p>
  {#if error !== null}
    <p class="quiet" role="status">{error}</p>
  {/if}
</div>
