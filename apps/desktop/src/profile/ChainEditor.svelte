<script lang="ts">
  // ONE fallback chain, editable (profiles-editor T5, L30-3): the steps of a chain
  // the fallback store records for the profile this chat USES, as an ordered list
  // of `model + optional thinking` rows the user can add to, remove from and
  // reorder - within the owner's own cap of eight, and never emptied.
  //
  // The editor writes NOTHING until Save: typing is a draft. Cancel throws it
  // away, an unchanged draft has nothing to send (Save is disabled), and a draft
  // the owner would refuse never leaves: the reason is drawn under the list and
  // Save stays disabled. The draft is re-seeded when the STORE's chain changes (a
  // save answered, or the tab re-read); a push that carries the same chain keeps
  // whatever the user is typing, which is what makes the editor safe to leave open
  // while the chat streams (the L18 lesson).
  import { untrack } from 'svelte'
  import type { ChatProfileStepView_Serialize as Step } from '../contract/bindings'
  import {
    MAX_CHAIN_STEPS,
    addStep,
    chainProblem,
    draftOf,
    moveStep,
    removeStep,
    sameChain,
    stepsOf,
    type DraftStep,
  } from './chain-draft'
  import { problemText } from './chain-copy'
  import type { ProfileCopy } from './copy'

  let {
    agent,
    label,
    steps,
    copy,
    saving = false,
    onSave,
  }: {
    /** The store's own chain key: an agent's name, or `*`. Sent back untouched. */
    agent: string
    /** What the tab calls this chain: the agent's key, or the `*` label. */
    label: string
    /** The chain the store holds right now. */
    steps: Step[]
    copy: ProfileCopy
    /** A write is in flight for this chat: every field and action is disabled. */
    saving?: boolean
    /** Ask the shell to save this chain in the profile. */
    onSave: (agent: string, steps: Step[]) => void
  } = $props()

  /** The store's chain as a value: the effect below watches THIS, not the array. */
  const stored = $derived(JSON.stringify(steps.map((step) => [step.model, step.thinking])))

  // Seeded ONCE from the chain the editor was drawn with: `untrack` because this
  // is a starting value, never a subscription to the prop (the effect below is
  // the only thing that follows the store's chain).
  let draft = $state<DraftStep[]>(untrack(() => draftOf(steps)))

  /** The store's chain the draft was last seeded from; `null` before the first
   * effect run, which seeds the draft from the props it mounted with. */
  let seeded: string | null = null

  $effect(() => {
    if (stored === seeded) return
    seeded = stored
    // Untracked on purpose: the effect depends on the store's own VALUE, so a
    // push that rebuilds an equal chain re-runs nothing (L18).
    draft = untrack(() => draftOf(steps))
  })

  const problem = $derived(chainProblem(draft))
  const dirty = $derived(!sameChain(draft, steps))
  const canSave = $derived(!saving && dirty && problem === null)

  function edit(index: number, field: 'model' | 'thinking', value: string): void {
    draft = draft.map((step, at) => (at === index ? { ...step, [field]: value } : step))
  }

  function save(): void {
    if (!canSave) return
    onSave(agent, stepsOf(draft))
  }
</script>

<div class="chain" role="group" aria-label={label}>
  <p class="cname">{label}</p>
  <ol class="steps">
    {#each draft as step, index (index)}
      <li>
        <span class="n" aria-hidden="true">{index + 1}</span>
        <input
          class="model"
          type="text"
          aria-label={copy.chainModel(index + 1)}
          value={step.model}
          disabled={saving}
          oninput={(event) => edit(index, 'model', event.currentTarget.value)}
        />
        <input
          class="think"
          type="text"
          aria-label={copy.chainThinking(index + 1)}
          placeholder={copy.chainThinkingHint}
          value={step.thinking}
          disabled={saving}
          oninput={(event) => edit(index, 'thinking', event.currentTarget.value)}
        />
        <button
          class="mini"
          type="button"
          aria-label={copy.chainMoveUp(index + 1)}
          disabled={saving || index === 0}
          onclick={() => (draft = moveStep(draft, index, -1))}>↑</button
        >
        <button
          class="mini"
          type="button"
          aria-label={copy.chainMoveDown(index + 1)}
          disabled={saving || index === draft.length - 1}
          onclick={() => (draft = moveStep(draft, index, 1))}>↓</button
        >
        <button
          class="mini"
          type="button"
          aria-label={copy.chainRemoveStep(index + 1)}
          disabled={saving || draft.length <= 1}
          onclick={() => (draft = removeStep(draft, index))}>×</button
        >
      </li>
    {/each}
  </ol>
  <div class="acts">
    <button
      class="btn"
      type="button"
      disabled={saving || draft.length >= MAX_CHAIN_STEPS}
      onclick={() => (draft = addStep(draft))}>{copy.chainAddStep}</button
    >
    <button class="btn" type="button" disabled={!canSave} onclick={save}>{copy.chainSave}</button>
    <button
      class="btn"
      type="button"
      disabled={saving || !dirty}
      onclick={() => (draft = draftOf(steps))}>{copy.chainCancel}</button
    >
  </div>
  {#if problem !== null}
    <p class="bad" role="status">{problemText(copy, problem)}</p>
  {/if}
</div>

<style>
  .cname {
    margin: 0;
    color: var(--text);
    font-size: 12.5px;
    font-weight: 600;
  }

  .steps {
    display: flex;
    flex-direction: column;
    gap: 3px;
    margin: 4px 0 0;
    padding: 0;
    list-style: none;
  }

  .steps li {
    display: flex;
    align-items: center;
    gap: 5px;
  }

  .n {
    width: 14px;
    flex: none;
    color: var(--faint);
    font: 600 11px ui-monospace, 'Cascadia Code', Consolas, monospace;
  }

  input {
    min-width: 0;
    padding: 3px 6px;
    border: 1px solid var(--border);
    border-radius: 6px;
    background: var(--bg);
    color: var(--text);
    font: 12px ui-monospace, 'Cascadia Code', Consolas, monospace;
  }

  .model {
    flex: 1 1 60%;
  }

  .think {
    flex: 0 1 30%;
  }

  input:focus-visible {
    outline: 1px solid var(--text);
    outline-offset: -1px;
  }

  input:disabled,
  .mini:disabled {
    opacity: 0.5;
    cursor: default;
  }

  .mini {
    flex: none;
    width: 20px;
    height: 20px;
    padding: 0;
    border: 1px solid var(--border);
    border-radius: 5px;
    background: transparent;
    color: var(--muted);
    font-size: 11px;
    line-height: 1;
    cursor: pointer;
  }

  .mini:hover:not(:disabled) {
    background: var(--surface2);
    color: var(--text);
  }

  .acts {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    margin-top: 6px;
  }

  .acts .btn {
    font-size: 11.5px;
    padding: 3px 9px;
  }

  .bad {
    margin: 6px 0 0;
    color: var(--danger);
    font-size: 11.5px;
    line-height: 1.5;
  }
</style>
