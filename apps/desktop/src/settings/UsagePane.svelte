<script lang="ts">
  // The Usage pane (settings-0.1.12 S6's deferred half): the subscription usage
  // the shell's own sources report, one card per provider. It reads the slice
  // Rust pushed and asks for its own read ONCE when it appears - a run of the
  // sources takes seconds, so the cheap correct answer is one run when the user
  // opens the section - and "Check again" asks for another. No timer, no
  // spinner, no blocking.
  //
  // The numbers are the sources' own readings: one row per window with the share
  // used and the countdown to its reset, in the context chip's language (a small
  // bar and the rounded share, the same reader's scale for the tone). A provider
  // that answered "no data" says so and draws no gauge at all, and a limit the
  // provider marked reached wears the warning tag, so the word carries it and
  // the colour never does.
  //
  // A package that is not installed is ONE line and one button: the install runs
  // in a terminal window the user reads, never silently from here.
  import { dispatchIntent, openTerminal } from '../bridge'
  import type { UsageView_Serialize as UsageView } from '../contract/bindings'
  import { paneOf } from './usage'
  import {
    CANNOT_READ,
    CHECK_AGAIN,
    INSTALL,
    INSTALL_NOTE,
    LIMIT_REACHED,
    MISSING,
    MISSING_WHY,
    NO_DATA,
    NO_DATA_WHY,
    NO_PROVIDERS,
    NOTE,
    PLAN,
    PROVIDER_FAILED,
    READING,
    RETRY_SAFE,
  } from './usage-copy'

  let { usage }: { usage: UsageView | null } = $props()

  /** What the terminal said when it could not open, or `null`. */
  let terminalError = $state<string | null>(null)

  const pane = $derived(paneOf(usage))

  /**
   * The pane's one read, asked when the dialog draws this section: nothing runs
   * the sources at app open (a run costs seconds, and a section the user never
   * opens should cost nothing).
   */
  $effect(() => {
    read()
  })

  function read(): void {
    void dispatchIntent({ type: 'RefreshUsage' })
  }

  async function install(): Promise<void> {
    terminalError = await openTerminal('InstallUsageSources')
  }
</script>

{#if pane.kind === 'reading'}
  <!-- The quiet waiting line: the read runs on its own thread in the shell. -->
  <p class="quiet" role="status">{READING}</p>
{:else if pane.kind === 'missing'}
  <p class="quiet">{MISSING}</p>
  <p class="why">{MISSING_WHY}</p>
  <div class="acts">
    <button class="btn accent" type="button" onclick={() => void install()}>{INSTALL}</button>
    <button class="btn" type="button" onclick={read}>{CHECK_AGAIN}</button>
  </div>
  <p class="note">{INSTALL_NOTE}</p>
{:else if pane.kind === 'unreadable' || pane.kind === 'failed'}
  <!-- Something the read needed could not be used: say why, in the shell's own
       words, and offer the same safe retry. -->
  <p class="quiet" role="status">{CANNOT_READ(pane.reason)}</p>
  <div class="acts">
    <button class="btn" type="button" onclick={read}>{CHECK_AGAIN}</button>
  </div>
  <p class="note">{RETRY_SAFE}</p>
{:else}
  {#if pane.providers.length === 0}
    <p class="quiet">{NO_PROVIDERS}</p>
  {:else}
    {#each pane.providers as provider (provider.provider)}
      <div class="card usage-provider">
        <div class="head">
          <span class="uname">{provider.provider}</span>
          {#if provider.plan !== null}
            <span class="uplan">{PLAN(provider.plan)}</span>
          {/if}
        </div>

        {#if provider.status === 'NoData'}
          <!-- No credentials, nothing fetched: no gauge, and no zero either. -->
          <p class="quiet">{NO_DATA}</p>
          <p class="note">{NO_DATA_WHY}</p>
        {:else if provider.status === 'Failed'}
          <p class="quiet" role="status">
            {PROVIDER_FAILED}{provider.error === null ? '' : `: ${provider.error}`}
          </p>
        {:else if provider.limits.length === 0}
          <p class="quiet">{NO_PROVIDERS}</p>
        {:else}
          {#each provider.limits as limit (limit.name)}
            <div class="ulimit" class:reached={limit.reached}>
              <div class="ulimit-head">
                <span class="ulimit-name">{limit.name}</span>
                {#if limit.reached}
                  <span class="tag warn">{LIMIT_REACHED}</span>
                {/if}
              </div>
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
      </div>
    {/each}
  {/if}

  <div class="acts">
    <button class="btn" type="button" onclick={read}>{CHECK_AGAIN}</button>
  </div>
  <p class="note">{NOTE}</p>
{/if}

{#if terminalError !== null}
  <p class="quiet" role="status">{terminalError}</p>
{/if}
