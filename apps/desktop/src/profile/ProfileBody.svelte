<script lang="ts">
  // The body of the Profile tab, read-only (06-profile-tab.html): one accordion
  // group per profile the store defines - its agents, its fallback chains - and one
  // for the pin of this chat's worktree. ONE group is open at a time, as the
  // artboard draws it, and the profile this chat RUNS opens by itself so the view
  // never starts empty-handed.
  //
  // A store that did not answer is marked WHERE ITS CONTENT WOULD BE (a profile
  // list, the chains of a profile, the pin layers) instead of being drawn as an
  // empty one: nothing here can tell "this store is empty" from "nobody asked".
  import type { ChatProfileView_Serialize as ChatProfile } from '../contract/bindings'
  import { pinLayerName, profileSummary, type ProfileCopy } from './copy'
  import { storeRead, usedName } from './rules'
  import ProfileAgents from './ProfileAgents.svelte'
  import ProfileChains from './ProfileChains.svelte'
  import ProfileGroup from './ProfileGroup.svelte'
  import ProfilePins from './ProfilePins.svelte'

  let { view, copy }: { view: ChatProfile; copy: ProfileCopy } = $props()

  const profilesRead = $derived(storeRead(view, 'profiles'))
  const fallbacksRead = $derived(storeRead(view, 'fallbacks'))
  const pinsRead = $derived(storeRead(view, 'pins'))

  /** The pin group's quiet summary: the layer that wins here, or why none does. */
  const pinSummary = $derived.by(() => {
    if (!pinsRead) return ''
    const pins = view.pins
    if (pins === null) return ''
    if (pins.worktree_unresolved) return copy.noWorktree
    const winner = pins.winner
    if (winner === null) return ''
    return `${pinLayerName(winner.source, copy)} ${winner.profile ?? copy.pinNone}`
  })

  /** The pin group's id: the one group that is not a profile. */
  const PIN_GROUP = 'profile-group-pin'

  /** The id of one profile's group, in the order the store reports them. */
  function groupId(index: number): string {
    return `profile-group-${index}`
  }

  /**
   * The group that opens by itself: the profile this chat RUNS, as the artboard's
   * read views follow it, and the first one when the store does not define it.
   */
  const first = $derived.by(() => {
    if (!profilesRead || view.profiles.length === 0) return null
    const used = usedName(view)
    const index = view.profiles.findIndex((profile) => profile.name === used)
    return groupId(index < 0 ? 0 : index)
  })

  /** The group the user opened; `undefined` until then, so the first one opens. */
  let chosen = $state<string | null | undefined>(undefined)
  const opened = $derived(chosen === undefined ? first : chosen)

  /** Opens a group, or closes it when it is the one already open. */
  function toggle(id: string): void {
    chosen = opened === id ? null : id
  }
</script>

<div class="groups">
  {#if !profilesRead}
    <p class="quiet">{copy.storeMissing('profiles')}</p>
  {:else if view.profiles.length === 0}
    <p class="quiet">{copy.noProfiles}</p>
  {:else}
    {#each view.profiles as profile, index (profile.name)}
      {@const id = groupId(index)}
      <ProfileGroup
        title={profile.name}
        summary={profileSummary(profile, copy)}
        expanded={opened === id}
        panelId={id}
        onToggle={() => toggle(id)}
      >
        <ProfileAgents agents={profile.agents} {copy} />
        <h4 class="sub">{copy.chainTitle}</h4>
        <ProfileChains chains={profile.chains} read={fallbacksRead} {copy} />
      </ProfileGroup>
    {/each}
  {/if}

  <ProfileGroup
    title={copy.pinTitle}
    summary={pinSummary}
    expanded={opened === PIN_GROUP}
    panelId={PIN_GROUP}
    onToggle={() => toggle(PIN_GROUP)}
  >
    <ProfilePins pins={view.pins} read={pinsRead} {copy} />
  </ProfileGroup>
</div>

<style>
  /* The artboard's `.pv-body`: the thread's width again, one group under another. */
  .groups {
    display: flex;
    flex-direction: column;
    gap: 10px;
    max-width: var(--chat-max);
    margin: 14px auto 0;
  }

  .sub {
    margin: 12px 0 0;
    color: var(--faint);
    font-size: 11px;
    font-weight: 600;
    letter-spacing: 0.05em;
    text-transform: uppercase;
  }

  .quiet {
    margin: 0;
    color: var(--muted);
    font-size: 12.5px;
    line-height: 1.5;
  }
</style>
