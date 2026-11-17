<script lang="ts">
  // What the profile this chat RUNS is set to, in the two pairs Settings > Profiles
  // draws for its own active profile (L19): the Orchestrator row (its model and its
  // effort) and the Fallback pair (the chain's models in order, and the effort only
  // when every step asks for the same one), plus the profile's ORCHESTRATION MODE.
  //
  // The mode is the PROFILE's, not this chat's (L20, L22): `profiles.json` stores it
  // under `workflow.<profile>.orchestratorMode`, which is why a change in the
  // terminal reaches every chat using that profile. An absent entry is `organic`,
  // gentle-pi's own default, and the scope line says so in words.
  //
  // It WRITES NOTHING: no switch, no control, not even a disabled one. The writing
  // switch is T3 (`set_config` of `workflow`). A pair the profile does not record -
  // no `orchestrator` agent, no chain - is drawn as NOTHING, never as a guess.
  import type { ChatProfileDefinitionView_Serialize as ProfileDefinition } from '../contract/bindings'
  import type { ProfileCopy } from './copy'
  import { fallbackChainOf, fallbackPair, modeName, orchestratorOf } from './rules'

  let { profile, copy }: { profile: ProfileDefinition; copy: ProfileCopy } = $props()

  const orchestrator = $derived(orchestratorOf(profile))
  const chain = $derived(fallbackChainOf(profile))
  const fallback = $derived(chain === null ? null : fallbackPair(chain))
  const mode = $derived(modeName(profile.mode, copy))
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
  <div class="mode">
    <span class="mlabel">{copy.modeLabel}</span>
    <span class="mvalue">{mode}</span>
    <span class="mscope">{copy.modeScope}</span>
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

  /* The mode line: the label, the stored value, and its scope in the quiet ink. */
  .mode {
    display: flex;
    flex-wrap: wrap;
    align-items: baseline;
    gap: 8px;
    padding-top: 6px;
    border-top: 1px solid var(--border);
  }

  .mlabel {
    width: 96px;
    flex: none;
    color: var(--faint);
    font-size: 11px;
    font-weight: 600;
    letter-spacing: 0.05em;
    text-transform: uppercase;
  }

  .mvalue {
    color: var(--text);
    font-size: 12.5px;
    font-weight: 600;
  }

  .mscope {
    color: var(--muted);
    font-size: 11.5px;
    line-height: 1.5;
  }
</style>
