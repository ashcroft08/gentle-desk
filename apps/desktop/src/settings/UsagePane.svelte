<script lang="ts">
  // The Usage pane (settings-0.1.12 S6's deferred half; settings-redesign S8, L5,
  // L7): the subscription usage the shell's own sources report. It draws the slice
  // the dialog's ONE load answered (settings-snapshot), and reads nothing when it
  // appears: the run of the sources happens on the dialog's open, off the window's
  // thread, once, whether the user walks to this section or not. "Check again"
  // asks the dialog for another load, and `refreshing` is what makes that ask
  // visible while it is on its way: the same quiet reading line the first read
  // draws, never a frozen block.
  //
  // NOTHING IS A CARD HERE ANY MORE (L5, S8): one provider is a `<section>` of the
  // pane's whole column, named by its own heading, its plan at the right edge of
  // that line (D1), its limits as quiet sub-headings (D2) and its windows as rows
  // of ONE four-column grid the stylesheet declares once (`--ucols`, D4). A
  // hairline and 18px of air separate two providers (R1); the box, the radius and
  // the extra gap went with the cards the maintainer rejected.
  //
  // The numbers are the sources' own readings: one row per window with the share
  // used and the countdown to its reset, in the context chip's language (a small
  // bar and the rounded share, the same reader's scale for the tone). A provider
  // that answered "no data" says so and draws no gauge at all, a provider with no
  // window says that in its own sentence (D3), and a limit the provider marked
  // reached wears the warning tag, so the word carries it and the colour never
  // does.
  //
  // A package that is not installed is ONE line and one button: the install runs
  // in a terminal window the user reads, never silently from here.
  import { openTerminal } from '../bridge'
  import type { UsageView_Serialize as UsageView } from '../contract/bindings'
  import { language } from '../i18n/language.svelte'
  import { paneOf, windowCount } from './usage'
  import { usageCopy } from './usage-copy'

  let {
    usage,
    refreshing = false,
    onRefresh = () => {},
  }: {
    /** The usage as the dialog's one load answered, or null before any read. */
    usage: UsageView | null
    /** Whether a load the user asked for is in flight: the reading line. */
    refreshing?: boolean
    /** Asks the dialog for one more load: the pane's only read path. */
    onRefresh?: () => void
  } = $props()

  /** The pane's own words in the language the app holds (0.2.0, D4). */
  const copy = $derived(usageCopy(language.current))

  /** What the terminal said when it could not open, or `null`. */
  let terminalError = $state<string | null>(null)

  const pane = $derived(paneOf(usage))

  async function install(): Promise<void> {
    terminalError = await openTerminal('InstallUsageSources')
  }
</script>

{#if pane.kind === 'reading' || refreshing}
  <!-- The quiet waiting line: the one read runs on the shell's own thread. -->
  <p class="quiet" role="status">{copy.reading}</p>
{:else if pane.kind === 'missing'}
  <p class="quiet">{copy.missing}</p>
  <p class="why">{copy.missingWhy}</p>
  <div class="acts">
    <button class="btn accent" type="button" onclick={() => void install()}>{copy.install}</button>
    <button class="btn" type="button" onclick={onRefresh}>{copy.checkAgain}</button>
  </div>
  <p class="note">{copy.installNote}</p>
{:else if pane.kind === 'unreadable' || pane.kind === 'failed'}
  <!-- Something the read needed could not be used: say why, in the shell's own
       words, and offer the same safe retry. -->
  <p class="quiet" role="status">{copy.cannotRead(pane.reason)}</p>
  <div class="acts">
    <button class="btn" type="button" onclick={onRefresh}>{copy.checkAgain}</button>
  </div>
  <p class="note">{copy.retrySafe}</p>
{:else}
  {#if pane.providers.length === 0}
    <p class="quiet">{copy.noProviders}</p>
  {:else}
    <!-- One section per provider, the pane's whole column wide: the PANE scrolls,
         so nothing here has a scroll of its own and no provider is ever sliced. -->
    <div class="usecs">
      {#each pane.providers as provider, index (provider.provider)}
        {@const head = `usage-provider-${index}`}
        <section class="uprov" aria-labelledby={head}>
          <div class="uprov-head">
            <h4 class="uname" id={head}>{provider.provider}</h4>
            {#if provider.plan !== null}
              <span class="uplan">{copy.plan(provider.plan)}</span>
            {/if}
          </div>

          {#if provider.status === 'NoData'}
            <!-- No credentials, nothing fetched: no gauge, and no zero either. -->
            <p class="quiet">{copy.noData}</p>
            <p class="note">{copy.noDataWhy}</p>
          {:else if provider.status === 'Failed'}
            <p class="quiet" role="status">
              {copy.providerFailed}{provider.error === null ? '' : `: ${provider.error}`}
            </p>
          {:else if windowCount(provider) === 0}
            <!-- D3: a provider of its own that answered with no window says so,
                 and never borrows the line about the whole read. -->
            <p class="quiet">{copy.noWindows}</p>
          {:else}
            {#each provider.limits as limit (limit.name)}
              <div class="ulimit" class:reached={limit.reached}>
                <p class="ulimit-head">
                  <span class="ulimit-name">{limit.name}</span>
                  {#if limit.reached}
                    <span class="tag warn">{copy.limitReached}</span>
                  {/if}
                </p>
                {#each limit.windows as window (`${limit.name}-${window.label}`)}
                  <div class="uwindow">
                    <span class="ulabel">{window.label}</span>
                    <span class="ubar {window.tone}" aria-hidden="true">
                      <i style="width: {window.width}"></i>
                    </span>
                    <span class="upercent">{window.percent}%</span>
                    {#if window.resets !== null}
                      <span class="ureset">{window.resets}</span>
                    {/if}
                  </div>
                {/each}
              </div>
            {/each}
          {/if}
        </section>
      {/each}
    </div>
  {/if}

  <div class="acts">
    <button class="btn" type="button" onclick={onRefresh}>{copy.checkAgain}</button>
  </div>
  <p class="note">{copy.note}</p>
{/if}

{#if terminalError !== null}
  <p class="quiet" role="status">{terminalError}</p>
{/if}
