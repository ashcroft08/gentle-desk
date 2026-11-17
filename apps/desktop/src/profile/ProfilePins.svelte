<script lang="ts">
  // The pin of THIS chat's worktree, read-only (06-profile-tab.html, state 5): one
  // row per layer the `pins` value names, each with the mark it carries - the layer
  // that wins here, one that names a profile the store no longer defines, one that is
  // not a pin at all. A chat with no resolvable worktree is drawn as "pins need a Git
  // workspace": no pin, no error, and the same calm box the unavailable state uses.
  //
  // A layer the store keeps silent about (a VALID profile another layer outranks is
  // not reported at all, `desktop-rpc.md:211-234`) is not drawn and no third mark is
  // invented for it.
  import type { ChatProfilePinsView_Serialize as Pins } from '../contract/bindings'
  import { pinLayerName, pinMarkWord, type ProfileCopy } from './copy'

  let {
    pins,
    read = true,
    copy,
  }: {
    /** What the `pins` store answered for this chat's worktree, if it answered. */
    pins: Pins | null
    /** Whether the pins store's content is in this view at all. */
    read?: boolean
    copy: ProfileCopy
  } = $props()
</script>

{#if !read || pins === null}
  <!-- The store did not answer: marked where the layers would be. -->
  <p class="quiet">{copy.storeMissing('pins')}</p>
{:else if pins.worktree_unresolved}
  <!-- State 5's no-worktree case: nothing to read, and nothing is wrong. -->
  <div class="worktree" role="status">
    <p class="bt">{copy.noWorktree}</p>
    <p class="why">{copy.noWorktreeWhy}</p>
  </div>
{:else if pins.layers.length === 0}
  <p class="quiet">{copy.pinsNone}</p>
{:else}
  <dl class="pins">
    {#each pins.layers as layer (layer.source)}
      <div class="pinrow">
        <dt class="plab">{pinLayerName(layer.source, copy)}</dt>
        <dd>
          <span class="mono">{layer.profile ?? copy.pinNone}</span>
          <span class="tag" class:winner={layer.mark === 'winner'}>
            {pinMarkWord(layer.mark, copy)}
          </span>
        </dd>
      </div>
    {/each}
  </dl>
{/if}

<style>
  .pins {
    display: flex;
    flex-direction: column;
    gap: 4px;
    margin: 6px 0 0;
  }

  .pinrow {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .plab {
    width: 76px;
    flex: none;
    color: var(--faint);
    font-size: 11px;
    font-weight: 600;
    letter-spacing: 0.05em;
    text-transform: uppercase;
  }

  dd {
    display: flex;
    align-items: center;
    gap: 6px;
    margin: 0;
    font-size: 12.5px;
  }

  .mono {
    font-family: ui-monospace, 'Cascadia Code', Consolas, monospace;
    font-size: 12px;
  }

  /* The house's tag: the state is its WORD, and the colour only repeats it. */
  .tag {
    padding: 2px 8px;
    border: 1px solid currentColor;
    border-radius: 999px;
    color: var(--muted);
    font-size: 11px;
    font-weight: 600;
    white-space: nowrap;
  }

  .tag.winner {
    color: var(--accent);
  }

  .worktree {
    margin-top: 6px;
    padding: 10px 12px;
    border: 1px solid var(--border);
    border-radius: 10px;
    background: var(--surface2);
    box-shadow: inset 3px 0 0 var(--warning);
    font-size: 12.5px;
  }

  .worktree .bt {
    margin: 0 0 4px;
    font-weight: 600;
  }

  .worktree .why {
    margin: 0;
    color: var(--muted);
    font-size: 12px;
    line-height: 1.5;
  }

  .quiet {
    margin: 8px 0 0;
    color: var(--muted);
    font-size: 12.5px;
  }
</style>
