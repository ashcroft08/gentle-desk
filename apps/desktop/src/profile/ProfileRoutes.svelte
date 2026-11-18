<script lang="ts">
  // What the profile this chat RUNS is set to, in the two pairs Settings > Profiles
  // draws for its own active profile (L19): the Orchestrator row (its model and its
  // effort) and the Fallback pair (the chain's models in order, and the effort only
  // when every step asks for the same one), then the profile's ORCHESTRATION MODE
  // as a switch (ProfileMode, T3-mode) and the terminal's other two outcomes
  // (ProfileApply, T3b).
  //
  // A pair the profile does not record - no `orchestrator` agent, no chain - is
  // drawn as NOTHING, never as a guess.
  import type {
    ChatProfileApplyNoticeView,
    ChatProfileDefinitionView_Serialize as ProfileDefinition,
    OrchestratorModeView,
  } from '../contract/bindings'
  import ProfileApply from './ProfileApply.svelte'
  import ProfileMode from './ProfileMode.svelte'
  import type { ProfileCopy } from './copy'
  import { fallbackChainOf, fallbackPair, orchestratorOf } from './rules'

  let {
    profile,
    copy,
    saving = false,
    saveFailed = false,
    applying = false,
    applyNotice = null,
    onSave,
    onApply,
  }: {
    profile: ProfileDefinition
    copy: ProfileCopy
    /** A mode save is in flight: both options are disabled until it answers. */
    saving?: boolean
    /** The last mode save did not happen: one short line says so. */
    saveFailed?: boolean
    /** An APPLY is in flight: both outcomes are disabled until it answers. */
    applying?: boolean
    /** The last apply wrote nothing: its own short line says why. */
    applyNotice?: ChatProfileApplyNoticeView | null
    /** Ask the shell to save this mode in the profile. */
    onSave?: (mode: OrchestratorModeView) => void
    /** Ask the shell to apply this profile: this chat, or the global default. */
    onApply?: (mode: 'session' | 'default') => void
  } = $props()

  const orchestrator = $derived(orchestratorOf(profile))
  const chain = $derived(fallbackChainOf(profile))
  const fallback = $derived(chain === null ? null : fallbackPair(chain))
</script>

<section class="routes" aria-label={copy.routesLabel}>
  {#if orchestrator !== null || fallback !== null}
    <dl class="pairs">
      {#if orchestrator !== null}
        <div class="pair">
          <dt>{copy.orchestratorLabel}</dt>
          <dd class="mono">{orchestrator.model ?? ''}</dd>
          <dd class="think">{orchestrator.thinking ?? ''}</dd>
        </div>
      {/if}
      {#if fallback !== null}
        <div class="pair">
          <dt>{copy.fallbackLabel}</dt>
          <dd class="mono fallmodels">{fallback.model}</dd>
          <dd class="think">{fallback.thinking ?? ''}</dd>
        </div>
      {/if}
    </dl>
  {/if}

  <!-- The profile's mode, and whose mode it is: every chat using this profile. -->
  <ProfileMode {profile} {copy} {saving} {saveFailed} {onSave} />

  <!-- The terminal's other two outcomes for THIS profile (T3b): one button each,
       and a line that names the scope each one touches. -->
  <div class="applied">
    <ProfileApply copy={copy.apply} {applying} notice={applyNotice} {onApply} />
  </div>
</section>

<style>
  /* The artboard's `.pv-routes`: the head's width, one pair under another. */
  .routes {
    display: flex;
    flex-direction: column;
    gap: 8px;
    max-width: var(--chat-max);
    margin: 12px auto 0;
  }

  .pairs {
    display: flex;
    flex-direction: column;
    gap: 4px;
    margin: 0;
  }

  .pair {
    display: flex;
    flex-wrap: wrap;
    align-items: baseline;
    gap: 8px;
  }

  dt {
    width: 96px;
    flex: none;
    color: var(--faint);
    font-size: 11px;
    font-weight: 600;
    letter-spacing: 0.05em;
    text-transform: uppercase;
  }

  dd {
    margin: 0;
  }

  .mono {
    color: var(--text);
    font-family: ui-monospace, 'Cascadia Code', Consolas, monospace;
    font-size: 12px;
  }

  .fallmodels {
    overflow-wrap: anywhere;
  }

  .think {
    color: var(--accent);
    font-size: 12px;
  }

  /* The two apply outcomes (ProfileApply.svelte): one row of the profile's own
     settings, separated from the mode the way the mode is separated from the
     pairs. */
  .applied {
    padding-top: 8px;
    border-top: 1px solid var(--border);
  }
</style>
