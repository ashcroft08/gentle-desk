<script lang="ts">
  // The model chip (0.1.11, S7): the session's model and thinking level as
  // two pill dropdowns in their own row BELOW the composer form. The lists
  // are the app's own themed popovers, never native select popups. The page
  // never guesses: what the child did not say is not shown, and a failed ask
  // is a state, not an error the composer turns into.
  import { dispatchIntent } from '../bridge'
  import type { ModelChipView } from '../contract/bindings'
  import PillSelect from './PillSelect.svelte'
  import {
    LEVEL_LABEL,
    MODEL_LABEL,
    MODELS_EMPTY,
    MODELS_LOADING,
    MODELS_UNAVAILABLE,
  } from './copy'

  let { chip }: { chip: ModelChipView | null } = $props()

  const status = $derived(chip?.status.type ?? 'Unknown')
  const models = $derived(
    (chip?.models ?? []).map((model) => ({
      value: `${model.provider}/${model.model_id}`,
      label: model.name,
      hint: model.provider,
    })),
  )
  const current = $derived(
    chip?.current === null || chip?.current === undefined
      ? null
      : `${chip.current.provider}/${chip.current.model_id}`,
  )
  const levels = $derived(
    (chip?.thinking_levels ?? []).map((level) => ({ value: level, label: level })),
  )
  const level = $derived(chip?.thinking_level ?? null)

  function choseModel(value: string): void {
    const slash = value.indexOf('/')
    if (slash <= 0) return
    void dispatchIntent({
      type: 'SelectModel',
      data: { provider: value.slice(0, slash), model_id: value.slice(slash + 1) },
    })
  }

  function choseLevel(value: string): void {
    if (value === '') return
    void dispatchIntent({ type: 'SelectThinkingLevel', data: { level: value } })
  }
</script>

{#if chip !== null}
  <div class="chip">
    {#if chip?.status.type === 'Ready' && models.length > 0}
      <PillSelect label={MODEL_LABEL} value={current} options={models} filterable onchange={choseModel} />
      {#if levels.length > 0}
        <PillSelect label={LEVEL_LABEL} value={level} options={levels} faint onchange={choseLevel} />
      {/if}
    {:else if status === 'Loading' || status === 'Unknown'}
      <p class="state">{MODELS_LOADING}</p>
    {:else if status === 'Empty'}
      <p class="state">{MODELS_EMPTY}</p>
    {:else if chip?.status.type === 'Failed'}
      <p class="state" role="status">{MODELS_UNAVAILABLE} {chip.status.data}</p>
    {/if}
  </div>
{/if}
