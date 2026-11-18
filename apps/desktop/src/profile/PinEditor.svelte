<script lang="ts">
  // ONE pin layer's editor (profiles-editor T6, S2, S3): the picker of the
  // profiles the store defines - plus "no pin", which CLEARS the layer - and the
  // one Save of this layer. It writes nothing until Save, an unchanged choice is
  // not a write, a name the profiles store does not define is refused under the
  // picker, and every control is disabled while a save is in flight.
  //
  // The choice is re-seeded when the STORE's answer for this layer changes (a save
  // answered, or the tab re-read); a push carrying the same layer keeps what the
  // user picked, which is what makes the editor safe to leave open while the chat
  // streams (the L18 lesson).
  //
  // A layer whose VALID pin another layer OUTRANKS is not reported by the store at
  // all (`desktop-rpc.md:211-234`), so its picker starts on a choice nobody can
  // send and Save waits for the user: the tab never claims to know a pin it was
  // not told.
  import { untrack } from 'svelte'
  import type { ChatProfilePinLayerView_Serialize as PinLayer } from '../contract/bindings'
  import { pinLayerName, type ProfileCopy } from './copy'
  import {
    NO_PIN,
    canSavePin,
    choiceOf,
    pinOptions,
    pinProblem,
    profileOf,
    type PinChoice,
  } from './pin-draft'
  import { pinProblemText } from './pin-copy'

  let {
    layer,
    copy,
    defined,
    saving = false,
    onSave,
  }: {
    /** The pin file this editor writes, as the store reported it. */
    layer: PinLayer
    copy: ProfileCopy
    /** The profile names the store defines: the picker's own options. */
    defined: string[]
    /** A write is in flight for this chat: the picker and Save are disabled. */
    saving?: boolean
    /** Writes this layer's pin; `null` clears it. */
    onSave: (source: string, profile: string | null) => void
  } = $props()

  /** The store's own answer for this layer, as a value: the effect below watches
   * THIS, not the object, so a push rebuilding an equal answer re-seeds nothing. */
  const stored = $derived(JSON.stringify([layer.status, layer.profile, layer.outranked]))

  let choice = $state<PinChoice>(untrack(() => choiceOf(layer)))
  let seeded: string | null = null

  $effect(() => {
    if (stored === seeded) return
    seeded = stored
    choice = untrack(() => choiceOf(layer))
  })

  const problem = $derived(pinProblem(choice, defined))
  const saveable = $derived(canSavePin(layer, choice, defined, saving))
  const label = $derived(pinLayerName(layer.source, copy))

  function save(): void {
    if (!saveable) return
    onSave(layer.source, profileOf(choice))
  }
</script>

<select
  class="pick"
  aria-label={copy.pinPick(label)}
  value={choice ?? ''}
  disabled={saving}
  onchange={(event) => (choice = event.currentTarget.value)}
>
  {#if layer.outranked}
    <option value="" disabled>{copy.pinUnreported}</option>
  {/if}
  <option value={NO_PIN}>{copy.pinNoPin}</option>
  {#each pinOptions(layer, defined) as name (name)}
    <option value={name}>{name}</option>
  {/each}
</select>
<button class="btn save" type="button" disabled={!saveable} onclick={save}>
  {copy.pinSaveLayer(label)}
</button>
{#if problem !== null}
  <span class="bad" role="status">{pinProblemText(copy, problem)}</span>
{/if}

<style>
  /* The picker is drawn with the same quiet frame as a chain editor's field. */
  .pick {
    min-width: 0;
    padding: 3px 6px;
    border: 1px solid var(--border);
    border-radius: 6px;
    background: var(--bg);
    color: var(--text);
    font: 12px ui-monospace, 'Cascadia Code', Consolas, monospace;
  }

  .pick:focus-visible {
    outline: 1px solid var(--text);
    outline-offset: -1px;
  }

  .pick:disabled,
  .save:disabled {
    opacity: 0.5;
    cursor: default;
  }

  .save {
    font-size: 11.5px;
    padding: 3px 9px;
  }

  .bad {
    color: var(--danger);
    font-size: 11.5px;
    line-height: 1.5;
  }
</style>
