<script lang="ts">
  // The Profiles pane (0.1.12, S1, T2, L9, L11): read-only, as the roadmap
  // agreed. Two things are drawn, and nothing is written:
  //
  // - the orchestrator switch of the chats opened from NOW ON. It is the
  //   session's, never a stored preference, so the note under it says so and a
  //   new run starts on gentle-pi's own default again;
  // - the ACTIVE profile only (L11): its name, its orchestrator and fallback
  //   routes, and one row per agent with the model it runs and the effort it
  //   asks for. A field the file leaves out is drawn as nothing, never guessed.
  //
  // The file is the shell's, so editing, creating and switching happen in the
  // terminal: the one button opens that window, and the pane asks for a fresh
  // read when the pane opens and again when the desk takes the focus back -
  // which is what "the panel refreshes when the terminal closes" means.
  import { dispatchIntent, openTerminal } from '../bridge'
  import type {
    OrchestratorModeView,
    ProfilesView_Serialize as ProfilesView,
  } from '../contract/bindings'
  import {
    ACTIVE_NOTE,
    AGENT_COLUMN,
    EDIT_IN_TERMINAL,
    EDIT_NOTE,
    EFFORT_COLUMN,
    FALLBACK_LABEL,
    FAILED,
    MODE_FULL_DELEGATION,
    MODE_LABEL,
    MODE_NOTE,
    MODE_ORGANIC,
    MODEL_COLUMN,
    NO_PROFILES,
    ORCHESTRATOR_LABEL,
    READING,
    RETRY_SAFE,
    TRY_AGAIN,
  } from './profiles-copy'

  let { profiles }: { profiles: ProfilesView | null } = $props()

  /** What the terminal said when it could not open, or `null`. */
  let terminalError = $state<string | null>(null)

  const mode = $derived<OrchestratorModeView>(profiles?.mode ?? 'organic')
  const active = $derived(profiles?.profiles.find((profile) => profile.active) ?? null)
  /** The chain gentle-pi falls back to, when the profile names one. */
  const fallback = $derived(active?.roles.find((role) => role.role === 'fallback') ?? null)
  const failed = $derived(
    profiles !== null && profiles.status.type === 'Failed' ? profiles.status.data : null,
  )

  /** One read per open, so the pane never draws another run's file. */
  $effect(() => {
    void dispatchIntent({ type: 'RefreshProfiles' })
  })

  /**
   * The terminal window closes and the desk is focused again: one read. It is
   * the same push any other change takes, never a timer.
   */
  $effect(() => {
    const read = () => void dispatchIntent({ type: 'RefreshProfiles' })
    window.addEventListener('focus', read)
    return () => window.removeEventListener('focus', read)
  })

  function choose(next: OrchestratorModeView): void {
    if (next === mode) return
    void dispatchIntent({ type: 'SetOrchestratorMode', data: { mode: next } })
  }

  function read(): void {
    void dispatchIntent({ type: 'RefreshProfiles' })
  }

  async function edit(): Promise<void> {
    terminalError = await openTerminal('Profiles')
  }
</script>

<div class="mode">
  <span class="mode-label" id="profiles-mode">{MODE_LABEL}</span>
  <div class="seg" role="radiogroup" aria-labelledby="profiles-mode">
    <button
      type="button"
      role="radio"
      aria-checked={mode === 'organic'}
      onclick={() => choose('organic')}>{MODE_ORGANIC}</button
    >
    <button
      type="button"
      role="radio"
      aria-checked={mode === 'full-delegation'}
      onclick={() => choose('full-delegation')}>{MODE_FULL_DELEGATION}</button
    >
  </div>
  <p class="mode-note">{MODE_NOTE}</p>
</div>

{#if profiles === null || profiles.status.type === 'Unknown'}
  <p class="quiet">{READING}</p>
{:else if failed !== null}
  <!-- The file is there and cannot be used: say why, and that a read is safe. -->
  <p class="quiet" role="status">{FAILED(failed)}</p>
  <p class="quiet">{RETRY_SAFE}</p>
  <button class="btn" type="button" onclick={read}>{TRY_AGAIN}</button>
{:else if active === null}
  <p class="quiet">{NO_PROFILES}</p>
{:else}
  <div class="profile">
    <p class="pname">{active.name}</p>
    <p class="pmark">{ACTIVE_NOTE}</p>
    <dl class="routes">
      {#if active.orchestrator !== null}
        <div class="pair orchestrator">
          <dt>{ORCHESTRATOR_LABEL}</dt>
          <dd class="mono">{active.orchestrator.model ?? ''}</dd>
          <dd class="think">{active.orchestrator.thinking ?? ''}</dd>
        </div>
      {/if}
      {#if fallback !== null}
        <div class="pair fallback">
          <dt>{FALLBACK_LABEL}</dt>
          <dd class="mono">{fallback.model ?? ''}</dd>
          <dd class="think">{fallback.thinking ?? ''}</dd>
        </div>
      {/if}
    </dl>
  </div>

  <table class="agents">
    <thead>
      <tr>
        <th scope="col">{AGENT_COLUMN}</th>
        <th scope="col">{MODEL_COLUMN}</th>
        <th scope="col" class="effort">{EFFORT_COLUMN}</th>
      </tr>
    </thead>
    <tbody>
      {#each active.roles as role (role.role)}
        <tr>
          <td class="agent">{role.role}</td>
          <td class="model" title={role.model ?? ''}>{role.model ?? ''}</td>
          <td class="effort">{role.thinking ?? role.tier ?? ''}</td>
        </tr>
      {/each}
    </tbody>
  </table>
{/if}

<div class="acts">
  <button class="btn accent" type="button" onclick={() => void edit()}>
    {EDIT_IN_TERMINAL}
  </button>
</div>
<p class="note">{EDIT_NOTE}</p>
{#if terminalError !== null}
  <p class="quiet" role="status">{terminalError}</p>
{/if}
