<script lang="ts">
  // What the profile this chat RUNS is set to, in the two pairs Settings > Profiles
  // draws for its own active profile (L19): the Orchestrator row (its model and its
  // effort) and the Fallback pair (the chain's models in order, and the effort only
  // when every step asks for the same one), plus the profile's ORCHESTRATION MODE
  // as a switch (T3-mode, L23).
  //
  // The mode is the PROFILE's, not this chat's (L20, L22): `profiles.json` stores it
  // under `workflow.<profile>.orchestratorMode`, which is why a change in the
  // terminal reaches every chat using that profile. An absent entry is `organic`,
  // gentle-pi's own default.
  //
  // The switch LOOKS and KEYS like Settings > Profiles' own (`.seg` radiogroup,
  // two buttons, no custom keys): the page keeps that one's words and behaviour,
  // and its own copy of the styles because app.css scopes them to `.settings-pane`.
  // It WRITES through the shell, which sends the whole document over this chat's
  // own child; a pair the profile does not record - no `orchestrator` agent, no
  // chain - is drawn as NOTHING, never as a guess.
  import type {
    ChatProfileDefinitionView_Serialize as ProfileDefinition,
    OrchestratorModeView,
  } from '../contract/bindings'
  import type { ProfileCopy } from './copy'
  import { fallbackChainOf, fallbackPair, orchestratorOf } from './rules'

  let {
    profile,
    copy,
    saving = false,
    saveFailed = false,
    onSave,
  }: {
    profile: ProfileDefinition
    copy: ProfileCopy
    /** A save is in flight: both options are disabled until it answers. */
    saving?: boolean
    /** The last save did not happen: one short line says so, nothing is written. */
    saveFailed?: boolean
    /** Ask the shell to save this mode in the profile. */
    onSave?: (mode: OrchestratorModeView) => void
  } = $props()

  const orchestrator = $derived(orchestratorOf(profile))
  const chain = $derived(fallbackChainOf(profile))
  const fallback = $derived(chain === null ? null : fallbackPair(chain))

  /**
   * One option of the switch. The stored mode is never written again (the same
   * rule the Settings pane follows), and nothing is sent while a save is in
   * flight - the disabled buttons say so, and this guard keeps a keyboard or a
   * test from going around them.
   */
  function choose(next: OrchestratorModeView): void {
    if (saving || next === profile.mode) return
    onSave?.(next)
  }
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
    <div class="seg" role="radiogroup" aria-label={copy.modeLabel}>      <button
        type="button"
        role="radio"
        aria-checked={profile.mode === 'organic'}
        disabled={saving}
        onclick={() => choose('organic')}>{copy.modeOrganic}</button
      >
      <button
        type="button"
        role="radio"
        aria-checked={profile.mode === 'full-delegation'}
        disabled={saving}
        onclick={() => choose('full-delegation')}>{copy.modeFullDelegation}</button
      >
    </div>
    <p class="mnote">{copy.modeNote(profile.name)}</p>    {#if saveFailed}
      <!-- One short line, tied to this chat: the mode was not saved. -->
      <p class="mfail" role="status">{copy.modeNotSaved}</p>
    {/if}
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

  /* The mode line: the label, the switch, the stored value, and the note under
     them. `.seg` is Settings > Profiles' own look, copied because app.css keeps
     its rules scoped to `.settings-pane`. */
  .mode {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 4px 10px;
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

  .seg {
    display: inline-flex;
    border: 1px solid var(--border);
    border-radius: 8px;
    overflow: hidden;
  }

  .seg [role='radio'] {
    border: 0;
    background: transparent;
    color: var(--muted);
    font: inherit;
    font-size: 12.5px;
    padding: 5px 12px;
    cursor: pointer;
  }

  .seg [role='radio']:hover {
    background: var(--surface2);
    color: var(--text);
  }

  .seg [role='radio'][aria-checked='true'] {
    background: var(--accent);
    color: var(--bg);
    font-weight: 600;
  }

  .seg [role='radio']:focus-visible {
    outline: 1px solid var(--text);
    outline-offset: -2px;
  }

  .seg [role='radio']:disabled {
    cursor: default;
    opacity: 0.55;
  }

  /* The note under the switch: saved in the PROFILE, never this chat's own. */
  .mnote {
    flex-basis: 100%;
    margin: 0;
    color: var(--muted);
    font-size: 11.5px;
    line-height: 1.5;
  }

  .mfail {
    flex-basis: 100%;
    margin: 0;
    color: var(--danger);
    font-size: 11.5px;
    line-height: 1.5;
  }
</style>
