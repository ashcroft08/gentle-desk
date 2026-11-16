<script lang="ts">
  // The Profiles pane (0.1.12, S1, T2, L9, L11, L17): read-only, as the roadmap
  // agreed, drawn in two sub-tabs because the maintainer found one pane too long
  // ("esta mucha la informacion, hay forma de mejorarlo con pestañas"). Two
  // things are drawn, and nothing is written:
  //
  // - the Profile tab: the orchestrator switch of the chats opened from NOW ON -
  //   it is the session's, never a stored preference, so the note under it says
  //   so and a new run starts on gentle-pi's own default again - and the ACTIVE
  //   profile only (L11): its name, its orchestrator and fallback routes, and
  //   the one way out of the pane, the terminal. A field the file leaves out is
  //   drawn as nothing, never guessed;
  // - the Agents tab: one row per agent with the model it runs and the effort it
  //   asks for. The chain the card already draws is NOT repeated as a row (L17):
  //   the role named `fallback` is the card's own pair, so it is never a row.
  //
  // The file is the shell's, so editing, creating and switching happen in the
  // terminal: the one button opens that window, and the pane asks for a fresh
  // read when the pane opens, when the desk takes the focus back - which is what
  // "the panel refreshes when the terminal closes" means - and when the native
  // watcher says these two files changed on disk while the pane is mounted
  // (rpc-settings-sync T3b-2).
  import { dispatchIntent, openTerminal } from '../bridge'
  import type {
    OrchestratorModeView,
    ProfilesView_Serialize as ProfilesView,
  } from '../contract/bindings'
  import { language } from '../i18n/language.svelte'
  import { coalesced, followSettingsFiles } from './files-changed'
  import {
    PROFILE_PANEL_IDS,
    PROFILE_TABS,
    PROFILE_TAB_IDS,
    profileTabAfter,
    type ProfileTab,
  } from './profile-tabs'
  import { profilesCopy } from './profiles-copy'

  let { profiles }: { profiles: ProfilesView | null } = $props()

  /** The pane's own words in the language the app holds (0.2.0, D4). */
  const copy = $derived(profilesCopy(language.current))

  /** The tab the pane draws: the Profile, so the first view is the short one. */
  let tab = $state<ProfileTab>('profile')

  /** What the terminal said when it could not open, or `null`. */
  let terminalError = $state<string | null>(null)

  const mode = $derived<OrchestratorModeView>(profiles?.mode ?? 'organic')
  const active = $derived(profiles?.profiles.find((profile) => profile.active) ?? null)
  /** The chain gentle-pi falls back to, when the profile names one. */
  const fallback = $derived(active?.roles.find((role) => role.role === 'fallback') ?? null)
  /** The table's rows: every role except the fallback, which is the card's pair. */
  const agents = $derived(active?.roles.filter((role) => role.role !== 'fallback') ?? [])
  const failed = $derived(
    profiles !== null && profiles.status.type === 'Failed' ? profiles.status.data : null,
  )

  /** One read per open, so the pane never draws another run's file. */
  const read = coalesced(() => dispatchIntent({ type: 'RefreshProfiles' }))

  $effect(() => {
    void read()
  })

  /**
   * The terminal window closes and the desk is focused again: one read. It is
   * the same push any other change takes, never a timer.
   */
  $effect(() => {
    const again = () => void read()
    window.addEventListener('focus', again)
    return () => window.removeEventListener('focus', again)
  })

  /**
   * While this pane is on screen, the native watcher's notice re-reads the two
   * files it names - and unmounting the pane takes the listener with it.
   */
  $effect(() => followSettingsFiles('profiles', read))

  function choose(next: OrchestratorModeView): void {
    if (next === mode) return
    void dispatchIntent({ type: 'SetOrchestratorMode', data: { mode: next } })
  }

  /** A key that walks the sub-tabs also takes the focus with it. */
  function keyed(event: KeyboardEvent, from: ProfileTab): void {
    const next = profileTabAfter(event.key, from)
    if (next === null) return
    event.preventDefault()
    tab = next
    document.getElementById(PROFILE_TAB_IDS[next])?.focus()
  }

  async function edit(): Promise<void> {
    terminalError = await openTerminal('Profiles')
  }
</script>

<!-- The two sub-tabs (L17): the pair the pane switches between, one tab stop. -->
<!-- The file's own state, above the tabs: both tabs read the same document, so
     the reading, the failure and the empty case are the pane's, not a tab's. -->
{#if profiles === null || profiles.status.type === 'Unknown'}
  <p class="quiet">{copy.reading}</p>
{:else if failed !== null}
  <!-- The file is there and cannot be used: say why, and that a read is safe. -->
  <p class="quiet" role="status">{copy.failed(failed)}</p>
  <p class="quiet">{copy.retrySafe}</p>
  <button class="btn" type="button" onclick={() => void read()}>{copy.tryAgain}</button>
{:else if active === null}
  <p class="quiet">{copy.noProfiles}</p>
{/if}

<div class="subtabs" role="tablist" aria-label={copy.tabsLabel}>
  {#each PROFILE_TABS as item (item)}
    <button
      type="button"
      role="tab"
      id={PROFILE_TAB_IDS[item]}
      aria-controls={PROFILE_PANEL_IDS[item]}
      aria-selected={tab === item}
      tabindex={tab === item ? 0 : -1}
      onclick={() => (tab = item)}
      onkeydown={(event) => keyed(event, item)}
    >
      {copy.tabs[item]}
    </button>
  {/each}
</div>

<!-- Profile (L17): the switch of the chats opened from now on, the active
     profile's card, and the one action - the terminal. -->
<div
  class="tabpane"
  role="tabpanel"
  id={PROFILE_PANEL_IDS.profile}
  aria-labelledby={PROFILE_TAB_IDS.profile}
  hidden={tab !== 'profile'}
>
  <div class="mode">
    <span class="mode-label" id="profiles-mode">{copy.modeLabel}</span>
    <div class="seg" role="radiogroup" aria-labelledby="profiles-mode">
      <button
        type="button"
        role="radio"
        aria-checked={mode === 'organic'}
        onclick={() => choose('organic')}>{copy.modeOrganic}</button
      >
      <button
        type="button"
        role="radio"
        aria-checked={mode === 'full-delegation'}
        onclick={() => choose('full-delegation')}>{copy.modeFullDelegation}</button
      >
    </div>
    <p class="mode-note">{copy.modeNote}</p>
  </div>

  {#if active !== null}
    <div class="profile">
      <p class="pname">{active.name}</p>
      <p class="pmark">{copy.activeNote}</p>
      <dl class="routes">
        {#if active.orchestrator !== null}
          <div class="pair orchestrator">
            <dt>{copy.orchestratorLabel}</dt>
            <dd class="mono">{active.orchestrator.model ?? ''}</dd>
            <dd class="think">{active.orchestrator.thinking ?? ''}</dd>
          </div>
        {/if}
        {#if fallback !== null}
          <div class="pair fallback">
            <dt>{copy.fallbackLabel}</dt>
            <dd class="mono">{fallback.model ?? ''}</dd>
            <dd class="think">{fallback.thinking ?? ''}</dd>
          </div>
        {/if}
      </dl>
    </div>
  {/if}

  <div class="acts">
    <button class="btn accent" type="button" onclick={() => void edit()}>
      {copy.editInTerminal}
    </button>
  </div>
  <p class="note">{copy.editNote}</p>
  {#if terminalError !== null}
    <p class="quiet" role="status">{terminalError}</p>
  {/if}
</div>

<!-- Agents (L17): the roles the profile names, one row each. The fallback is the
     card's pair and never a row here. -->
<div
  class="tabpane"
  role="tabpanel"
  id={PROFILE_PANEL_IDS.agents}
  aria-labelledby={PROFILE_TAB_IDS.agents}
  hidden={tab !== 'agents'}
>
  {#if active !== null}
    <table class="agents">
      <thead>
        <tr>
          <th scope="col">{copy.agentColumn}</th>
          <th scope="col">{copy.modelColumn}</th>
          <th scope="col" class="effort">{copy.effortColumn}</th>
        </tr>
      </thead>
      <tbody>
        {#each agents as role (role.role)}
          <tr>
            <td class="agent">{role.role}</td>
            <td class="model" title={role.model ?? ''}>{role.model ?? ''}</td>
            <td class="effort">{role.thinking ?? role.tier ?? ''}</td>
          </tr>
        {/each}
      </tbody>
    </table>
  {/if}
</div>
