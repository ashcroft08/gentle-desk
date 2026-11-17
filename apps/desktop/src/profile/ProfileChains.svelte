<script lang="ts">
  // One profile's fallback chains, read-only (06-profile-tab.html, `.chainro`): the
  // agent each chain is for and the models gentle-pi would move through. The `*` key
  // is the profile-wide chain and is LABELLED as such - it is the one key this view
  // interprets - while every other key, every model and every effort is drawn
  // exactly as stored (D7). When the store that holds them did not answer, the group
  // says so instead of drawing an empty list.
  import type { ChatProfileChainView_Serialize as Chain } from '../contract/bindings'
  import { chainName, type ProfileCopy } from './copy'

  let {
    chains,
    read = true,
    copy,
  }: {
    chains: Chain[]
    /** Whether the fallbacks store's content is in this view at all. */
    read?: boolean
    copy: ProfileCopy
  } = $props()
</script>

{#if !read}
  <!-- The store that holds the chains did not answer: say so, never "no chains". -->
  <p class="quiet">{copy.storeMissing('fallbacks')}</p>
{:else if chains.length === 0}
  <p class="quiet">{copy.noChains}</p>
{:else}
  <ul class="chains">
    {#each chains as chain (chain.agent)}
      <li>
        <p class="cname">{chainName(chain.agent, copy)}</p>
        <ol class="steps">
          {#each chain.steps as step, index (index)}
            <li>
              <span class="step" aria-hidden="true">{index + 1}</span>
              <span class="mono">{step.model}</span>
              {#if step.thinking !== null}
                <span class="think">{step.thinking}</span>
              {/if}
            </li>
          {/each}
        </ol>
      </li>
    {/each}
  </ul>
{/if}

<style>
  .chains {
    display: flex;
    flex-direction: column;
    gap: 8px;
    margin: 6px 0 0;
    padding: 0;
    list-style: none;
  }

  .cname {
    margin: 0;
    color: var(--text);
    font-size: 12.5px;
    font-weight: 600;
  }

  .steps {
    display: flex;
    flex-direction: column;
    gap: 2px;
    margin: 4px 0 0;
    padding: 0;
    list-style: none;
  }

  .steps li {
    display: flex;
    align-items: baseline;
    gap: 8px;
    padding: 1px 0;
    font-size: 12.5px;
  }

  .step {
    width: 16px;
    flex: none;
    color: var(--faint);
    font-family: ui-monospace, 'Cascadia Code', Consolas, monospace;
    font-size: 11px;
    font-weight: 600;
  }

  .mono {
    font-family: ui-monospace, 'Cascadia Code', Consolas, monospace;
    font-size: 12px;
  }

  .think {
    margin-left: auto;
    color: var(--accent);
    font-family: ui-monospace, 'Cascadia Code', Consolas, monospace;
    font-size: 12px;
  }

  .quiet {
    margin: 8px 0 0;
    color: var(--muted);
    font-size: 12.5px;
  }
</style>
