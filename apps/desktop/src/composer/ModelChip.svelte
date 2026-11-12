<script lang="ts">
  // The model chip (0.1.11): the session's model and thinking level as two
  // pill dropdowns inside the composer bar, right of the box and left of the
  // send button, the way the user's screenshot shows it. The page never
  // guesses: what the child did not say is not shown, and a failed ask is a
  // state, not an error the composer turns into.
  //
  // One turn at a time is the composer's business, not the chip's: choosing a
  // model while the agent works is the child's session change, and pi answers
  // it or refuses it. The chip renders the refusal as the notice the app
  // already pushes, not as a disabled control.
  import { dispatchIntent } from '../bridge'
  import Icon from '../icons/Icon.svelte'
  import type { ModelChipView } from '../contract/bindings'
  import {
    LEVEL_LABEL,
    MODEL_LABEL,
    MODELS_EMPTY,
    MODELS_LOADING,
    MODELS_UNAVAILABLE,
  } from './copy'

  let { chip }: { chip: ModelChipView | null } = $props()

  const status = $derived(chip?.status.type ?? 'Unknown')
  const models = $derived(chip?.models ?? [])
  const current = $derived(chip?.current ?? null)
  const levels = $derived(chip?.thinking_levels ?? [])
  const level = $derived(chip?.thinking_level ?? null)

  function choseModel(event: Event): void {
    const value = (event.currentTarget as HTMLSelectElement).value
    const slash = value.indexOf('/')
    if (slash <= 0) return
    void dispatchIntent({
      type: 'SelectModel',
      data: { provider: value.slice(0, slash), model_id: value.slice(slash + 1) },
    })
  }

  function choseLevel(event: Event): void {
    const value = (event.currentTarget as HTMLSelectElement).value
    if (value === '') return
    void dispatchIntent({ type: 'SelectThinkingLevel', data: { level: value } })
  }
</script>

{#if chip !== null}
  <div class="chip">
    {#if chip?.status.type === 'Ready' && models.length > 0}
      <label class="pill">
        <select
          aria-label={MODEL_LABEL}
          value={current === null ? '' : `${current.provider}/${current.model_id}`}
          onchange={choseModel}
        >
          {#if current === null}
            <option value="" disabled hidden>{MODEL_LABEL}</option>
          {/if}
          {#each models as model (model.provider + '/' + model.model_id)}
            <option value={model.provider + '/' + model.model_id}>{model.name}</option>
          {/each}
        </select>
        <Icon name="chevronDown" size={12} />
      </label>
      {#if levels.length > 0}
        <label class="pill level">
          <select aria-label={LEVEL_LABEL} value={level ?? ''} onchange={choseLevel}>
            {#if level === null}
              <option value="" disabled hidden>{LEVEL_LABEL}</option>
            {/if}
            {#each levels as option (option)}
              <option value={option}>{option}</option>
            {/each}
          </select>
          <Icon name="chevronDown" size={12} />
        </label>
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
