<script lang="ts">
  // The pin of THIS chat's worktree (06-profile-tab.html, state 5; profiles-editor
  // T6): one row per pin layer the store reports, each with the mark it carries -
  // the layer that wins here, one that names a profile the store no longer
  // defines, one that is not a pin at all - and, for every layer this worktree
  // offers a write target for, the terminal's PIN EDITOR ([`PinEditor`]): a
  // picker of the profiles the store defines (or "no pin") and ONE Save per layer.
  // Anything that is not writable keeps the read-only drawing.
  //
  // A chat with no resolvable worktree is drawn as "pins need a Git workspace": no
  // pin, no editor, no error, and the same calm box the unavailable state uses.
  //
  // A layer whose VALID pin another layer OUTRANKS is not reported by the store at
  // all (`desktop-rpc.md:211-234`): it is drawn as a row without a value, which is
  // exactly what the editor needs and never a claim to know it.
  import type {
    ChatProfilePinLayerView_Serialize as PinLayer,
    ChatProfilePinsView_Serialize as Pins,
  } from '../contract/bindings'
  import { pinLayerName, pinMarkWord, type ProfileCopy } from './copy'
  import PinEditor from './PinEditor.svelte'

  let {
    pins,
    read = true,
    copy,
    defined = [],
    saving = false,
    failed = false,
    onSave = undefined,
  }: {
    /** What the `pins` store answered for this chat's worktree, if it answered. */
    pins: Pins | null
    /** Whether the pins store's content is in this view at all. */
    read?: boolean
    copy: ProfileCopy
    /** The profile names the store defines: the picker's own options. */
    defined?: string[]
    /** A write is in flight for this chat: every editor is disabled. */
    saving?: boolean
    /** The last pin write did not happen: one short line says so, once. */
    failed?: boolean
    /** Writes one layer's pin; absent where nothing may be written from here. */
    onSave?: (source: string, profile: string | null) => void
  } = $props()

  /** Whether this worktree offers at least one layer to write from here. */
  const editing = $derived(
    read &&
      pins !== null &&
      !pins.worktree_unresolved &&
      pins.layers.some((layer) => layer.writable),
  )

  /** One row is editable only when the group is and that layer may be written. */
  function editable(layer: PinLayer): boolean {
    return editing && layer.writable && onSave !== undefined
  }
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
  <!-- The one notice of the whole group: a pin write that did not happen. -->
  {#if editing && failed}
    <p class="fail" role="status">{copy.pinNotSaved}</p>
  {/if}
  <dl class="pins">
    {#each pins.layers as layer (layer.source)}
      <div class="pinrow">
        <dt class="plab">{pinLayerName(layer.source, copy)}</dt>
        <dd>
          {#if editable(layer)}
            <PinEditor
              {layer}
              {copy}
              {defined}
              {saving}
              onSave={onSave as (source: string, profile: string | null) => void}
            />
          {:else}
            <span class="mono">{layer.profile ?? copy.pinNone}</span>
          {/if}
          {#if layer.mark !== null}
            <span class="tag" class:winner={layer.mark === 'winner'}>
              {pinMarkWord(layer.mark, copy)}
            </span>
          {/if}
        </dd>
      </div>
    {/each}
  </dl>
  {#if editing}
    <p class="note">{copy.pinNote}</p>
  {/if}
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

  .fail {
    margin: 6px 0 0;
    color: var(--danger);
    font-size: 11.5px;
    line-height: 1.5;
  }

  .note {
    margin: 8px 0 0;
    color: var(--muted);
    font-size: 11.5px;
    line-height: 1.5;
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
