<script lang="ts">
  // One profile's fallback chains (06-profile-tab.html, `.chainro`): the agent each
  // chain is for and the models gentle-pi would move through. The `*` key is the
  // profile-wide chain and is LABELLED as such - it is the one key this view
  // interprets - while every other key, every model and every effort is drawn
  // exactly as stored (D7).
  //
  // EDITED for the profile this chat USES, and read-only for every other one
  // (profiles-editor T5, L30-3): with `onSave` the chains become one
  // [`ChainEditor`] each - the ordered steps, add/remove/reorder within the
  // owner's cap, Save and Cancel per chain - and the tab says, once, that the
  // chains live in the PROFILE. Without it (a profile this chat does not use) the
  // same list is drawn as text, exactly as before.
  //
  // When the store that holds the chains did not answer, the group says so instead
  // of drawing an empty list or an editor for content nobody read.
  import type { ChatProfileChainView_Serialize as Chain, ChatProfileStepView_Serialize as Step } from '../contract/bindings'
  import { chainName, type ProfileCopy } from './copy'
  import ChainEditor from './ChainEditor.svelte'

  let {
    chains,
    profile = '',
    read = true,
    copy,
    saving = false,
    failed = false,
    onSave,
  }: {
    chains: Chain[]
    /** The name of the profile these chains belong to, for the note's own line. */
    profile?: string
    /** Whether the fallbacks store's content is in this view at all. */
    read?: boolean
    copy: ProfileCopy
    /** A chain save is in flight for this chat: every editor is disabled. */
    saving?: boolean
    /** The last chain save did not happen: one short line says so, once. */
    failed?: boolean
    /** Saves one chain in the profile; absent for a profile this chat does not use. */
    onSave?: (agent: string, steps: Step[]) => void
  } = $props()

  /** Whether this profile's chains are the EDITABLE ones. */
  const editing = $derived(onSave !== undefined && read)
</script>

{#if !read}
  <!-- The store that holds the chains did not answer: say so, never "no chains". -->
  <p class="quiet">{copy.storeMissing('fallbacks')}</p>
{:else if chains.length === 0}
  <p class="quiet">{copy.noChains}</p>
{:else if editing}
  <!-- The one notice of the whole group: a chain write that did not happen. -->
  {#if failed}
    <p class="fail" role="status">{copy.chainNotSaved}</p>
  {/if}
  {#each chains as chain (chain.agent)}
    <div class="one">
      <ChainEditor
        agent={chain.agent}
        label={chainName(chain.agent, copy)}
        steps={chain.steps}
        {copy}
        {saving}
        onSave={onSave as (agent: string, steps: Step[]) => void}
      />
    </div>
  {/each}
  <p class="note">{copy.chainNote(profile)}</p>
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

  /* The editable half: no list wrapper, so the editors stack with one gap. */
  .one + .one {
    margin-top: 10px;
  }

  .note {
    margin: 8px 0 0;
    color: var(--muted);
    font-size: 11.5px;
    line-height: 1.5;
  }

  .fail {
    margin: 6px 0 0;
    color: var(--danger);
    font-size: 11.5px;
    line-height: 1.5;
  }

  .quiet {
    margin: 8px 0 0;
    color: var(--muted);
    font-size: 12.5px;
  }
</style>
